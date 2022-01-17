# rustBoot [![](https://tokei.rs/b1/github/nihalpasham/rustBoot?category=code)](https://github.com/nihalpasham/rustBoot)
rustBoot is a standalone bootloader, written entirely in `Rust`, designed to run on anything from a microcontroller to a system on chip. It can be used to boot into bare-metal firmware or Linux.

![rustBoot](https://user-images.githubusercontent.com/20253082/131207587-5c0caba7-f70a-4062-bd53-5035fd6df668.png "rustBoot - Just a secure bootloader and nothing more!")

## Why build another `bootloader`? 

![Why rustBoot](https://user-images.githubusercontent.com/20253082/131207633-8fb5afc9-e879-407e-bf33-3a342f1adad3.png "Why build another bootloader")

## So, how does `rustBoot` help?

rustBoot aims to offer an OS and micro-architecture agnostic (i.e. highly portable) secure bootloader which is standards-based, easy to integrate (into existing embedded software projects) and eliminates duplication of work for every project. 

![What is rustBoot](https://user-images.githubusercontent.com/20253082/131283947-98b77b33-65e9-4a6a-b554-4ec6fb4813c2.png "So, how does rustBoot help")

## Goals and objectives:

A secure-bootloader is a complex project, despite its small size. The effort involved in developing/integrating one can be overwhelming. (key-management, code-safety, trust-chains, reliable back-ups etc.)

rustBoot's purpose is to help simplify the entire process. Some of its primary goals are  

- **Compliance with the [IETF-SUIT](https://datatracker.ietf.org/wg/suit/about/) standard** i.e.
    - one of its requirements is to not require the use of specific protocols or data link interfaces to transfer `updates` to a device. 
    - transferring an `update` should be delegated to the firmware/OS to avoid `size or computational` limitations (along with a drastic reduction in attack surface).
- **Reliable updates:**
    - rustBoot will perform swap operations via the `A/B or multi-slot partitioning method` to replace currently active firmware with a newly received update and at the same time store a back-up copy of it in a (passive) secondary partition.
- **Predictablility over Performance:** 
    - one of rustBoot's core design objectives is to keep it simple and avoid complexity. So, there will be little to no application of meta or async programming constructs. 
    - not that we need the extra performance, rustBoot can already hit sub-second secure boot-times as we've stripped it down to the bare-essentials.
- **Zero-dynamic memory allocation:**
    - to make it highly portable, apart from its modular design, rustBoot relies on a zero dymnamic memory allocation architecture i.e. no heap required. 
- **Memory safety & type-state programming:** 
    - the entire bootloader is written in rust's safe-fragment with a limited set of well-defined api(s) for unsafe HW access.
    - as a consequence, it makes rustBoot immune to a whole host of memory safety bugs. ex: things like parsing image-headers (i.e. container-formats) in rustBoot is much safer.
    - rustBoot takes advantage of rust's powerful type-system to make `invalid boot-states unrepresentable at compile time` and along with constructs such as sealed states, global singletons, it improves the overall security of the entire code-base.
- **Formal guarantees:** an *aspirational* goal at this point but we think its do-able
    - *property-based testing via symbolic execution:* to formally verify rustBoot's parser.
    - *deductive verification:* for critical sections of code (ex: swapping contents of boot and update partitions).

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
    - *Note: I'm still contemplating switching to something like embedded-storage. Please feel free to chime-in if you have suggestions here.*
- **memory-layout:** choose a suitable memory layout based on the board's micro-architecture and model. 
- **firmware-api:** use the `UpdateInterface` api to trigger and confirm firmware updates (in your firmware). 

Note - downloading and installing the update is to be handled by whatever firmware/OS you're running.

## rustBoot's high-level design

![rustBoot – Secure bootloader architecture](https://user-images.githubusercontent.com/20253082/131221352-12e742c9-f88f-42ba-98a5-f0f3e6109e94.png "rustBoot – Secure bootloader architecture")
![rustBoot – Application interface](https://user-images.githubusercontent.com/20253082/131221381-c1c81a2a-b93f-41ee-b6c0-a201d286eee0.png "rustBoot – Application interface")

## Features:

- A/B or multi-slot partitioning of the flash device. 
- elliptic curve cryptography for integrity and authenticity verification using [RustCrypto](https://github.com/RustCrypto) 
- a tiny hardware abstraction layer for non-volatile memory (i.e. flash) programming.
- anti-rollback protection via version numbering. 
- a fully memory safe core-bootloader implementation with safe parsers and flash-update logic.
- power-interruptible firmware updates along with the assurance of fall-back availability.

## Future roadmap:

- switch to a `rust-based KMI` for manifest-header creation, key-generation and firmware signing to improve scalability and security (currently the lone available example uses `wolfboot's` python implementation for this). 
- support for external flash devices and serial/console logging interfaces.
- support for ARM TrustZone-M and A and certified secure hardware elements - `microchip ATECC608a, NXP SE050, STSAFE-100`
- support for a highly secure and efficient `firmware transport` method over end-end mutually authenticated and encrypted channels via [ockam-networking-libraries](https://github.com/ockam-network/ockam/tree/develop/documentation/use-cases/end-to-end-encryption-with-rust#readme).
- many more examples with `test implementations` for a variety of boards.

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
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the MIT license, shall be dual licensed as above, without any additional terms or conditions.