---
title: Cryptographic Self-Tests (POST KATs)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - FIPS-140-3-Level-3
  - Common-Criteria-EAL5+
bootloader: true
firmware: false
hardware: false
---

# Cryptographic Self-Tests (POST KATs)

## Summary

On every boot, the bootloader runs Known Answer Tests (KATs) against all
cryptographic primitives before using them to verify firmware. If any KAT
fails, the bootloader halts with a diagnostic code and enters a safe failure
state (cannot boot any firmware, even a known-good one).

## Motivation

Cryptographic implementations can fail due to:
- Hardware faults (bit flips in flash/cache/registers from radiation or aging)
- Fault injection attacks (voltage/clock glitching to skip crypto operations)
- Memory corruption of algorithm lookup tables
- Timing attacks that induce wraparound in loop counters

KATs detect all of these before a single byte of firmware is decrypted. This
is a fundamental requirement of FIPS 140-3 certification.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Power-on self-tests (POST) for all algorithms | FIPS 140-3 §7.10.2 | KATs run before any crypto operation |
| Conditional self-tests on API call | FIPS 140-3 §7.10.3 | Post-verify KAT on signature verification |
| Fail-secure on KAT failure | FIPS 140-3 §7.10.1 | Halt with diagnostic; cannot boot any image |
| Self-test of all cryptographic functions | EAL5+ (FPT_TST.1) | AES-GCM, SHA-512, ECDSA, LMS/XMSS KATs |

## Design

### Algorithm Test Vectors

Each KAT is a known-answer test: fixed input → known expected output. The test
vector is embedded in the bootloader binary.

| Primitive | KAT Type | Vector Source |
|-----------|----------|---------------|
| AES-256-GCM | Encrypt KAT + Decrypt KAT | NIST CAVP AES-GCM sample vectors |
| SHA-512 | Hash KAT (3 vectors: empty, short, long) | NIST CAVP SHA-512 sample vectors |
| ECDSA P-256 | Sign KAT (fixed key, fixed message → verify) | NIST CAVP ECDSA sample vectors |
| LMS (SHA-256/192) | Verify KAT | NIST SP 800-208 sample |
| XMSS (SHA-512) | Verify KAT | NIST SP 800-208 sample |
| HMAC-SHA256 | MAC KAT | NIST CAVP HMAC sample vectors |

### Boot Sequence

```
1. Reset
2. Hardware initialization (clock, flash, RAM)
3. IWDG lock (prevents disabling watchdog)
4. CRYPTO POST:
   a. AES-256-GCM KAT
   b. SHA-512 KAT
   c. ECDSA P-256 KAT
   d. LMS Verify KAT
   e. XMSS Verify KAT
   f. HMAC-SHA256 KAT
   g. RNG KAT (health test)
5. If any KAT fails:
   → blink diagnostic LED pattern (Morse code fault code)
   → enter infinite halt loop (safe failure)
   → IWDG will eventually reset; repeat POST (possible transient fault)
6. If all KATs pass:
   → proceed to decryption + verification
```

### Conditional Self-Tests

Some KATs run after specific operations (conditional self-tests per FIPS 140-3
§7.10.3):
- After each signature verification: re-run the signature algorithm KAT
- After RNG call: re-run the RNG health test
- Periodically: re-run AES-GCM encrypt/decrypt KAT

## Interface

```rust
pub trait CryptoPost {
    /// Run all power-on self-tests.
    /// Returns Ok(()) only if every KAT passes.
    fn run_post() -> Result<PostReport, PostError>;

    /// Run conditional self-test after cryptographic operation.
    fn conditional_kat(algorithm: AlgorithmId) -> Result<(), PostError>;
}

pub struct PostReport {
    pub kat_count: u8,
    pub passed: u8,
    pub failed: u8,
    pub first_failure: Option<AlgorithmId>,
}
```

## Verification

### Testing Requirements

- Each KAT: known input → known output → assert match
- Corrupt each KAT vector in flash → assert KAT failure
- Verify that POST halts on first failure (no silent continuation)
- Verify that conditional KAT runs after each signature verify call
- Timing: measure POST duration, assert ≤ N ms in WCET analysis

### Kani Proofs

| Harness | Property |
|---------|----------|
| `post_halts_on_first_failure` | If any KAT fails, `run_post()` returns `Err` before calling the next KAT |
| `kat_vectors_in_rodata` | All KAT vectors are in read-only memory (verified by linker section) |
| `post_bounded_execution` | POST completes within bounded number of instructions |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| SHA-512 KAT correctness | Verus | Hash of known input matches expected output (functional proof) |
| AES-GCM KAT correctness | Creusot | Encrypt/decrypt of KAT vectors produces correct ciphertext/tag |
| POST never writes to mutable globals | Creusot | All KAT state is stack-local |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `kat_vectors_arbitrary` | Random KAT vectors — verify POST rejects them all |

### Proptests

| Property | Check |
|----------|-------|
| `kat_always_deterministic` | Running POST twice on the same binary produces the same result |
| `conditional_kat_always_after_verify` | After every `verify()` call, conditional KAT state is correct |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- KAT vectors in flash could be corrupted by an attacker with physical access.
  Mitigation: KAT data is in the bootloader's read-only section. With RDP
  Level 2, flash reads are disabled. The KAT comparison itself is a
  constant-time compare to prevent timing-based bypass.
- POST adds boot time overhead (~5-50 ms depending on MCU speed and algorithm
  count). Acceptable for boot-time operation; verify with WCET analysis.