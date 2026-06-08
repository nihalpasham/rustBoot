---
title: Active Tamper Detection (TAMP)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 2
priority: high
standards:
  - FIPS-140-3-Level-3
  - Common-Criteria-EAL5+
bootloader: true
firmware: false
hardware: true
---

# Active Tamper Detection (TAMP)

## Summary

Use the MCU's tamper detection peripheral (STM32L5/U5 TAMP, F411 software
monitoring) to detect physical intrusion attempts. On tamper detection,
immediately zeroize all cryptographic key material, set the tamper flag in
OTP, and enter a secure halt state.

## Motivation

FIPS 140-3 Level 3 requires tamper-evident physical security. EAL5+
(FPT_PHP.3) requires resistance to physical attack. Without tamper detection,
an attacker with physical access can probe, modify, or extract device contents
without the device ever knowing.

## Standards Compliance

| Requirement | Standard | How Met |
|-------------|----------|---------|
| Tamper evidence | FIPS 140-3 Level 3 §7.9.7 | Tamper event set in OTP, permanent |
| Physical tamper resistance | EAL5+ (FPT_PHP.3) | TAMP peripheral + active monitoring |
| CSP zeroization on tamper | FIPS 140-3 §7.9.7 | Immediate zeroize, then halt |
| Tamper detection + response | NIST SP 800-193 §4.2.1.4 | On tamper: zeroize keys, halt, blink |

## Design

### STM32L5/U5 TAMP Peripheral

The TAMP peripheral provides:
- Up to 5 external tamper pins (connected to enclosure switches)
- 2 internal tamper events (RTC/backup register tamper)
- Voltage/temperature tamper monitoring (PVT)
- Configurable filter and debounce
- Automatic erase of backup registers on tamper
- Tamper interrupt to CPU

### Tamper Sources

| Source | MCU | Detection | Response Time |
|--------|-----|-----------|---------------|
| Enclosure pin 1 (lid open) | L5/U5 | TAMP pin, active high | <1µs |
| Enclosure pin 2 (screw removed) | L5/U5 | TAMP pin, active low | <1µs |
| VBAT voltage out of range | L5/U5 | PVT comparator | <10µs |
| Internal temperature spike | L5/U5 | Temperature sensor | <100µs |
| RTC register tamper | L5/U5 | Register modification detection | <1µs |
| Clock glitch detection | L5/U5 | CSS (Clock Security System) | <1ms |
| Voltage glitch (F411) | All | ADC monitoring (software) | ~100µs |

### Tamper Response

```
Tamper Detected
    │
    ├─► Hardware: TAMP peripheral erases backup registers
    │
    ├─► ISR: TamperInterrupt
    │       ├─► set_tamper_flag_otp()  (permanent indicator)
    │       ├─► zeroize_all_keys()      (CPU registers, stack)
    │       ├─► zeroize_imrs()          (measurement registers)
    │       ├─► blink_led(TAMPER_PATTERN)
    │       └─► infinite halt loop
    │
    └─► Next boot:
            ├─► check_tamper_flag_otp()  → true
            ├─► zeroize_all_keys()       (again, defense in depth)
            └─► HALT with tamper diagnostic
```

### Software-Only Tamper (F411/H7)

For MCUs without a TAMP peripheral:
- GPIO pin configured as external interrupt (rising/falling edge)
- Pull-up on enclosure switch → falling edge = lid opened
- ADC channel monitors VBAT voltage periodically
- IWDG ensures periodic re-check
- Response: same zeroize + halt

## Interface

```rust
pub trait TamperDetector {
    /// Configure tamper detection pins and sources.
    fn configure(config: TamperConfig) -> Result<(), TamperError>;

    /// Check if a tamper event has ever been detected (persistent flag).
    fn was_tampered() -> bool;

    /// Read the last tamper source.
    fn last_source() -> Option<TamperSource>;

    /// Clear tamper flag (only possible with factory authority).
    fn clear_tamper() -> Result<(), TamperError>;

    /// Zeroize all key material (called on tamper).
    fn zeroize() -> Result<(), TamperError>;
}

pub struct TamperConfig {
    pub pin1_enable: bool,
    pub pin2_enable: bool,
    pub vbat_monitor_enable: bool,
    pub temperature_monitor_enable: bool,
}

pub enum TamperSource {
    EnclosurePin1,
    EnclosurePin2,
    VoltageOutOfRange,
    TemperatureSpike,
    ClockGlitch,
    RtcTamper,
    SoftwareWatchdog,
}
```

## Verification

### Testing Requirements

- Simulate tamper pin event → assert `was_tampered()` returns true
- Assert keys zeroized after tamper (verify via volatile read)
- Assert tamper flag persists across reset (OTP storage)
- Assert that after tamper, device does not boot firmware
- Run 1000 false-positive tests (brownout, normal enclosure handling) → no false alarms
- For F411: simulate GPIO interrupt → assert same response

### Kani Proofs

| Harness | Property |
|---------|----------|
| `tamper_flag_persistent` | After tamper flag is set, it remains set across all reset types |
| `tamper_response_halt` | After tamper, CPU does not execute application code |
| `zeroize_on_tamper_safe` | Zeroize does not panic (all pointers valid) |
| `tamper_flag_monotonic` | `clear_tamper()` cannot be called without factory authority |

### Creusot/Verus Proofs

| Property | Tool | Details |
|----------|------|---------|
| Zeroize is complete | Verus | All key storage locations are written to 0 before halt |
| Exception safety on tamper | Creusot | If zeroize itself faults, IWDG reset still clears registers |

### Fuzz Targets

| Target | Input |
|--------|-------|
| `tamper_config_validation` | Arbitrary TamperConfig values — verify never panics |

### Proptests

| Property | Check |
|----------|-------|
| `tamper_response_is_deterministic` | Same tamper source always produces same response |
| `tamper_flag_is_persistent` | After N random resets, tamper flag persists |

## Status

- **Specification**: Draft
- **Implementation**: Not started
- **Tests**: Not written
- **Kani proofs**: Not written
- **Creusot/Verus proofs**: Not written
- **Fuzz targets**: Not written
- **Proptests**: Not written

## Residual Risk

- An attacker who can precisely bypass the tamper detection (e.g., short the
  enclosure switch while opening) defeats this measure. Defense-in-depth:
  multiple tamper sources (mechanical + voltage + temperature).
- The tamper flag stored in OTP is permanent. A false positive (accidental
  tamper during shipping) bricks the device. Mitigation: thorough
  debounce/filter configuration, factory override for RMA.