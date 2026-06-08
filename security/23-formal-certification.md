---
title: Formal Certification (EAL5+ / FIPS 140-3)
status: draft
date: 2026-06-08
version: 0.1.0
phase: 3
priority: medium
standards:
  - Common-Criteria-EAL5+
  - FIPS-140-3
bootloader: process
firmware: process
hardware: process
---

# Formal Certification (EAL5+ / FIPS 140-3)

## Summary

Formal security certification is a process, not code. This measure documents
the path to Common Criteria EAL5+ and FIPS 140-3 Level 3 certification for
the rustBoot bootloader and associated hardware platform.

## Certification Requirements

### EAL5+ (Common Criteria)

Based on BSI-CC-PP-0084-2014 (Secure Sub-System in SoC):

| Assurance Class | Components Required |
|----------------|-------------------|
| ADV (Development) | ADV_ARC.1, ADV_FSP.5, ADV_IMP.1, ADV_INT.2, ADV_TDS.4 |
| AGD (Guidance) | AGD_OPE.1, AGD_PRE.1 |
| ALC (Lifecycle) | ALC_CMC.4, ALC_CMS.5, ALC_DEL.1, ALC_DVS.2, ALC_LCD.1, ALC_TAT.2 |
| ATE (Tests) | ATE_COV.2, ATE_DPT.3, ATE_FUN.1, ATE_IND.2 |
| AVA (Vulnerability) | AVA_VAN.5 |
| ASE (Security Target) | ASE_CCL.1, ASE_ECD.1, ASE_INT.1, ASE_OBJ.2, ASE_REQ.2, ASE_SPD.1, ASE_TSS.1 |

### FIPS 140-3 Level 3

| Area | Requirements |
|------|-------------|
| Specification | Cryptographic boundary, module interfaces |
| Authentication | Identity-based operator authentication |
| Physical security | Tamper-evident enclosure, coatings |
| CSP management | Key generation, zeroization, encrypted CSPs |
| Self-tests | POST KATs, conditional tests |
| Lifecycle | CMVP-validated algorithms, CAVP certificates |
| Non-invasive | Side-channel testing (TE.11) |

## Certification Process

### Phase 1: Preparation (6-12 months)

1. Select a certification lab (e.g., SGS Brightsight, Riscure, UL)
2. Develop Security Target document (ST)
3. Prepare functional specification (semiformal, EAL5+)
4. Prepare high-level design (ADV_TDS.4)
5. Prepare security architecture (ADV_ARC.1)
6. Prepare implementation representation (ADV_IMP.1)
7. Establish configuration management (ALC_CMC.4)

### Phase 2: Evaluation (12-24 months)

1. Submit TOE (Target of Evaluation) to lab
2. Lab performs testing:
   - Functional testing (ATE_FUN.1)
   - Coverage analysis (ATE_COV.2)
   - Depth testing (ATE_DPT.3)
   - Independent testing (ATE_IND.2)
   - Vulnerability analysis (AVA_VAN.5)
3. Lab prepares Evaluation Technical Report (ETR)

### Phase 3: Certification (3-6 months)

1. Certification body reviews ETR
2. Certificate issued (valid for 5 years, renewable)
3. Maintenance: assurance continuity for updates

## Estimated Costs

| Item | Low | High |
|------|-----|------|
| Lab engagement fee | €200K | €500K |
| Documentation preparation | €50K | €150K |
| Testing (assumes 2 rounds) | €100K | €300K |
| Certification body fee | €20K | €50K |
| Engineering support | €100K | €300K |
| **Total** | **€470K** | **€1.3M** |

## Timeline

| Milestone | Duration | Cumulative |
|-----------|----------|------------|
| Security Target development | 6 mo | 6 mo |
| Functional spec + design docs | 6 mo | 12 mo |
| Lab evaluation | 12-18 mo | 24-30 mo |
| Certification decision | 3-6 mo | 27-36 mo |

## Dependencies

- Rust compiler must be evaluated as part of the toolchain (LC_TAT.2)
- Rust standard library (core) must be considered in the evaluation
- Third-party crate security must be assessed (supply chain)
- MCU platform must be certified separately or evaluated as composite

## Status

- **Preparation**: Not started
- **Lab selection**: Not started
- **Security Target**: Not written
- **Budget**: Not allocated

## Residual Risk

- Certification cost may exceed project budget. EAL5+ is the most expensive
  evaluation level commonly pursued. Alternative: EAL4+ with AVA_VAN.5
  augmentation provides meaningful security at lower cost.
- Rust ecosystem certification immaturity. Most CC evaluations target C/C++
  code. Rust's safety features are an advantage, but the evaluation lab must
  have Rust expertise.
- Toolchain qualification: Rust nightly + LLVM must be qualified. Changes in
  the compiler between evaluations are a lifecycle maintenance challenge.