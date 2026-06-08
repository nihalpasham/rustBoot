---
title: rustBoot Security Specification
status: draft
date: 2026-06-08
version: 1.0.0
standards:
  - FIPS-140-3-Level-3
  - Common-Criteria-EAL5+
  - NIST-SP-800-193
  - NSA-CNSA-2.0
  - IETF-SUIT-RFC9019
  - NIST-SP-800-208
---

# rustBoot Security Specification

## Scope

Military-grade secure boot for ARM Cortex-M microcontrollers (STM32F411, STM32L5,
STM32H7, STM32U5) with post-quantum readiness, hardware-backed root of trust,
and formal verification (Kani, Creusot, Verus, proptest, cargo-fuzz).

## Target Standards

| Standard | Scope | Level Required |
|----------|-------|----------------|
| FIPS 140-3 | Cryptographic module | Level 3 |
| Common Criteria | Secure boot + SoC PP | EAL5+ (AVA_VAN.5) |
| NIST SP 800-193 | Platform firmware resiliency | Protection + Detection + Recovery |
| NSA CNSA 2.0 | National Security Systems | LMS/XMSS + AES-256 + SHA-384/512 |
| NIST SP 800-208 | Post-quantum signatures | LMS or XMSS |
| IETF SUIT RFC 9019 | Firmware update manifest | Authentication + integrity + optional confidentiality |
| BSI-CC-PP-0084-2014 | Secure Sub-System in SoC | EAL5+ augmented |

## Security Measures Index

### Phase 1 — Critical (Must Implement)

| # | Measure | Priority | Standards | BL | FW | HW | Status |
|---|---------|----------|-----------|----|----|----|--------|
| 01 | Firmware Encryption at Rest (AES-256-GCM) | critical | FIPS-140-3, CNSA-2.0, SUIT | yes | no | no | draft |
| 02 | Hardware-Backed Anti-Rollback (OTP Counter) | critical | NIST-800-193, EAL5+, CNSA-2.0 | yes | no | yes | draft |
| 03 | Cryptographic Self-Tests (POST KATs) | critical | FIPS-140-3 | yes | no | no | draft |
| 04 | Debug Port Lockdown (JTAG/SWD) | critical | EAL5+, NIST-800-193 | yes | no | yes | draft |
| 05 | Post-Quantum Firmware Signing (LMS/XMSS) | critical | CNSA-2.0, NIST-800-208 | yes | no | no | draft |
| 06 | Measured Boot Chain (SHA-384) | critical | NIST-800-193, EAL5+, SUIT | yes | shared | no | draft |
| 07 | Fail-Secure Behavior & Recovery | critical | NIST-800-193, EAL5+ | yes | no | no | draft |
| 08 | Cryptographic Key Management | critical | FIPS-140-3, EAL5+, CNSA-2.0 | yes | no | yes | draft |
| 09 | Independent Watchdog (IWDG) Protection | critical | NIST-800-193 | yes | shared | yes | draft |
| 10 | Continuous-Power Attack Defense | critical | NIST-800-193, EAL5+ | yes | no | yes | draft |

### Phase 2 — Hardware-Backed (MCU-Dependent)

| # | Measure | Priority | Standards | BL | FW | HW | Status |
|---|---------|----------|-----------|----|----|----|--------|
| 11 | Constant-Time & Side-Channel Resistance | high | FIPS-140-3, EAL5+ | shared | shared | no | draft |
| 12 | Volatile Key Material & Zeroization | high | FIPS-140-3 | yes | shared | no | draft |
| 13 | Runtime Integrity Verification | high | NIST-800-193 | yes | shared | no | draft |
| 14 | Active Tamper Detection (TAMP) | high | FIPS-140-3, EAL5+ | yes | no | yes | draft |
| 15 | TrustZone Isolation Architecture | high | NIST-800-193, EAL5+ | yes | shared | yes | draft |
| 16 | DICE / Hardware Root of Trust | high | EAL5+, SUIT | yes | no | yes | draft |

### Phase 3 — Advanced (Optional)

| # | Measure | Priority | Standards | BL | FW | HW | Status |
|---|---------|----------|-----------|----|----|----|--------|
| 17 | White-Box Cryptography Strategy | medium | FIPS-140-3 | yes | no | no | draft |
| 18 | Honeypot / Decoy Partitions | medium | NIST-800-193 | yes | no | no | draft |
| 19 | Remote Attestation | medium | SUIT, RATS | yes | yes | no | draft |
| 20 | Control-Flow Integrity & Stack Protection | medium | EAL5+ | shared | shared | no | draft |
| 21 | Fault Injection Countermeasures | medium | EAL5+, FIPS-140-3 | shared | shared | yes | draft |
| 22 | SUIT Manifest Support (CBOR/COSE) | medium | IETF-RFC9019 | yes | no | no | draft |
| 23 | Formal Certification (EAL5+/FIPS 140-3) | medium | — | process | process | process | draft |
| 24 | Vulnerability Analysis & Penetration Testing | medium | EAL5+, FIPS-140-3 | process | process | process | draft |
| 25 | Supply Chain Security & Provisioning | medium | EAL5+, NIST-800-193 | process | process | process | draft |

