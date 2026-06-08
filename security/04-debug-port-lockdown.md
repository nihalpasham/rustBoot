---
title: Debug Port Lockdown (JTAG/SWD)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - Common-Criteria-EAL5+
  - NIST-SP-800-193
bootloader: true
firmware: false
hardware: true
---

# Debug Port Lockdown (JTAG/SWD)

## Summary

Permanently disable the JTAG/SWD debug interface after provisioning. For MCUs
that support debug authentication (STM32L5, H7, U5), implement a signed-
certificate-based re-enable mechanism. For F411 (no HW debug authentication),
blast the RDP (Readout Protection) option bytes to the highest level, making
debug port activation irreversible.

## Motivation

The debug port is the single most powerful attack vector on any MCU:
- Read all flash contents (firmware, keys, configuration)
- Halt CPU, inspect registers, modify memory
- Bypass security checks by setting breakpoints
- Extract cryptographic keys by probing memory after decryption

Military-grade security requires that after provisioning, the debug port is
either permanently disabled or requires a cryptographically signed certificate
to re-enable.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Physical tamper resistance of debug interface | EAL5+ (FPT_PHP.3) | RDP Level 2 or DBG_AUTH with signed cert |
| Protection against unauthorized physical access | NIST SP 800-193 §4.1 | Debug interface disabled at boot |
| Identity-based authentication for debug | FIPS 140-3 Level 3 | Signed certificate authentication (L5/H7/U5) |

## Design

### STM32 Readout Protection Levels

| Level | Name | Debug Access | Impact |
|-------|------|-------------|--------|
| 0 | None | Full debug | Development only |
| 1 | Limited | Debug allowed but flash reads blocked | User code visible? No |
| 2 | No debug | Debug permanently disabled | Irreversible; mass erase also disabled |

For Phase 1, we use Level 2 on all MCUs. This is a one-time operation: once
RDP Level 2 is set, the debug port is dead forever. No firmware update can
re-enable it. The device is a black box.

### Debug Authentication (STM32L5/H7/U5 Option)

These MCUs support debug authentication: a challenge-response protocol using
ECDSA P-256 or LMS/XMSS signatures. To re-enable debug:
1. Device generates a random challenge
2. Responder (authorized party with HSM) signs the challenge
3. Device verifies signature against a provisioned root certificate
4. If valid, debug is temporarily re-enabled until next reset

This allows:
- Factory debugging before shipment (using factory key)
- Field failure analysis (returned units with known key)
- Secure disposal (mass erase after authentication)

## Interface

```rust
pub trait DebugAuth {
    /// Lock debug port permanently (RDP Level 2).
    /// After this call, only mass erase can unlock.
    /// Verified: this is a one-way transition.
    fn lock_permanently() -> Result<(), DebugAuthError>;

    /// Check current debug lock state.
    fn is_locked() -> bool;

    /// Attempt to authenticate with a signed certificate.
    /// Only available on MCUs with DBG_AUTH hardware (L5, H7, U5).
    #[cfg(feature = "debug_auth")]
    fn authenticate(
        challenge: &[u8; 32],
        signature: &[u8],
        certificate: &[u8],
    ) -> Result<(), DebugAuthError>;
}
```

### Boot Flow

```
1. Read RDP level from option bytes
2. If RDP Level < 2:
     a. Verify bootloader integrity
     b. Set RDP Level 2 (one-way)
     c. System reset
3. If RDP Level == 2:
     a. Proceed with normal boot
     b. (Debug interface is dead)
4. If DBG_AUTH challenge is pending (L5/H7/U5 only):
     a. Generate challenge
     b. Output on UART (if enabled)
     c. Wait for signed response
     d. Verify and authenticate or reject
```

## Verification

### Testing Requirements

- Lock → assert `is_locked() == true`
- Lock twice → second call returns `AlreadyLocked` error
- Attempt to set RDP Level 0 from firmware → returns error (option byte change requires reset)
- DBG_AUTH: correct cert → assert auth succeeds
- DBG_AUTH: wrong cert → assert auth fails
- DBG_AUTH: expired cert → assert auth fails
- Observation: after lock, SWD scan finds no devices

### Kani Proofs

| Harness | Property |
|---------|----------|
| `lock_is_one_way` | After `lock_permanently()`, any subsequent call returns `AlreadyLocked` |
| `is_locked_consistent` | `is_locked()` returns the same value within a boot cycle |
| `debug_auth_cert_chain_bounds` | Certificate parsing does not overflow buffers |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Lock state machine monotonic | Creusot | State transitions: Unlocked→Locked is irreversible |
| Auth cert signature verification soundness | Verus | Certificate verification is equivalent to ECDSA verify with known root |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `debug_auth_certificates` | Arbitrary bytes as debug auth challenge/response |

### Proptests

| Property | Check |
|----------|-------|
| `lock_monotonic` | Random sequence of lock/check operations never shows relock |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- RDP Level 2 is irreversible. A bug in the bootloader that sets Level 2
  prematurely would brick all devices. Mitigation: Level 2 is only set during
  factory provisioning, not on every boot. The bootloader checks but does not
  set Level 2 unless explicitly requested by a signed provisioning command.
- For F411, there is no debug authentication. If a device needs post-shipment
  debugging, it's impossible. Mitigation: use L5/U5 for applications requiring
  field debug capability.