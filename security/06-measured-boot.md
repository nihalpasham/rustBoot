---
title: Measured Boot Chain (SHA-384)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - NIST-SP-800-193
  - Common-Criteria-EAL5+
  - IETF-SUIT-RFC9019
bootloader: true
firmware: shared
hardware: false
---

# Measured Boot Chain (SHA-384)

## Summary

Establish an immutable chain of measurements across boot stages. Each stage
measures (SHA-384 hash) the next stage before executing it. The measurements
are recorded in measurement registers (IMRs) that can only be extended, never
reset. The final measurement digest provides a fingerprint of the entire boot
state for attestation.

## Motivation

Signature verification proves that firmware hasn't been tampered with since it
was signed. Measured boot proves exactly what software is running and in what
order. This is essential for:
- Remote attestation: prove device state to a verifier
- Supply chain integrity: verify that each stage is authentic
- Forensic analysis: determine exactly what booted
- Multi-stage boot: each stage trusts only the stage that measured it

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Boot-time integrity verification | NIST SP 800-193 §4.3 | SHA-384 measurement of each stage |
| Non-bypassable measurement chain | NIST SP 800-193 §4.2.1.3 | ROM measures rustBoot; rustBoot measures firmware |
| Measurement for attestation | RFC 9019 §5 | Measurement log available for remote attestation |
| Extend-only measurement registers | EAL5+ (FDP_SDI.2) | IMRs: `new = SHA384(old || data)` |

## Design

### Measurement Chain

```
Immutable ROM (silicon)
    → measures: SHA384(rom_code) → IMR[0]
    → verifies + loads rustBoot
    → measures: SHA384(rustBoot_binary) → IMR[1]
    → transfers control

rustBoot Bootloader
    → measures: SHA384(bootloader_config) → IMR[2]
    → measures: SHA384(firmware_encrypted) → IMR[3]
    → decrypts firmware
    → measures: SHA384(firmware_decrypted) → IMR[4]
    → measures: SHA384(firmware_config) → IMR[5]
    → verifies signatures
    → extends IMR chain
    → boots firmware

Application Firmware
    → reads IMR[0..5] for attestation
    → extends: SHA384(IMR[5] || app_config) → IMR[6]
```

### Measurement Register Implementation

```rust
pub struct Imr {
    // Not a real register — software-emulated SHA-384 state.
    // For TrustZone MCUs, stored in secure-RAM only accessible to secure world.
    value: [u8; 48], // SHA-384 output
}

impl Imr {
    /// Create a new IMR initialized to SHA-384 of all zeros.
    pub fn new() -> Self { /* ... */ }

    /// Extend: IMR = SHA-384(IMR || data).
    /// This is the only way to change the IMR value.
    pub fn extend(&mut self, data: &[u8]) {
        let mut hasher = Sha384::new();
        hasher.update(&self.value);
        hasher.update(data);
        self.value = hasher.finalize();
    }
}
```

Property: IMR values are monotonic in the sense that SHA-384 preimage
resistance prevents an attacker from finding input data that produces a
desired IMR value. This is the standard TCG measured boot property.

## Interface

```rust
pub trait MeasuredBoot {
    /// Number of supported IMRs.
    const IMR_COUNT: usize = 8;

    /// Read current value of IMR[index].
    /// Returns Error if index >= IMR_COUNT.
    fn read_imr(index: usize) -> Result<[u8; 48], MBError>;

    /// Extend IMR[index] with data.
    /// Verified: IMR value after extend is deterministically
    /// determined by old value and data.
    fn extend_imr(index: usize, data: &[u8]) -> Result<(), MBError>;

    /// Export the measurement log (ordered list of extend operations).
    /// Used for remote attestation verification.
    fn measurement_log() -> Result<MeasurementLog, MBError>;
}

pub struct MeasurementLog {
    pub entries: [MeasurementEntry; MAX_LOG_ENTRIES],
    pub count: usize,
}

pub struct MeasurementEntry {
    pub index: usize,
    pub data_hash: [u8; 48], // Hash of the data that was extended
}
```

## Verification

### Testing Requirements

- Create IMR → extend → assert value changed
- Extend twice with different data → assert different final values
- Extend with same data twice → assert deterministic (same final value)
- Read IMR[0..7] → all valid
- Read IMR[8] → `MBError::InvalidIndex`
- Measurement log contains all extend operations in order

### Kani Proofs

| Harness | Property |
|---------|----------|
| `imr_extend_monotonic` | IMR value changes after every extend operation (preimage resistance not provable in Kani, but we prove the function executes) |
| `imr_bounds_no_overflow` | `extend_imr(index >= IMR_COUNT)` returns error |
| `imr_extend_deterministic` | Same (IMR, data) → same new IMR value |
| `measurement_log_order` | Log entries are returned in insertion order |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| SHA-384 hash chain correctness | Verus | IMR[n] = SHA384(IMR[n-1] || data) is functionally correct |
| Extend-only invariant | Creusot | No function exists that sets IMR directly (only extend) |
| Measurement log completeness | Creusot | Every extend operation appears in the log |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `imr_extend_arbitrary` | Random data extended into IMRs — verify never panics |

### Proptests

| Property | Check |
|----------|-------|
| `imr_extend_commutativity` | Extension order is not commutative (SHA-384 collision resistance makes this practically provable) |
| `imr_all_values_distinct` | Different boot paths produce different IMR values |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- Without a hardware anchor (TrustZone on L5/U5), software-emulated IMRs in
  rustBoot can be tampered with if an attacker gains code execution before the
  bootloader. This is mitigated by the ROM boot process: on STM32, the ROM
  bootloader is immutable and always runs first.
- The measurement log is stored in RAM and lost on power-off. Persistent
  attestation requires a secure element or external attestation infrastructure.