### Support Files

| # | File | Purpose |
|---|------|---------|
| — | references/standards-matrix.md | Full mapping: measure -> standard -> SFR -> test |
| — | references/threat-model-update.md | Updated STRIDE with new attack vectors |

## Legend

- **BL**: Implemented in bootloader (immutable, runs before firmware)
- **FW**: Implemented in firmware/application
- **shared**: Split responsibility between bootloader and firmware
- **HW**: Requires hardware feature (MCU peripheral, OTP, fuse, etc.)
- **process**: Not code — requires procedural changes, lab engagement

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    Immutable Boot ROM                      │
│  (First-stage bootloader, fused in silicon by STM32)       │
└────────────────────┬─────────────────────────────────────┘
                     │ measures + verifies
┌────────────────────▼──────────────────────────────────────┐
│                   rustBoot Bootloader                      │
│  ┌────────────┐  ┌───────────┐  ┌──────────────────────┐  │
│  │ Crypto POST │  │ IWDG Lock │  │ Decrypt + Verify +  │  │
│  │ (KATs)     │  │           │  │ Measured Boot Chain  │  │
│  └────────────┘  └───────────┘  └──────────────────────┘  │
│                                                           │
│  ┌────────────┐  ┌───────────┐  ┌──────────────────────┐  │
│  │ Anti-      │  │ Debug     │  │ Fail-Secure /        │  │
│  │ Rollback   │  │ Lockdown  │  │ Rollback             │  │
│  └────────────┘  └───────────┘  └──────────────────────┘  │
└────────────────────┬─────────────────────────────────────┘
                     │ boots authenticated/decrypted
┌────────────────────▼──────────────────────────────────────┐
│             Application Firmware (OS/Tasks)                │
│  ┌────────────┐  ┌───────────┐  ┌──────────────────────┐  │
│  │ IWDG Kick  │  │ Runtime   │  │ Remote Attestation   │  │
│  │            │  │ Integrity │  │ (optional)           │  │
│  └────────────┘  └───────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Trait Architecture

```rust
// Each security measure maps to one or more traits.
// Sub-traits are independently verifiable (Kani, Creusot, Verus)

pub trait CryptoEngine {
    fn decrypt(data: &mut [u8], nonce: &[u8; 12]) -> Result<(), CryptoError>;
    fn verify(data: &[u8], sig: &[u8], pk: &[u8]) -> Result<(), CryptoError>;
    fn hash(data: &[u8], out: &mut [u8; 64]) -> Result<(), CryptoError>;
    fn post_kat() -> Result<(), CryptoError>;
}

pub trait MonotonicCounter {
    fn current() -> Result<u64, CounterError>;
    fn increment() -> Result<(), CounterError>;
}

pub trait DebugAuth {
    fn is_locked() -> bool;
    fn lock() -> Result<(), DebugAuthError>;
}

pub trait TamperDetector {
    fn is_tampered() -> bool;
    fn zeroize() -> Result<(), TamperError>;
}

pub trait MeasuredBoot {
    fn extend(stage: BootStage, data: &[u8]) -> Result<(), MBError>;
    fn attest(nonce: &[u8], out: &mut [u8]) -> Result<usize, AttestError>;
}

pub trait Watchdog {
    fn lock(period_ms: u32) -> Result<(), WdtError>;
    fn kick() -> Result<(), WdtError>;
    /// Return remaining time before watchdog expiry
    fn remaining_ms() -> u32;
}
```

## Per-MCU Capability Matrix

| Feature | F411 (M4F) | L5 (M33+TZ) | H7 (M7) | U5 (M33+TZ) |
|---------|------------|-------------|---------|-------------|
| AES-256-GCM | software (aes crate) | HW CRYP | HW CRYP | HW CRYP |
| Anti-rollback | RTC backup reg | OTP fuses | OTP fuses | OTP fuses |
| Crypto self-test | software | HW + SW | HW + SW | HW + SW |
| Debug lockdown | option bytes | DBG_AUTH HW | DBG_AUTH HW | DBG_AUTH+ |
| IWDG | yes | yes | yes | yes |
| TrustZone | no | yes | no | yes |
| Tamper detection | SW only | TAMP HW | SW only | TAMP HW+ |
| PQ signatures | software | software | software | software |
| RNG | RNG peri | TRNG | RNG peri | TRNG |
| Unique ID | 96-bit UID | 96-bit UID | 96-bit UID | 96-bit UID |

## Verification Methodology

Every security measure requires:

- **Unit tests**: Functional correctness, edge cases, error paths
- **Kani proofs**: Bounded verification of buffer bounds, arithmetic, state transitions
- **Creusot/Verus proofs**: Deductive verification of key derivation, constant-time, invariant preservation
- **Property tests (proptest)**: Parser roundtrip, state machine invariants, never-panics
- **Fuzz targets (cargo-fuzz)**: Arbitrary-byte parser hardening
- **MC/DC coverage**: >80% line coverage, full branch coverage on critical logic