#!/usr/bin/env bash
# Stack depth analysis for rustBoot
set -euo pipefail

echo "=== Stack Usage Analysis ==="

# Determine target triple
TARGET="${1:-thumbv7em-none-eabihf}"
FEATURES="${2:-stm32f411,nistp256,sha256}"

echo "Target: ${TARGET}"
echo "Features: ${FEATURES}"

if ! command -v cargo-call-stack &>/dev/null; then
    echo "cargo-call-stack not installed."
    echo "Install with: cargo install cargo-call-stack"
    echo ""
    echo "NOTE: cargo-call-stack v0.1.16 requires nightly-2023-11-13."
    echo "For newer toolchains, patch and rebuild from source:"
    echo "  git clone https://github.com/japaric/cargo-call-stack"
    echo "  cd cargo-call-stack"
    echo "  # Update SUPPORTED_NIGHTLY_HASH in src/main.rs to match your nightly"
    echo "  cargo install --path ."
    exit 1
fi

echo ""
echo "=== Running cargo-call-stack ==="

if cargo call-stack --package rustBoot --target "${TARGET}" \
    --features "${FEATURES}" --format top 2>&1; then
    echo ""
    echo "=== Stack analysis complete ==="
else
    echo ""
    echo "=== Fallback: static analysis ==="
    echo "cargo-call-stack failed (common on toolchain mismatch)."
    echo "Trying direct build with -Z emit-stack-sizes..."

    RUSTFLAGS="-Z emit-stack-sizes" \
        cargo build --package rustBoot --target "${TARGET}" \
        --features "${FEATURES}" --release -Z build-std=core,alloc 2>&1 || {
        echo ""
        echo "=== Manual stack analysis ==="
        echo "To analyze stack usage manually:"
        echo "1. Build with: RUSTFLAGS=\"-Z emit-stack-sizes\" cargo build ..."
        echo "2. Extract .stack_sizes section: llvm-readobj --stack-sizes <elf>"
        echo "3. Generate call graph with: cargo rustc -- --emit=llvm-ir"
        echo ""
        echo "See docs/wcet.md for details."
    }
fi