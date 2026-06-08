---
title: Fail-Secure Behavior & Recovery
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - NIST-SP-800-193
  - Common-Criteria-EAL5+
  - FIPS-140-3
bootloader: true
firmware: false
hardware: false
---

# Fail-Secure Behavior & Recovery

## Summary

Every failure in the boot chain results in a safe, deterministic recovery
state. No single failure bricks the device or boots untrusted firmware. The
recovery path is itself authenticated and verified. All failure states are
documented with their recovery outcomes.

## Motivation

Security systems are judged by what happens when they fail. A bootloader that
panics on error, enters an undefined state, or silently falls through to
unverified firmware is worse than a system that refuses to boot. The
bootloader must have explicit, tested, and verified behavior for every possible
failure.

NIST SP 800-193 requires three properties: Protection, Detection, and
Recovery. EAL5+ (FPT_FLS.1) requires failure with preservation of secure
state. This measure implements the Recovery pillar.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Failure with preservation of secure state | EAL5+ (FPT_FLS.1) | All failure → known safe state |
| Automated recovery without undue loss | EAL5+ (FPT_RCV.3) | Golden fallback partition |
| Recovery path is authenticated | NIST SP 800-193 §4.4 | Recovery image also signed + verified |
| Detection + recovery | NIST SP 800-193 §4.3–4.4 | All detection events have defined recovery |
| No unauthenticated mutable code execution | NIST SP 800-193 §4.2.1.4 | Recovery path verifies signature |

## Design

### State Machine: Boot -> Failure -> Recovery

```
               ┌──────────────────────┐
               │   Boot Start          │
               └──────────┬───────────┘
                          │
                    ┌─────▼──────┐
                    │  POST KATs  │──── failure ──► HALT (blink diagnostic)
                    └─────┬──────┘
                          │ pass
                    ┌─────▼──────┐
                    │  Decrypt    │──── failure ──► Rollback to golden
                    └─────┬──────┘
                          │ pass
                    ┌─────▼──────┐
                    │  Verify     │──── failure ──► Rollback to golden
                    │  Signature  │
                    └─────┬──────┘
                          │ pass
                    ┌─────▼──────┐
                    │  Anti-      │──── failure ──► HALT (rollback attack)
                    │  Rollback   │
                    └─────┬──────┘
                          │ pass
                    ┌─────▼──────┐
                    │  Measure    │
                    │  & Boot     │
                    └────────────┘
```

### Failure Classes

| Class | Cause | Behavior | Recovery |
|-------|-------|----------|----------|
| CRYPTO_POST_FAIL | KAT failed | Blink diagnostic LED, halt | IWDG reset → retry POST |
| DECRYPT_FAIL | AES-GCM tag mismatch | Rollback to golden partition | Boot golden firmware |
| VERIFY_FAIL | Signature invalid | Rollback to golden partition | Boot golden firmware |
| ROLLBACK_DETECTED | Version ≤ counter | Halt (rollback attack) | Only recoverable via authenticated update |
| BOUNDS_VIOLATION | Partition math error | Halt (likely flash corruption) | Only recoverable via factory intervention |
| TAMPER_DETECTED | Tamper sensor triggered | Zeroize keys, halt | Device is compromised |

### Golden Partition

Each device has a small golden partition containing a minimal, factory-signed
firmware that provides:
- Enough functionality to receive a new firmware update
- Verified by the same signature chain
- Stored in a separate flash region (hardware-protected if supported)

The golden partition cannot be updated (or can only be updated with a special
factory key held by an HSM). This prevents an attacker from corrupting the
recovery path.

### Diagnostic Blink Patterns

| Condition | LED Pattern | Frequency |
|-----------|-------------|-----------|
| POST KAT failure | 5 fast blinks, pause | 100ms on, 100ms off |
| Decrypt failure | 4 blinks, pause | 200ms on, 200ms off |
| Signature failure | 3 blinks, pause | 300ms on, 300ms off |
| Rollback detected | 2 blinks, pause | 500ms on, 500ms off |
| Tamper detected | Continuous on | — |
| Golden boot | 1 blink, pause | 1s on, 1s off |

## Verification

### Testing Requirements

- Every error path in the bootloader returns a documented failure state
- After any failure, certified: IWDG reset leads to a clean retry
- Golden partition: verify is booted when primary fails
- Golden partition: verify signature before execution
- Power loss during rollback → resume on next boot
- Error states are unique and testable (each class produces different output)

### Kani Proofs

| Harness | Property |
|---------|----------|
| `recovery_state_machine_deadlock_free` | From any state, at least one transition is valid |
| `golden_partition_authenticated` | Golden partition is always signature-verified before execution |
| `all_error_paths_covered` | Every fallible function returns a typed error; every error is handled |
| `fail_secure_state_preserved` | After any failure, CPU does not execute mutable code before recovery |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Recovery DAG completeness | Verus | From every failure state, the recovery path reaches a bootable state |
| No silent fallthrough | Creusot | Every match on `Result` has an explicit `Err` arm |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `failure_injection` | Inject failures at random boot stages — verify recovery path always taken |

### Proptests

| Property | Check |
|----------|-------|
| `all_failure_causes_handled` | Enumeration of all error types → each maps to a recovery action |
| `recovery_idempotent` | Multiple recovery cycles produce the same result |
| `power_loss_at_any_point` | Simulated power loss at any boot stage → deterministic next-boot outcome |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- A simultaneous failure of both primary and golden partitions (catastrophic
  flash corruption) leaves no recovery path. Mitigation: golden partition is in
  a separate flash bank or hardware-protected region.
- IWDG reset during rollback is the only recovery from a halted state. If IWDG
  itself fails (clock failure), the device stays halted. Mitigation: IWDG uses
  an independent RC oscillator, not the main system clock.