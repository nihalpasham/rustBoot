# rustBoot FMEA

## Scoring
- Severity: 1 (minor) to 10 (catastrophic)
- Occurrence: 1 (rare) to 10 (certain)
- Detection: 1 (certain) to 10 (impossible)
- RPN = Severity × Occurrence × Detection

## Boot Flow FMEA

| Component | Failure Mode | Effect | Detection | Mitigation | Sev | Occ | Det | RPN |
|-----------|-------------|--------|-----------|------------|:---:|:---:|:---:|:---:|
| Flash read | Corrupt magic number | Boot fails to identify image | Magic check | Return InvalidImage error | 8 | 2 | 2 | 32 |
| Flash read | Corrupt firmware size | Incorrect hash computation | Size bounds check | Return InvalidFirmwareSize | 9 | 2 | 3 | 54 |
| SHA-256 | Hash mismatch | Integrity check fails | Hash comparison | Return `IntegrityCheckFailed` | 9 | 1 | 1 | 9 |
| ECDSA verify | Signature invalid | Auth check fails | Signature verification | Return FwAuthFailed | 9 | 1 | 1 | 9 |
| Flash write | Write fails during swap | Partial update | Sector flag check | Resume on reboot | 7 | 3 | 2 | 42 |
| Flash erase | Erase fails | Swap incomplete | Sector flag check | Resume on reboot | 7 | 2 | 2 | 28 |
| Power loss | Interrupted during swap | Inconsistent state | Sector flag state machine | Resume from last known state | 8 | 4 | 1 | 32 |
| Version check | Version <= current version | Anti-rollback bypass | Version comparison returns `FwAuthFailed` | Return `FwAuthFailed` | 6 | 1 | 3 | 18 |
| Config parser | Malformed updt.txt | Incorrect update config | nom parser error | Return InvalidValue | 5 | 2 | 2 | 20 |
| DTB parser | Malformed device tree | Boot failure | DT parser validation | Return InvalidImage | 7 | 1 | 2 | 14 |

## High-Risk Items (RPN > 50)

### Corrupt firmware size (RPN: 54)
- **Mitigation**: Size must be <= PARTITION_SIZE - IMAGE_HEADER_SIZE
- **Verification**: `open_partition()` bounds check
- **Action**: Add fuzz target for header parsing with arbitrary sizes

## Accepted Risks
- Flash wear-out: No wear leveling implemented. Acceptable for expected update count (<1000).
- No firmware encryption: Acceptable for trusted flash environments.