# rustBoot Architecture

## System Context

```
┌─────────────────────────────────────────────────────────┐
│                     rustBoot System                       │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │  BOOT    │  │  UPDATE  │  │  SWAP    │  ← Flash      │
│  │ Partition│  │ Partition│  │ Partition│    Partitions  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘               │
│       │              │              │                     │
│  ┌────▼──────────────▼──────────────▼─────┐              │
│  │         rustBoot Core (rustBoot)        │              │
│  │  - Image parsing (TLV)                  │              │
│  │  - Crypto (SHA-256, ECDSA)              │              │
│  │  - State machine (New→Updating→Testing→Success) │              │
│  │  - Device tree (FIT, DTB)               │              │
│  └────────────────┬────────────────────────┘              │
│                   │                                       │
│  ┌────────────────▼────────────────────────┐              │
│  │         rustBoot HAL (rustBoot-hal)      │              │
│  │  - FlashInterface trait                  │              │
│  │  - Board-specific flash drivers          │              │
│  └────────────────┬────────────────────────┘              │
│                   │                                       │
│  ┌────────────────▼────────────────────────┐              │
│  │         rustBoot Update (rustBoot-update) │              │
│  │  - A/B swap logic                        │              │
│  │  - Power-interruptible sector copy       │              │
│  │  - Rollback                              │              │
│  └────────────────┬────────────────────────┘              │
│                   │                                       │
│  ┌────────────────▼────────────────────────┐              │
│  │         Board Bootloaders                │              │
│  │  - Cortex-M entry (cortex-m-rt)          │              │
│  │  - Cortex-A entry (custom ASM)           │              │
│  │  - AArch64 Linux boot                    │              │
│  └─────────────────────────────────────────┘              │
└─────────────────────────────────────────────────────────┘
```

## Module Decomposition

| Module | Responsibility | Criticality |
|--------|---------------|-------------|
| `rustBoot/src/image/` | Image format, state machine, partition management | A (Safety) |
| `rustBoot/src/crypto/` | ECDSA signature verification | A (Safety) |
| `rustBoot/src/parser.rs` | TLV header parsing | A (Safety) |
| `rustBoot/src/cfgparser.rs` | Update config parsing | B (Security) |
| `rustBoot/src/constants.rs` | Board-specific flash layout | A (Safety) |
| `rustBoot/src/flashapi.rs` | Flash API trait | A (Safety) |
| `rustBoot/src/dt/` | Device tree parsing/patching | B (Security) |
| `rustBoot/src/fs/` | FAT filesystem support | C (Mission) |
| `boards/hal/` | Hardware abstraction layer | A (Safety) |
| `boards/update/` | A/B swap and boot flow | A (Safety) |
| `rbsigner/` | Firmware signing tool | B (Security) |
| `xtask/` | Build orchestration | D (Support) |

## Data Flow

```
Signing Environment:
  Private Key → rbsigner → Signed Firmware (TLV header + ECDSA sig)

Device Boot:
  Flash → Magic Check → Integrity Check (SHA-256) → Auth Check (ECDSA)
  → State Machine → Boot or Rollback

Update Flow:
  New firmware in UPDATE → Verify → Sector-by-sector swap
  → Mark Testing → Reboot → If OK: Mark Success
  → If fail: Rollback
```

## State Machine

```
                    ┌──────────┐
                    │   New    │
                    └────┬─────┘
                         │ update_trigger()
                    ┌────▼─────┐
                    │ Updating │
                    └────┬─────┘
                         │ swap complete
                    ┌────▼─────┐
                    │ Testing  │ ←── reboot
                    └────┬─────┘
                         │ update_success()
                    ┌────▼─────┐
                    │ Success  │
                    └──────────┘
                         │ (next update)
                         └──→ New
```