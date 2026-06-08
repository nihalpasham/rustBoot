# rustBoot Architecture

## System Overview

rustBoot is a `no_std` embedded-safe Rust bootloader implementing secure A/B firmware updates with ECDSA (NIST P-256 / secp256k1) image verification. It supports Cortex-M MCUs (STM32, nRF52840, RP2040) and AArch64 targets (RPi4, iMX8M Nano) with two image format options: a native TLV format and the Linux-standard FIT (Flattened Image Tree) format. The core crate enforces `#![deny(clippy::unwrap_used|expect_used|panic|todo|unimplemented)]` at the crate root and bans crate-level `unsafe` code.

## Boot Flow

```
Power-on Reset
     │
     ▼
Boot ROM loads rustBoot (hardware entry point)
     │
     ▼
MCU-specific init (clocks, GPIO, UART, MMU for AArch64)
     │
     ▼
Partitions opened via open_partition() → BOOT, UPDATE, SWAP
     │
     ▼
State machine evaluates ImageType variant:
  ├─ BootInNewState       → verify integrity + authenticity → mark Testing → boot
  ├─ BootInTestingState   → previous update still unconfirmed → verify → if OK:
  │                           mark Success → boot; if fail: rollback
  ├─ BootInSuccessState   → confirmed good → verify → boot
  ├─ UpdateInNewState     → new update present → mark Updating
  ├─ UpdateInUpdatingState → swap BOOT ↔ UPDATE via scratch sector
  └─ NoStateSwap          → SWAP partition has no state
     │
     ▼
Image integrity check (SHA-256 hash against TLV Digest256)
     │
     ▼
Image authenticity check (ECDSA P-256 / secp256k1 signature)
     │
     ▼
Jump to firmware entry point (or rollback on failure)
```

## Partition Layout

Three physical partitions in flash:

```
┌──────────────────────────────────────────────┐
│ BOOT Partition   │ Active firmware image     │
│                  │ (verified + running)       │
├──────────────────────────────────────────────┤
│ UPDATE Partition │ Staged update image        │
│                  │ (downloaded, not yet booted)│
├──────────────────────────────────────────────┤
│ SWAP Partition   │ Scratch space for A/B swap │
│                  │ (sector-by-sector copy)     │
└──────────────────────────────────────────────┘
```

Each partition stores:
- **Header** (256 bytes): Magic (`RUST` = `0x54535552`), size, TLV entries
- **Firmware payload**: Sector-aligned binary
- **Trailer** (8 bytes): Magic (`BOOT` = `0x544F4F42`), state byte, sector flags

Per-MCU flash geometry (from `constants.rs`):

| Target      | Sector Size | Partition Size | Boot Addr    | Update Addr  | Swap Addr    |
|-------------|-------------|----------------|--------------|--------------|--------------|
| nRF52840    | 4 KB        | 160 KB         | 0x2F000      | 0x58000      | 0x57000      |
| STM32F411   | 128 KB      | 128 KB         | 0x08020000   | 0x08040000   | 0x08060000   |
| STM32F446   | 128 KB      | 128 KB         | 0x08020000   | 0x08040000   | 0x08060000   |
| STM32F469   | 128 KB      | 384 KB         | 0x08020000   | 0x08080000   | 0x080E0000   |
| STM32H723   | 128 KB      | 256 KB         | 0x08020000   | 0x08060000   | 0x080A0000   |
| STM32F746   | 256 KB      | 256 KB         | 0x08040000   | 0x08080000   | 0x080C0000   |
| STM32F334   | 6 KB        | 6 KB           | 0x0800B800   | 0x0800D000   | 0x0800E800   |
| RP2040      | 4 KB        | 64 KB          | 0x10010000   | 0x10020000   | 0x10030000   |

## State Machine

The boot state machine is encoded in the `ImageType` enum with 6 valid partition-state pairs, using the sealed trait pattern to restrict valid transitions at compile time:

```
                    ┌──────────────┐
                    │  BootInNew   │
                    │  (verify +   │
                    │   authenticate)│
                    └──────┬───────┘
                           │ into_testing_state()
                    ┌──────▼───────┐
                    │BootInTesting │◄──────────────┐
                    │ (pending     │                │
                    │  confirmation)│               │
                    └──────┬───────┘               │
                           │ into_success_state()   │
                    ┌──────▼───────┐               │
                    │BootInSuccess │───────────────┘
                    │ (confirmed   │ into_testing_state()
                    │  good)       │
                    └──────────────┘

                    ┌──────────────┐
                    │UpdateInNew   │
                    └──────┬───────┘
                           │ into_updating_state()
                    ┌──────▼───────┐
                    │UpdateIn      │
                    │Updating      │ → sector swap
                    └──────────────┘

                    ┌──────────────┐
                    │  NoStateSwap │
                    │  (scratch)   │
                    └──────────────┘
```

