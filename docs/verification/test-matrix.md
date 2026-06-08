# rustBoot Verification Matrix

## Test Coverage Overview

- **Total tests**: 184 (164 unit + 20 integration)
- **Kani proof harnesses**: 18
- **Fuzz targets**: 4
- **Property tests (proptest)**: 7

## Safety Goal to Test Mapping

| Safety Goal | Test Type | Test Name | Kani Harness | Fuzz Target | Status |
|---|---|---|---|---|---|
| **G1: Image Integrity** | unit | `crypto::signatures::nistp256_verify_good_signature` | `partition_size_bounds` | — | ✅ |
| G1 | unit | `crypto::signatures::nistp256_verify_bad_signature` | — | — | ✅ |
| G1 | unit | `crypto::signatures::nistp256_verify_malformed_signature` | — | — | ✅ |
| G1 | unit | `crypto::signatures::nistp256_verify_wrong_key` | — | — | ✅ |
| G1 | unit | `crypto::signatures::verify_ecc256_bad_sig_returns_auth_failed` | — | — | ✅ |
| G1 | unit | `crypto::signatures::verify_ecc256_zero_length_signature` | — | — | ✅ |
| G1 | unit | `crypto::signatures::verify_ecc256_invalid_algorithm_id` | — | — | ✅ |
| G1 | unit | `crypto::signatures::import_pubkey_nistp256_ok` | — | — | ✅ |
| G1 | unit | `crypto::signatures::import_pubkey_unsupported_returns_error` | — | — | ✅ |
| G1 | unit | `crypto::signatures::import_pubkey_nistp384_returns_error` | — | — | ✅ |
| G1 | unit | `crypto::signatures::import_pubkey_secp256k1_returns_error` | — | — | ✅ |
| G1 | integration | `integration::test_image_header_size_invariants` | — | — | ✅ |
| **G2: Image Authenticity** | unit | (same 11 crypto tests as G1) | — | — | ✅ |
| G2 | unit | `parser::tests::parse_signature` | — | — | ✅ |
| **G3: State Machine Correctness** | unit | `image::image::tests::test_valid_state_transitions_graph` | `state_transition_dag_no_cycles` | — | ✅ |
| G3 | unit | `image::image::tests::test_state_transition_methods_compile` | `state_encode_decode_roundtrip` | — | ✅ |
| G3 | unit | `image::image::tests::test_typestate_from_values` | `sect_flags_from_bounded` | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_from_valid` | `sect_flags_all_values` | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_from_invalid` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_helpers` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_mutation` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_part_id_values` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_part_id_equality` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_image_type_match_exhaustive` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_all_has_flags_mutually_exclusive` | — | — | ✅ |
| G3 | unit | `image::image::tests::test_sect_flags_mutation_from_each_state` | — | — | ✅ |
| G3 | proptest | `state_decoding_never_panics` | — | — | ✅ |
| G3 | proptest | `state_encoding_roundtrip` | — | — | ✅ |
| G3 | proptest | `invalid_state_flags_produce_errors` | — | — | ✅ |
| G3 | integration | `integration::test_state_transition_table_exhaustive` | — | — | ✅ |
| G3 | integration | `integration::test_valid_transitions_map_to_methods` | — | — | ✅ |
| G3 | integration | `integration::test_documented_valid_transitions` | — | — | ✅ |
| G3 | integration | `integration::test_typestate_encoding` | — | — | ✅ |
| G3 | integration | `integration::test_part_id_values` | — | — | ✅ |
| G3 | integration | `integration::test_sect_flags_values` | — | — | ✅ |
| G3 | integration | `integration::test_image_type_exhaustive` | — | — | ✅ |
| G3 | integration | `integration::test_state_flags_distinct` | — | — | ✅ |
| G3 | integration | `integration::test_sect_flags_none_returns_none` | — | — | ✅ |
| G3 | integration | `integration::test_sect_flags_valid_return_some` | — | — | ✅ |
| G3 | integration | `integration::test_error_display` | — | — | ✅ |
| G3 | integration | `integration::test_error_equality` | — | — | ✅ |
| G3 | integration | `integration::test_tags_ids` | — | — | ✅ |
| **G4: Partition Arithmetic Safety** | kani | — | `partition_offset_arithmetic` | — | ✅ |
| G4 | kani | — | `partition_open_bounds` | — | ✅ |
| G4 | kani | — | `partition_size_bounds` | — | ✅ |
| G4 | kani | — | `constants_consistent` | — | ✅ |
| G4 | integration | `integration::test_rbconstants_correctness` | — | — | ✅ |
| G4 | integration | `integration::test_platform_constants_consistency` | — | — | ✅ |
| **G5: Parser Robustness** | unit | `parser::tests::padding_test` | `parser_extract_version_bounds` | `image_header_parser` | ✅ |
| G5 | unit | `parser::tests::parse_version` | `parser_extract_timestamp_bounds` | `config_parser` | ✅ |
| G5 | unit | `parser::tests::parse_timestamp` | `parser_extract_version_no_panic` | `dtb_parser` | ✅ |
| G5 | unit | `parser::tests::parse_img_type` | `parser_extract_timestamp_no_panic` | `fit_parser` | ✅ |
| G5 | unit | `parser::tests::parse_digest` | `parser_extract_img_type_no_panic` | — | ✅ |
| G5 | unit | `parser::tests::parse_pubkey_digest` | `parser_extract_digest_no_panic` | — | ✅ |
| G5 | unit | `parser::tests::parse_signature` | `parser_extract_pubkey_digest_no_panic` | — | ✅ |
| G5 | unit | `parser::tests::get_tlv_digest256` | `parser_extract_signature_no_panic` | — | ✅ |
| G5 | unit | `parser::tests::get_tlv_pubkey_digest` | `flatten_bounds` | — | ✅ |
| G5 | unit | `cfgparser::tests::test_config_keys` | `version_comparison_properties` | — | ✅ |
| G5 | unit | `cfgparser::tests::test_image_name` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_image_version` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_ready_for_update` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_update_status` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_active_conf` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_passive_conf` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_parse_config` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_update_status_all_variants` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_ready_for_update_both_values` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_alphanumericwithhypen_variants` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_config_keys_edge_cases` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_image_name_edge_cases` | — | — | ✅ |
| G5 | unit | `cfgparser::tests::test_parse_config_empty_passive` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_valid_curve` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_unknown_curve_returns_error` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_empty_algo_returns_error` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_truncated_blob_does_not_panic` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_extra_long_algo_string` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_parse_algo_multiple_confignodes` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_valid_utf8_with_null` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_valid_utf8_no_null` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_invalid_utf8` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_empty` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_very_long_valid_utf8` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_very_long_with_null` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_embedded_nulls` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_only_null` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_as_str_multiple_trailing_nulls` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_correct_size` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_all_zeros` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_distinct_values` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_partial_usage` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_empty_array` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_single_hash` | — | — | ✅ |
| G5 | unit | `dt::fit::tests::test_flatten_all_boundaries` | — | — | ✅ |
| G5 | proptest | `parser_never_panics_on_arbitrary_input` | — | — | ✅ |
| G5 | proptest | `parser_version_output_len_valid` | — | — | ✅ |
| G5 | proptest | `parser_timestamp_output_len_valid` | — | — | ✅ |
| G5 | proptest | `cfgparser_never_panics_on_arbitrary_input` | — | — | ✅ |
| G5 | integration | `integration::test_version_parsing` | — | — | ✅ |
| G5 | integration | `integration::test_check_for_eof` | — | — | ✅ |
| G5 | integration | `integration::test_check_for_padding` | — | — | ✅ |
| G5 | integration | `integration::test_version_comparison_logic` | — | — | ✅ |
| **G6: Unsafe Code Containment** | static | `#![deny(unsafe_code)]` at `lib.rs:13` | — | — | ✅ |
| G6 | static | 7 module-level `#[allow(unsafe_code)]` with documented invariants | — | — | ✅ |
| G6 | static | 1 inline `#[allow(unsafe_code)]` at `dt/internal.rs:93` | — | — | ✅ |
| **G7: Build Determinism** | script | `sbom.sh` | — | — | ✅ |
| G7 | script | `verify.sh` | — | — | ✅ |
| G7 | audit | `cargo audit` | — | — | ✅ |
| G7 | deny | `cargo deny check` | — | — | ✅ |

