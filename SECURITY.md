# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| main (rolling) | ✅ |
| tagged releases | ✅ (latest only) |

Only the latest tagged release and the `main` branch receive security patches.
Older releases must be updated to the latest version to receive fixes.

## Reporting a Vulnerability

rustBoot is a secure bootloader for embedded systems. If you discover a
security vulnerability, please do NOT file a public issue.

Contact the maintainers directly:

- **Email**: See commit history for maintainer contact
- **GitHub Security Advisories**: Use the "Report a vulnerability" link under
  the repository's Security tab
- **PGP**: Maintainer PGP keys are available in the commit metadata

We will acknowledge receipt within 48 hours and provide an initial assessment
within 5 business days. Fix timelines depend on severity:

| Severity | Target Fix Time |
|----------|-----------------|
| Critical (boot bypass, signature bypass) | 14 days |
| High (integrity bypass, rollback) | 30 days |
| Medium (denial of service, partial bypass) | 60 days |
| Low (informational) | Next release |

## Responsible Disclosure

We request that vulnerabilities be disclosed responsibly:

1. Report privately (see above)
2. Allow reasonable time for a fix before public disclosure
3. Do not exploit the vulnerability

## Scope

The following are in scope for security reports:

- Boot flow bypass
- Signature verification bypass
- Anti-rollback bypass
- Integrity check bypass
- Secure boot state machine violations
- Cryptographic weaknesses in the boot path
- Supply chain vulnerabilities in dependencies

## Out of Scope

- Physical attacks requiring specialized equipment (unless low-cost)
- Attacks requiring debug interfaces to be enabled (production config assumed)
- Vulnerabilities in third-party HAL implementations (report to respective project)

## Threat Model

See `docs/threat-model/stride.md` for the current threat model covering 20+
identified threats across all STRIDE categories (Spoofing, Tampering,
Repudiation, Information Disclosure, Denial of Service, Elevation of
Privilege). Each threat is mapped to assets, mitigations, and verification
status.

## Safety Case

See `docs/safety_case.md` for the formal safety case documenting 7 safety goals
(G1–G7), hazards and mitigations, a full verification matrix, and residual risk
assessment. The safety case covers:

- G1: Image Integrity (SHA-256)
- G2: Image Authenticity (ECDSA P-256)
- G3: State Machine Correctness
- G4: Partition Arithmetic Safety
- G5: Parser Robustness
- G6: Unsafe Code Containment
- G7: Build Determinism

## Formal Verification

rustBoot employs three layers of formal verification:

- **Kani** (Rust model checker): 19 proof harnesses in `kani/src/lib.rs`
  verifying bounded safety of parser extract functions, partition offset
  arithmetic, state machine transitions, and compile-time constant consistency.
  All proofs pass with `cargo kani --workspace`.
- **Proptest** (property-based testing): State encoding/decoding roundtrips,
  parser never-panic guarantees, invalid input rejection, and output length
  invariants across 6+ properties.
- **Fuzz** (coverage-guided): 4 fuzz targets (`image_header_parser`,
  `config_parser`, `fit_parser`, `dtb_parser`) with verified never-panic
  guarantees on arbitrary inputs.

Changes to security-critical code (parser, crypto, state machine) must include
corresponding formal verification evidence.

## Fuzz Testing

4 fuzz targets in `fuzz/fuzz_targets/` ensure parsers never panic:
- `image_header_parser` — TLV image header (arbitrary byte sequences)
- `config_parser` — Update configuration file
- `fit_parser` — FIT image format
- `dtb_parser` — Device tree blob

All targets are verified to never panic, panic, or abort on any input.