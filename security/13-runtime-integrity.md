---
title: Runtime Integrity Verification
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - NIST-SP-800-193
bootloader: true
firmware: shared
hardware: false
---

# Runtime Integrity Verification

## Summary

After boot, the system periodically re-verifies the integrity of critical
memory regions (flash firmware, configuration data, measurement registers).
For MCUs with TrustZone (L5/U5), this is performed by a secure-world monitor
that is invisible to the application. For F411/H7, a dedicated runtime task
performs the verification.

## Motivation

Boot-time verification is necessary but insufficient:
- An attacker with continuous power can modify flash after boot
- DMA-capable peripherals can corrupt memory
- Flash bit flips (from aging or radiation) occur after boot
- Supply voltage variations can cause read disturb errors

NIST SP 800-193 requires detection of unauthorized changes after the boot
process is complete.

## Design

### TrustZone-Protected Monitor (L5/U5)

```
Secure World (TrustZone):
  ┌─────────────────────────────┐
  │ Secure Monitor Timer Interrupt │
  │ Fires every N ms              │
  │                              │
  │ 1. Check IMR values (not modified)  │
  │ 2. SHA-384 over bootloader flash    │
  │ 3. SHA-384 over firmware flash      │
  │ 4. Compare against known-good hash  │
  │ 5. If mismatch: set tamper flag     │
  │ 6. Clear timer, return to NS world  │
  └─────────────────────────────┘
```

The secure monitor is invisible to the non-secure world. The application
cannot disable it, modify its code, or read its measurement results.

### Software-Only Runtime Check (F411/H7)

A dedicated low-priority task in the application:

```rust
fn runtime_integrity_task() -> ! {
    loop {
        // Wait for periodic timer
        wfi();

        // Verify bootloader region (read-only)
        let boot_hash = sha384(bootloader_flash_slice());
        if boot_hash != EXPECTED_BOOT_HASH {
            set_tamper_flag(TamperSource::BootFlashModified);
        }

        // Verify firmware region
        let fw_hash = sha384(firmware_flash_slice());
        if fw_hash != EXPECTED_FW_HASH {
            set_tamper_flag(TamperSource::FirmwareModified);
        }

        // Verify measurement registers (IMRs) haven't been reset
        let imr = read_imr(IMR_FIRMWARE);
        if imr != EXPECTED_IMR {
            set_tamper_flag(TamperSource::ImrModified);
        }
    }
}
```

### Integrity Verification Schedule

| Region | Frequency | MCU | Method |
|--------|-----------|-----|--------|
| Bootloader flash | 1× per IWDG cycle | All | SHA-384 hash compare |
| Firmware flash | 1× per IWDG cycle | All | SHA-384 hash compare |
| IMR values | 1× per IWDG cycle | All | Compare against expected |
| Config data | 1× per 10 IWDG cycles | All | CRC-32 |
| Stack boundary | 1× per ISR entry | All | Stack canary check |

## Interface

```rust
pub trait RuntimeIntegrity {
    /// Set up runtime integrity check schedule.
    fn configure(period_ms: u32) -> Result<(), IntegrityError>;

    /// Trigger a full integrity check.
    fn check_all() -> Result<IntegrityReport, IntegrityError>;

    /// Read the last integrity check result.
    fn last_report() -> IntegrityReport;
}

pub struct IntegrityReport {
    pub bootloader_match: bool,
    pub firmware_match: bool,
    pub imr_match: bool,
    pub config_match: bool,
    pub timestamp: u64,
}
```

## Verification

### Testing Requirements

- Boot → wait for runtime check → assert `check_all()` returns match
- Corrupt a single byte in firmware flash → next check reports mismatch
- Corrupt bootloader flash → next check reports mismatch
- Reset IMR → check reports mismatch
- Run 1000 integrity checks → assert no false positives
- Timer accuracy: verify check fires within configured period ±10%

### Kani Proofs

| Harness | Property |
|----------|---------|
| `integrity_check_bounded` | Each integrity check completes within bounded time |
| `tamper_flag_once_set` | Once a mismatch is detected, the tamper flag stays set |
| `integrity_check_deterministic` | Same flash content produces same result |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Integrity monitor cannot be bypassed (TZ) | Verus | Non-secure world cannot mask timer interrupt |
| Hash comparison is sound | Creusot | SHA-384 preimage resistance implies hash match = content match |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `integrity_check_flash_content` | Random flash content patterns — verify never panics |

### Proptests

| Property | Check |
|----------|-------|
| `integrity_matches_unmodified` | Unmodified flash always passes integrity check |
| `any_bit_flip_detected` | Any single-bit flip in flash is detected |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- Software-only runtime integrity (F411/H7) can itself be subverted if the
  firmware is compromised. The attacker would patch the integrity check task
  to skip verification. Only TrustZone provides a true hardware root for
  runtime monitoring.
- SHA-384 of the full firmware is slow (seconds). With a ~30s IWDG period,
  the integrity check consumes significant CPU time. Mitigation: HMAC-based
  incremental verification or hardware hash accelerator.