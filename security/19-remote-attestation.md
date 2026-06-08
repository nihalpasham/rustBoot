---
title: Remote Attestation
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - IETF-SUIT-RFC9019
  - RATS-Architecture
bootloader: true
firmware: true
hardware: false
---

# Remote Attestation

## Summary

The device produces a signed attestation report containing the measurement
log (IMR values from measured boot). A remote verifier can request and verify
this report to determine whether the device is in a trusted state (correct
firmware, correct boot chain, no tamper).

## Motivation

Remote attestation proves to a network operator that:
- The device booted the correct, signed firmware
- No tampering occurred during boot
- The device identity is authentic
- The device is running the expected software version

This is essential for:
- Network access control (only verified devices connect)
- Update compliance enforcement
- Intrusion detection (unexpected measurements = compromise)
- Supply chain verification

## Design

### Attestation Flow

```
Verifier                     Device
    │                          │
    ├── Request attestation ───┤
    │   (nonce = random 32B)   │
    │                          │
    │                     ┌────┴────┐
    │                     │ 1. Read IMRs   │
    │                     │ 2. Read measurement log │
    │                     │ 3. Sign with attestation key │
    │                     │ 4. Build report   │
    │                     └────┬────┘
    │                          │
    ├── Attestation report ────┤
    │   (report + signature)   │
    │                          │
    ├── Verify report:         │
    │  • DeviceID cert chain   │
    │  • Attestation key cert  │
    │  • Report signature      │
    │  • IMR values match known│
    │  • Nonce is fresh        │
    │  └── Trust decision      │
```

### Attestation Report Format

```rust
pub struct AttestationReport {
    /// Device identity certificate chain (DICE or provisioned)
    pub device_id_chain: DiceChain,

    /// Fresh nonce from verifier (prevents replay)
    pub nonce: [u8; 32],

    /// IMR values at time of request
    pub imr_values: [[u8; 48]; IMR_COUNT],

    /// Measurement log (ordered extend operations)
    pub measurement_log: MeasurementLog,

    /// Firmware version and security version
    pub firmware_version: u32,
    pub security_version: u32,

    /// Tampter flag
    pub tampered: bool,

    /// Signature over the above fields using attestation key
    pub signature: [u8; 64],
}
```

## Verification

### Testing Requirements

- Generate attestation report → verify signature → assert valid
- Corrupt IMR → regenerate → verify → assert mismatch
- Tampered device → tamper flag is set in report
- Nonce mismatch → verifier rejects

### Kani Proofs

| Harness | Property |
|---------|----------|
| `report_bounded_size` | Report size is bounded and fits in output buffer |
| `signature_includes_all_fields` | Signature covers all report fields (no omission) |

### Proptests

| Property | Check |
|----------|-------|
| `attestation_deterministic` | Same device state always produces same report (except nonce) |
| `nonce_prevents_replay` | Different nonces produce different signatures |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Proptests**: Not written

## Residual Risk

- Requires the device to have network connectivity (or a communication channel
  to the verifier). Offline devices cannot be remotely attested.
- The attestation key is derived from the device identity secret. If the
  identity secret is compromised, attestation cannot be trusted.