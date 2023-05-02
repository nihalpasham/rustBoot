![GitHub](https://img.shields.io/github/license/nihalpasham/rustBoot) [![ci](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml/badge.svg)](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml) [![chat](https://img.shields.io/badge/chat-rustBoot%3Amatrix.org-brightgreen)](https://matrix.to/#/#rustBoot:matrix.org)
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
- [x] a `signing utility` to sign bare-metal firmware and fit-image(s), written in pure rust.

## Features planned:

- [ ] support for external flash devices (ex: SPI flash) and serial/console logging interfaces.
- [ ] support for `ARM TrustZone-M and A` and certified `secure hardware elements` - microchip ATECC608a, NXP SE050, STSAFE-100
- [ ] support for secure, distributed and efficient `firmware transport` over [ipfs](https://ipfs.tech/).

## Documentation:

You can read the book for <a href="https://nihalpasham.github.io/rustBoot-book/index.html" target="_blank">`free online`.</a>. 

> Note: `rustBoot` and the `book` are still in development (i.e. a work in progress).

## Acknowledgment: 

rustBoot exists as we could not find a suitable (open-source) option that meets our security goals. It is the result of an exhaustive evaluation of 'pretty much' the entire embedded-bootloader landscape. 

Having said that, it does take inspiration from similar projects (such as u-boot, zephyr, mcuboot, coreboot, wolfBoot etc). However, the key differentiator is security-above-all-else. To that extent, its built entirely in rust, takes full advantage of rust's memory safety guarantees while leveraging safer parsing libraries, compile-time state-transition checks coupled with (safe) community sourced rust-crates (such as boards, HALs drivers etc.)

## Support:

For questions, issues, feature requests, and other changes, please file an issue in the github project.

## License:

rustBoot is licensed under 
 
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributing:
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.