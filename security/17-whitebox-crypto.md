---
title: White-Box Cryptography Strategy
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - FIPS-140-3
bootloader: true
firmware: false
hardware: false
---

# White-Box Cryptography Strategy

## Summary

White-box cryptography embeds the cryptographic key into the algorithm
implementation such that the key is never present as plaintext. The algorithm
(AES) is implemented as a network of encoded lookup tables (Chow et al.
approach). Even an attacker with full code access and debug capabilities cannot
extract the key.

## Motivation

Standard AES stores the key in memory during operation. White-box AES
transforms the key into a series of obfuscated lookup tables. The tables
perform AES operations but are randomized and encoded so that reverse-
engineering the tables does not reveal the key.

This is particularly relevant for:
- Devices where an attacker has full code execution
- Obfuscated key storage when hardware key storage is unavailable
- Defense-in-depth on devices without TrustZone

## Design Approach

White-box AES is an advanced cryptographic technique with significant
complexity. The approach is:

1. **Evaluate existing white-box implementations**: No verified Rust
   implementation exists. Evaluate C implementations (e.g., Chow et al.,
   Karroumi) for portability.
2. **Implement as a research project**: White-box crypto must be analyzed for
   known attacks (BGE attack, algebraic attacks, side-channel analysis).
3. **Verification difficulty**: White-box AES is extremely difficult to
   formally verify because verification tools would expose the key.

### Architecture

```
Firmware Encryption Key (FEK)
    │
    ▼ White-Box Compilation (offline tool)
    ┌──────────────────────────────┐
    │ White-Box AES Tables         │
    │ 18 encoded lookup tables     │
    │ Key embedded in table values │
    │ Table randomization + mixing │
    │ External encoding on IO      │
    └──────────┬───────────────────┘
               │ (stored in bootloader flash as opaque data)
               ▼
    Decrypt using white-box implementation (no key input)

Result: No key in registers, no key in flash, no key anywhere.
Secret is the table construction, not a numeric key.
```

### Current Status

White-box cryptography is **not ready for production**. Known attacks exist
against most published white-box AES schemes. Only implementations with
proven security against BGE-type attacks and algebraic attacks should be
used. This is deferred until:

1. NIST standardizes a white-box primitive (none exists as of 2026)
2. A CAVP-validated white-box implementation exists for our targets
3. The threat model specifically requires key extraction resistance at this
   level

## Status

- **Implementation**: Not started (research-only)
- **Deferred until**: NIST standard, CAVP validation, or specific threat
  model requirement

## Residual Risk

- Most published white-box AES schemes have been broken. The BGE attack (2005)
  breaks Chow et al. with ~2^30 complexity. Only implementations with
  proven 128-bit security should be considered.
- No Rust ecosystem support exists. Would require a C FFI or custom
  implementation.
- Verification is extremely challenging: Kani/Creusot/Verus cannot easily
  reason about obfuscated lookup tables.