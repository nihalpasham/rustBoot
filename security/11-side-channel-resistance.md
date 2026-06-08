---
title: Constant-Time & Side-Channel Resistance
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - FIPS-140-3-Level-3
  - Common-Criteria-EAL5+
bootloader: shared
firmware: shared
hardware: false
---

# Constant-Time & Side-Channel Resistance

## Summary

All cryptographic operations must execute in constant time relative to secret
data. No branch, memory access, or operation timing may depend on key material,
plaintext, or signature values. This defeats timing analysis, power analysis
(SPA/DPA), and electromagnetic side-channel attacks.

## Motivation

Side-channel attacks extract secrets by measuring physical characteristics of
computation. Even with perfect algorithms, timing variations in the
implementation leak information. FIPS 140-3 Level 3 requires non-invasive
security testing (Section 7.11). EAL5+ requires vulnerability analysis
including side-channel attack resistance.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Non-invasive security (side-channel) testing | FIPS 140-3 §7.11 | Constant-time implementation + associated data randomization |
| Covert channel analysis | EAL5+ (AVA_VAN.5) | Timing channel analysis documented |
| Mitigation of other attacks | FIPS 140-3 §7.11 | Countermeasures against SPA/DPA/EMA |

## Design

### Principles

1. **No secret-dependent branches**: `if secret_bit { ... } else { ... }` is
   forbidden. All branches must be data-independent or use constant-time
   selection (e.g., bit masking).
2. **No secret-dependent memory accesses**: `lookup_table[secret_index]` leaks
   the index via cache timing. Use constant-time table access or avoid tables.
3. **No secret-dependent loop bounds**: `for i in 0..secret_len` is forbidden.
4. **Fixed ALU operation patterns**: Multiplications and additions must be
   executed regardless of operand values.

### Code Review Checklist

Every cryptographic function must pass this checklist:

- [ ] No `if` statement with secret condition
- [ ] No `match` expression on secret value
- [ ] No `for`/`while` loop with secret upper bound
- [ ] No array index with secret value
- [ ] No ternary operator (`? :`) with secret condition
- [ ] All comparisons use `subtle::ConstantTimeEq` or equivalent
- [ ] All conditional operations use `subtle::ConditionallySelectable`
- [ ] All function execution time is independent of inputs
- [ ] Stack frame size is independent of inputs
- [ ] Loop iterations are fixed count for any input

### Verified Constant-Time Primitives

| Primitive | Constant-Time? | Verification |
|-----------|---------------|--------------|
| HMAC-SHA256 | Yes (via sha2 crate) | Relies on crate guarantees |
| AES-256-GCM (software) | Yes (aes crate, bitsliced) | Relies on crate guarantees |
| ECDSA verify (p256 crate) | Yes (masked feature) | Relies on crate guarantees |
| LMS verify (custom) | MUST VERIFY | Creusot constant-time proof |
| XMSS verify (custom) | MUST VERIFY | Creusot constant-time proof |
| Memory compare (tag verify) | Yes (custom ct_eq) | Custom implementation, Creusot-proofed |
| Key zeroization | Yes (volatile_write) | Creusot proof |

### Constant-Time Memory Compare

```rust
use subtle::ConstantTimeEq;

/// Compare two slices in constant time.
/// Returns true if equal, false otherwise.
/// Time is independent of content and position of first mismatch.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    // constant-time: always compare all bytes
    a.ct_eq(b).unwrap_u8() == 1
}
```

### Random Delay Insertion

To further obscure timing characteristics, insert random delays at critical
verification points:

```rust
fn verify_with_timing_obfuscation(data: &[u8], sig: &[u8]) -> Result<(), Error> {
    // Random delay before verification
    let delay_ticks = (trng_u32() % 100) + 50;
    spin_delay(delay_ticks);

    let result = verify_constant_time(data, sig);

    // Random delay after verification (independent of result)
    let delay_ticks = (trng_u32() % 100) + 50;
    spin_delay(delay_ticks);

    result
}
```

## Verification

### Testing Requirements

- `ct_eq(a, b)` returns true for identical slices
- `ct_eq(a, b)` returns false for different slices
- `ct_eq(a, b)` timing is within ±5 CPU cycles regardless of content
- All crypto operations: measure min/max timing over 10000 runs; assert
  max-min < threshold
- Verify that no `if` statements or `match` arms exist on secret data

### Kani Proofs

| Harness | Property |
|---------|----------|
| `ct_eq_constant_time` | Execution path is independent of comparison result |
| `ct_eq_all_bytes_read` | All bytes in both slices are read (no early exit) |
| `key_zeroization_volatile` | Zeroization uses volatile writes, verified by MIR |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| `ct_eq` has no secret branches | Creusot | Analysis of MIR shows no branch on comparison result |
| AES-GCM table access is constant-time | Verus | Lookup table indices are independent of key |
| Verify functions have no conditional returns | Creusot | All return paths are identical |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `constant_time_operations` | Random inputs to ct_eq — verify never panics |

### Proptests

| Property | Check |
|----------|-------|
| `ct_eq_correctness` | For any two arbitrary byte sequences, ct_eq matches byte-by-byte equality |
| `ct_eq_timing_stable` | Timing does not correlate with input content (statistical test) |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- The `aes`, `sha2`, and `p256` crates claim constant-time operation but are
  not CAVP-validated for our specific MCU targets. Side-channel resistance
  depends on the quality of these external implementations.
- Random delay insertion reduces correlation but does not eliminate all
  statistical side-channels. Full SPA/DPA resistance requires hardware
  countermeasures (ST M33 with random delay, etc.).