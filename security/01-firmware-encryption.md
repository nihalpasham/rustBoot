---
title: Firmware Encryption at Rest (AES-256-GCM)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - FIPS-140-3-Level-3
  - NSA-CNSA-2.0
  - IETF-SUIT-RFC9019
bootloader: true
firmware: false
hardware: false
---

# Firmware Encryption at Rest (AES-256-GCM)

## Summary

Encrypt all firmware images stored in flash using AES-256-GCM authenticated
encryption. Decrypt on every boot before verification and execution. The
encryption key is derived from the device-unique STM32 UID (96-bit) combined
with a per-device secret provisioned in OTP. GCM mode provides both
confidentiality and integrity (authentication tag).

## Motivation

Without encryption, anyone with physical access can desolder the flash chip and
read the entire firmware binary via SPI/NOR interface. This exposes:
- Proprietary algorithms and business logic
- Cryptographic key material (even if embedded)
- Vulnerability discovery surface for targeted attacks
- Reverse-engineering for clone production

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| CSP encrypted outside crypto boundary | FIPS 140-3 §7.9 | Encrypted ciphertext in flash; key in CPU registers only |
| AES-256 required | CNSA 2.0 | AES-256-GCM using `aes` crate (software) or STM32 CRYP (hardware) |
| Optional confidentiality for SUIT | RFC 9019 §7 | GCM provides authenticated encryption |
| Nonce management per device | CNSA 2.0 | Nonce = device UID + monotonic counter, never reused |

## Design

### Key Derivation

```
OTP_Secret (32 bytes, burned per-device at factory)
         │
         ▼
    HMAC-SHA256(key = OTP_Secret, data = UID_96_bits)
         │
         ▼
    Firmware_Encryption_Key (32 bytes, AES-256)
         │
         ▼
    Key lives in CPU registers ONLY during decryption.
    Zeroized after each firmware block is decrypted.
```

Never stored in flash. Never stored in RAM between boots. Derived fresh on
each boot from UID + OTP secret.

### Firmware Image Layout (Encrypted)

```
┌──────────────────────────────────────────────┐
│             TLV Image Header                  │
│  (magic, version, type, hash, signature)      │
├──────────────────────────────────────────────┤
│              GCM Nonce (12 bytes)              │
├──────────────────────────────────────────────┤
│         AES-256-GCM Ciphertext                │
│         (encrypted firmware binary)            │
├──────────────────────────────────────────────┤
│          GCM Authentication Tag (16 bytes)     │
└──────────────────────────────────────────────┘
```

The signing tool (rbsigner) encrypts before signing:
1. Generate random 12-byte nonce
2. Encrypt firmware with AES-256-GCM (key from per-device UID or broadcast key)
3. Compute SHA-384 over ciphertext + nonce + tag
4. Sign with ECDSA (and LMS/XMSS for PQ) over the hash
5. Assemble TLV header + nonce + ciphertext + tag

### Boot Load Sequence

1. Read TLV header, extract nonce + ciphertext + tag
2. Derive encryption key from UID + OTP secret
3. Decrypt in-place: `aes_gcm_decrypt(ciphertext, nonce, key, tag)`
4. If tag mismatch → `CryptoError::IntegrityFailure` → fail-secure
5. Verify SHA-384 hash of plaintext
6. Verify ECDSA + LMS/XMSS signatures
7. Execute

## Interface

```rust
pub trait FirmwareEncryption {
    /// Decrypt firmware in-place. Nonce is stored in TLV header.
    /// Key is derived internally from device UID + OTP secret.
    fn decrypt_firmware(
        ciphertext: &mut [u8],
        nonce: &[u8; 12],
        tag: &[u8; 16],
    ) -> Result<(), CryptoError>;
}
```

## Implementation

### STM32F411 (Software AES)

- Use `aes` crate with `aes-gcm` crate (no_std compatible)
- CMSIS-DSP or bit-sliced AES for constant-time
- Key derived via HMAC-SHA256 using `sha2` crate

### STM32L5/H7/U5 (Hardware CRYP)

- STM32 CRYP peripheral supports AES-GCM natively
- DMA transfer for zero-copy decryption
- Key loaded into CRYP_KEYRx registers, never exposed to CPU memory
- HAL trait `CryptoEngine` wraps CRYP registers

## Verification

### Testing Requirements

- Encrypt known plaintext → decrypt → assert roundtrip
- Encrypt → corrupt ciphertext byte → assert tag mismatch
- Encrypt → corrupt nonce → assert tag mismatch
- Encrypt → decrypt with wrong key → assert failure
- Zero-length plaintext
- Maximum partition-size plaintext (bounds check)

### Kani Proofs

| Harness | Property |
|---------|----------|
| `encrypt_decrypt_roundtrip_bounded` | `decrypt(encrypt(p)) == p` for all `p` up to 256 bytes |
| `nonce_never_zero` | Derived nonce is never all-zeros |
| `tag_verification_rejects_corruption` | Any single-bit corruption in ciphertext → tag mismatch |
| `key_derivation_output_bounds` | Derived key fits in 32-byte buffer |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Key derivation correctness | Creusot | HMAC-SHA256 produces correct length output |
| GCM decryption functional correctness | Verus | Decryption reverses encryption (algebraic) |
| Constant-time key comparison | Creusot | No branching on secret data during tag verification |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `encrypted_image_header_parser` | Arbitrary bytes: TLV + nonce + ciphertext + tag |

### Proptests

| Property | Check |
|----------|-------|
| `encrypt_decrypt_roundtrip_any_key` | For any valid key, roundtrip succeeds |
| `ciphertext_stochastic_rejection` | Random ciphertext always returns error |
| `key_derivation_deterministic` | Same UID + secret → same key (required for update) |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- Device UID is read from STM32 registers — visible to any code running on the
  device. The OTP secret mitigates this: UID alone cannot derive the key.
- Software AES (F411) is slower than hardware. Benchmark to ensure boot time
  remains acceptable.
- GCM nonce reuse is catastrophic. Nonce = random per-image, verified at
  decryption time. Signing tool must never reuse a nonce for the same key.
- Key derivation is OTP secret + UID. If both are extracted simultaneously
  (invasive probe of OTP + UID registers), the key is compromised.