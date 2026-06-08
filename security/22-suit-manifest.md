---
title: SUIT Manifest Support (CBOR/COSE)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - IETF-SUIT-RFC9019
bootloader: true
firmware: false
hardware: false
---

# SUIT Manifest Support (CBOR/COSE)

## Summary

Implement support for the IETF SUIT (Software Updates for Internet of Things)
manifest format (RFC 9019) using CBOR encoding and COSE (CBOR Object Signing
and Encryption) for cryptographic protection. The SUIT manifest describes the
firmware update: what, where, how, and with what authorization.

## Motivation

SUIT is the emerging standard for IoT firmware updates. It provides:
- A standardized manifest format (CBOR-based, compact)
- Multiple signature support (multi-authorization)
- Dependency management (update must be applied in order)
- Conditional update commands (vendor/class/device-specific logic)
- Encryption and authentication in a single manifest

Adopting SUIT makes our update mechanism interoperable with standard tooling
and aligns with NATO/IETF recommendations.

## Design

### SUIT Manifest Structure

```
SUIT_Manifest = {
    1 : 0,                    // manifest version
    2 : vendor_id,            // RFC 4122 UUID
    3 : class_id,             // RFC 4122 UUID
    4 : payload_image,        // firmware image reference
    5 : install_sequence,     // CBOR commands to install
    6 : validate_sequence,    // CBOR commands to validate
    7 : text_manifest,        // human-readable description
    // Authentication wrapper (COSE Sign)
    8 : COSE_Sign_Tagged,     // signature over manifest
}
```

### Integration with Existing Format

The rustBoot TLV format can be extended to optionally wrap a SUIT manifest:

```
TLV Tag: SUIT_MANIFEST
    → Value: full SUIT/COSE manifest (CBOR binary)

TLV Tag: FIRMWARE_IMAGE
    → Value: AES-256-GCM encrypted firmware (same as now)
```

The bootloader:
1. Parses the SUIT manifest (CBOR)
2. Verifies the COSE signature
3. Reads the install sequence
4. Extracts the firmware dependency info
5. Decrypts and verifies the firmware

## Interface

```rust
pub trait SuitParser {
    /// Parse a SUIT manifest from CBOR bytes.
    fn parse_manifest(data: &[u8]) -> Result<SuitManifest, SuitError>;

    /// Verify COSE signature over the manifest.
    fn verify_manifest(manifest: &SuitManifest) -> Result<(), SuitError>;

    /// Check if this manifest applies to this device (vendor/class match).
    fn matches_device(manifest: &SuitManifest) -> bool;

    /// Get firmware payload reference from manifest.
    fn payload_reference(manifest: &SuitManifest) -> Result<FwRef, SuitError>;
}

pub struct SuitManifest {
    pub version: u8,
    pub vendor_id: uuid::Uuid,
    pub class_id: uuid::Uuid,
    pub payload: Option<FwRef>,
    pub install: Vec<u8>,
    pub validate: Vec<u8>,
    pub signature: Vec<u8>,
}
```

## Verification

### Testing Requirements

- Parse valid SUIT manifest → correct fields extracted
- Parse with COSE signature → signature verification passes
- Parse with invalid COSE signature → signature verification fails
- Parse manifest with wrong vendor ID → `matches_device()` returns false
- Parse malformed CBOR → `SuitError::ParseFailed`
- Parse with truncated data → error

### Kani Proofs

| Harness | Property |
|---------|----------|
| `suit_parse_bounds` | CBOR parsing does not read beyond input bounds |
| `cose_verify_bounds` | COSE signature verification does not overflow buffers |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `suit_manifest_parser` | Arbitrary bytes as SUIT manifest |
| `cose_signature_parser` | Arbitrary bytes as COSE signature |

### Proptests

| Property | Check |
|----------|-------|
| `suit_parse_no_panic` | Parser never panics on arbitrary input |
| `identifier_uniqueness` | Vendor/class IDs are validated for length |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- SUIT/COSE uses CBOR which requires a parser. The `minicbor` crate or custom
  parser must be verified for no_std compatibility and bounds safety.
- The SUIT manifest adds overhead: manifest can be 500-2000 bytes for complex
  updates. Acceptable for all STM32 targets.