# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | ✅ |

## Reporting a Vulnerability

rustBoot is a secure bootloader for embedded systems. If you discover a
security vulnerability, please do NOT file a public issue.

Contact the maintainers directly:

- **Email**: See commit history for maintainer contact
- **GitHub Security Advisories**: Use the "Report a vulnerability" link under
  the repository's Security tab

We will acknowledge receipt within 48 hours and provide a timeline for a fix.

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

See `docs/threat-model/stride.md` for the current threat model.