---
title: Control-Flow Integrity & Stack Protection
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - Common-Criteria-EAL5+
bootloader: shared
firmware: shared
hardware: false
---

# Control-Flow Integrity & Stack Protection

## Summary

Implement control-flow integrity (CFI) protections to detect and prevent
exploitation of memory corruption vulnerabilities. Stack canaries detect
buffer overflows. Forward-edge CFI prevents indirect call hijacking. Return
address protection prevents ROP/JOP attacks.

## Motivation

Even with all other protections, a memory safety bug in the bootloader or
firmware could allow an attacker to hijack control flow. CFI makes
exploitation significantly harder by ensuring that indirect branches only
target known-valid locations.

## Design

### Stack Canaries

Insert a random canary value at function entry and check it at function exit.

```rust
// Compiler-driven stack canary (enabled in rustc via -Z stack-protector)
// rustc does not yet have stable stack protector support.
// For now, manual canary check in critical functions:

fn verify_firmware(image: &[u8]) -> Result<(), Error> {
    let canary = read_random_canary();
    // ... function body ...
    if read_random_canary() != canary {
        // Stack corruption detected
        zeroize_keys();
        panic_with_diagnostic(StackCorruption);
    }
    Ok(())
}
```

### Forward-Edge CFI (Indirect Call Protection)

For indirect function calls through function pointers and trait objects:

```rust
// Use enum dispatch instead of trait objects for security-critical paths.
// This makes all call targets statically known.

pub enum CryptoDispatch {
    Software(SoftwareCrypto),
    Hardware(HardwareCrypto),
}

impl CryptoDispatch {
    fn decrypt(&self, data: &mut [u8]) -> Result<(), Error> {
        match self {
            CryptoDispatch::Software(c) => c.decrypt(data),
            CryptoDispatch::Hardware(c) => c.decrypt(data),
        }
    }
}
```

### Return Address Protection

Without hardware PAC (Pointer Authentication) support:
- Use `#[inline(never)]` on security-critical functions so return addresses
  are always on the stack (predictable layout)
- Arm M-profile PACBTI extension is available on Cortex-M85, not on M33/M4/M7
- Software shadow stack: maintain a separate copy of return addresses in a
  protected region

## Verification

### Testing Requirements

- Stack canary corruption → detection → correct error path
- Canary values are random across boots (different each reset)
- Indirect calls: verify dispatch table is read-only

### Kani Proofs

| Harness | Property |
|---------|----------|
| `canary_check_on_all_returns` | All security-critical functions have canary check |
| `dispatch_table_readonly` | Crypto dispatch table is in read-only memory |

### Proptests

| Property | Check |
|----------|-------|
| `canary_random_unique` | Canary values from consecutive boots are unique |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Proptests**: Not written

## Residual Risk

- Rust's memory safety guarantees reduce the need for CFI compared to C/C++.
  However, `unsafe` blocks and hardware interactions create risk.
- Software shadow stack adds ~10-20% overhead for function calls. Use only
  for security-critical functions.