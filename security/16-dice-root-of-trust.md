---
title: DICE / Hardware Root of Trust
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - Common-Criteria-EAL5+
  - IETF-SUIT-RFC9019
bootloader: true
firmware: false
hardware: true
---

# DICE / Hardware Root of Trust

## Summary

Implement the TCG DICE (Device Identifier Composition Engine) architecture to
create a unique identity for each device. The DICE chain derives layered
certificates: from the immutable device secret (OTP), through the bootloader,
to the application firmware. Each stage cryptographically binds its
measurements into the next stage's certificate.

## Motivation

Without a hardware root of trust:
- All devices share the same signing key (if provisioned in bootloader flash)
- Compromising one device compromises all devices
- No unique device attestation possible
- Supply chain trust is binary (signed or not) rather than measurable

DICE provides:
- Unique per-device identity
- Measured boot evidence
- Device attestation without per-device key injection
- Certificate chain rooted in hardware

## Design

### DICE Layering

```
OTP Device Secret (UDS — Unique Device Secret)
    │  (32 bytes, provisioned per-device at manufacturing)
    │
    ▼ DICE layer 0 (immutable ROM / silicon)
    ┌──────────────────────────────┐
    │  DeviceID Cert (UDS-derived) │
    │  ├─ Public key K_0           │
    │  └─ Measurement = 0          │
    └──────────┬───────────────────┘
               │ Derives K_1 = HMAC(UDS, Measurement_0)
               ▼ DICE layer 1 (rustBoot)
    ┌──────────────────────────────┐
    │  Alias Cert (K_1-derived)    │
    │  ├─ Public key K_1           │
    │  ├─ Measurement = SHA384(rustBoot code)    │
    │  └─ Signed by K_0            │
    └──────────┬───────────────────┘
               │ Derives K_2 = HMAC(K_1, Measurement_1)
               ▼ DICE layer 2 (Application Firmware)
    ┌──────────────────────────────┐
    │  Alias Cert (K_2-derived)    │
    │  ├─ Public key K_2           │
    │  ├─ Measurement = SHA384(firmware + config) │
    │  └─ Signed by K_1            │
    └──────────────────────────────┘
```

Each layer:
1. Hashes its own code to produce a measurement
2. Uses that measurement to derive the next layer's key
3. Signs the next layer's certificate
4. Provides evidence of measurement to the next layer

### Device Identity

The DeviceID certificate (K_0) is the root of trust for the device. It:
- Is derived from the OTP Unique Device Secret (UDS)
- Is unique per device (different UDS → different K_0)
- Cannot be extracted: K_0 is derived and used, never stored
- Provides a verifiable device identity

### Attestation

Remote verifiers can:
1. Request the device's certificate chain (DeviceID + Alias certs)
2. Verify that each certificate is signed by the previous layer's key
3. Extract the measurement values
4. Compare measurements against known-good values
5. Determine if the device is in a trusted state

## Interface

```rust
pub trait DiceEngine {
    /// Get the device's DICE certificate chain.
    fn certificate_chain() -> Result<DiceChain, DiceError>;

    /// Get the device's unique identity (DeviceID public key).
    fn device_id() -> Result<[u8; 64], DiceError>;

    /// Derive the next DICE layer's key.
    fn derive_next_layer(measurement: &[u8; 64]) -> Result<[u8; 32], DiceError>;

    /// Sign the next layer's certificate.
    fn sign_next_cert(next_pubkey: &[u8; 64], measurement: &[u8; 64])
        -> Result<Vec<u8>, DiceError>;
}

pub struct DiceChain {
    pub device_id_cert: Certificate,
    pub alias_certs: [Certificate; MAX_LAYERS],
    pub layer_count: usize,
}
```

## Verification

### Testing Requirements

- Same UDS → same DeviceID key (deterministic derivation)
- Different UDS → different DeviceID (unique per device)
- Certificate chain verification: each cert signed by previous layer
- Corrupted measurement → next layer key changes → cert chain breaks
- Extraction resistance: UDS never appears in memory or in any output
- Signature of next layer cert verifies with current layer key

### Kani Proofs

| Harness | Property |
|---------|----------|
| `dice_derivation_bounded` | DICE key derivation completes within bounded operations |
| `dice_cert_chain_length` | Chain length is bounded by MAX_LAYERS |
| `measurement_deterministic` | Same stage code produces same measurement |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| DICE key derivation correctness | Verus | K_n = HMAC(K_{n-1}, Measurement_{n-1}) per TCG DICE spec |
| Certificate chain verifies | Creusot | Each cert signature validates against previous layer's key |
| UDS never leaks | Verus | UDS is used only in HMAC derivation, never stored or output |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `dice_measurement_injection` | Arbitrary measurement values into DICE chain |

### Proptests

| Property | Check |
|----------|-------|
| `dice_deterministic_identity` | Same device always produces same DeviceID |
| `dice_measurement_binds_identity` | Changing measurement changes derived keys |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- The UDS (Unique Device Secret) in OTP is the root of all device identity.
  If extracted via invasive probing, the attacker can forge that device's
  identity. Mitigation: OTP is designed to resist probing, protected by RDP
  Level 2.
- DICE requires factory provisioning infrastructure to burn UDS per device.
  This adds manufacturing cost and complexity. Without it, all devices share
  the same identity (a shared UDS defeats the purpose of DICE).