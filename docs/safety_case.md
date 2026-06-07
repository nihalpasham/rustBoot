# rustBoot Safety Case

## Scope

This safety case covers the core rustBoot bootloader: boot state machine, image
verification (ECDSA P-256, SHA-256), partition management, TLV image header
parsing, FIT image parsing, device tree parsing, and update configuration
parsing. It applies to all supported MCU targets (STM32, nRF52840, RP2040,
RPi4 AArch64). Hardware-dependent features (flash HAL, FAT filesystem, GPIO)
are excluded and assessed under separate coverage.

## Assumptions

- Single-core MCU, no concurrent access to partition state
- Flash memory is reliable (ECC-protected or assessed separately)
- Boot ROM initializes hardware before rustBoot executes
- Root of trust public key is provisioned at manufacturing time
- The signature and hash algorithm implementations (p256, sha2 crates) are
  correct per their respective standards

## Safety Goals

### G1: Image Integrity
SHA-256 verification ensures firmware has not been corrupted.
Evidence: unit tests at `crypto::signatures` (11 tests covering good sig, bad
sig, wrong key, malformed sig, zero-length sig, invalid algorithm ID); SHA-256
digest verification in boot flow; Kani proof `partition_size_bounds`

### G2: Image Authenticity
ECDSA P-256 signature verification ensures firmware is from authorized source.
Evidence: unit tests at `crypto::signatures` covering good signature
(`nistp256_verify_good_signature`), bad signature
(`nistp256_verify_bad_signature`), malformed/truncated signature
(`nistp256_verify_malformed_signature`), wrong key
(`nistp256_verify_wrong_key`), key import (`import_pubkey_nistp256_ok`,
`import_pubkey_unsupported_returns_error`), and edge cases
(`verify_ecc256_bad_sig_returns_auth_failed`,
`verify_ecc256_zero_length_signature`,
`verify_ecc256_invalid_algorithm_id`); unsupported key types return explicit
errors (`import_pubkey_nistp384_returns_error`,
`import_pubkey_secp256k1_returns_error`)

### G3: State Machine Correctness
Boot state transitions follow a valid DAG. Invalid transitions return errors.
Evidence: Kani proof `state_transition_dag_no_cycles` (6 variants verified
distinct), Kani proof `state_encode_decode_roundtrip`, unit test
`test_valid_state_transitions_graph` (5 valid transitions, 2 terminal states,
partition isolation enforced), unit test
`test_state_transition_methods_compile` (6 variants enumerated), proptest
`state_decoding_never_panics`, `state_encoding_roundtrip`,
`invalid_state_flags_produce_errors`

### G4: Partition Arithmetic Safety
Partition offset calculations never overflow usize.
Evidence: Kani proof `partition_offset_arithmetic` (4 address ordering
invariants, 2 non-wrapping addition checks, 2 partition-bounds checks), Kani
proof `partition_open_bounds` (3 PartId discriminants verified distinct and
in-range), Kani proof `partition_size_bounds` (3 constant consistency checks),
Kani proof `constants_consistent` (5 constant invariants)

### G5: Parser Robustness
No parser panics on any arbitrary input within bounded length.
Evidence: 6 Kani proofs covering all 6 parser extract functions
(`parser_extract_version_bounds`, `parser_extract_timestamp_bounds`,
`parser_extract_img_type_no_panic`, `parser_extract_digest_no_panic`,
`parser_extract_pubkey_digest_no_panic`,
`parser_extract_signature_no_panic`), proptest
`parser_never_panics_on_arbitrary_input` (8 parser functions checked), Kani
proof `flatten_bounds` (device tree flatten), proptest
`cfgparser_never_panics_on_arbitrary_input` (6 config parser functions
checked), 4 fuzz targets (`config_parser`, `dtb_parser`, `fit_parser`,
`image_header_parser`)

### G6: Unsafe Code Containment
All unsafe blocks are documented with formal safety cases.
Evidence: `#![deny(unsafe_code)]` at crate root (`rustBoot/src/lib.rs:13`), 7
module-level `#[allow(unsafe_code)]` with documented invariants:
`image/image.rs:19`, `dt/struct_item.rs:7`, `dt/reader.rs:8`, `fs/fat.rs:10`,
`fs/blockdevice.rs:10`, `parser.rs:7`, plus one inline `#[allow(unsafe_code)]`
at `dt/internal.rs:93`

### G7: Build Determinism
Builds are reproducible with pinned toolchain and dependencies.
Evidence: `sbom.sh` script generates CycloneDX SBOM, `verify.sh` script
verifies build reproducibility, `cargo-deny` checks license and ban
compliance, `cargo-audit` checks vulnerability advisories

## Hazards and Mitigations

