# High-Assurance rustBoot Bootloader Agent

## Mission

Harden [vlordier/rustBoot](https://github.com/vlordier/rustBoot) — a no_std, embedded-safe Rust bootloader for STM32, nRF52840, RP2040, and RPi4 (AArch64) — to NATO-grade software assurance.

`rustBoot` implements secure A/B firmware updates with ECDSA (NIST P-256) image verification via device tree (FIT) or custom TLV image formats on Cortex-M and AArch64 targets.

The goal is not just "make it compile." The goal is:

- protocol correctness (FIT/ITB, TLV image format)
- deterministic embedded boot behavior
- bounded memory use (no_std, no alloc)
- no uncontrolled panics in core boot path
- robust image parsing (no malformed-image exploits)
- verifiable A/B swap state machines
- excellent test evidence (unit, property, fuzz, formal)
- MC/DC coverage on safety-critical logic
- reproducible builds with SBOM

## Repository Layout

```
rustBoot/src/
  lib.rs              — crate root, safety lint denies
  image/image.rs      — boot state machine, partition mgmt
  image/parse.rs      — TLV header parser (nom-based)
  crypto/signatures.rs — ECDSA verifier (NistP256, Secp256k1)
  dt/fit.rs           — FIT image parser
  dt/reader.rs        — device tree reader
  dt/writer.rs        — device tree writer
  dt/patch.rs         — device tree patching
  cfgparser.rs        — update config parser
boards/update/src/update/
  update_flash.rs     — A/B swap logic, rustboot_start()
boards/bootloaders/   — per-MCU main.rs (stm32*, nrf52840, rp2040, rpi4)
boards/hal/           — HAL abstractions, register maps (nxp, stm32, nrf, rpi)
rbsigner/src/         — CLI signing tool
rbsigner/src/curve.rs — key types
rbsigner/src/mcusigner.rs — MCU image signing
rbsigner/src/fitsigner.rs  — FIT image signing
docs/threat-model/    — STRIDE threat model
docs/fmea/            — FMEA with RPN
docs/requirements/    — requirements traceability
```

## Current State (PR1-PR9 merged)

- 0 warnings, 0 errors on both `rustBoot` and `rbsigner` crates
- 76/76 tests passing (rustBoot 63, rbsigner 13)
- `#![deny(clippy::unwrap_used|expect_used|panic|todo|unimplemented)]` at crate root
- All `unsafe` blocks documented with safety cases
- Property tests (proptest), fuzz harnesses (cargo-fuzz), Kani proof skeletons
- MC/DC coverage CI with baseline line-coverage threshold (--fail-under-linenum 26)
- Build reproducibility script (sbom.sh, verify.sh)
- cargo-deny, cargo-audit, cargo-machete in CI
- All dependencies upgraded (p256 0.13, sha2 0.10, nom 8, signature 2.2, defmt 1.0, byteorder 1.5)
- Edition 2021 migration complete
- Comprehensive security documentation (SECURITY.md, threat model, FMEA, requirements traceability, architecture docs, verification matrix)

## Hard Rules

- Keep `no_std` compatibility in core `rustBoot` crate.
- Do not introduce `std` into core crates (only `rbsigner`/`xtask` may use std).
- Do not introduce heap allocation (`alloc` feature-gated if needed).
- Do not introduce uncontrolled `panic!`, `unwrap`, `expect`, `todo!`, `unimplemented!`.
- Do not add blocking behavior without explicit timeout semantics in HAL.
- Do not silently drop boot images or update states unless documented and tested.
- Do not weaken security guarantees (ECDSA verification, SHA-256 integrity) to simplify.
- Do not hide unsupported features; expose as explicit errors or feature-gated TODOs.
- Do not merge code without tests.

## Required Linting

Every crate (rustBoot, rbsigner, xtask, boards/update, boards/hal) must enforce:

```rust
#![deny(warnings)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
```

`unsafe` blocks require:

```rust
// SAFETY:
// - explain invariant
// - explain caller obligations
// - explain why safe Rust cannot express this
// - link to test or verification evidence
```

## Agent Workflow

For every change:

1. Inspect the existing implementation.
2. Identify affected boot or protocol behavior.
3. Compare against expected behavior (upstream rustBoot behavior, FIT spec).
4. Add unit tests.
5. Add property tests if parsing/state logic touched.
6. Add fuzz cases if parsing/decoding is touched.
7. Add Kani checks for bounded buffer/state logic.
8. Run all quality gates (clippy, test, audit, deny, coverage).
9. Document residual risk.

## Required Test Categories

### Unit Tests

Required for:
- image header parsing (TLV, FIT)
- boot state transitions (A/B swap)
- ECDSA signature verification
- SHA-256 integrity checks
- partition descriptor management
- TLV encoding/decoding
- error type conversion
- device tree FDT block parsing
- FIT hash/authentication verification

### Property Tests with `proptest`

Use for:
- parser never panics on arbitrary bytes
- decode rejects invalid TLV/FIT headers safely
- encode/decode roundtrip for image headers
- serialized length is bounded
- state transitions preserve invariants
- counters do not overflow silently
- malformed images never produce valid boot states

### Fuzzing

Use `cargo fuzz` for:
- TLV image header parser
- FIT image parser
- device tree parser
- update configuration parser

Existing fuzz targets in `fuzz/`:
```
fuzz_targets/fit_parser.rs
fuzz_targets/tlv_parser.rs
fuzz_targets/dtb_parser.rs
```

### Kani

Use for bounded verification of:
- fixed-capacity image buffers
- boot state machine transitions
- sequence arithmetic (partition offsets)
- parser cursor logic
- bounds-checked serialization

### MC/DC Coverage

The CI runs MC/DC coverage via `cargo-llvm-cov` with `--mcdc`. Threshold: `--fail-under-linenum 26`.

## Required CI Commands (justfile)

```bash
just fmt-check       # cargo fmt --check
just clippy           # cargo clippy —workspace —all-targets —all-features — -D warnings
just test             # cargo test —workspace —all-features
just audit            # cargo audit
just deny             # cargo deny check
just coverage         # cargo llvm-cov with threshold
just sbom             # generate SBOM
just verify           # verify build reproducibility
```

For embedded targets:

```bash
just nrf52840 build
just stm32f411 build
just rpi4 build
```

## Boot Safety Checklist

For every boot feature:

```text
[ ] malformed image rejected (TLV, FIT)
[ ] truncated image rejected
[ ] oversized image rejected (partition bounds)
[ ] invalid signature rejected
[ ] invalid SHA-256 hash rejected
[ ] unknown/supported feature returns explicit error
[ ] no panic path in core boot flow
[ ] no unchecked arithmetic (partition math)
[ ] no unbounded allocation
[ ] deterministic boot behavior
[ ] fuzz target added if parser touched
[ ] property test added if state machine touched
[ ] formal check considered if bounded logic

## Embedded Runtime Checklist

```text
[ ] no_std build passes
[ ] panic strategy documented (abort on fatal)
[ ] allocator not required (or feature-gated)
[ ] all buffers are bounded
[ ] all partition sizes checked against flash geometry
[ ] boot failure has safe reset path (rollback)
[ ] logging/tracing is feature-gated (defmt, log)
[ ] binary size checked
```

## Specific Areas to Improve

### 1. Parser Hardening

- nom 8 parser: no panics on arbitrary bytes (current state)
- TLV header: no cursor overrun, no integer overflow in length fields
- FIT parser: no invalid hash promoted to valid verification
- Device tree parser: no stack overflow on deeply nested DTBs

### 2. Boot State Machine Safety

Represent A/B partition state explicitly:

```rust
enum BootState {
    BootInNewState,       // fresh image, needs verification
    BootInTestingState,   // image under test, can rollback
    BootInSuccessState,   // confirmed good
    UpdateInNewState,     // new update available
    UpdateInUpdatingState,// update in progress
}
```

Invalid transitions must return typed errors.

### 3. Image Verification

- ECDSA P-256: verify against known-good public keys
- SHA-256: verify firmware integrity
- Image authenticity: signature + hash chain

### 4. Rollback Safety

- Testing-state images rollback on failed boot
- Update failure reverts to previous known-good
- Partition descriptor state management (flash writes)

### 5. Feature Flags

```text
default = []
nrf52840 / stm32f411 / stm32f446 / stm32f469 / stm32h723 / stm32f746 / stm32f334 / rp2040 / rpi4 = []
nistp256 / secp256k1 = []
defmt / log = []
```

Core protocol logic remains `no_std`.

## Definition of Done

A change is done only when:

```text
[ ] no_std build passes
[ ] std test build passes (rbsigner)
[ ] clippy passes with denied warnings
[ ] no unwrap/expect/panic/todo/unimplemented in core logic
[ ] unit tests added
[ ] proptest properties added if parsing/state logic touched
[ ] fuzz target added for parsers
[ ] Kani harness added for bounded logic
[ ] unsupported behavior explicitly documented
[ ] residual risk documented
```

## Agent Response Format

After each task, report:

```text
Changed files:
- ...

Boot behavior affected:
- ...

Verification evidence:
- ...

Tests added:
- ...

Fuzzing added:
- ...

Formal verification added:
- ...

Commands run:
- ...

Known gaps:
- ...

Next hardening step:
- ...
```

## Immediate Next Steps

1. Keep all safety lint denies active; fix any regressions immediately.
2. Eliminate remaining `#[allow(clippy::*)]` on panic/wrap/expect/todo/unimplemented.
3. Improve MC/DC coverage (currently 29.32% baseline).
4. Add more Kani proof harnesses (partition math, state transitions).
5. Extend proptest rounds and fuzz corpus.
6. Upgrade remaining `#[allow(deprecated)]` items (generic-array re-export).
7. Formalize the vendored `aarch64-cpu` fork (upgrade to v11.2.0, extract `mair_el3` locally).
8. Document remaining gaps: hardware-dependent features, `rustboot_start()` diverging return type.