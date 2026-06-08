---
title: TrustZone Isolation Architecture
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - NIST-SP-800-193
  - Common-Criteria-EAL5+
bootloader: true
firmware: shared
hardware: true
---

# TrustZone Isolation Architecture

## Summary

Use ARM TrustZone-M (STM32L5/U5) to create a hardware-enforced secure world
for the bootloader and cryptographic operations. The application firmware runs
in the non-secure world and cannot access secure-world memory, registers, or
peripherals. Communication between worlds happens through a defined Secure
Gate (SGATE) API.

## Motivation

Software security is vulnerable to:
- Bugs in the application firmware that expose secure data
- DMA-capable peripherals reading secure memory
- Debug probes inspecting the entire address space
- Malicious firmware updates with backdoors

TrustZone creates a hardware boundary that is enforced by the CPU's bus
matrix. Even if the application firmware is completely compromised, secure
world data and code are inaccessible.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Isolation of security functions | NIST SP 800-193 §4.2.1.3 | TrustZone secure/non-secure split |
| Non-bypassability | NIST SP 800-193 §4.2.1.3 | Secure world cannot be entered from NS without SGATE |
| Least privilege | EAL5+ | Only necessary code in secure world |
| Secure vs normal world separation | EAL5+ (ADV_ARC) | GTSPEC + SAU + IDAU configuration |

## Design

### Memory Map

```
0x0000_0000 ┌──────────────────────────┐
            │  Secure Flash (bootloader) │ ← Secure world only
            │  Keys, crypto routines     │
            ├────────────────────────────┤
            │  Non-Secure Flash (firmware)│ ← Both worlds
            ├────────────────────────────┤
            │  Secure SRAM               │ ← Secure world only
            │  Keys, IMRs, FIQ stack     │
            ├────────────────────────────┤
            │  Non-Secure SRAM           │ ← Both worlds
            │  Application data          │
            ├────────────────────────────┤
            │  Secure Peripherals        │ ← Secure world only
            │  CRYP, HASH, TAMP, TRNG    │
            ├────────────────────────────┤
            │  Non-Secure Peripherals    │ ← Both worlds
            │  UART, SPI, GPIO, TIM      │
            └────────────────────────────┘
```

### Boot Sequence with TrustZone

```
1. ROM bootloader (immutable, secure)
2. Configures SAU (Security Attribution Unit) + IDAU
3. Boots rustBoot in Secure World
   ├─ All crypto operations in Secure World
   ├─ Key derivation, decrypt, verify
   ├─ Write measurement to Secure IMRs
   └─ Configure SGATE API
4. Configure GTSPEC (non-secure callable regions)
5. Switch to Non-Secure World
6. Boot application firmware (Non-Secure)
7. Application calls SGATE to request secure services
```

### SGATE API

The application firmware communicates with the secure world through a
defined set of Secure Gateway calls:

```rust
// Non-Secure Callable (NSC) region
// These functions are callable from Non-Secure world
// but execute in Secure world.

#[nonsecure_callable]
pub fn secure_hash(data: &[u8], out: &mut [u8; 64]) -> Result<(), SecureError> {
    // Execute in Secure world context
    // Validate input buffer (NS world could provide invalid pointers)
    // Compute SHA-512
    // Write to output buffer
}

#[nonsecure_callable]
pub fn secure_attest(nonce: &[u8; 32], out: &mut [u8]) -> Result<usize, SecureError> {
    // Read Secure IMRs
    // Sign measurement log with attestation key
    // Return signed report
}

#[nonsecure_callable]
pub fn secure_random(out: &mut [u8]) -> Result<(), SecureError> {
    // Use TRNG (secure peripheral)
    // Fill output buffer
}
```

### Secure World Contents

| Component | Why Secure | Size (Flash) |
|-----------|------------|--------------|
| Crypto key derivation | Protects device identity key | ~2 KB |
| AES-256-GCM decrypt | Protects encryption key | ~4 KB |
| LMS/XMSS verify | Protects public keys | ~8 KB |
| SHA-512 | Integrity verification | ~2 KB |
| Measurement registers (IMRs) | Tamper-proof boot evidence | ~0.5 KB |
| Key storage (temporary) | Zeroized after use | ~0.1 KB (SRAM) |
| Tamper handler | Cannot be disabled | ~0.5 KB |

## Verification

### Testing Requirements

- Secure world code can access secure memory → Ok
- Non-secure world code accessing secure memory → bus fault
- SGATE call: secure function executes correctly
- SGATE call with invalid pointer: returns error (no crash)
- Boot sequence: verify SAU/IDAU configured correctly
- TrustZone configuration: verify via debugger (if unlocked)

### Kani Proofs

| Harness | Property |
|---------|----------|
| `secure_memory_isolation` | Non-secure pointer to secure region is detected and rejected |
| `sgate_input_validation` | All SGATE functions validate NS-provided pointers before use |
| `secure_world_imr_not_writable_from_ns` | Secure IMRs cannot be modified from Non-Secure |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| TrustZone configuration correctness | Verus | SAU regions correctly partition memory |
| SGATE API correctness | Creusot | Each SGATE function validates inputs and does not leak secure data |
| Secure boot chain non-bypassable | Verus | No path exists from NS reset to firmware execution without passing through secure world |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `sgate_invalid_pointers` | Various invalid Non-Secure pointers to SGATE functions |

### Proptests

| Property | Check |
|----------|-------|
| `sgate_all_paths_validated` | Every SGATE function validates all pointer arguments before dereference |
| `secure_storage_not_leaked` | No SGATE function returns secure storage contents |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- TrustZone-M cannot protect against invasive physical attacks (microprobing,
  focused ion beam). Once an attacker gets past the package, all memory is
  accessible. Mitigation: RDP Level 2 + encrypted flash + tamper detection.
- Only L5 and U5 support TrustZone-M. F411 and H7 cannot use this isolation.
  For those MCUs, the bootloader runs in the same privilege level as
  everything else, and isolation relies on secure coding practices.