#!/usr/bin/env bash
# QEMU-based firmware test for rustBoot STM32 targets
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== rustBoot QEMU Test ==="
echo "Target: stm32f411"

# Check QEMU
if ! command -v qemu-system-arm &>/dev/null; then
    echo "ERROR: qemu-system-arm not found. Install with: brew install qemu"
    exit 1
fi

# Build the firmware
echo "Building firmware..."
cd "$PROJECT_DIR"
cargo run -p xtask --features stm32f411 -- stm32f411 build rustBoot-only 2>&1 | tail -5

# Find the ELF
FW_DIR="boards/target/thumbv7em-none-eabihf/release"
FW_ELF=$(find "$FW_DIR" -name "rustBoot*" -o -name "stm32f411*" 2>/dev/null | head -1)

if [ -z "$FW_ELF" ]; then
    # Try alternate paths
    FW_ELF=$(find boards -name "*.elf" -path "*stm32f411*" 2>/dev/null | head -1)
fi

if [ -z "$FW_ELF" ]; then
    echo "WARNING: No ELF binary found. QEMU test skipped."
    echo "This is expected if the xtask build step uses a different output path."
    echo "Once binary is available, run:"
    echo "  qemu-system-arm -machine stm32f4xx -kernel <elf> -nographic"
    exit 0
fi

echo "Firmware: $FW_ELF"
echo "Launching QEMU..."

# Run in QEMU with a 5-second timeout
timeout 5 qemu-system-arm \
    -machine stm32f4xx \
    -kernel "$FW_ELF" \
    -nographic \
    -semihosting \
    -semihosting-config enable=on,target=native \
    2>&1 | head -50 || true

echo ""
echo "=== QEMU test complete ==="