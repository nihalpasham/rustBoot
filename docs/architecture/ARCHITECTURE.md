# rustBoot Architecture

## System Overview

rustBoot is a `no_std` Rust secure bootloader implementing A/B firmware
updates with ECDSA (NIST P-256) image verification. It supports both
custom TLV image format (bare-metal) and Flattened Image Tree (FIT)
format (Linux booting) on Cortex-M and AArch64 targets.

## Boot Flow

<!-- docs/diagrams/boot-flow.mmd -->
```mermaid
sequenceDiagram
    participant ROM as Boot ROM
    participant RB as rustBoot
    participant Flash as Flash Memory
    participant FW as Target Firmware

    ROM->>RB: 1. Load from flash, jump to entry
    RB->>Flash: 2. Open BootPartition
    RB->>Flash: 3. Open UpdatePartition
    RB->>RB: 4. Evaluate boot state machine
    alt BootInTestingState
        RB->>RB: Rollback triggered
        RB->>Flash: Write SuccessState
    else UpdateInUpdatingState
        RB->>RB: Update swap triggered
        RB->>Flash: Swap Update → Boot
    else BootInNewState or BootInSuccessState
        RB->>RB: SHA-256 integrity check
        RB->>RB: ECDSA P-256 authenticity check
        alt Verification fails
            RB->>RB: Emergency update / rollback
        else Verification passes
            RB->>RB: Boot proceeds
        end
    end
    RB->>RB: 5. Re-open Boot partition
    RB->>Flash: 6. Read firmware base address
    RB->>FW: 7. hal_boot_from(address) → jump
```

## Partition Layout

<!-- docs/diagrams/partition-layout.mmd -->
```mermaid
block-beta
    columns 3
    block:Flash:3
        columns 1
        block:Boot:1
            columns 1
            space
            block:headers:3
                columns 3
                a["HDR"] b["FW Image"] c["TRAILER"]
            end
            space
        end
        block:Update:1
            columns 1
            space
            block:headers2:3
                columns 3
                d["HDR"] e["FW Image"] f["TRAILER"]
            end
            space
        end
        block:Swap:1
            columns 1
            space
            g["Swap Metadata"] h["State Flags"]
            space
        end
    end
    style Boot fill:#4a9eff77
    style Update fill:#ff9a4a77
    style Swap fill:#9aff4a77
```

Three partitions managed by `PartDescriptor`:

- **BootPartition**: Active firmware slot (boots from here)
- **UpdatePartition**: Update staging slot (new firmware written here)
- **SwapPartition**: Swap metadata and rollback state

Each partition contains:
- Image header (TLV format): version, timestamp, type, digests, signature
- Firmware binary
- Trailer: state flags, magic number

## State Machine

<!-- docs/diagrams/state-machine.mmd -->
```mermaid
stateDiagram-v2
    [*] --> BootInNewState : Power-on

    state BootInNewState {
        [*] --> VerifyIntegrity
        VerifyIntegrity --> VerifyAuthenticity
        VerifyAuthenticity --> BootReady
    }

    BootInNewState --> BootInTestingState : into_testing_state()
    BootInTestingState --> BootInSuccessState : into_success_state()
    BootInTestingState --> BootInNewState : rollback

    state UpdateInNewState {
        [*] --> WaitingForUpdate
    }

    UpdateInNewState --> UpdateInUpdatingState : into_updating_state()
    UpdateInUpdatingState --> BootInTestingState : swap complete

    note right of BootInNewState : State: 0xFF\nFresh image
    note right of BootInTestingState : State: 0x70\nUnder test
    note right of BootInSuccessState : State: 0x00\nConfirmed
```

Five `ImageType` states with four valid transitions:

```
BootInNewState ──► BootInTestingState ──► BootInSuccessState
                      │
                      └── (rollback → BootInNewState, via update_trigger)

UpdateInNewState ──► UpdateInUpdatingState
```

All 20 remaining transition pairs are invalid and return
`RustbootError::InvalidState`.

## Image Formats

### TLV Format (native)

