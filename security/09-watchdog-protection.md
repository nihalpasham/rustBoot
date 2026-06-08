---
title: Independent Watchdog (IWDG) Protection
status: draft
date: 2026-06-08
version: 0.1.0
phase: 1
priority: critical
standards:
  - NIST-SP-800-193
bootloader: true
firmware: shared
hardware: true
---

# Independent Watchdog (IWDG) Protection

## Summary

The Independent Watchdog (IWDG) is locked during the bootloader phase and runs
from its own dedicated RC oscillator. It forces a system reset if the firmware
fails to refresh it within a configured time window. This defeats continuous-
power attacks by ensuring the system cannot run indefinitely without periodic
re-authentication.

## Motivation

An attacker who provides external power (battery, lab supply) keeps the device
running without ever triggering the normal power-on reset. Without a watchdog:
- POST runs once, never again
- Memory state becomes stale
- Debug probes can step through code at leisure
- Secrets can be extracted from running memory

The IWDG defeats this by forcing a system-wide reset at a configurable
interval. When the watchdog fires:
1. CPU resets
2. Bootloader runs POST (crypto KATs)
3. Decrypts and verifies firmware
4. Measures boot chain
5. Boots firmware again

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Recovery from unresponsive state | NIST SP 800-193 §4.4 | IWDG forces system reset |
| Independent of main system clock | NIST SP 800-193 §4.4.2 | IWDG uses its own RC oscillator |
| Cannot be disabled by software | — | IWDG locked via option bytes or key register |

## Design

### IWDG Configuration

| Parameter | Bootloader Phase | Firmware Phase |
|-----------|-----------------|----------------|
| Prescaler | /4 | Configured in signed manifest |
| Counter reload | 0x0FFF (12.8s with /4 @ 32kHz) | As specified in manifest |
| Window mode | Disabled | Configurable |
| Lock | Locked via `IWDG_KR 0xCCCC` | Cannot be unlocked |

The watchdog period is part of the signed firmware manifest. The firmware can
request a specific watchdog period (e.g., 30s for a sensor node, 5s for a
safety-critical controller). The bootloader enforces the configured period.

### Watchdog Period Negotiation

```
1. Bootloader runs with IWDG period = minimum safe value (e.g., 12.8s)
2. Decrypts and verifies firmware manifest
3. Reads `wdt_period_ms` from signed manifest
4. If manifest period < minimum_safe_period:
       → reject (attacker trying to shorten watchdog)
5. Reconfigure IWDG with firmware-requested period
6. Lock IWDG
7. Boot firmware
8. Firmware must kick IWDG within the period
```

### Bootloader Behavior

The bootloader kicks the IWDG between each major stage to prevent watchdog
reset during decryption/verification of large images:

```
1. Reset
2. IWDG: 100ms timeout (tight — no boring waits allowed)
3. POST KATs (kick IWDG before/after each KAT)
4. Decrypt firmware (kick IWDG every 64KB)
5. Verify signature (kick IWDG)
6. Reconfigure IWDG to firmware-requested period
7. Lock IWDG
8. Boot firmware
```

### Firmware Behavior

The firmware must kick the IWDG at least once within the configured period.
Recommended pattern:
- Dedicated lowest-priority task that kicks the watchdog
- If any higher-priority task starves the watchdog task → reset
- Hard real-time systems: kick at the end of the main control loop

## Interface

```rust
pub trait Watchdog {
    /// Lock watchdog with initial timeout (bootloader phase).
    /// After locking, only a system reset can change the configuration.
    fn lock_initial(timeout_ms: u32) -> Result<(), WdtError>;

    /// Reconfigure watchdog period (called after firmware manifest parsed).
    fn reconfigure(timeout_ms: u32) -> Result<(), WdtError>;

    /// Kick (refresh) the watchdog. Must be called before timeout expires.
    fn kick() -> Result<(), WdtError>;

    /// Read remaining time before watchdog expiry (approximate).
    fn remaining_ms() -> u32;

    /// Check if watchdog was the cause of the last reset.
    fn last_reset_was_watchdog() -> bool;
}
```

## Verification

### Testing Requirements

- Lock → kick within period → no reset
- Kick → wait past timeout → verify reset (simulated via timeout injection)
- Lock → attempt to unlock → assert error
- Reconfigure → assert new period applies
- `last_reset_was_watchdog()` → true after simulated WDT timeout
- Firmware-kick test: verify bootloader's periodic kick mechanism

### Kani Proofs

| Harness | Property |
|---------|----------|
| `watchdog_lock_irreversible` | After `lock_initial()`, no sequence of calls can unlock the watchdog |
| `watchdog_period_bounds` | Any configured period is within hardware limits |
| `watchdog_kick_within_bounds` | `kick()` does not overflow any counters |
| `remaining_ms_no_underflow` | `remaining_ms()` returns 0 instead of underflowing |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Reconfiguration is safe | Verus | Reconfiguring IWDG does not trigger immediate reset |
| Kick times are monotonic | Creusot | `remaining_ms()` decreases monotonically between kicks |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `watchdog_period_manifest` | Malformed manifest watchdog period values |

### Proptests

| Property | Check |
|----------|-------|
| `period_accepts_valid_configs` | Valid watchdog periods are accepted |
| `period_rejects_invalid_configs` | Periods < minimum or > maximum are rejected |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- IWDG uses an independent RC oscillator (~32 kHz) that is less accurate than
  the main HSI/HSE. The timeout can vary by ±10-20% across temperature and
  voltage. The firmware must account for this margin.
- An attacker who can disable or glitch the IWDG oscillator defeats the
  watchdog. Mitigation: the IWDG oscillator is internal to the MCU and not
  accessible from external pins.