## DTB Patch Tests (G5 coverage)

| Test Name | Module | Status |
|---|---|---|
| `test_correct_endianess_zero` | `dt::patch` | ✅ |
| `test_correct_endianess_all_ones` | `dt::patch` | ✅ |
| `test_correct_endianess_byte_swap` | `dt::patch` | ✅ |
| `test_correct_endianess_identity_values` | `dt::patch` | ✅ |
| `test_correct_endianess_mid_bytes` | `dt::patch` | ✅ |
| `test_update_dtb_header_typical` | `dt::patch` | ✅ |
| `test_update_dtb_header_zero_appended` | `dt::patch` | ✅ |
| `test_update_dtb_header_subtract_larger_than_add` | `dt::patch` | ✅ |
| `test_make_node_with_props_valid` | `dt::patch` | ✅ |
| `test_make_node_with_props_multiple_props` | `dt::patch` | ✅ |
| `test_make_node_with_props_empty_props` | `dt::patch` | ✅ |
| `test_make_node_with_props_short_name` | `dt::patch` | ✅ |
| `test_make_new_strings_block_with_valid` | `dt::patch` | ✅ |
| `test_make_new_strings_block_with_empty_name_returns_err` | `dt::patch` | ✅ |
| `test_make_new_strings_block_with_too_many_names` | `dt::patch` | ✅ |
| `test_make_new_strings_block_with_invalid_dtb` | `dt::patch` | ✅ |
| `test_parse_raw_node_chosen_node` | `dt::patch` | ✅ |
| `test_parse_raw_node_root_path` | `dt::patch` | ✅ |
| `test_parse_raw_node_invalid_path_returns_err_or_empty` | `dt::patch` | ✅ |
| `test_parse_raw_node_chosen_offset_tracking` | `dt::patch` | ✅ |
| `test_check_chosen_node_removes_bootargs_and_initrd` | `dt::patch` | ✅ |
| `test_check_chosen_node_all_removed` | `dt::patch` | ✅ |
| `test_check_chosen_node_empty_items` | `dt::patch` | ✅ |
| `test_get_padded_node_len_chosen` | `dt::patch` | ✅ |
| `test_get_padded_node_len_root_path_uses_chosen` | `dt::patch` | ✅ |
| `test_get_node_start_and_end_chosen` | `dt::patch` | ✅ |
| `test_get_node_start_and_end_different_node_size` | `dt::patch` | ✅ |
| `test_patch_dtb_node_basic` | `dt::patch` | ✅ |
| `test_patch_dtb_node_empty_patches` | `dt::patch` | ✅ |
| `test_patch_chosen_node_valid` | `dt::patch` | ✅ |
| `test_patch_chosen_node_multiple_calls` | `dt::patch` | ✅ |