### State byte values (stored in partition trailer):

| State          | Byte Value |
|----------------|------------|
| StateNew       | 0xFF       |
| StateUpdating  | 0x70       |
| StateTesting   | 0x10       |
| StateSuccess   | 0x00       |
| NoState        | (N/A)      |

### Valid transitions:

| From              | To                | Method                  | Partition |
|-------------------|-------------------|-------------------------|-----------|
| BootInNewState    | BootInTestingState  | `into_testing_state()`  | Boot      |
| BootInNewState    | BootInSuccessState  | `into_success_state()`  | Boot      |
| BootInTestingState | BootInSuccessState | `into_success_state()`  | Boot      |
| BootInSuccessState | BootInTestingState | `into_testing_state()`  | Boot      |
| UpdateInNewState  | UpdateInUpdatingState | `into_updating_state()` | Update    |

## Image Formats

### TLV Format (rustBoot native)

Native format for MCU targets. Image header (256 bytes max):

```
┌──────────┬──────────┬──────────────────────────────────┐
│ Offset   │ Size     │ Field                            │
├──────────┼──────────┼──────────────────────────────────┤
│ 0        │ 4        │ Magic: "RUST" (0x54535552)       │
│ 4        │ 4        │ Header size + firmware size       │
│ 8        │ 8        │ TLV: Version (tag 0x0001)        │
│ 16       │ 12       │ TLV: Timestamp (tag 0x0002)      │
│ 28       │ 6        │ TLV: ImgType (tag 0x0004)        │
│ 34       │ 38       │ TLV: Digest256 (tag 0x0003)      │
│ 72       │ ...      │ TLV: PubkeyDigest (tag 0x0010)   │
│ ...      │ ...      │ TLV: Signature (tag 0x0020)      │
│ ...      │ 2        │ EndOfHeader (tag 0x0000)          │
│ ...      │ var      │ Padding (0xFF)                    │
├──────────┼──────────┼──────────────────────────────────┤
│ 0x100    │ var      │ Firmware payload                   │
│ trailer  │ 4        │ Trail Magic: "BOOT" (0x544F4F42)  │
│ trailer+4│ 1        │ State byte                        │
│ trailer+5│ 3        │ Sector flags                      │
└──────────┴──────────┴──────────────────────────────────┘
```

TLV tag IDs:

| Tag             | ID (bytes)  | Length Field |
|-----------------|-------------|--------------|
| Version         | [0x01, 0x00]| 4 bytes      |
| TimeStamp       | [0x02, 0x00]| 8 bytes      |
| ImgType         | [0x04, 0x00]| 2 bytes      |
| Digest256       | [0x03, 0x00]| 32 bytes     |
| Digest384       | [0x13, 0x00]| 48 bytes     |
| PubkeyDigest    | [0x10, 0x00]| 32/48 bytes  |
| Signature       | [0x20, 0x00]| 64 bytes     |
| EndOfHeader     | [0x00, 0x00]| 0 bytes      |

### FIT Format (Linux)

FIT images use the Flattened Device Tree (FDT) structure as defined in the U-Boot FIT specification. Parsed by `dt/fit.rs` using the device tree reader (`dt/reader.rs`). The FIT structure contains:

```
/dts-v1/;
/ {
    description = "FIT image for rustBoot";
    #address-cells = <1>;

    images {
        kernel {
            description = "Linux kernel";
            data = <...>;
            type = "kernel";
            arch = "arm64";
            os = "linux";
            compression = "none";
            load = <0x80080000>;
            entry = <0x80080000>;
            hash-1 {
                algo = "sha256";
                value = <...>;
            };
        };
        fdt {
            data = <...>;
            type = "flat_dt";
            ...
        };
    };
    configurations {
        default = "config-1";
        config-1 {
            kernel = "kernel";
            fdt = "fdt";
            signature {
                algo = "p256";
                value = <...>;
                key-hint = "dev-key";
            };
        };
    };
};
```

The FIT configuration bundles kernel, FDT, ramdisk, and a signature node. Verification reads the hash from each image subnode and verifies the configuration-level ECDSA signature.

## Cryptographic Verification

```
Signed Firmware (TLV or FIT)
         │
         ▼
┌─────────────────────────────┐
│ SHA-256(Hash)               │
│   Read header.Digest256     │
│   Hash firmware bytes       │
│   == digest ?  OK : FAIL    │
└─────────────────────────────┘
         │ OK
         ▼
┌─────────────────────────────┐
│ ECDSA P-256 / secp256k1     │
│   Read header.Signature     │
│   Hash header + firmware    │
│   Verify(Sig, Hash, PubKey) │
│   == OK ?  AUTH : REJECT    │
└─────────────────────────────┘
         │ AUTH
         ▼
    Boot proceeds
```

