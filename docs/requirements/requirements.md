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

| Req ID | Module | Test / Verification | Status |
|--------|--------|---------------------|--------|
| REQ-001 | `crypto/signatures.rs` | `nistp256_verify_good_signature`, `nistp256_verify_bad_signature`, `nistp256_verify_malformed_signature`, `nistp256_verify_wrong_key`, `verify_ecc256_bad_sig_returns_auth_failed`, `verify_ecc256_zero_length_signature`, `verify_ecc256_invalid_algorithm_id`, `import_pubkey_nistp256_ok`, `import_pubkey_unsupported_returns_error`, `import_pubkey_nistp384_returns_error`, `import_pubkey_secp256k1_returns_error` | Verified |
| REQ-001 | `integration` | `test_image_header_size_invariants` | Verified |
| REQ-001 | Kani | `partition_size_bounds` | Verified |
| REQ-002 | `integration` | `test_version_comparison_logic` | Verified |
| REQ-002 | Kani | `version_comparison_properties` | Verified |
| REQ-003 | `image/image.rs` | `test_sect_flags_mutation`, `test_sect_flags_mutation_from_each_state` | Verified |
| REQ-003 | integration | `test_sect_flags_values`, `test_sect_flags_none_returns_none`, `test_sect_flags_valid_return_some` | Verified |
| REQ-003 | Kani | `sect_flags_from_bounded`, `sect_flags_all_values`, `state_encode_decode_roundtrip` | Verified |
| REQ-004 | `image/image.rs` | `test_valid_state_transitions_graph` (Testing → Success transition) | Verified |
| REQ-004 | integration | `test_state_transition_table_exhaustive`, `test_documented_valid_transitions` | Verified |
| REQ-004 | Kani | `state_transition_dag_no_cycles` | Verified |
| REQ-005 | `image/image.rs` | `test_valid_state_transitions_graph`, `test_state_transition_methods_compile`, `test_image_type_match_exhaustive`, `test_typestate_from_values`, `test_sect_flags_helpers` | Verified |
| REQ-005 | integration | `test_state_transition_table_exhaustive`, `test_valid_transitions_map_to_methods`, `test_typestate_encoding`, `test_state_flags_distinct` | Verified |
| REQ-005 | proptest | `state_decoding_never_panics`, `state_encoding_roundtrip`, `invalid_state_flags_produce_errors` | Verified |
| REQ-005 | Kani | `state_transition_dag_no_cycles`, `state_encode_decode_roundtrip` | Verified |
| REQ-006 | `boards/hal/src/lib.rs` | `FlashInterface` trait implementations (compile-time per-MCU) | Verified |
| REQ-007 | `parser.rs` | `padding_test`, `parse_version`, `parse_timestamp`, `parse_img_type`, `parse_digest`, `parse_pubkey_digest`, `parse_signature`, `get_tlv_digest256`, `get_tlv_pubkey_digest` | Verified |
| REQ-007 | integration | `test_tags_ids`, `test_rbconstants_correctness` | Verified |
| REQ-007 | Kani | `parser_extract_version_bounds`, `parser_extract_timestamp_bounds`, `parser_extract_version_no_panic`, `parser_extract_timestamp_no_panic`, `parser_extract_img_type_no_panic`, `parser_extract_digest_no_panic`, `parser_extract_pubkey_digest_no_panic`, `parser_extract_signature_no_panic` | Verified |
| REQ-007 | proptest | `parser_never_panics_on_arbitrary_input`, `parser_version_output_len_valid`, `parser_timestamp_output_len_valid` | Verified |
| REQ-007 | fuzz | `image_header_parser` | Verified |
| REQ-008 | `rustBoot/src/constants.rs` | Per-feature partition constants (compile-time constants) | Verified |
| REQ-008 | integration | `integration::test_platform_constants_consistency` | Verified |
| REQ-008 | Kani | `partition_offset_arithmetic`, `partition_open_bounds`, `partition_size_bounds`, `constants_consistent` | Verified |
| REQ-009 | `dt/fit.rs` | `test_parse_algo_valid_curve`, `test_parse_algo_unknown_curve_returns_error`, `test_parse_algo_empty_algo_returns_error`, `test_parse_algo_truncated_blob_does_not_panic`, `test_parse_algo_extra_long_algo_string`, `test_parse_algo_multiple_confignodes`, `test_flatten_correct_size`, `test_flatten_all_zeros`, `test_flatten_distinct_values`, `test_flatten_partial_usage` | Verified |
| REQ-009 | `dt/patch.rs` | `test_patch_dtb_node_basic`, `test_patch_dtb_node_empty_patches`, `test_patch_chosen_node_valid`, `test_patch_chosen_node_multiple_calls`, `test_check_chosen_node_removes_bootargs_and_initrd`, `test_check_chosen_node_all_removed`, `test_check_chosen_node_empty_items`, `test_get_padded_node_len_chosen`, `test_get_node_start_and_end_chosen` | Verified |
| REQ-009 | fuzz | `fit_parser`, `dtb_parser` | Verified |
| REQ-009 | Kani | `flatten_bounds` | Verified |
| REQ-010 | `lib.rs` | `#![deny(clippy::panic)]`, `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`, `#![deny(clippy::todo)]`, `#![deny(clippy::unimplemented)]` | Verified |
| REQ-010 | `crypto/signatures.rs`, `image/image.rs`, `parser.rs`, etc. | All `#[allow(clippy::unwrap_used)]` scoped to `#[cfg(test)]` modules | Verified |