---
title: Post-Quantum Firmware Signing (LMS/XMSS)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - NSA-CNSA-2.0
  - NIST-SP-800-208
  - IETF-SUIT-RFC9019
bootloader: true
firmware: false
hardware: false
---

# Post-Quantum Firmware Signing (LMS/XMSS)

## Summary

Add hash-based post-quantum signature verification (LMS and XMSS per NIST SP
800-208) to the boot authentication chain. The bootloader verifies an LMS or
XMSS signature over the firmware image in addition to the existing ECDSA P-256
signature. Dual-signing allows migration: devices can be updated to drop ECDSA
once CNSA 2.0 mandates exclusive PQ use (2030).

## Motivation

ECDSA P-256 and all elliptic-curve cryptography will be broken by a
cryptanalytically-relevant quantum computer (CRQC). NSA CNSA 2.0 mandates
LMS or XMSS for firmware signing as the only approved quantum-resistant
algorithms for this use case. Timeline:
- 2025: support and prefer CNSA 2.0
- 2030: exclusively use CNSA 2.0

The bootloader's root of trust is the hardest component to upgrade. By adding
PQ signatures now, we future-proof devices with a 10-15 year lifespan.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| PQ signatures for firmware | CNSA 2.0, NIST SP 800-208 | LMS with SHA-256/192 or XMSS |
| Signature verification on device | CNSA 2.0 | CAVP-validated verify implementation |
| Support multiple signature algorithms | RFC 9019 §7 | Dual ECDSA + LMS/XMSS |
| LMS approval for all classification levels | CNSA 2.0 Table I | All LMS parameter sets approved |

## Design

### Algorithm Selection

| Algorithm | Parameters | Signature Size | Security Level | Boot ROM Space |
|-----------|-----------|---------------|----------------|----------------|
| LMS | SHA-256/192, H=5, W=1 | ~8 KB | 192-bit | Recommended by NSA |
| LMS | SHA-256/192, H=10, W=8 | ~24 KB | 192-bit | More signatures per key |
| XMSS | SHA-512, H=10, W=16 | ~2.5 KB | 256-bit | Smaller signatures |
| XMSS | SHA-512, H=20, W=16 | ~5 KB | 256-bit | More signatures |

Recommended: LMS SHA-256/192 H=5 W=1 (NSA recommended, smallest code
footprint, sufficient for firmware signing with key rotation).

### Dual-Signing Format

```
TLV Header:
  Tag: LMS_SIGNATURE
  Length: N
  Value: [LMS signature bytes]

  Tag: ECDSA_SIGNATURE
  Length: N
  Value: [ECDSA P-256 signature bytes]

  Tag: PUBLIC_KEY_DIGEST
  Length: 32
  Value: [SHA-256 of LMS public key hash]
```

The bootloader verifies both signatures independently:
- If LMS verifies AND ECDSA verifies → boot (transition phase)
- If LMS verifies AND ECDSA fails → boot (PQ migration, ECDSA deprecated)
- If LMS fails AND ECDSA verifies → boot with warning (ECDSA-only fallback)
- If both fail → reject

This transition policy is signed into the bootloader itself and can be updated
through the normal update mechanism (PQ-signed, of course).

### LMS State Management

LMS/XMSS are stateful signatures: the signer must track how many signatures
have been produced with each key. On the device, verification is stateless
(no tracking needed). The signer (rbsigner tool) must:
1. Maintain a persistent counter per LMS key
2. Never reuse a key after its signature limit is reached
3. Implement key rotation through the SUIT manifest

This is enforced in the signing infrastructure, not in the bootloader.

## Interface

```rust
pub trait PostQuantumVerify {
    /// Verify an LMS signature over `data` using the provisioned LMS public key.
    fn verify_lms(data: &[u8], signature: &[u8]) -> Result<(), CryptoError>;

    /// Verify an XMSS signature over `data` using the provisioned XMSS public key.
    fn verify_xmss(data: &[u8], signature: &[u8]) -> Result<(), CryptoError>;

    /// Verify both PQ and ECDSA. Returns policy-appropriate result.
    fn dual_verify(
        data: &[u8],
        ecdsa_sig: &[u8],
        pq_sig: &[u8],
        policy: VerifyPolicy,
    ) -> Result<(), CryptoError>;
}

pub enum VerifyPolicy {
    /// Both must pass
    Strict,
    /// ECDSA is authoritative, PQ is advisory
    EcdsaPrimary,
    /// PQ is authoritative, ECDSA is advisory
    PqPrimary,
    /// PQ only (2030+)
    PqOnly,
}
```

## Verification

### Testing Requirements

- LMS verify of known-good signature → Ok
- LMS verify with corrupted signature → Err
- LMS verify with wrong key → Err
- LMS verify with truncated signature → Err (bounds)
- XMSS: same as LMS (all test types)
- Dual verify with all policy variants
- Transition policy: test EcdsaPrimary, PqPrimary, PqOnly, Strict
- LMS public key import from raw bytes

### Kani Proofs

| Harness | Property |
|---------|----------|
| `lms_signature_bounds` | Signature buffer accesses are within bounds for all parameter sets |
| `xmss_signature_bounds` | Signature buffer accesses are within bounds for all parameter sets |
| `verify_rejects_truncated` | Any input shorter than minimum signature size returns `Err` |
| `dual_verify_policy_exhaustive` | All VerifyPolicy variants are handled without panic |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| LMS Merkle tree verification | Verus | Proof that verify algorithm correctly reconstructs the Merkle root |
| XMSS L-tree construction | Verus | Correctness of WOTS+ chain computation |
| Dual-verify policy coverage | Creusot | All 4 policy variants produce a deterministic result |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `lms_signature_parser` | Arbitrary bytes as LMS signature |
| `xmss_signature_parser` | Arbitrary bytes as XMSS signature |
| `dual_verify_policy` | Combinations of valid/invalid signatures with policy variants |

### Proptests

| Property | Check |
|----------|-------|
| `lms_verify_deterministic` | Same signature + data always produces same verify result |
| `dual_verify_respects_policy` | For each policy, the result matches expected logic |
| `signature_size_bounds` | All signature sizes are within documented limits |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- LMS/XMSS signatures are large (5–25 KB vs 64 bytes for ECDSA). Embedded
  flash must provision enough space. Mitigation: typical LMS H=5 W=1 signature
  ~8 KB fits within a single flash page on all STM32 targets.
- LMS/XMSS must be CAVP-validated for CNSA 2.0 compliance. The Rust
  implementation must undergo CAVP testing. No CAVP-validated Rust LMS/XMSS
  library exists yet — this is a blocker.
- State management is critical on the signer side. A state rollback in the
  signing infrastructure would allow signature forgery. Mitigation: HSM-backed
  signing with hardware state management.