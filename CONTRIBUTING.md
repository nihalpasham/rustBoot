# Contributing to rustBoot

## Welcome

Thank you for your interest in rustBoot. This project aims to provide a secure
bootloader suitable for safety- and security-conscious embedded deployments.

## Getting Started

1. Read the [README](README.md)
2. Set up the development environment per `rust-toolchain.toml`
3. Run `just verify` before submitting changes
4. Review open issues and PRs

## Development Workflow

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run `just verify` (format, clippy, test, audit, deny)
5. Run local verification steps appropriate to your changes (see sections below)
6. Submit a pull request

## Code Style

- Run `cargo fmt --all` before committing
- All warnings must be addressed (CI has `-D warnings`)
- No `unwrap()`, `expect()`, `panic!()`, or `todo!()` in production code
- All `unsafe` blocks must have a `// Safety:` comment
- Prefer typed errors over panics
- New features must include tests

## Pull Request Requirements

Every PR must pass all of the following CI gates (defined in `.github/workflows/ci.yml`):

- [ ] **Formatting** — `cargo fmt --all --check`
- [ ] **Lint** — `cargo clippy -D warnings` on all packages and feature combinations
- [ ] **Tests** — `cargo test --package rustBoot --lib` (unit) + per-MCU parser tests (184 total)
- [ ] **Audit** — `cargo audit` (no new vulnerability advisories)
- [ ] **Deny** — `cargo deny check` (no license or dependency issues)
- [ ] **Coverage** — MC/DC via `cargo-llvm-cov` (--fail-under-linenum 26)
- [ ] **Builds** — All 8 MCU targets across 3 architectures (Cortex-M, AArch64, Cortex-M0+)
- [ ] **Binary Size** — Per-target size check (warn > 64KB)
- [ ] **Kani** — Formal verification proofs (19 harnesses in `kani/src/lib.rs`)
- [ ] **Fuzz** — 4 fuzz targets never panic (`fuzz/fuzz_targets/`)
- [ ] **QEMU** — Boot emulation test (stm32f411 under qemu-system-arm)
- [ ] **SBOM** — CycloneDX SBOM generated via `just sbom`
- [ ] Include tests for new functionality
- [ ] Update documentation if behavior changes
- [ ] Reference any related issue or requirement ID

## Architectural Principles

- `no_std` compatibility for core library
- Deterministic boot flow (explicit state machine)
- Defense in depth (SHA-256 + ECDSA + anti-rollback + fallback)
- Fail closed (return errors, never panic in production)
- Bounded resource usage (no unbounded allocation in boot path)

## Running Kani Proofs

Kani is a formal verification tool for Rust that proves absence of panics,
overflow, and assertion failures within bounded input spaces. rustBoot maintains
19 Kani proof harnesses in `kani/src/lib.rs`.

**Setup:**
```bash
cargo install kani-verifier
```

**Run all proofs:**
```bash
cd kani && cargo kani --workspace
```

**Run a specific harness:**
```bash
cd kani && cargo kani --harness <harness_name>
```

Available harnesses cover:
- State machine: `state_transition_dag_no_cycles`, `state_encode_decode_roundtrip`,
  `sect_flags_from_bounded`, `sect_flags_all_values`
- Partition arithmetic: `partition_offset_arithmetic`, `partition_open_bounds`,
  `partition_size_bounds`, `constants_consistent`
- Parser bounds: `parser_extract_version_bounds`, `parser_extract_timestamp_bounds`,
  `parser_extract_img_type_no_panic`, `parser_extract_digest_no_panic`,
  `parser_extract_pubkey_digest_no_panic`, `parser_extract_signature_no_panic`
- Utilities: `flatten_bounds`, `version_comparison_properties`

Kani proofs require the `kani` Rust toolchain and are run on `ubuntu-latest` in CI.

## Running Fuzz Targets

rustBoot uses `cargo fuzz` with 4 fuzz targets in `fuzz/fuzz_targets/`:
- `image_header_parser` — TLV image header parser
- `config_parser` — Update configuration file parser
- `fit_parser` — FIT image parser
- `dtb_parser` — Device tree blob parser

**Setup:**
```bash
cargo install cargo-fuzz
```

**Run a target (default 1 hour, or until crash found):**
```bash
cd fuzz && cargo fuzz run <target_name> -- -max_total_time=3600
```

**Run with a specific corpus:**
```bash
cd fuzz && cargo fuzz run <target_name> fuzz/corpus/<target_name> -- -max_total_time=3600
```

All fuzz targets are verified to never panic on any input. CI runs each target
for a minimum period; local runs should target at least 1 hour before submitting
parser changes.

## Running QEMU Tests

QEMU integration tests boot the compiled firmware under emulation for the
stm32f411 target.

**Setup:**
```bash
sudo apt-get install qemu-system-arm   # Linux
brew install qemu                      # macOS
```

**Run the QEMU boot test:**
```bash
cargo run -p xtask --features stm32f411 -- stm32f411 build rustBoot-only
./scripts/qemu_test.sh
```

The QEMU gate is allowed to fail (continue-on-error) in CI as it depends on
the host QEMU version and peripheral models.

## Safety Case and Formal Models

rustBoot maintains a formal safety case at `docs/safety_case.md` covering 7
safety goals (G1–G7):

| Goal | Description | Verification |
|------|-------------|-------------|
| G1 | Image Integrity (SHA-256) | Unit tests + Kani proofs |
| G2 | Image Authenticity (ECDSA P-256) | 11 unit tests covering all error paths |
| G3 | State Machine Correctness | 19 Kani proofs + exhaustive transition tests |
| G4 | Partition Arithmetic Safety | 4 Kani proofs (overflow verification) |
| G5 | Parser Robustness | 6 Kani proofs + 4 fuzz targets + proptests |
| G6 | Unsafe Code Containment | deny(unsafe_code) at crate root |
| G7 | Build Determinism | SBOM + cargo-deny + cargo-audit |

The safety case documents hazards, mitigations, residual risk, and a full
verification matrix mapping each goal to tests, Kani harnesses, fuzz targets,
and property tests.

Formal verification includes:
- **Kani** (Rust model checker) — 19 harnesses for bounded verification of
  parser, partition math, and state machine logic
- **Proptest** — Property-based tests for parser safety, state encoding/decoding
  roundtrips, and invalid input rejection
- **Fuzz** — Coverage-guided fuzzing of all four parser entry points

Changes to parser, state machine, or partition arithmetic code must update the
safety case and add corresponding verification evidence (Kani harness, fuzz
target, or proptest as appropriate).

## License

By contributing, you agree that your contributions will be licensed under the
MIT License as specified in the repository.