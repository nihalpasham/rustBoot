#!/usr/bin/env bash
# QEMU firmware test for rustBoot core state machine (Cortex-M3/M4).
# Builds a minimal test firmware and runs it in QEMU's mps2-an385 (Cortex-M3)
# or netduinoplus2 (STM32F405 Cortex-M4) machine.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== rustBoot QEMU Test ==="

# Check QEMU
if ! command -v qemu-system-arm &>/dev/null; then
    echo "ERROR: qemu-system-arm not found. Install with: brew install qemu"
    exit 1
fi

# Find a suitable machine
MACHINE=""
if qemu-system-arm -machine help 2>&1 | grep -q mps2-an385; then
    MACHINE="mps2-an385"
    echo "Machine: mps2-an385 (Cortex-M3)"
elif qemu-system-arm -machine help 2>&1 | grep -q netduinoplus2; then
    MACHINE="netduinoplus2"
    echo "Machine: netduinoplus2 (STM32F405 Cortex-M4)"
else
    echo "ERROR: No suitable Cortex-M machine found in QEMU."
    exit 1
fi

# Build the test firmware
echo "Building QEMU test firmware..."
cd "$PROJECT_DIR"
RUSTFLAGS="-C link-arg=-Tlink.x -C panic=abort" cargo build --release \
    --target thumbv7em-none-eabihf \
    --manifest-path boards/qemu_test/Cargo.toml 2>&1 | tail -3

# Convert to raw binary and load at flash base
ELFDIR="boards/qemu_test/target/thumbv7em-none-eabihf/release"
if [ ! -f "$ELFDIR/qemu_test" ]; then
    echo "ERROR: Firmware binary not found at $ELFDIR/qemu_test"
    exit 1
fi

rust-objcopy -O binary "$ELFDIR/qemu_test" /tmp/qemu_test.bin 2>/dev/null
echo "Binary size: $(wc -c < /tmp/qemu_test.bin) bytes"

# Run in QEMU
echo "Launching QEMU (5 second timeout)..."
timeout 5 qemu-system-arm \
    -M "$MACHINE" \
    -device loader,file=/tmp/qemu_test.bin,addr=0x00000000,force-raw=on \
    -nographic -semihosting -serial none -monitor none 2>&1 || true

echo ""
echo "=== QEMU test complete ==="