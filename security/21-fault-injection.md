---
title: Fault Injection Countermeasures
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - Common-Criteria-EAL5+
  - FIPS-140-3
bootloader: shared
firmware: shared
hardware: true
---

# Fault Injection Countermeasures

## Summary

Implement hardware and software countermeasures against fault injection
attacks (voltage glitching, clock glitching, electromagnetic pulses, laser
injection). These attacks aim to corrupt instruction execution, skip security
checks, or corrupt cryptographic computations.

## Motivation

Fault injection is a well-known attack vector against embedded systems:
- Voltage glitching: brief over/under-voltage causes instructions to be
  skipped or corrupted
- Clock glitching: extra clock cycles cause pipeline corruption
- EM injection: focused electromagnetic pulses induce bit flips
- Laser: focused light causes transistor state changes

A successful fault injection can bypass a security check (e.g., skip a
signature verification check) even with perfect software.

## Design

### Software Countermeasures

| Technique | Implementation | Overhead |
|-----------|---------------|----------|
| Redundant comparison | Each security check performed 3×, majority vote | ~3× time |
| Variable delay | Random delay before/after security checks | ~10-50% time |
| Double-check | Security-critical if-conditions checked twice | ~2× time |
| Trip-wire variables | Sentinel values between critical operations | ~1% memory |
| CRC on critical data | CRC-32 on state, config, and keys | ~1% time |

Example:

```rust
fn verify_signature(data: &[u8], sig: &[u8]) -> Result<(), Error> {
    // Trip-wire: check that start sentinel is intact
    if START_SENTINEL != SENTINEL_VALUE {
        return Err(Error::FaultDetected);
    }

    // Redundant verification (3× majority)
    let r1 = verify_ecdsa_inner(data, sig);
    let r2 = verify_ecdsa_inner(data, sig);
    let r3 = verify_ecdsa_inner(data, sig);

    // Majority vote
    let pass_count = [r1, r2, r3].iter()
        .filter(|r| r.is_ok())
        .count();

    // Trip-wire: check that end sentinel is intact
    if END_SENTINEL != SENTINEL_VALUE {
        return Err(Error::FaultDetected);
    }

    if pass_count >= 2 {
        Ok(())
    } else {
        Err(Error::SignatureInvalid)
    }
}
```

### Hardware Countermeasures

| MCU | Feature | What It Detects |
|-----|---------|-----------------|
| L5 | PVT (Power/Voltage/Temperature) sensor | Voltage out of safe range |
| L5/U5 | CSS (Clock Security System) | Clock frequency deviation |
| L5/U5 | TAMP | Physical tamper |
| All | IWDG | Code execution stall |
| All | CRC peripheral | Memory corruption |
| H7 | Dcache parity | Cache bit flips |

### Critical Code Sections

The following code sections are protected with redundant execution:
1. Signature verification in `verify_authenticity()`
2. Anti-rollback counter increment
3. Decryption key derivation
4. Mode transition in state machine
5. IWDG configuration
6. Any `if` condition that gates security-critical behavior

## Verification

### Testing Requirements

- Redundant comparison: inject fault (skip instruction) → majority catches
- CRC on state: corrupt state → CRC mismatch detected
- Sentinel check: corrupt sentinel → fault detected
- Each countermeasure is tested individually

### Kani Proofs

| Harness | Property |
|---------|----------|
| `redundant_check_majority` | 2/3 majority correctly handles single fault |
| `sentinel_values_integrity` | Sentinel values are never modified by normal code paths |
| `crc_detects_corruption` | Any single-bit corruption in CRC-protected data is detected |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Triple redundancy is sound | Verus | If any two of three checks pass, the result is correct |
| Trip-wires are invariant | Creusot | Sentinel values are never modified by normal execution |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `fault_injected_boot` | Inject single-bit faults at random boot stages — verify always detected |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- Software-only countermeasures can themselves be faulted. An attacker who
  can skip 3+ consecutive instructions can bypass triple redundancy.
- Hardware countermeasures (PVT, CSS) are available only on L5/U5. F411 and
  H7 must rely exclusively on software techniques.
- Fault injection requires adversary to have physical access and specialized
  equipment (~$1000-100,000 depending on technique). This raises the bar but
  does not eliminate the risk.