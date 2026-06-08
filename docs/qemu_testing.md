# QEMU Testing for rustBoot

## Overview
QEMU emulates ARM Cortex-M processors, allowing firmware testing without
physical hardware. This enables:
- CI-based smoke testing of firmware boot
- Regression testing of boot flow
- Early validation of firmware images

## Supported Targets

| Target | QEMU Machine | Status |
|--------|-------------|--------|
| stm32f411 | `-machine stm32f4xx` | Basic boot |
| stm32f446 | `-machine stm32f4xx` | Untested |
| rpi4 (AArch64) | `-machine raspi4b` | Needs AArch64 QEMU |

## Running Locally

```bash
./scripts/qemu_test.sh
```

## CI Integration
The QEMU test runs as a CI job with `continue-on-error: true` so toolchain
incompatibilities won't block the pipeline.

## Limitations
- QEMU does not perfectly emulate all peripherals
- Flash timing is not realistic
- Cannot test real-world timing-dependent behavior
- Semihosting support varies by machine model