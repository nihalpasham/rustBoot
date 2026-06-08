---
title: Hardware-Backed Anti-Rollback (OTP Monotonic Counter)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - NIST-SP-800-193
  - Common-Criteria-EAL5+
  - NSA-CNSA-2.0
bootloader: true
firmware: false
hardware: true
---

# Hardware-Backed Anti-Rollback (OTP Monotonic Counter)

## Summary

Store a monotonic firmware version counter in hardware that cannot be
decremented or reset by software. Each firmware image carries a security
version number in its signed manifest. The bootloader rejects any image whose
version ≤ the hardware counter. The counter is incremented only through the
authenticated update process. OTP (One-Time Programmable) fuses or battery-
backed registers provide the hardware anchor.

## Motivation

The current software-only anti-rollback checks version fields in flash. An
attacker with physical access (external battery, debug probe) can:
- Modify the version field in the TLV header
- Re-flash an older signed firmware image with a known vulnerability
- Bypass the version check by altering flash contents directly

Hardware monotonic counters make rollback physically impossible without
destroying the device (OTP) or triggering tamper response (backup registers).

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Anti-rollback monotonic counter | NIST SP 800-193 §4.2.2.2 | OTP or battery-backed monotonic counter |
| Reject firmware ≤ current version | NIST SP 800-193 §4.2.5 | Version comparison before boot |
| Secure version storage, not software-modifiable | EAL5+ (FDP_SDI.2) | OTP fuses or RTC backup registers |
| Security version number in signed metadata | CNSA 2.0 | Version in TLV header, covered by signature |
| Counter persistence across power loss | EAL5+ (FPT_RCV.3) | OTP is non-volatile; backup regs have battery |

## Design

### Counter Storage

| MCU | Storage | Capacity | Wear | Notes |
|-----|---------|----------|------|-------|
| F411 | RTC backup register + VBAT | 32-bit | unlimited | Requires coin cell on VBAT |
| L5 | OTP fuses (64× 32-bit words) | 64×32 bits | 1× per OTP word | Each word blown once |
| H7 | OTP fuses | 64×32 bits | 1× per OTP word | — |
| U5 | OTP fuses | 128×32 bits | 1× per OTP word | — |

For OTP-based implementations, the counter uses a bitmap scheme:
- Each OTP bit represents one version number
- Version N = 1 bit blown. Version N+1 = same bit (unused) + next bit blown
- When all bits consumed → end-of-life (device cannot accept further updates)

For F411 (RTC backup registers):
- 32-bit backup register holds the counter
- VBAT pin powers the RTC domain when main power is off
- Counter incremented by writing a new value
- Protected by RTC write protection sequence

### Boot Flow

```
1. Read hardware counter → stored_version
2. Parse firmware TLV header → firmware_version
3. If firmware_version <= stored_version:
       return CounterError::RollbackAttempted
4. Increment hardware counter → stored_version = firmware_version
5. Continue boot
```

The counter is incremented ONLY after successful cryptographic verification of
the new firmware. This ensures:
- If verification fails, the counter is NOT incremented
- If verification succeeds, the counter is advanced to prevent re-installation
- Partial upgrades (power loss after counter increment but before flash write)
  are handled by the A/B swap state machine

### Interface

```rust
pub trait MonotonicCounter {
    /// Read current monotonic counter value.
    /// Returns None if storage is uninitialized (first boot).
    fn current() -> Result<Option<u64>, CounterError>;

    /// Atomically increment counter to at least `target`.
    /// If counter >= target, this is a no-op.
    /// Verified: counter never decreases.
    fn increment_to(target: u64) -> Result<(), CounterError>;

    /// Returns true if counter has reached end-of-life
    /// (all OTP bits consumed or backup register at max).
    fn is_exhausted() -> bool;
}
```

## Verification

### Testing Requirements

- Read counter → increment → read again → assert increased
- Increment twice → assert second value > first
- Attempt to decrement → assert error (compile-time or runtime)
- Power-loss simulation: increment → restore counter → assert idempotent
- Exhausted counter: all OTP bits blown → `increment_to()` returns `Exhausted`

### Kani Proofs

| Harness | Property |
|---------|----------|
| `counter_monotonic_invariant` | After any sequence of increment operations, counter never decreases |
| `increment_to_target_bounds` | `increment_to(t)` always results in `current() >= t` |
| `otp_bitmap_no_double_blow` | Each OTP bit is only blown once |
| `counter_arithmetic_no_overflow` | `increment_to(u64::MAX)` returns `Exhausted`, not wrap |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Monotonic invariant induction | Verus | For any sequence of `increment_to()` calls, `current()` is monotonic |
| Bitmap allocation soundness | Creusot | OTP word is blown exactly once per version, never reused |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `otp_counters_arbitrary_bits` | Random bit patterns in OTP words — verify never produces invalid state |

### Proptests

| Property | Check |
|----------|-------|
| `counter_increment_sequence` | Random sequence of increments never decreases value |
| `nondeterministic_counter_reads` | Concurrent/power-loss reads return valid state |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- F411 RTC backup registers are software-writable with the right unlock
  sequence. A compromised bootloader or TrustZone bypass could increment the
  counter. Mitigation: RDP Level 2 lock prevents debug access; TrustZone on
  L5/U5 restricts register access to secure world only.
- OTP is one-shot. When all bits are consumed, the device cannot accept
  security updates at all. Mitigation: careful provisioning of OTP word count
  based on expected device lifetime and update frequency.