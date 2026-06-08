---
title: Continuous-Power Attack Defense
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
firmware: shared
hardware: true
---

# Continuous-Power Attack Defense

## Summary

An attacker who provides external power (battery, lab supply) to the device
bypasses the normal power-on reset. This gives them unlimited time to probe
memory, step through code, glitch the power rail, or extract secrets while the
device is running. This measure describes the combined defenses required to
prevent compromise even under sustained power.

## Motivation

Without specific defenses, a continuous-power attack enables:
- **Stale memory**: memory contents are never cleared by a POR
- **Unlimited debug time**: attacker can interact with the device forever
- **Infinite retries**: fault injection attacks can be attempted millions of
  times
- **Runtime inspection**: probes can monitor bus activity, cache contents
- **State corruption**: attacker can gradually corrupt memory content

## Threat Model

```
┌────────────────────────────────────────────────────────┐
│  Attacker provides external battery / lab PSU          │
│  Device never resets → IWDG is the ONLY forced reset   │
│                                                         │
│  Attack vectors under continuous power:                 │
│  ├─ JTAG/SWD (blocked by RDP Level 2)                  │
│  ├─ Bus probing (SPI flash traffic sniffing)            │
│  ├─ Voltage glitching → fault injection                 │
│  ├─ Clock glitching → skip security checks              │
│  ├─ Temperature → accelerate aging, cause faults        │
│  ├─ EM radiation → side-channel extraction              │
│  └─ Power analysis → SPA/DPA on crypto operations       │
└────────────────────────────────────────────────────────┘
```

## Design

### Defense Layers

```
┌──────────────────────────────────────────────────────────┐
│  Layer 1: IWDG Forced Periodic Reset                     │
│  Every N seconds → full system reset                     │
│  POST re-runs → state is fresh → attacker loses context  │
│  (This is the PRIMARY defense against continuous power)  │
├──────────────────────────────────────────────────────────┤
│  Layer 2: Bus-Level Encryption                           │
│  AES-256-GCM ciphertext at rest in flash                 │
│  Bus probes see only encrypted data                      │
│  Key derived from UID + OTP — never in flash             │
├──────────────────────────────────────────────────────────┤
│  Layer 3: Debug Port Dead (RDP Level 2)                 │
│  JTAG/SWD physically disconnected                        │
│  No debug probe can halt or inspect the CPU              │
├──────────────────────────────────────────────────────────┤
│  Layer 4: Volatile Key Material                          │
│  Keys live in CPU registers only during decryption       │
│  Zeroized after each block (64KB window)                 │
│  On any exception → zeroize before panic                 │
├──────────────────────────────────────────────────────────┤
│  Layer 5: Tamper Detection                               │
│  Active tamper pins monitor enclosure integrity          │
│  Voltage/temperature monitoring (L5/U5 PVT)              │
│  On tamper → immediate key zeroization                   │
├──────────────────────────────────────────────────────────┤
│  Layer 6: Constant-Time Operations                       │
│  No timing side-channels on secret data                  │
│  All crypto uses constant-time comparisons               │
│  Random delays in critical verification paths            │
└──────────────────────────────────────────────────────────┘
```

### IWDG Reset Cycle Under Continuous Power

```
Time:  ||<-- IWDG period (~30s) -->||<-- IWDG period (~30s) -->||
       ||                          ||                          ||
State: Reset→POST→Decrypt→Boot→Run Reset→POST→Decrypt→Boot→Run

Attacker observation:
- Each cycle lasts ~30s
- Of that, ~1s is in bootloader (POST + decrypt + verify)
- ~29s in application runtime
- Every 30s: all state lost, POST re-runs
```

The attacker cannot accumulate state across reset boundaries. Each reset is a
clean slate: RAM is re-initialized, keys are derived fresh, measurement
registers are reset. The only persistent state is the OTP monotonic counter
and the flash partitions.

### Application Compatibility

**For real-time systems**, the IWDG period is negotiated via the signed
firmware manifest. The application designer chooses:

| Application Type | Typical IWDG Period | Notes |
|-----------------|---------------------|-------|
| Safety-critical control | 100ms–1s | Fast recovery, tight latency |
| Sensor node | 10–60s | Periodic data collection |
| Communication gateway | 1–10s | Responsive enough for network |
| Batch processing | 1–60min | Long compute, must budget |

The period signed into the manifest: even if the application firmware is
compromised, the attacker cannot extend the watchdog beyond what the manifest
allows.

## Verification

### Testing Requirements

- IWDG fires within configured period (± hardware tolerance)
- Decryption + verification completes within the bootloader's IWDG window
- Key zeroization on any exception (test via HardFault injection)
- RDP Level 2 persists across IWDG resets (it should — it's in option bytes)
- Flash encryption: after IWDG reset, all keys are fresh
- Measurement registers: after IWDG reset, all IMRs are re-initialized

### Kani Proofs

| Harness | Property |
|---------|----------|
| `reset_clears_key_registers` | After any reset vector entry, no key material exists in general-purpose registers |
| `post_runs_after_every_reset` | POST is called on every reset, regardless of reset cause |
| `measurement_registers_fresh` | IMRs are initialized to zero on every reset |
| `key_derivation_called_per_boot` | `derive_fek()` is called exactly once per boot |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| No persistent secrets across reset | Verus | After zeroization + reset, no key material survives |
| Reset cause is authenticated | Creusot | Bootloader correctly determines reset cause (POR vs IWDG vs external) |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `reset_cause_injection` | Random reset cause values — verify correct behavior |

### Proptests

| Property | Check |
|----------|-------|
| `all_reset_causes_reach_post` | Every possible reset cause results in POST execution |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- If the IWDG oscillator itself fails (extremely unlikely), the watchdog never
  fires. STM32 IWDG uses a dedicated RC oscillator that is independent of the
  main system clock. Failure rate is per the MCU's FIT (Failures In Time) data.
- An attacker with an external clock source could provide a faster clock to
  reduce the IWDG period to near-zero, starving the application. Mitigation: no
  external pin can influence IWDG timing — it's entirely internal.