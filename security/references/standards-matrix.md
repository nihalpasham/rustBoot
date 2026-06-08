---
title: Standards Compliance Matrix
status: draft
date: 2026-06-08
version: 0.1.0
---

# Standards Compliance Matrix

## Measure → Standard → SFR → Test

| # | Security Measure | FIPS 140-3 | CC EAL5+ | NIST 800-193 | CNSA 2.0 | SUIT RFC 9019 | SFRs | Primary Tests |
|---|-----------------|------------|----------|-------------|----------|---------------|------|---------------|
| 01 | FW Encryption | §7.9 CSP encrypt | FCS_COP.1 | — | AES-256 | §7 confidentiality | FCS_COP.1/SKC | ct_eq, roundtrip, tag mismatch |
| 02 | Anti-Rollback | — | FDP_SDI.2 | §4.2.2.2, §4.2.5 | SVN req | — | FDP_SDI.2 | monotonic, increment, exhaust |
| 03 | Crypto POST | §7.10 POST | FPT_TST.1 | — | — | — | FPT_TST.1, FCS_COP.1/KAT | KAT pass/fail, conditional KAT |
| 04 | Debug Lockdown | §7.4 auth | FPT_PHP.3 | §4.1 | — | — | FPT_PHP.3 | RDP level, DBG_AUTH cert |
| 05 | PQ Signatures | — | FCS_COP.1 | — | LMS/XMSS | §7.3 auth | FCS_COP.1/SIG | LMS verify, XMSS verify, dual |
| 06 | Measured Boot | — | FDP_SDI.2 | §4.3 | SHA-384 | §5 measurement | FDP_SDI.2, FCS_COP.1/Hash | extend, read IMR, log |
| 07 | Fail-Secure Recovery | §7.10.1 | FPT_FLS.1, FPT_RCV.3 | §4.4 | — | — | FPT_FLS.1, FPT_RCV.3 | error paths, golden boot |
| 08 | Key Management | §7.9, §7.11 | FCS_CKM.1, FCS_CKM.4 | — | key sizes | — | FCS_CKM.1, FCS_CKM.4 | derive, zeroize, provision |
| 09 | IWDG Protection | — | — | §4.4.2 | — | — | — | lock, kick, timeout |
| 10 | Continuous-Power | §7.9.7 | FPT_FLS.1 | §4.2.1.4 | — | — | FPT_FLS.1 | key zeroize, RAM fresh, DIK |
| 11 | Side-Channel | §7.11 | AVA_VAN.5 | — | — | — | — | ct timing, no secret branches |
| 12 | Key Zeroization | §7.9.7 | FCS_CKM.4 | — | — | — | FCS_CKM.4 | volatile_write, Drop, exception |
| 13 | Runtime Integrity | — | — | §4.3, §4.4 | — | — | FDP_SDI.2 | periodic hash, tamper flag |
| 14 | Tamper Detection | §7.9.7 | FPT_PHP.3 | §4.2.1.4 | — | — | FPT_PHP.3 | pin trigger, zeroize, persist |
| 15 | TrustZone | — | ADV_ARC.1 | §4.2.1.3 | — | — | ADV_ARC.1, FDP_IFC.1 | NS→S isolation, SGATE |
| 16 | DICE | — | FCS_CKM.1 | — | — | §5 trust anchor | FCS_CKM.1 | chain derive, cert verify |
| 17 | White-Box | §7.9 | — | — | — | — | — | (research) |
| 18 | Honeypot | — | — | §4.3 | — | — | — | detect, escalate, zeroize |
| 19 | Attestation | — | — | — | — | §5 measurement, RATS | — | report gen, verify, nonce |
| 20 | CFI | — | ADV_ARC.1 | — | — | — | ADV_ARC.1 | canary, dispatch table |
| 21 | Fault Injection | §7.11 | AVA_VAN.5 | — | — | — | — | redundant checks, sentinel, CRC |
| 22 | SUIT Manifest | — | — | — | — | §6 manifest, §7 auth | — | parse, COSE verify, match |
| 23 | Certification | §7.12 lifecycle | ALC, ASE, ADV, ATE, AVA | — | — | — | All EAL5+ | lab evaluation |
| 24 | Vuln Analysis | §7.11 test | AVA_VAN.5 | — | — | — | AVA_VAN.5 | fuzz, pen test, side-channel |
| 25 | Supply Chain | ALC lifecycle | ALC_DVS.2, ALC_CMC.4 | — | — | §10 trust | ALC_DVS.2, ALC_CMC.4 | repro build, SBOM, provision |

## Standard → Measures

### FIPS 140-3 Level 3 Required

| Section | Measure(s) |
|---------|-----------|
| §7.1 Cryptographic module specification | 01, 08 |
| §7.2 Module interfaces | 08, 15 |
| §7.3 Roles, services, authentication | 04 |
| §7.4 Software/firmware security | 03, 07 |
| §7.5 Operating environment | 15 |
| §7.6 Physical security | 04, 14, 10 |
| §7.7 Non-invasive security | 11, 21 |
| §7.8 Sensitive security parameter mgmt | 08, 12, 10 |
| §7.9 Self-tests | 03 |
| §7.10 Life-cycle assurance | 23, 24, 25 |
| §7.11 Mitigation of other attacks | 11, 14, 21 |

### Common Criteria EAL5+ (BSI-CC-PP-0084) Required

| SFR Class | Family | Measure(s) |
|-----------|--------|-----------|
| FCS | FCS_RNG.1 | 08 |
| FCS | FCS_COP.1 | 01, 03, 05, 06 |
| FCS | FCS_CKM.1, FCS_CKM.4 | 08, 12, 16 |
| FDP | FDP_SDI.2 | 02, 06, 13 |
| FDP | FDP_IFC.1 | 15 |
| FIA | FIA_UAU.2 | 04 |
| FIA | FIA_AFL.1 | 04 |
| FPT | FPT_TST.1 | 03 |
| FPT | FPT_FLS.1 | 07, 10 |
| FPT | FPT_RCV.3 | 07 |
| FPT | FPT_PHP.3 | 04, 14 |
| ADV | ADV_ARC.1 | 15, 20 |
| AVA | AVA_VAN.5 | 11, 21, 24 |
| ALC | ALC_DVS.2 | 25 |
| ALC | ALC_CMC.4 | 25 |

### NIST SP 800-193 Required

| Principle | Requirement | Measure(s) |
|-----------|-------------|-----------|
| Protection | Authenticated update (RTU) | 01, 05 |
| Protection | Flash write protection | 04, 09 |
| Protection | Non-bypassability | 15, 07 |
| Detection | Boot integrity verification | 06 |
| Detection | Runtime integrity | 13 |
| Detection | Tamper detection | 14 |
| Recovery | Backup/recovery image | 07 |
| Recovery | Golden boot | 07 |
| Recovery | Authenticated recovery | 07 |

### NSA CNSA 2.0 Required

| Algorithm | CNSA 2.0 Parameters | Measure(s) |
|-----------|---------------------|-----------|
| AES | AES-256 only | 01 |
| SHA | SHA-384 or SHA-512 | 06 |
| FW signing | LMS or XMSS (NIST SP 800-208) | 05 |
| Key exchange | ML-KEM-1024 (future) | — |
| Signatures (future) | ML-DSA-87 (future) | — |