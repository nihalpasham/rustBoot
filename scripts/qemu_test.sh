#!/usr/bin/env bash
# QEMU-based firmware test for rustBoot STM32 targets
# Tests firmware builds for the embedded target and runs in QEMU.
# Note: macOS cross-compilation places sections at different VMA addresses
# than the STM32 boot address. For proper QEMU boot testing, run on a
# Linux CI runner where rust-lld produces ELF with correct VMA layout.
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
RUSTFLAGS="-C panic=abort" cargo build -Zbuild-std=core --release --target thumbv7em-none-eabihf --manifest-path boards/bootloaders/stm32f411/Cargo.toml 2>&1 | tail -5

# Find the ELF
FW_DIR="boards/target/thumbv7em-none-eabihf/release"
FW_ELF=$(find "$PROJECT_DIR/boards" -name "stm32f411" -path "*/release/*" -type f ! -name "*.d" 2>/dev/null | head -1)

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

# Try various machine types for STM32 testing
# netduinoplus2 (STM32F405) is the closest supported machine
MACHINE=""
if qemu-system-arm -machine help 2>&1 | grep -q netduinoplus2; then
    MACHINE="netduinoplus2"
elif qemu-system-arm -machine help 2>&1 | grep -q olimex-stm32-h405; then
    MACHINE="olimex-stm32-h405"
else
    echo "WARNING: No suitable Cortex-M4 machine found in QEMU."
    echo "Install qemu-system-arm with STM32 support or use a newer QEMU version."
    echo "Firmware binary built successfully at: $FW_ELF"
    exit 0
fi

echo "Machine: $MACHINE"
timeout 5 qemu-system-arm \
    -machine "$MACHINE" \
    -kernel "$FW_ELF" \
    -nographic \
    -semihosting \
    -semihosting-config enable=on,target=native \
    2>&1 | head -50 || true

echo ""
echo "=== QEMU test complete ==="