---
title: Volatile Key Material & Zeroization
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - FIPS-140-3-Level-3
bootloader: true
firmware: shared
hardware: false
---

# Volatile Key Material & Zeroization

## Summary

All cryptographic key material must be volatile: it exists only in CPU
registers or stack memory during its active window, and is explicitly
zeroized (overwritten with zeros) immediately after use. No key material
persists across function calls, boot cycles, or exception handlers.

## Motivation

FIPS 140-3 Level 3 requires that Critical Security Parameters (CSPs) be
zeroized when no longer needed. Key material left in memory is extractable
via:
- Debug probe (blocked by RDP Level 2, but defense-in-depth)
- DMA readout by malicious on-chip peripherals
- Cold boot attack (RAM retains data for seconds after power-off)
- Fault injection that redirects execution to a memory dump routine

## Design

### Key Lifetime

```
allocate_key_buffer() → [key in stack] → use_key() → zeroize() → scope end
       │                                         │
       │                                         └── compiler_fence()
       │                                             (prevents reordering)
       └── must not be static/global
```

### Rules

1. **No static key buffers**: Keys must never be stored in `static` variables.
   All key buffers are stack-local.
2. **No global key references**: `&'static [u8]` to key data is forbidden.
3. **Immediate zeroization**: Zeroize within the same function that uses the
   key, before returning.
4. **Compiler fence**: A `compiler_fence(SeqCst)` prevents the compiler from
   reordering the zeroization past any subsequent operations.
5. **Volatile write**: Each byte is written with `write_volatile`.
6. **Exception safety**: Exception handlers zeroize any in-progress key buffer
   before performing other recovery actions.

### Implementation

```rust
#[derive(Derivative)]
#[derivative(Drop)]
pub struct KeyBuffer<const N: usize> {
    #[derivative(Drop = "custom")]
    data: [u8; N],
}

impl<const N: usize> Drop for KeyBuffer<N> {
    fn drop(&mut self) {
        // SAFETY: write_volatile prevents compiler from optimizing
        // away the zeroization. FIPS 140-3 §7.9.7 requirement.
        for byte in self.data.iter_mut() {
            unsafe {
                core::ptr::write_volatile(byte, 0);
            }
        }
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}
```

### Exception Safety

On HardFault, MemManage, or BusFault during a cryptographic operation:

```rust
#[exception]
fn HardFault(_frame: &ExceptionFrame) -> ! {
    // SAFETY: This is the last opportunity to zeroize before system reset.
    // We clear a fixed address range where key data may reside.
    // The IWDG will reset the system shortly if we don't.
    zeroize_key_area();
    loop {
        // Wait for IWDG reset
    }
}
```

## Verification

### Testing Requirements

- Allocate `KeyBuffer<32>`, write key, drop → assert all bytes are 0
- Key is not accessible after drop (compiler error if referenced)
- No `static` references to key data exist (checked by `#[deny(static_mut_refs)]`)
- Exception handler zeroization: invoke HardFault during key use → assert key
  area is cleared after fault handler (measured by LED/blink pattern)

### Kani Proofs

| Harness | Property |
|---------|----------|
| `key_buffer_drop_zeroes` | After `drop()`, all N bytes are 0 |
| `key_buffer_no_leak` | After drop, no copies of key data remain on stack |
| `zeroize_with_volatile` | `write_volatile` is used for every byte (proven by MIR analysis) |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Zeroization is not elided | Creusot | MIR contains the zeroing store instructions after Drop::drop |
| No reference to key outlives buffer | Verus | Borrow checker ensures lifetime constraints |
| Exception zeroization is reachable | Verus | HardFault handler is called for every exception type |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `key_buffer_drop_patterns` | Various key buffer sizes and contents → verify zeroed |

### Proptests

| Property | Check |
|----------|-------|
| `key_buffer_zeroed_regardless_of_content` | Any content is zeroed after drop |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- Stack memory: after zeroization, the physical RAM still retains a residual
  charge for milliseconds. Cold boot attacks with DRAM are not applicable; SRAM
  (used in STM32) fades quickly but not instantaneously. The IWDG reset
  combined with zeroization minimizes this window.
- Compiler optimizations: `compiler_fence` and `write_volatile` prevent
  optimization but require manual verification for every new compiler version.
  Adding `#[inline(never)]` and Creusot MIR analysis as additional checks.