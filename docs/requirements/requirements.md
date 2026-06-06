# rustBoot Requirements

## REQ-001: Secure Boot
The bootloader MUST verify cryptographic integrity (SHA-256) and authenticity (ECDSA NIST P-256) of every firmware image before execution.

**Verification**: `verify_integrity::<SHA256_DIGEST_SIZE>()` + `verify_authenticity::<HDR_IMG_TYPE_AUTH>()`

## REQ-002: Anti-Rollback
The bootloader MUST reject firmware images with version <= current booted version, unless a rollback is explicitly triggered.

**Verification**: `get_firmware_version()` comparison in `rustboot_update()`

## REQ-003: Power-Interruptible Update
The bootloader MUST support sector-by-sector A/B swap that can be interrupted by power loss and resumed on next boot without bricking the device.

**Verification**: Sector flag state machine (New → Swapping → Backup → Updated)

## REQ-004: Fallback on Failure
If the updated firmware fails to boot (detected via Testing state on next boot), the bootloader MUST automatically roll back to the previous firmware.

**Verification**: `BootInTestingState` triggers `rustboot_update(true)` in `rustboot_start()`

## REQ-005: Deterministic Boot Flow
The bootloader MUST follow a deterministic state machine: New → Updating → Testing → Success, with no undefined states reachable.

**Verification**: `ImageType` enum with type-state enforcement via `Sealed` trait

## REQ-006: Multi-Platform Support
The bootloader MUST support ARM Cortex-M, Cortex-A, and AArch64 targets with a hardware abstraction layer for flash operations.

**Verification**: `FlashInterface` trait with implementations for nRF52840, STM32, RP2040, RPi4, i.MX 8M Nano

## REQ-007: Signed Image Format
All firmware images MUST use a TLV-based header format with magic number, version, timestamp, image type, SHA-256 digest, public key digest, and ECDSA signature.

**Verification**: `parser.rs` TLV parsing with `Tags` enum

## REQ-008: Configurable Partition Layout
The bootloader MUST support board-specific flash partition layouts (BOOT, UPDATE, SWAP) defined at compile time.

**Verification**: `constants.rs` per-feature partition constants

## REQ-009: FIT Image Support
For Linux targets (RPi4, i.MX 8M Nano), the bootloader MUST support Flattened Image Tree (FIT) format with DTB patching.

**Verification**: `dt/fit.rs`, `dt/patch.rs`

## REQ-010: No Panic in Production
The bootloader MUST NOT panic in production code paths. All fallible operations must return typed errors.

**Verification**: `#![deny(clippy::panic)]`, `#![deny(clippy::unwrap_used)]` (planned — see PR2)

## Traceability Matrix

| Req ID | Module | Test | Status |
|--------|--------|------|--------|
| REQ-001 | `image/image.rs` | `verify_integrity`, `verify_authenticity` | Verified |
| REQ-002 | `update/update_flash.rs` | version comparison | Verified |
| REQ-003 | `update/update_flash.rs` | sector flag state machine | Verified |
| REQ-004 | `update/update_flash.rs` | `rustboot_start()` rollback | Verified |
| REQ-005 | `image/image.rs` | `ImageType` enum | Verified |
| REQ-006 | `hal/src/lib.rs` | `FlashInterface` trait | Verified |
| REQ-007 | `parser.rs` | TLV parser tests | Verified |
| REQ-008 | `constants.rs` | per-feature constants | Verified |
| REQ-009 | `dt/fit.rs`, `dt/patch.rs` | FIT tests | Verified |
| REQ-010 | `lib.rs` | lint enforcement | **Partial** — PR2 in review |