![GitHub](https://img.shields.io/github/license/nihalpasham/rustBoot) ![GitHub Workflow Status (event)](https://img.shields.io/github/workflow/status/nihalpasham/rustBoot/ci) [![chat](https://img.shields.io/badge/chat-rustBoot%3Amatrix.org-brightgreen)](https://matrix.to/#/#rustBoot:matrix.org)
# rustBoot 
rustBoot is a standalone bootloader, written entirely in `Rust`, designed to run on anything from a microcontroller to a system on chip. It can be used to boot into bare-metal firmware or Linux.

![rustBoot](https://user-images.githubusercontent.com/20253082/131207587-5c0caba7-f70a-4062-bd53-5035fd6df668.png "rustBoot - Just a secure bootloader and nothing more!")

## Why rustBoot?

rustBoot aims to offer an OS and micro-architecture agnostic (i.e. highly portable) secure bootloader which is standards-compatible and easy to integrate into existing embedded software projects.

![What is rustBoot](https://user-images.githubusercontent.com/20253082/131283947-98b77b33-65e9-4a6a-b554-4ec6fb4813c2.png "So, how does rustBoot help")

## Features currently supported:

- [x] support for `ARM Cortex-M, Cortex-A` micro-architectures
- [x] support for multi-slot partitioning of microcontroller flash memory. This allows us to implement the `boot/update` approach for bare-metal `firmware updates`.
- [x] support for `Aarch64 linux` booting
- [x] elliptic curve cryptography for integrity and authenticity verification using [`RustCrypto`](https://github.com/RustCrypto) crates
- [x] a tiny hardware abstraction layer for non-volatile memory (i.e. flash) access.
- [x] anti-rollback protection via version numbering.
- [x] a fully memory safe core-bootloader implementation with safe parsers and firmware-update logic.
- [x] power-interruptible firmware updates along with the assurance of fall-back availability.

## Features planned:

- [ ] switch to `rust-based signing tools` for manifest-header creation, key-generation and firmware signing to improve scalability and security (currently examples use a python implementation for this). 
- [ ] support for external flash devices (ex: SPI flash) and serial/console logging interfaces.
- [ ] support for `ARM TrustZone-M and A` and certified `secure hardware elements` - microchip ATECC608a, NXP SE050, STSAFE-100
- [ ] support for a highly secure and efficient `firmware transport` method over end-end mutually authenticated and encrypted channels via [ockam-networking-libraries](https://github.com/ockam-network/ockam/tree/develop/documentation/use-cases/end-to-end-encryption-with-rust#readme).

## Documentation:

You can read the book for [`free online`](https://nihalpasham.github.io/rustBoot-book/index.html). 

> Note: `rustBoot` and the `book` are still in development (i.e. a work in progress).

## Acknowledgment: 

rustBoot's design was influenced by [wolfBoot](https://github.com/wolfSSL/wolfBoot). It borrows wolfBoot's `reliable-update` design idea and builds on it with rust's memory safety guarantees, safer parsing libraries, compile-time state-transition checks and easy integration with crates (such as boards, HALs drivers etc.) developed by the [embedded-rust](https://crates.io/categories/embedded) community.

## Support:

For questions, issues, feature requests, and other changes, please file an issue in the github project.

## License:

rustBoot is licensed under 
 
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributing:
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.