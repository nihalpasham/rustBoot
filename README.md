![GitHub](https://img.shields.io/github/license/nihalpasham/rustBoot) [![ci](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/nihalpasham/rustBoot/actions/workflows/ci.yml) [![chat](https://img.shields.io/badge/chat-rustBoot%3Amatrix.org-brightgreen)](https://matrix.to/#/#rustBoot:matrix.org)
# rustBoot 
rustBoot is a standalone bootloader, written entirely in `Rust`, designed to run on anything from a microcontroller to a system on chip. It can be used to boot into bare-metal firmware or Linux.

![rustBoot](https://user-images.githubusercontent.com/20253082/131207587-5c0caba7-f70a-4062-bd53-5035fd6df668.png "rustBoot - Just a secure bootloader and nothing more!")

## Why rustBoot?

rustBoot aims to offer an OS and micro-architecture agnostic (i.e. highly portable) secure bootloader which is standards-based, easy to integrate into existing embedded software projects.

![What is rustBoot](https://user-images.githubusercontent.com/20253082/131283947-98b77b33-65e9-4a6a-b554-4ec6fb4813c2.png "So, how does rustBoot help")

## Project layout:

This project's folder structure is divided into 2 workspaces.
- **core-bootloader:** 
     - resides in its own folder called `rustBoot`
- **hardware abstraction layer**
    - the *boards* folder contains hardware-specific code. It contains the following folders
        - **rustBoot-hal:** contains the flash hardware abstraction layer (read/write/erase operations) for a specific board.
        - **rust-update:** this crate/folder contains board-agnostic A/B update logic.
        - **firmware:** contains board-specific firmware (i.e. boot and update).
        - **bootloaders:** contains bootloader implementations for different boards.

Additionally, the project includes a folder called `xtask` to simplify the `build-sign-flash` process.

For detailed instructions on usage, you can take a look at the `readme` page for each board under - `boards/bootloaders/{board-name}`

In short, you'll need 3 things:
- **flash-api:** implement the `FlashInterface` trait for your board (abstracts out the necessary HW-specific flash operations such as writing/readin/erasing). 
- **memory-layout:** choose a suitable memory layout based on the board's micro-architecture and model. 
- **firmware-api:** use the `UpdateInterface` api to trigger and confirm firmware updates (in your firmware). 

Note - downloading and installing the update is to be handled by whatever firmware/OS you're running.

## Features:

- A/B or multi-slot partitioning of the flash device. 
- elliptic curve cryptography for integrity and authenticity verification using [RustCrypto](https://github.com/RustCrypto) 
- a tiny hardware abstraction layer for non-volatile memory (i.e. flash) programming.
- anti-rollback protection via version numbering. 
- a fully memory safe core-bootloader implementation with safe parsers and flash-update logic.
- power-interruptible firmware updates along with the assurance of fall-back availability.

## Planned roadmap:

- switch to a `rust-based firmware signing tools` for manifest-header creation, key-generation and firmware signing to improve scalability and security (currently examples use `wolfboot's` python implementation for this).
- support for external flash devices and serial/console logging interfaces.
- support for ARM TrustZone-M and A and certified secure hardware elements - `microchip ATECC608a, NXP SE050, STSAFE-100`
- support for a highly secure and efficient `firmware transport` method over end-end mutually authenticated and encrypted channels via [ockam-networking-libraries](https://github.com/ockam-network/ockam/tree/develop/documentation/use-cases/end-to-end-encryption-with-rust#readme).
- more `test implementations` examples for a variety of boards. All examples are included in the bootloaders folder

## Documentation:

**[todo!]** - `rustBoot-book` goes here.

## Acknowledgment: 

rustBoot's design was influenced by [wolfBoot](https://github.com/wolfSSL/wolfBoot). It borrows wolfBoot's `reliable-update` design idea and builds on it with rust's memory safety guarantees, safer parsing libraries, compile-time state-transition checks and easy integration with crates (such as boards, HALs drivers etc.) developed by the [embedded-rust](https://crates.io/categories/embedded) community.

## Support:

For questions, issues, feature requests, and other changes, please file an issue in the github project.

## License:

rustBoot is licensed under 
 
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributing:
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.