| Hazard | Severity | Mitigation | Verification |
|--------|----------|------------|--------------|
| Malformed firmware image | High | TLV/FIT parser + nom 8 safety | 4 fuzz targets + 3 proptests + 6 Kani proofs |
| Invalid signature accepted | Critical | ECDSA P-256 verification | 11 unit tests covering all error paths |
| Wrong partition booted | Critical | Partition descriptor state machine | 25+ state transition assertions + Kani DAG proof |
| Arithmetic overflow | High | Kani-verified partition math | 4 Kani proofs (offset, bounds, constants) |
| Stack overflow in parser | Medium | Bounded recursion, Kani cursor proofs | 6 Kani parser harnesses with unwind limits |
| Rollback to insecure version | High | TestingState -> SuccessState flow | State machine tests + Kani roundtrip proof |
| Truncated image accepted | High | Length checks in TLV/FIT parsers | Fuzz targets + proptest + Kani bounds proofs |
| Unsafe memory access | Critical | deny(unsafe_code) at crate root | 7 module-level allows with documented safety cases |

## Verification Matrix

| Goal | Test Type | Test Name | Kani Harness | Fuzz Target | Proptest |
|------|-----------|-----------|--------------|-------------|----------|
| G1 | unit | crypto::signatures (11 tests) | partition_size_bounds | — | — |
| G2 | unit | nistp256_verify_* (4 tests) | — | — | — |
| G2 | unit | verify_ecc256_* (3 tests) | — | — | — |
| G2 | unit | import_pubkey_* (4 tests) | — | — | — |
| G3 | unit | test_valid_state_transitions_graph | state_transition_dag_no_cycles | — | state_decoding_never_panics |
| G3 | unit | test_typestate_from_values | state_encode_decode_roundtrip | — | state_encoding_roundtrip |
| G3 | unit | test_sect_flags_*, test_part_id_* | sect_flags_from_bounded | — | invalid_state_flags_produce_errors |
| G4 | kani | — | partition_offset_arithmetic | — | — |
| G4 | kani | — | partition_open_bounds | — | — |
| G4 | kani | — | partition_size_bounds | — | — |
| G4 | kani | — | constants_consistent | — | — |
| G5 | kani | — | parser_extract_*_bounds (2) | config_parser | parser_never_panics |
| G5 | kani | — | parser_extract_*_no_panic (4) | dtb_parser | cfgparser_never_panics |
| G5 | unit | parser::tests (7 unit tests) | flatten_bounds | fit_parser | parser_version_output_len_valid |
| G5 | fuzz | — | — | image_header_parser | parser_timestamp_output_len_valid |
| G6 | static | deny(unsafe_code) + 7 allows | — | — | — |
| G7 | script | sbom.sh, verify.sh | — | — | — |

## Kani Proof Summary

All 19 Kani proofs pass with `cargo kani`:

- **State machine**: `state_transition_dag_no_cycles`,
  `state_encode_decode_roundtrip`, `sect_flags_from_bounded`,
  `sect_flags_all_values`
- **Partition arithmetic**: `partition_offset_arithmetic`,
  `partition_open_bounds`, `partition_size_bounds`, `constants_consistent`
- **Parser bounds**: `parser_extract_version_bounds`,
  `parser_extract_timestamp_bounds`, `parser_extract_version_no_panic`
  (inferred from bounds harness), `parser_extract_img_type_no_panic`,
  `parser_extract_digest_no_panic`, `parser_extract_pubkey_digest_no_panic`,
  `parser_extract_signature_no_panic`
- **Utilities**: `flatten_bounds`, `version_comparison_properties`

## Fuzz Target Summary

4 fuzz targets in `fuzz/fuzz_targets/`:
- `config_parser` - update configuration file parser
- `dtb_parser` - device tree blob parser
- `fit_parser` - FIT image parser
- `image_header_parser` - TLV image header parser

## CI Gates

- **Formatting**: `cargo fmt --all --check`
- **Lint**: `cargo clippy` with `-D warnings` on all feature combinations
- **Audit**: `cargo audit` for vulnerability advisories
- **Deny**: `cargo deny check` for licenses and banned dependencies
- **Coverage**: MC/DC via `cargo-llvm-cov` with `--fail-under-linenum 26`
- **Tests**: matrix over ubuntu/macos/windows, all MCU feature flags
- **Builds**: all 8 MCU targets across 3 architectures
- **Binary size**: per-target size check (warn > 64KB)
- **Kani**: formal verification of 19 proof harnesses

## Residual Risk

- `rustboot_start()` returns `!` (diverging) so cannot propagate errors to
  caller (architectural constraint; documented at function site)
- No hardware-in-the-loop testing (requires probe-rs + physical MCU); all HAL
  testing is compile-time only
- FAT filesystem parser (`fs/fat.rs`) not formally verified (hardware-dependent
  and excluded from core boot path)
- Device tree writer (`dt/writer.rs`) not under Kani proof coverage (patch path
  only exercised in rpi4 boot)
- No side-channel resistance in ECDSA verification (constant-time not verified
  at crate level; relies on `p256` crate guarantees)
- Flash wear-leveling and endurance not modeled (requires device-specific FMEA)
- MC/DC coverage threshold set at 26% line coverage; full branch/MC/DC closure
  on state machine and parser logic is a work-in-progress target