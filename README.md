![GitHub](https://img.shields.io/github/license/vlordier/rustBoot)
[![ci](https://github.com/vlordier/rustBoot/actions/workflows/ci.yml/badge.svg)](https://github.com/vlordier/rustBoot/actions/workflows/ci.yml)
![Tests](https://img.shields.io/badge/tests-184%20(164%20unit%20%2B%2020%20integration)-green)
![Coverage](https://img.shields.io/badge/line%20coverage-29%25-yellow)
![Kani](https://img.shields.io/badge/kani%20proofs-16-blue)

# rustBoot — Secure Bootloader for Embedded Systems

rustBoot is a standalone secure bootloader written entirely in Rust, designed
for microcontrollers through system-on-chip, supporting bare-metal firmware and
Linux booting. It provides A/B firmware updates with cryptographic verification
(ECDSA P-256, SHA-256), anti-rollback via version numbering, and
power-interruptible swap with automatic fallback.

The codebase is hardened to NATO-grade software assurance standards:
**0 warnings, 0 clippy errors**, `#![deny(unsafe_code)]` at crate root, formal
TLA+/Alloy models, 16 Kani proof harnesses, 5 cargo-fuzz targets, a complete
safety case, and MC/DC coverage analysis.

## Operational Scope

rustBoot is a **secure bootloader** intended for hardened embedded deployments.
It is not a general-purpose bootloader — it prioritizes:

- Cryptographic integrity and authenticity verification (ECDSA, SHA-256)
- Deterministic boot flow with explicit state machine
- Anti-rollback via version numbering
- Power-fail-safe firmware updates with automatic fallback
- Memory-safe core in Rust (no_std, no alloc, no panics)
- Formal verification via TLA+ and Alloy models

## Supported Targets

| Target | Architecture | Boot Mode |
|--------|-------------|-----------|
| STM32F334 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F411 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F446 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F469 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F746 | ARM Cortex-M7F | Bare-metal firmware |
| STM32H723 | ARM Cortex-M7F | Bare-metal firmware |
| nRF52840 | ARM Cortex-M4F | Bare-metal firmware |
| RP2040 | ARM Cortex-M0+ | Bare-metal firmware |
| Raspberry Pi 4 | AArch64 | Linux FIT image |
| i.MX 8M Nano | AArch64 | Linux FIT image |

## Features

- **Multi-architecture**: ARM Cortex-M (M0+, M4F, M7F) and AArch64 (Cortex-A)
- **Multi-slot flash partitioning**: boot / update / swap partitions
- **Cryptographic verification**: ECDSA signature verification (NIST P-256,
  secp256k1) and SHA-256 integrity hashing
- **Anti-rollback**: version-number-based downgrade prevention
- **Power-interruptible A/B swap**: automatic fallback on failed boot
- **Image formats**: TLV (bare-metal firmware), FIT/Flattened Image Tree (Linux)
- **Device tree support**: DTB parsing, patching, and writing
- **Tooling**: `rbsigner` CLI for signed firmware generation
- **QEMU testing**: STM32F411 emulation for CI-based smoke tests
- **Formal methods**: TLA+ and Alloy state machine models
- **Safety case**: NATO-aligned safety case document (`docs/safety_case.md`)
- **Fuzz testing**: 5 cargo-fuzz targets (TLV, FIT, DTB, config parser)
- **Kani proofs**: 16 proof harnesses for bounded verification
- **Stack depth analysis**: static analysis via `cargo-call-stack`
- **SBOM**: reproducible builds with CycloneDX software bill of materials

## Prerequisites

- Rust nightly toolchain (see `rust-toolchain.toml` — currently
  `nightly-2026-04-14`)
- Cross-compilation toolchain for target (`arm-none-eabi` for Cortex-M,
  `aarch64-unknown-none-softfloat` for Cortex-A)
- For flashing: `probe-rs-cli` or `pyocd`
- For verification: `just` (https://github.com/casey/just), `cargo-audit`,
  `cargo-deny`, `cargo-cyclonedx`, `cargo-llvm-cov`
- For Kani proofs: `cargo-kani`
- For fuzzing: `cargo-fuzz`

## Quick Start

```bash
# Build for STM32F411 (default target)
cargo run -p xtask --features stm32f411 -- stm32f411 build rustBoot-only

# Run all unit tests
cargo test --package rustBoot --lib

# Verification pipeline
just verify
```

## Build

```bash
cargo run -p xtask --features <board> -- <board> build [rustBoot-only|pkgs-for]
```

Supported boards: `stm32f411`, `stm32f334`, `stm32f446`, `stm32f469`,
`stm32f746`, `stm32h723`, `nrf52840`, `rp2040`, `rpi4`.

## Build Verification

```bash
# Format check
cargo fmt --all --check

# Clippy (all crates)
cargo clippy --package rustBoot --all-targets --features stm32f411 -- -D warnings
cargo clippy --package rbsigner --all-targets --all-features -- -D warnings
cargo clippy --package xtask --all-targets --features stm32f411 -- -D warnings

# Tests (rustBoot + rbsigner)
cargo test --package rustBoot --lib
cargo test --package rbsigner --all-features

# Integration tests
cargo test --package rustBoot

# Kani proofs
cd kani && cargo kani
```

## Verification Pipeline

All CI gates are defined in the `justfile`. The canonical command is `just verify`:

| Gate | Command | Description |
|------|---------|-------------|
| Format | `cargo fmt --all --check` | Rustfmt compliance |
| Clippy | `cargo clippy -D warnings` | Lint with deny-all-warnings |
| Unit tests | `cargo test --package rustBoot --lib` | 164 unit tests |
| Integration tests | `cargo test --package rustBoot` | 20 integration tests |
| rbsigner tests | `cargo test --package rbsigner` | 13 tests |
| Audit | `cargo audit` | Dependency vulnerability scan |
| Deny | `cargo deny check` | License and dependency ban check |
| Coverage | `cargo llvm-cov` | Line + MC/DC coverage |
| SBOM | `cargo cyclonedx` | CycloneDX software bill of materials |
| Kani | `cd kani && cargo kani` | 16 bounded proof harnesses |
| Fuzz | `cargo fuzz run <target>` | 5 fuzz targets |
| Stack analysis | `./scripts/stack_analysis.sh` | Stack depth via `cargo-call-stack` |
| QEMU smoke test | `./scripts/qemu_test.sh` | STM32F411 emulation boot test |

## Security

rustBoot is designed with defense in depth:

- **Memory safety**: entire core crate is `#![deny(unsafe_code)]`; no `unsafe`
  blocks in the boot path
- **No panics**: `#![deny(clippy::unwrap_used, clippy::expect_used,
  clippy::panic, clippy::todo, clippy::unimplemented)]` at crate root
- **Cryptographic chain**: SHA-256 integrity → ECDSA P-256 signature
  verification → anti-rollback version check
- **Fail closed**: all errors returned as typed enums; no silent fallback to
  untrusted images
- **Formal models**: TLA+ and Alloy specifications of the boot state machine
- **Fuzz-tested parsers**: TLV headers, FIT images, DTB blobs, config files
  tested against arbitrary/malformed inputs
- **Safety case**: comprehensive `docs/safety_case.md` with assumptions,
  hazards, risk assessment, and verification evidence

### Report a Vulnerability

See [SECURITY.md](SECURITY.md). Do **not** file public issues for security
vulnerabilities.

## Documentation

| Document | Path |
|----------|------|
| Architecture overview | `docs/architecture/architecture.md` |
| Safety case | `docs/safety_case.md` |
| Threat model (STRIDE) | `docs/threat-model/stride.md` |
| FMEA | `docs/fmea/fmea.md` |
| Requirements traceability | `docs/requirements/requirements.md` |
| Verification matrix | `docs/verification/test-matrix.md` |
| Formal models (TLA+) | `docs/formal/tla_plus/boot_state_machine.tla` |
| Formal models (Alloy) | `docs/formal/alloy/boot_state_machine.als` |
| QEMU testing guide | `docs/qemu_testing.md` |
| WCET / stack depth analysis | `docs/wcet.md` |
| Contributing guide | `CONTRIBUTING.md` |
| Security policy | `SECURITY.md` |
| rustBoot Book (WIP) | https://nihalpasham.github.io/rustBoot-book/index.html |

## License

MIT license. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributions are subject to the
MIT license terms.

### Quick Contribution Checklist

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] No `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` added
- [ ] All `unsafe` blocks have `// Safety:` comments
- [ ] New features include tests
- [ ] Documentation updated if behavior changes