- **Integrity check**: SHA-256 hash of firmware payload vs. header Digest256 field (`verify_integrity` in `image.rs`)
- **Authenticity check**: ECDSA P-256 (via `p256` crate) or secp256k1 (via `k256` crate) signature verification (`verify_ecc256_signature` in `signatures.rs`)
- **Public key**: Embedded at compile time per board, imported via `import_pubkey()` from a fixed byte array
- **Supported curves**: NIST P-256 (`nistp256` feature), secp256k1 (`secp256k1` feature), ed25519 (placeholder)

## Crates

### `rustBoot` (core no_std library)

| Module          | Responsibility                                     | Feature-gated |
|-----------------|-----------------------------------------------------|---------------|
| `image/`        | State machine (`ImageType`, `RustbootImage`), partition descriptors, sealed trait pattern | `mcu`         |
| `parser.rs`     | Nom-based TLV parser, `Tags` enum                   | `mcu`         |
| `rbconstants.rs`| Shared constants (`IMAGE_HEADER_SIZE`, `Tags` enum, hash/signature sizes) | Always        |
| `constants.rs`  | Per-MCU flash layout (sector size, partition addresses) | `mcu`         |
| `crypto/`       | ECDSA signature verification (NistP256, Secp256k1)   | nistp256/secp256k1 |
| `dt/`           | FIT parser, DTB reader/writer, device tree patching   | Always        |
| `flashapi.rs`   | `FlashApi` trait (write, erase, trailer_write)        | `mcu`         |
| `cfgparser.rs`  | Update config file parser (active/passive conf)       | Always        |
| `fs/`           | FAT filesystem (block device, controller, filesystem) | Always        |
| `lib.rs`        | Crate root, `RustbootError` enum, `Result` type       | —             |

### `rbsigner` (CLI signing tool)

`std`-only crate for signing firmware images offline.

| Module       | Responsibility                       |
|--------------|---------------------------------------|
| `main.rs`    | CLI dispatch                          |
| `mcusigner.rs` | Sign TLV-format MCU firmware images |
| `fitsigner.rs` | Sign FIT-format images             |
| `curve.rs`   | Key type definitions                  |

### `boards/update` (A/B swap logic)

The update crate implements the power-interruptible sector-by-sector swap between BOOT and UPDATE partitions via the SWAP scratch partition. Key components:

- `FlashUpdater<Interface>`: Implements `FlashApi` trait for writing, erasing, and trailer management
- `copy_sector()`: Sector-by-sector copy with swap flags for power-loss recovery
- Trailer flags: `has_new_flag()`, `has_swapping_flag()`, `has_backup_flag()`, `has_updated_flag()`
- `rustboot_start()`: Entry point called from board `main.rs`

### `boards/hal` (Hardware Abstraction Layer)

Per-MCU HAL implementations:

| Vendor    | MCUs                    |
|-----------|-------------------------|
| STM       | F334, F411, F446, F469, F746, H723 |
| nRF       | nRF52840                |
| Pico      | RP2040                  |
| RPi       | RPi4 (BCM2711, AArch64) |
| NXP       | iMX8M Nano (AArch64)    |

AArch64 support includes vendored `aarch64-cpu` register definitions and custom MMU/exception handling.

### `xtask` (Build helper)

Build system orchestration, embedded binary processing.

## Verification Architecture

| Category            | Count | Coverage                                       |
|---------------------|-------|-------------------------------------------------|
| Unit tests          | 164   | Image header parsing, state transitions, crypto, FIT parsing, DTB, config parser, constants |
| Integration tests   | 20    | `rustboot_start()` boot flow, A/B swap, TLV roundtrip |
| Kani proofs         | 16    | Bounded verification of state transitions, partition offset arithmetic, parser extractors, sector flags, constants, encode/decode roundtrip |
| Fuzz targets        | 4     | `image_header_parser`, `fit_parser`, `dtb_parser`, `config_parser` |
| Property tests (proptest) | 4 files | Parser: never panics on arbitrary bytes. State machine: invariants. Config parser: roundtrip. DTB struct items: invariants. |
| Formal models       | 2     | TLA+ (`docs/formal/tla_plus/boot_state_machine.tla`), Alloy (`docs/formal/alloy/boot_state_machine.als`) |
| MC/DC coverage      | 26%   | Baseline line-coverage threshold (`--fail-under-linenum 26`) |

### Kani Proof Harnesses (`kani/src/lib.rs`)

- `sect_flags_from_bounded`
- `partition_size_bounds`
- `partition_open_bounds`
- `sect_flags_all_values`
- `partition_offset_arithmetic`
- `state_transition_dag_no_cycles`
- `parser_extract_version_bounds`
- `parser_extract_timestamp_bounds`
- `constants_consistent`
- `version_comparison_properties`
- `parser_extract_img_type_no_panic`
- `parser_extract_digest_no_panic`
- `parser_extract_pubkey_digest_no_panic`
- `parser_extract_signature_no_panic`
- `flatten_bounds`
- `state_encode_decode_roundtrip`