## Kani Proof Harnesses (18 total)

| Harness | Module | Verifies |
|---|---|---|
| `sect_flags_from_bounded` | `image::image` | State flag decoding never returns None for valid values |
| `sect_flags_all_values` | `image::image` | All SectFlags variants encode to expected bytes |
| `partition_size_bounds` | `constants` | Partition size constants internally consistent |
| `partition_open_bounds` | `image::image` | PartId discriminants distinct and in-range |
| `partition_offset_arithmetic` | `constants` | Address arithmetic never overflows usize |
| `state_transition_dag_no_cycles` | `image::image` | State values distinct with no self-loops |
| `state_encode_decode_roundtrip` | `image::image` | Valid SectFlags round-trip through encode/decode |
| `parser_extract_version_bounds` | `parser` | extract_version never panics on 16-byte input |
| `parser_extract_timestamp_bounds` | `parser` | extract_timestamp never panics on 24-byte input |
| `parser_extract_version_no_panic` | `parser` | extract_version panic-free (no size limit) |
| `parser_extract_timestamp_no_panic` | `parser` | extract_timestamp panic-free (no size limit) |
| `parser_extract_img_type_no_panic` | `parser` | extract_img_type never panics on 32-byte input |
| `parser_extract_digest_no_panic` | `parser` | extract_digest never panics on 48-byte input |
| `parser_extract_pubkey_digest_no_panic` | `parser` | extract_pubkey_digest never panics on 64-byte input |
| `parser_extract_signature_no_panic` | `parser` | extract_signature never panics on 96-byte input |
| `constants_consistent` | `constants` | Compile-time constants internally consistent |
| `version_comparison_properties` | `parser` | u32 comparison reflexive/antisymmetric/transitive |
| `flatten_bounds` | `dt::fit` | flatten() always returns exactly 128 bytes |

