# Canonical build and verification commands for rustBoot
#
# Usage: just <command>
# Requires `just` (https://github.com/casey/just)

default: verify

# Run all verification gates (fmt, clippy, test, audit, deny)
verify:
    cargo fmt --all --check
    cargo clippy --package rustBoot --all-targets --features nrf52840 -- -D warnings
    cargo clippy --package rbsigner --all-targets --all-features -- -D warnings
    cargo clippy --package xtask --all-targets --features nrf52840 -- -D warnings
    cargo test --package rustBoot --lib
    cargo test --package rustBoot --lib --features nrf52840 -- parser::tests --nocapture
    cargo test --package rbsigner --all-features
    cargo audit
    cargo deny check

# Build the rustBoot library for host
build-lib:
    cargo build --package rustBoot

# Build all bootloader firmware for a given board e.g. `just build nrf52840`
build board:
    cargo run -p xtask --features {{board}} -- {{board}} build rustBoot-only

# Run unit tests for all supported boards
test-all:
    cargo test --package rustBoot --lib
    cargo test --package rustBoot --lib --features nrf52840 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32f411 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32f446 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32f469 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32h723 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32f746 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features stm32f334 -- parser::tests --nocapture
    cargo test --package rustBoot --lib --features rp2040 -- parser::tests --nocapture

# Run cargo-audit to check for vulnerable dependencies
audit:
    cargo audit

# Run cargo-deny to check licenses and dependency bans
deny:
    cargo deny check

# Generate SBOM (Software Bill of Materials)
sbom:
    cargo install cargo-cyclonedx
    cargo cyclonedx --format json --output-file sbom.cdx.json

# Format all source code
fmt:
    cargo fmt --all

# Run clippy for all supported configurations
clippy:
    cargo clippy --package rustBoot --all-targets --features nrf52840 -- -D warnings
    cargo clippy --package rbsigner --all-targets --all-features -- -D warnings
    cargo clippy --package xtask --all-targets --features nrf52840 -- -D warnings