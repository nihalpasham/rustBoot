# WCET and Stack Depth Analysis

## Stack Usage

### Current Results

Stack depth analysis requires `cargo-call-stack` (v0.1.16), which needs
nightly-2023-11-13 or a patched build for newer toolchains.

Run:
```bash
./scripts/stack_analysis.sh
```

### Methodology

Stack depth is computed statically by `cargo-call-stack`, which:
1. Builds the entire crate with `-Z emit-stack-sizes` (nightly flag)
2. Parses the `.stack_sizes` ELF section from every compilation unit
3. Builds a whole-program call graph by analyzing LLVM-IR
4. For each root → leaf path, sums the stack frames

The tool uses `-Z build-std` to get stack sizes from `core`, `alloc`, and
`compiler_builtins` — critical since the bootloader is a `no_std` crate.

### Known Stack-Deep Paths

The deepest call chains typically involve:

1. **ECDSA verification** (`verify_ecc256_signature`)
   - Field arithmetic (elliptic-curve crate): ~1-2KB
   - SHA-256 hashing (sha2 crate): ~512B
   - Signature parsing: ~256B

2. **FIT image parsing** (`parse_fit`)
   - FDT blob walking (nom parser): ~1KB
   - Hash computation chain: ~512B
   - Verification dispatch: ~256B

3. **TLV image parsing** (`parse_tlv`)
   - nom combinator nesting: ~768B
   - Version/timestamp extraction: ~256B

4. **Boot state machine** (`rustboot_start`)
   - Partition descriptor I/O: ~256B
   - State transition logic: ~128B

### Stack Size Budget

| Target       | SRAM    | Typical Stack Budget |
|--------------|---------|---------------------|
| STM32F411    | 128KB   | 8KB                 |
| STM32F446    | 128KB   | 8KB                 |
| STM32F469    | 384KB   | 16KB                |
| STM32H723    | 564KB   | 32KB                |
| STM32F746    | 340KB   | 16KB                |
| STM32F334    | 64KB    | 4KB                 |
| nRF52840     | 256KB   | 16KB                |
| RP2040       | 264KB   | 8KB                 |
| RPi4         | 1GB+    | 128KB               |

## Alternative: Manual Stack Analysis

When `cargo-call-stack` cannot be used (e.g., toolchain mismatch), stack depth
can be estimated manually:

1. **Build with stack sizes:**
   ```bash
   RUSTFLAGS="-Z emit-stack-sizes" cargo build --target thumbv7em-none-eabihf \
       --features "stm32f411,nistp256,sha256" --release
   ```

2. **Extract `.stack_sizes` section:**
   ```bash
   rust-objdump --section=.stack_sizes target/thumbv7em-none-eabihf/release/rustBoot
   ```

3. **Sum frames per call chain:**
   Recover the call graph from LLVM-IR emitted by:
   ```bash
   cargo rustc -- --emit=llvm-ir
   ```
   Then trace each `call` instruction and sum the corresponding stack sizes.

### Known Limitations

- Recursive functions are conservatively estimated
- `#[inline]` and generic monomorphization create the most call paths
- Interrupt handlers and nested exceptions add separate stack frames
- The Rust compiler's register allocation affects actual stack usage
- LTO can eliminate or inline frames, reducing stack depth