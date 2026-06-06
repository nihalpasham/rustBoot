![GitHub](https://img.shields.io/github/license/nihalpasham/rustBoot)
[![ci](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml/badge.svg)](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml)

# rustBoot — Secure Bootloader for Embedded Systems

rustBoot is a standalone secure bootloader written entirely in Rust, designed for
microcontrollers through system-on-chip, supporting bare-metal firmware and Linux
booting. It provides A/B firmware update with cryptographic verification,
anti-rollback, and power-interruptible swap.

## Operational Scope

rustBoot is a **secure bootloader** intended for hardened embedded deployments.
It is not a general-purpose bootloader — it prioritizes:

- Cryptographic integrity and authenticity verification (ECDSA, SHA-256)
- Deterministic boot flow with explicit state machine
- Anti-rollback via version numbering
- Power-fail-safe firmware updates with automatic fallback
- Memory-safe core in Rust (no_std)

## Supported Targets

| Target | Architecture | Boot Mode |
|--------|-------------|-----------|
| nRF52840 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F334 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F411 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F446 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F469 | ARM Cortex-M4F | Bare-metal firmware |
| STM32F746 | ARM Cortex-M7F | Bare-metal firmware |
| STM32H723 | ARM Cortex-M7F | Bare-metal firmware |
| RP2040 | ARM Cortex-M0+ | Bare-metal firmware |
| Raspberry Pi 4 | AArch64 | Linux FIT image |
| i.MX 8M Nano | AArch64 | Linux FIT image |

## Features

- ARM Cortex-M, Cortex-A, AArch64 support
- Multi-slot flash partitioning (boot/update/swap)
- ECDSA signature verification (NIST P-256, secp256k1, ed25519 planned)
- SHA-256/384 integrity verification
- Anti-rollback via version numbering
- Power-interruptible A/B firmware swap with fallback
- Flattened Image Tree (FIT) support for Linux booting
- Device tree (DTB) parsing, patching, and writing
- Signed firmware generation utility (`rbsigner`)

## Prerequisites

- Rust nightly toolchain (see `rust-toolchain.toml`)
- `arm-none-eabi` or appropriate cross-compilation toolchain for target
- For flashing: `probe-rs-cli` or `pyocd`
- For full verification: `cargo-audit`, `cargo-deny`, `cargo-cyclonedx`

## Quick Start

```bash
# Verify the build works (replace nrf52840 with your target)
cargo run -p xtask --features nrf52840 -- nrf52840 build rustBoot-only

# Full verification pipeline
cargo fmt --all --check
cargo clippy --package rustBoot --all-targets --features nrf52840 -- -D warnings
cargo test --package rustBoot --lib
```

## Build

```bash
cargo run -p xtask --features <board> -- <board> build [rustBoot-only|pkgs-for]
```

Supported boards: `nrf52840`, `stm32f411`, `stm32f446`, `stm32f469`, `stm32h723`,
`stm32f746`, `stm32f334`, `rp2040`, `rpi4`.

## Verification Pipeline

The canonical verification command is `just verify`, which runs:

- `cargo fmt --all --check`
- `cargo clippy` with deny warnings
- `cargo test`
- `cargo audit`
- `cargo deny check`

See `justfile` for all available commands.

## Security

- **Report vulnerabilities**: See `SECURITY.md`
- **Threat model**: `docs/threat-model/`
- **FMEA**: `docs/fmea/`
- **Requirements traceability**: `docs/requirements/`

## Documentation

- [rustBoot Book](https://nihalpasham.github.io/rustBoot-book/index.html) (work in progress)

## License

MIT license. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributions are subject to the
MIT license terms.