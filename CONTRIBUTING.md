# Contributing to rustBoot

## Welcome

Thank you for your interest in rustBoot. This project aims to provide a secure
bootloader suitable for safety- and security-conscious embedded deployments.

## Getting Started

1. Read the [README](README.md)
2. Set up the development environment per `rust-toolchain.toml`
3. Run `just verify` before submitting changes
4. Review open issues and PRs

## Development Workflow

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run `just verify` (format, clippy, test, audit, deny)
5. Submit a pull request

## Code Style

- Run `cargo fmt --all` before committing
- All warnings must be addressed (CI has `-D warnings`)
- No `unwrap()`, `expect()`, `panic!()`, or `todo!()` in production code
- All `unsafe` blocks must have a `// Safety:` comment
- Prefer typed errors over panics
- New features must include tests

## Pull Request Requirements

Every PR must:

- [ ] Pass `cargo fmt --all --check`
- [ ] Pass `cargo clippy -D warnings`
- [ ] Pass `cargo test --workspace`
- [ ] Pass `cargo audit` (no new vulnerabilities)
- [ ] Pass `cargo deny check` (no license issues)
- [ ] Include tests for new functionality
- [ ] Update documentation if behavior changes
- [ ] Reference any related issue or requirement ID

## Architectural Principles

- `no_std` compatibility for core library
- Deterministic boot flow (explicit state machine)
- Defense in depth (SHA-256 + ECDSA + anti-rollback + fallback)
- Fail closed (return errors, never panic in production)
- Bounded resource usage (no unbounded allocation in boot path)

## License

By contributing, you agree that your contributions will be licensed under the
MIT License as specified in the repository.