## Fuzz Targets (4 total)

| Target | Module | Harness Status |
|---|---|---|
| `image_header_parser` | `parser` | ✅ Active |
| `config_parser` | `cfgparser` | ✅ Active |
| `fit_parser` | `dt::fit` | ✅ Active |
| `dtb_parser` | `dt::reader` | ✅ Active |

## Property Tests (7 total)

| Property | Module | Checks |
|---|---|---|
| `state_decoding_never_panics` | `image::image` | State decoding never panics on any u8 input |
| `state_encoding_roundtrip` | `image::image` | Valid state flags round-trip through encode/decode |
| `invalid_state_flags_produce_errors` | `image::image` | Invalid flag values produce errors, not panics |
| `parser_never_panics_on_arbitrary_input` | `parser` | 8 parser functions never panic on arbitrary bytes |
| `parser_version_output_len_valid` | `parser` | Version extraction returns at most 4 bytes |
| `parser_timestamp_output_len_valid` | `parser` | Timestamp extraction returns at most 8 bytes |
| `cfgparser_never_panics_on_arbitrary_input` | `cfgparser` | 6 config parser functions never panic on arbitrary strings |

## Static Analysis

| Tool | Status | Notes |
|---|---|---|
| `cargo fmt` | ✅ | Enforced in CI |
| `cargo clippy -D warnings` | ✅ | Enforced in CI |
| `cargo audit` | ✅ | Enforced in CI |
| `cargo deny check` | ✅ | Enforced in CI |

## CI Gates

- **Formatting**: `cargo fmt --all --check`
- **Lint**: `cargo clippy` with `-D warnings` on all feature combinations
- **Audit**: `cargo audit` for vulnerability advisories
- **Deny**: `cargo deny check` for licenses and banned dependencies
- **Coverage**: MC/DC via `cargo-llvm-cov` with `--fail-under-linenum 26`
- **Tests**: matrix over ubuntu/macos/windows, all MCU feature flags
- **Builds**: all 8 MCU targets across 3 architectures
- **Binary size**: per-target size check (warn > 64KB)
- **Kani**: formal verification of 18 proof harnesses

## Gaps

1. No hardware-in-the-loop integration tests for full boot flow (requires probe-rs + physical MCU)
2. MC/DC threshold at 26% line coverage; full branch/MC/DC closure on state machine and parser logic is WIP
3. No timing benchmarks
4. No fault injection tests
5. No Kani proofs for hash computation or signature verification
6. Side-channel resistance not verified at crate level (relies on `p256` crate guarantees)
7. No formal verification of FAT filesystem parser (`fs/fat.rs`)
8. No formal verification of device tree writer (`dt/writer.rs`)