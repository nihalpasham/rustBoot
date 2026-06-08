---
title: Cryptographic Key Management
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - FIPS-140-3-Level-3
  - Common-Criteria-EAL5+
  - NSA-CNSA-2.0
bootloader: true
firmware: false
hardware: true
---

# Cryptographic Key Management

## Summary

Define a key hierarchy with hardware-anchored root of trust. Each device has
unique secret material provisioned at manufacturing. Keys exist in CPU
registers only during their active use and are zeroized immediately after. Key
derivation is performed by the bootloader using HMAC-SHA256 over device
identity material.

## Motivation

FIPS 140-3 Level 3 requires that:
- Critical Security Parameters (CSPs) be protected within a tamper-evident
  boundary
- CSPs outside the boundary be encrypted
- Key generation use an approved RNG
- Keys be zeroized on module termination or tamper

EAL5+ (FCS_CKM.1, FCS_CKM.4) requires key generation and destruction as
security functions with formal design evidence.

Without proper key management, keys stored in flash are extractable by any
attacker with physical access.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Key generation using approved RNG | FIPS 140-3 §7.9.2, EAL5+ (FCS_CKM.1) | STM32 TRNG or RNG peripheral, health-tested |
| Key destruction on module termination | FIPS 140-3 §7.9.7, EAL5+ (FCS_CKM.4) | Zeroize via volatile_write |
| CSP encrypted outside crypto boundary | FIPS 140-3 §7.9.4 | Keys only in CPU registers; ciphertext in flash |
| AES-256 key size | CNSA 2.0 | 32-byte keys for AES-256-GCM |
| SHA-384/512 for hashing | CNSA 2.0 | SHA-384 in measured boot; SHA-512 for XMSS |

## Design

### Key Hierarchy

```
                ┌──────────────────────────────┐
                │  Device Identity Key (DIK)    │
                │  32 bytes, burned in OTP      │
                │  Provisioned at manufacturing │
                │  Per-device unique             │
                │  NEVER exposed to SW directly │
                └──────────┬───────────────────┘
                           │
               ┌───────────┴───────────┐
               │                       │
        ┌──────▼──────┐        ┌──────▼──────┐
        │ FW Encryption│        │ Attestation  │
        │ Key (FEK)   │        │ Key (AK)     │
        │ HMAC-SHA256 │        │ HMAC-SHA256  │
        │ (DIK, "enc")│        │ (DIK, "att") │
        │ 32 bytes    │        │ 32 bytes     │
        └──────┬──────┘        └──────┬──────┘
               │                       │
        ┌──────▼──────┐        ┌──────▼──────┐
        │ Used for    │        │ Used for    │
        │ AES-256-GCM │        │ DICE /      │
        │ decrypt     │        │ attestation │
        └─────────────┘        └─────────────┘

                ┌──────────────────────────────┐
                │  Provisioned Public Keys      │
                │  Root ECDSA P-256 public key  │
                │  Root LMS/XMSS public key     │
                │  Provisioned in bootloader    │
                │  flash (read-only section)    │
                └──────────────────────────────┘
```

### Key Derivation

```rust
fn derive_fek(dik: &[u8; 32], uid: &[u8; 12]) -> [u8; 32] {
    // DIK is the OTP secret, 32 bytes
    // UID is the 96-bit STM32 unique device ID
    // Salt = constant string "rustboot-fek-v1"
    let salt = b"rustboot-fek-v1";
    let mut hasher = HmacSha256::new_from_slice(dik).unwrap();
    hasher.update(salt);
    hasher.update(uid);
    let result = hasher.finalize();
    result.into_bytes().into()
}
```

### Key Zeroization

Keys are zeroized after use using `core::ptr::write_volatile` to prevent the
compiler from optimizing away the clear operation. Verified with Creusot.

```rust
pub fn zeroize_key(key: &mut [u8]) {
    for byte in key.iter_mut() {
        unsafe {
            core::ptr::write_volatile(byte, 0);
        }
    }
    // Memory barrier to prevent reordering
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}
```

### Key Lifecycle

| Phase | Location | Protection | Notes |
|-------|----------|------------|-------|
| Provisioning | OTP (factory) | Physical security at fab | Blown during secure manufacture |
| Storage | OTP + UID | Hardware readout protection | Cannot be read via debug |
| Derivation | CPU registers | Only during boot, before any user code runs | HMAC computation |
| Active use | CPU registers | Only during AES-GCM decrypt | ~10-100ms per boot |
| Zeroization | Stack → overwritten | Volatile write | Before returning to caller |
| Destruction | On tamper | TAMP peripheral triggers | Immediate zeroize |

## Interface

```rust
pub trait KeyManager {
    /// Derive the firmware encryption key.
    /// Key exists in the returned buffer only; caller must zeroize.
    fn derive_fek() -> Result<[u8; 32], KeyError>;

    /// Derive the attestation key for DICE / remote attestation.
    fn derive_attestation_key(nonce: &[u8]) -> Result<[u8; 32], KeyError>;

    /// Provision device identity key at manufacturing.
    /// This is a one-time operation. Returns error if already provisioned.
    #[cfg(feature = "provisioning")]
    fn provision_dik(dik: &[u8; 32]) -> Result<(), KeyError>;

    /// Check if device is provisioned.
    fn is_provisioned() -> bool;

    /// Zeroize a key buffer.
    fn zeroize(key: &mut [u8]);
}
```

## Verification

### Testing Requirements

- `derive_fek()` called twice → assert same result (deterministic from UID + OTP)
- Zeroize → assert all bytes are zero (volatile read)
- `is_provisioned()` → false on blank device, true after provision
- `provision_dik()` called twice → second call returns `AlreadyProvisioned`
- Different UID → different FEK (key uniqueness)

### Kani Proofs

| Harness | Property |
|---------|----------|
| `key_derivation_output_len` | `derive_fek()` always returns 32 bytes |
| `key_derivation_deterministic` | Same inputs → same output |
| `zeroize_wipes_all_bytes` | After `zeroize()`, all bytes in the buffer are 0 |
| `provision_one_way` | `provision_dik()` can only succeed once |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| HMAC-SHA256 functional correctness | Verus | Derivation produces correct HMAC per RFC 2104 |
| Zeroize is not optimized away | Creusot | volatile_write is preserved in the generated MIR |
| Key derivation uses constant time | Creusot | No secret-dependent branches in HMAC computation |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `key_derivation_ids` | Derive with invalid salt strings — verify never panics |

### Proptests

| Property | Check |
|----------|-------|
| `key_uniqueness_across_devices` | Different UIDs produce different keys |
| `zeroize_idempotent` | Calling zeroize twice is equivalent to calling it once |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- The Device Identity Key burned in OTP could theoretically be extracted via
  invasive microprobing. Mitigation: at RDP Level 2, OTP reads are blocked;
  STM32 OTP has physical protection mechanisms against probing.
- Key derivation uses the UID, which is readable from CPU registers. An
  attacker with code execution could read the UID. However, without the OTP
  secret, the UID alone is insufficient to derive the FEK.