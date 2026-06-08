---
title: Updated Threat Model (STRIDE)
status: draft
date: 2026-06-08
version: 0.1.0
---

# Updated Threat Model (STRIDE)

## Assets (Updated)

| ID | Asset | Priority | Protection Level After Implementation |
|----|-------|----------|--------------------------------------|
| A1 | Boot firmware integrity | Critical | SHA-384 + LMS/XMSS + ECDSA |
| A2 | Firmware signing private key | Critical | HSM-protected, never on device |
| A3 | Update firmware authenticity | Critical | SUIT manifest + dual signatures |
| A4 | Boot state (anti-rollback) | Critical | OTP monotonic counter |
| A5 | Device availability (no brick) | Critical | Golden partition + fail-secure |
| A6 | Firmware confidentiality | Critical | AES-256-GCM at rest |
| A7 | Device identity key (OTP) | Critical | Hardware readout protection |
| A8 | Measurement log (IMRs) | High | Extend-only, TrustZone-protected |
| A9 | Cryptographic keys in memory | High | Volatile, zeroized after use |
| A10 | Debug authentication key | High | HSM-signed certificates |
| A11 | Attestation report | Medium | Signed with device attestation key |

## Threats (Updated with New Measures)

### Spoofing

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| Fake firmware update | A2, A3 | LMS/XMSS + ECDSA dual verification | Mitigated | Mitigated (PQ ready) |
| Fake boot image | A1 | SHA-384 + dual signature | Mitigated | Mitigated (PQ ready) |
| Device identity spoofing | A7 | DICE certificate chain | Not covered | Mitigated |
| Replay of old attestation | A11 | Fresh nonce in attestation request | Not covered | Mitigated |

### Tampering

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| Modify firmware in flash | A1 | AES-GCM tag verification on decrypt | Mitigated | Mitigated (encrypted) |
| Modify sector flags | A4 | OTP anti-rollback cannot bypass flags | Mitigated | Mitigated (HW-backed) |
| Modify trailer magic | A4 | OTP counter independent of trailer magic | Mitigated | Mitigated |
| Modify encrypted firmware | A6 | AES-GCM tag mismatch → decrypt fails | Not covered | Mitigated |
| Flash readout via debug | A7 | RDP Level 2 + encrypted flash | Not covered | Mitigated |
| Modify measurement IMRs | A8 | TrustZone-protected, extend-only | Not covered | Mitigated (TZ) |
| Modify KAT vectors in flash | A1 | KAT vectors in bootloader code (read-only) | Not covered | Mitigated |
| Modify golden partition | A1 | Golden partition in separate, HW-protected flash | Not covered | Mitigated |

### Repudiation

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| No audit trail for boot decisions | A4 | Measurement log + attestation | Partial | Mitigated |

### Information Disclosure

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| Public key embedded in bootloader | N/A | Acceptable risk: public by design | Accepted | Accepted |
| Firmware binary extracted from flash | A6 | AES-256-GCM encryption at rest | **GAP** | Mitigated |
| Key material extracted from RAM | A9 | Volatile key buffer + zeroization | Not covered | Mitigated |
| Device identity extracted via debug | A7 | RDP Level 2 | Not covered | Mitigated |
| Side-channel extraction of keys | A9 | Constant-time operations, random delay | Not covered | Mitigated |
| TrustZone-secured key extraction | A9 | SGATE API, no direct memory access from NS | Not covered | Mitigated (TZ) |
| Firmware content via EM emission | A6 | Encrypted at rest; side-channel tested | Not covered | Mitigated (PH3) |

### Denial of Service

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| Corrupt firmware causes boot loop | A5 | Golden partition + fail-secure recovery | Mitigated | Mitigated |
| Power loss during update | A5 | A/B swap with power-interruptible states | Mitigated | Mitigated |
| Flash wear-out from updates | A5 | OTP counter limits update count explicitly | Accepted | Mitigated (bounded) |
| Continuous-power attack | A5 | IWDG forced periodic reset | Not covered | Mitigated |
| Fault injection → skip security check | A1 | Redundant checks, sentinel variables, CRC | Not covered | Mitigated |
| KAT failure → brick | A5 | IWDG reset: transient fault retries automatically | Not covered | Mitigated |
| Tamper sensor false positive → brick | A5 | Debounce, filter, factory override for RMA | Not covered | Mitigated |

### Elevation of Privilege

| Threat | Asset | New Mitigation | Old Status | New Status |
|--------|-------|---------------|------------|------------|
| Bypass bootloader | A1 | DICE chain + measured boot + TZ isolation | Partial | Mitigated (PH3) |
| Debug port re-enable | A1 | RDP Level 2 (irreversible) or DBG_AUTH | Not covered | Mitigated |
| TrustZone secure world entry via NS | A8, A9 | SAU/IDAU enforces SGATE-only entry | Not covered | Mitigated (TZ) |
| Fault injection → execute arbitrary code | A1 | CFI, stack canaries, redundant checks | Not covered | Mitigated (PH3) |

## Gaps (Post-Implementation Residual Risks)

| ID | Gap | Rationale | Mitigation |
|----|-----|-----------|------------|
| G1 | OTP extractable via invasive probing | Physical attack with FIB workstation | Defense-in-depth: RDP L2 + encrypted flash |
| G2 | IWDG oscillator failure | Extremely rare, per MCU FIT data | External watchdog as secondary option |
| G3 | Compromised signing HSM | Physical security at factory | HSM in tamper-proof enclosure |
| G4 | No runtime integrity on F411/H7 | No TrustZone → monitor can be disabled | IWDG periodic reset force-checks |
| G5 | White-box crypto not production-ready | No NIST standard, no Rust ecosystem | Deferred until NIST standardization |
| G6 | Certification cost | EAL5+ >€500K | EAL4+ alternative under evaluation |
| G7 | Rust compiler qualification | CC evaluation needs qualified TOEs | Engage lab with Rust experience |

## Attack Tree (Updated)

### Firmware Extraction Attack
```
Gain physical access to device
├─ Read flash directly
│  ├─ Desolder flash chip → encrypted → blocked by AES-GCM
│  └─ SWD/JTAG → RDP Level 2 → blocked
├─ Read flash while running
│  ├─ Debug probe → RDP Level 2 → blocked
│  └─ DMA attack → TrustZone isolation → blocked (TZ)
├─ Extract key from memory
│  ├─ Cold boot → SRAM fades quickly + zeroized → blocked
│  └─ Debug probe → RDP Level 2 → blocked
├─ Side-channel key extraction
│  ├─ Power analysis → constant-time → blocked
│  ├─ EM analysis → constant-time + random delay → blocked
│  └─ Timing → constant-time → blocked
└─ Fault injection to skip decrypt
    ├─ Voltage glitch → redundant check → blocked
    └─ Clock glitch → CSS detection → blocked (L5/U5)
```

### Rollback Attack
```
Modify version to old value
├─ Modify flash version field
│  └─ OTP counter → version enforced in hardware → blocked
├─ Modify OTP counter value
│  └─ OTP is write-once → cannot decrement → blocked
├─ Replace entire flash with old image
│  └─ Version ≤ OTP counter → blocked
└─ Replace OTP (physical)
    └─ Invasive probe + FIB → extremely expensive → deterrent
```