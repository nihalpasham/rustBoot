---
title: Honeypot / Decoy Partitions
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - NIST-SP-800-193
bootloader: true
firmware: false
hardware: false
---

# Honeypot / Decoy Partitions

## Summary

Create decoy firmware images in unused partition slots that appear identical
to real firmware (encrypted, signed format). If an attacker attempts to boot
from a honeypot partition, the bootloader detects the trap and takes
punitive action: zeroize all keys, set permanent tamper flag, halt.

## Motivation

An attacker who modifies flash contents or inserts their own firmware image
cannot know which partitions are real and which are decoys. This adds a
layer of confusion for physical attackers, increasing the cost and risk of
flash modification attacks.

## Design

### Honeypot Placement

```
Flash Layout (example):
┌──────────────────┐
│ Bootloader       │ (immutable, read-protected)
├──────────────────┤
│ Golden Partition │ (factory-signed, read-only)
├──────────────────┤
│ Partition A      │ (real, active boot slot)
├──────────────────┤
│ Decoy Partition  │ (looks real, but is a trap)
├──────────────────┤
│ Partition B      │ (real, update slot)
├──────────────────┤
│ Decoy Partition  │ (another trap)
└──────────────────┘
```

### Decoy Characteristics

Each decoy partition:
- Contains an encrypted firmware image (AES-256-GCM ciphertext)
- Has a valid-looking TLV header with plausible magic numbers
- Has a signature (that will fail verification)
- Is indistinguishable from a real partition by flash content alone

### Detection and Response

When the bootloader selects a boot slot:
1. Read TLV header
2. Attempt decryption → AES-GCM tag will mismatch for decoy ciphertext
3. Signature verification will fail for forged signatures
4. After 3 consecutive failed boots from different partitions:
   - Mark only real partitions (by index) as bootable
   - If any non-real partition was attempted:
     → Zeroize all keys
     → Set OTP tamper flag
     → Halt permanently

### Trap Logic

```rust
fn boot_from_partition(part: PartId) -> Result<(), BootError> {
    if is_decoy_partition(part) {
        // Increment decoy boot counter
        decoy_boot_counter += 1;

        if decoy_boot_counter >= DECOY_THRESHOLD {
            // Punitive response
            tamper_detector.zeroize();
            tamper_detector.set_tamper_flag()?;
            halt_with_diagnostic(HoneypotTriggered);
        }

        // Return error to try next partition
        return Err(BootError::SignatureInvalid);
    }

    // Normal boot path
    decrypt_and_verify(part)
}
```

## Verification

### Testing Requirements

- Decoy partition detected → boot error (not crash)
- Boot from decoy 3 times → keys zeroized, tamper flag set
- Boot from real partition after decoy → still succeeds (if within threshold)
- Decoy partition format indistinguishable from real by examination
- Decoy threshold is configurable per product

### Kani Proofs

| Harness | Property |
|---------|----------|
| `decoy_detection_bounded` | Decoy detection logic completes without unbounded loops |
| `decoy_counter_overflow` | Decoy counter saturates at threshold (no overflow) |
| `tamper_on_decoy_escalation` | After N decoy boots, tamper action is always taken |

### Proptests

| Property | Check |
|----------|-------|
| `decoy_vs_real_discrimination` | Real partitions are never misidentified as decoys |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Proptests**: Not written

## Residual Risk

- An attacker who can read the bootloader's partition table (via debug or
  code execution) can identify which partitions are real and which are decoys.
  Mitigation: partition table is in the bootloader's read-only section (RDP
  Level 2 blocks reading).
- Decoy partitions consume flash space that could be used for firmware.
  Impact: ~2× partition overhead for real + decoy slots.
- False positive: if a real partition appears corrupted, it might be
  mistaken for a decoy. Mitigation: threshold-based detection (3 failures).