<!-- docs/diagrams/image-format.mmd -->
```mermaid
packet-beta
0-15: "Magic (4B)"
16-31: "Header Size (4B)"
32-47: "FW Size (4B)"
48-63: "Version (4B)"
64-79: "Timestamp (8B)"
80-95: "Image Type (4B)"
96-111: "SHA-256 Hash (32B)"
112-127: "Pubkey Hash (32B)"
128-143: "Padding"
144-159: "ECDSASig_R (32B)"
160-175: "ECDSASig_S (32B)"
176-191: "Padding to 512B"
```

Fixed-size header (512 bytes) with Type-Length-Value fields:
- Magic, Version, Timestamp, Image Type
- SHA-256 hash, Public key hash, ECDSA signature
- Firmware binary follows header

### FIT Format (Linux)
Flattened Image Tree format with device tree structure:
- Hash nodes (SHA-256 per sub-image)
- Signature nodes (ECDSA per configuration)
- Configurations binding kernel, FDT, ramdisk

## Cryptographic Verification

- **ECDSA P-256**: `NistP256Signature::verify()` via `DigestVerifier` trait
- **SHA-256**: `verify_integrity()` computes digest of firmware region
- **Public Key**: Embedded at compile time in `import_pubkey()`

## Crate Architecture

<!-- docs/diagrams/crate-architecture.mmd -->
```mermaid
graph TB
    subgraph "Application Layer"
        FW[Firmware Image]
    end
    subgraph "Bootloader Core"
        RB[rustBoot -- no_std]
        IMG[image/ -- State Machine]
        CRYPTO[crypto/ -- ECDSA SHA-256]
        PARSER[parser/ -- TLV]
        DT[dt/ -- Device Tree FIT]
        CFG[cfgparser/ -- Config]
    end
    subgraph "Board Support"
        BRD[boards/ -- HAL Update Logic]
        HAL[hal/ -- Register Maps Drivers]
    end
    subgraph "Tooling"
        XT[xtask/ -- Build]
        SIG[rbsigner/ -- Signing]
    end
    subgraph "Verification"
        TST[tests/ -- 184 total]
        KANI[kani/ -- 16 proofs]
        FUZZ[fuzz/ -- 5 targets]
        FORM[formal/ -- TLA+ Alloy]
    end
    RB --> IMG & CRYPTO & PARSER & DT & CFG
    BRD --> RB & HAL
    FW --> BRD
    SIG --> RB
    XT --> BRD
    KANI & FUZZ & TST --> RB
```

```
rustBoot/          Core no_std library (parsers, crypto, state machine, DT)
rbsigner/          CLI signing tool (std, for host builds)
xtask/             Build system helper (std)
boards/            Board support packages
  hal/             Hardware abstraction layer (register maps, drivers)
  update/          A/B swap logic (rustboot_start entry point)
  bootloaders/     Per-MCU main.rs entry points
  firmware/        Example firmware images
```

## Verification Architecture

- **Unit tests**: 164 covering parsers, crypto, state machine, DT ops
- **Integration tests**: 20 covering state transitions, constants, types
- **Kani proofs**: 16 bounded verification harnesses
- **Fuzz targets**: 5 (TLV header, FIT parser, DTB parser, config parser)
- **Property tests**: 3 proptest properties (parse safety, roundtrip, flags)
- **Formal models**: TLA+ and Alloy for state machine invariants

## Safety Case

See `docs/safety_case.md` for the full safety argument covering 7 safety
goals (G1-G7), 8 documented hazards, and a complete verification matrix.

## Directory Layout

```
rustBoot/src/
  lib.rs            — Crate root, lint denies
  image/image.rs    — Boot state machine, partition management
  image/parse.rs    — TLV header parser
  crypto/signatures.rs — ECDSA verification
  dt/               — Device tree (reader, writer, patcher, FIT parser)
  cfgparser.rs      — Update config parser
  rbconstants.rs    — Global constants

docs/
  architecture/     — This document
  safety_case.md    — NATO-grade safety argument
  threat-model/     — STRIDE threat model
  fmea/             — FMEA with RPN scoring
  requirements/     — Requirements traceability
  formal/           — TLA+ and Alloy formal models
  verification/     — Test matrix and verification evidence
  qemu_testing.md   — QEMU integration guide
  wcet.md           — Stack depth and WCET analysis
```