### Fuzz Targets (`fuzz/fuzz_targets/`)

- `image_header_parser.rs` — TLV header fuzzing
- `fit_parser.rs` — FIT image fuzzing
- `dtb_parser.rs` — Device tree blob fuzzing
- `config_parser.rs` — Update configuration fuzzing

## Safety Case

See `docs/safety_case.md` for the full safety argument. Key assumptions:

1. Single-core MCU, no concurrent access to partition state
2. Flash memory is reliable (ECC-protected or assessed separately)
3. Boot ROM initializes hardware before rustBoot executes
4. Root of trust public key is provisioned at manufacturing time
5. The signature and hash algorithm implementations (`p256`, `sha2` crates) are correct per their respective standards

## Directory Layout

```
rustBoot/
├── rustBoot/                          # Core no_std library
│   ├── src/
│   │   ├── lib.rs                     # Crate root, safety lints, RustbootError
│   │   ├── cfgparser.rs               # Update config parser (nom)
│   │   ├── constants.rs               # Per-MCU flash layout
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   └── signatures.rs          # ECDSA verification (NistP256, Secp256k1)
│   │   ├── dt/
│   │   │   ├── mod.rs
│   │   │   ├── common.rs              # DTB Error enum, types
│   │   │   ├── fit.rs                 # FIT image parser
│   │   │   ├── internal.rs            # FDT internal structures
│   │   │   ├── patch.rs               # Device tree patching
│   │   │   ├── reader.rs              # DTB reader
│   │   │   ├── struct_item.rs         # Struct item enum
│   │   │   └── writer.rs              # DTB writer
│   │   ├── flashapi.rs                # FlashApi trait
│   │   ├── fs/                        # FAT filesystem
│   │   │   ├── mod.rs
│   │   │   ├── blockdevice.rs
│   │   │   ├── controller.rs
│   │   │   ├── fat.rs
│   │   │   ├── filesystem.rs
│   │   │   └── structure.rs
│   │   ├── image/
│   │   │   ├── mod.rs
│   │   │   ├── image.rs               # State machine, ImageType, partitions
│   │   │   └── sealed.rs              # Sealed trait pattern
│   │   ├── parser.rs                  # TLV header parser (nom)
│   │   └── rbconstants.rs             # Shared constants, Tags enum
│   ├── tests/
│   │   └── integration.rs             # Integration tests (20)
│   └── examples/                      # Usage examples
├── rbsigner/                          # CLI signing tool (std)
│   └── src/
│       ├── main.rs
│       ├── curve.rs
│       ├── fitsigner.rs
│       └── mcusigner.rs
├── boards/
│   ├── bootloaders/                   # Per-MCU main.rs entry points
│   │   ├── stm32f334/
│   │   ├── stm32f411/
│   │   ├── stm32f446/
│   │   ├── stm32f469/
│   │   ├── stm32f746/
│   │   ├── stm32h723/
│   │   ├── nrf52840/
│   │   ├── rp2040/
│   │   ├── rpi4/
│   │   └── imx8mn/
│   ├── hal/                           # Hardware abstraction layer
│   │   └── src/
│   │       ├── stm/                   # STM32 MCU HAL
│   │       ├── nrf/                   # nRF52840 HAL
│   │       ├── pico/                  # RP2040 HAL
│   │       ├── rpi/                   # RPi4 HAL (AArch64)
│   │       └── nxp/                   # iMX8M Nano HAL (AArch64)
│   ├── update/                        # A/B swap logic
│   │   └── src/update/update_flash.rs
│   └── firmware/                      # Example firmware (blinky)
├── fuzz/                              # Fuzz targets
│   └── fuzz_targets/
│       ├── image_header_parser.rs
│       ├── fit_parser.rs
│       ├── dtb_parser.rs
│       └── config_parser.rs
├── kani/                              # Kani proof harnesses
│   └── src/lib.rs                     # 16 proofs
├── xtask/                             # Build system helper
├── stack-analysis/                    # Stack usage analysis
├── docs/
│   ├── architecture/
│   │   ├── ARCHITECTURE.md            # This file
│   │   └── architecture.md            # Previous version
│   ├── safety_case.md                 # Full safety argument
│   ├── threat-model/                  # STRIDE threat model
│   ├── fmea/                          # FMEA with RPN
│   ├── requirements/                  # Requirements traceability
│   ├── verification/                  # Verification matrix
│   ├── formal/
│   │   ├── tla_plus/boot_state_machine.tla
│   │   └── alloy/boot_state_machine.als
│   ├── qemu_testing.md
│   └── wcet.md                        # WCET analysis
└── Cargo.toml                         # Workspace root
```