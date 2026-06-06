#!/usr/bin/env bash
# Full verification pipeline for rustBoot.
#
# Usage: ./scripts/verify.sh
# Prerequisites: cargo-audit, cargo-deny (optional but recommended)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "===================================================="
echo "  rustBoot Verification Pipeline"
echo "===================================================="
echo ""

# 1. Formatting
echo "==> [1/7] Checking formatting..."
cargo fmt --all --check

# 2. Clippy
echo "==> [2/7] Running clippy (nrf52840)..."
cargo clippy --package rustBoot --all-targets --features nrf52840 -- -D warnings
echo "==> [2/7] Running clippy (rbsigner)..."
cargo clippy --package rbsigner --all-targets --all-features -- -D warnings
echo "==> [2/7] Running clippy (xtask)..."
cargo clippy --package xtask --all-targets --features nrf52840 -- -D warnings

# 3. Unit tests
echo "==> [3/7] Running unit tests..."
cargo test --package rustBoot --lib
# Also test parser per-feature
for feature in nrf52840 stm32f411 stm32f446 stm32f469 stm32h723 stm32f746 stm32f334 rp2040; do
    echo "  -> Testing with feature: $feature"
    cargo test --package rustBoot --lib --features "$feature" -- parser::tests --nocapture
done

# 4. Audit
echo "==> [4/7] Checking dependencies for vulnerabilities..."
if command -v cargo-audit &>/dev/null; then
    cargo audit
else
    echo "  SKIP: cargo-audit not installed (install: cargo install cargo-audit)"
fi

# 5. License check
echo "==> [5/7] Checking licenses..."
if command -v cargo-deny &>/dev/null; then
    cargo deny check
else
    echo "  SKIP: cargo-deny not installed (install: cargo install cargo-deny)"
fi

# 6. SBOM generation
echo "==> [6/7] Generating SBOM..."
./scripts/sbom.sh

# 7. Summary
echo ""
echo "===================================================="
echo "  Verification Complete"
echo "===================================================="

echo ""
echo "NOTE: Some checks are skipped if required tools are not installed."
echo "For full verification, install: cargo-audit, cargo-deny, cargo-cyclonedx"