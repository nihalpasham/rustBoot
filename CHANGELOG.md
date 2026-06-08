# Changelog

All notable changes to rustBoot are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- NATO-grade safety case (`docs/safety_case.md`) with 7 safety goals (G1-G7)
- Formal TLA+ and Alloy models of boot state machine (`docs/formal/`)
- 16 Kani proof harnesses for bounded verification
- 5 cargo-fuzz targets (TLV, FIT, DTB, config parser)
- QEMU integration testing for STM32 targets
- Stack depth analysis with `cargo-call-stack`
- Binary size CI gate with 64KB warning threshold
- Dependabot configuration for automated dependency updates
- Pre-commit hook configuration
- GitHub release workflow with binary artifacts
- Mermaid diagrams for boot flow, state machine, partition layout, image format
- Comprehensive ARCHITECTURE.md, updated README, CONTRIBUTING, SECURITY docs

### Changed
- `#![deny(unsafe_code)]` at crate root; 7 module-level permits with safety cases
- 8 unsafe blocks eliminated (made safe via safe byte-order operations)
- Test count: 63 → 184 (164 unit + 20 integration)
- Kani proofs: 2 → 16
- Fuzz targets: 2 → 5
- CI pipeline: 5 gates → 12 gates
- `defmt-rtt` dependency: 0.3.2 → 0.4.0 (fixes yanked critical-section)
- README: default target nrf52840 → stm32f411

### Fixed
- All 270 clippy warnings eliminated (0 remaining)
- All panic/todo/unimplemented in fit.rs replaced with typed errors
- rbsigner: all unwrap/expect/panic replaced with typed RbSignerError
- as-slice crate removed (Vec/array .as_slice() available in std since Rust 1.57)
- Edition 2021 migration complete
- All dependencies upgraded to latest compatible versions

### Security
- ECDSA P-256 verification with 7 unit tests (good/bad/malformed/wrong-key)
- SHA-256 integrity checks
- Anti-rollback via version numbering
- STRIDE threat model with 20+ threats documented
- FMEA with RPN scoring
- Full requirements traceability matrix