# rustBoot Threat Model (STRIDE)

## Assets
- **A1**: Boot firmware integrity (must not be tampered)
- **A2**: Private signing key used for firmware signing (must remain confidential)
- **A3**: Update firmware authenticity (must be from authorized source)
- **A4**: Boot state (must not be rolled back to vulnerable version)
- **A5**: Device availability (must not be bricked by failed update)

## Trust Boundaries
- **TB1**: Between firmware image on flash and bootloader
- **TB2**: Between update delivery mechanism and bootloader
- **TB3**: Between signing environment and device

## Threats

### Spoofing
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| Attacker provides fake firmware update | A2, A3 | ECDSA signature verification | Mitigated |
| Attacker provides fake boot image | A1 | SHA-256 integrity + ECDSA auth | Mitigated |

### Tampering
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| Attacker modifies firmware in flash | A1 | Integrity check on every boot | Mitigated |
| Attacker modifies sector flags | A4 | Sector flag state machine validation | Mitigated |
| Attacker modifies trailer magic | A4 | Trailer magic validation | Mitigated |

### Repudiation
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| No audit trail for boot decisions | A4 | Logging via defmt/log feature | Partial |

### Information Disclosure
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| Public key embedded in bootloader | N/A (public by design) | Acceptable risk: public key is public | Accepted |
| Firmware binary extracted from flash | A1 | Acceptable risk: no encryption | **GAP** |

### Denial of Service
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| Corrupt firmware causes boot loop | A5 | Fallback/rollback mechanism | Mitigated |
| Power loss during update | A5 | Power-interruptible swap | Mitigated |
| Flash wear-out from repeated updates | A5 | Acceptable risk: no wear leveling | Accepted |

### Elevation of Privilege
| Threat | Asset | Mitigation | Status |
|--------|-------|------------|--------|
| Attacker bypasses bootloader | A1 | Boot from ROM/OTP (hardware-dependent) | Partial |

## Attack Trees

### Firmware Tampering Attack
1. Gain physical access to flash
2. Read/modify firmware image
3. Recompute SHA-256 hash → requires signing key → **Blocked by ECDSA**

### Rollback Attack
1. Modify version field in header to an older value
2. SHA-256 hash will detect modification → but attacker can recompute
3. **Blocked by ECDSA signature**: any header modification invalidates the signature, which the attacker cannot forge without the private key

### Bricking Attack
1. Corrupt update firmware
2. Bootloader detects auth failure → falls back to previous version → **Blocked by fallback**

## Residual Risks
- **R1**: No firmware encryption (confidentiality not guaranteed)
- **R2**: No secure element integration (key stored in flash)
- **R3**: No signed boot state (trailer magic not authenticated)
- **R4**: No hardware watchdog integration
- **R5**: No secure debug disable verification