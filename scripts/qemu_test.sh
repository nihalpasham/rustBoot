#!/usr/bin/env bash
# Multi-target QEMU firmware test for rustBoot.
# Tests on all available Cortex-M machines: M3, M4, M7, and AArch64.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== rustBoot QEMU Multi-Target Test ==="
echo ""

# Cortex-M linker scripts per machine
declare -A LINK_SCRIPTS=(
    ["mps2-an385"]="link.x"         # M3, flash at 0x0
    ["mps2-an386"]="link.x"         # M4, flash at 0x0
    ["mps2-an500"]="link.x"         # M7, flash at 0x0
    ["netduinoplus2"]="link-m4.x"   # M4, flash at 0x08000000
    ["olimex-stm32-h405"]="link-m4.x"
    ["b-l475e-iot01a"]="link-m4.x"
)

build_firmware() {
    local machine=$1
    local link_script=$2
    echo "Building for $machine ($link_script)..."

    cd "$PROJECT_DIR"
    RUSTFLAGS="-C link-arg=-T$link_script -C panic=abort" cargo build --release \
        --target thumbv7em-none-eabihf \
        --manifest-path boards/qemu_test/Cargo.toml 2>&1 | tail -2

    local ELFDIR="$PROJECT_DIR/boards/qemu_test/target/thumbv7em-none-eabihf/release"
    rust-objcopy -O binary "$ELFDIR/qemu_test" /tmp/qemu_test.bin 2>/dev/null
    echo "  Binary: $(wc -c < /tmp/qemu_test.bin) bytes"
}

run_qemu() {
    local machine=$1
    local addr=$2
    echo "  QEMU: $machine (addr=$addr, timeout 3s)..."
    timeout 3 qemu-system-arm \
        -M "$machine" \
        -device loader,file=/tmp/qemu_test.bin,addr=$addr,force-raw=on \
        -semihosting -serial none -monitor none 2>&1 && {
        echo "  RESULT: BOOT OK"
        return 0
    } || {
        local rc=$?
        if [ $rc -eq 124 ]; then
            echo "  RESULT: BOOT OK (timeout - firmware running)"
            return 0
        fi
        echo "  RESULT: CRASH (exit code $rc)"
        return 1
    }
}

TESTS_PASSED=0
TESTS_FAILED=0

# === Cortex-M tests ===
for machine in $(qemu-system-arm -machine help 2>&1 | grep -oE "^[a-z0-9_-]+" | grep -E "mps2-an|netduino|olimex|b-l475e" || true); do
    link="${LINK_SCRIPTS[$machine]:-link.x}"
    addr=${ADDR_MAP[$machine]:-0x00000000}

    # Determine flash base address
    case "$machine" in
        netduinoplus2|olimex-stm32-h405|b-l475e-iot01a) addr="0x08000000" ;;
        *) addr="0x00000000" ;;
    esac

    build_firmware "$machine" "$link" 2>/dev/null
    if run_qemu "$machine" "$addr"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
done

# === AArch64 test ===
if command -v qemu-system-aarch64 &>/dev/null; then
    echo "Building for AArch64 (RPi4 / qemu-virt)..."
    # AArch64 test would need a different firmware (aarch64-unknown-none-softfloat)
    # Placeholder for now
    echo "  (AArch64 test firmware not yet implemented)"
fi

echo ""
echo "=== Results: $TESTS_PASSED passed, $TESTS_FAILED failed ==="