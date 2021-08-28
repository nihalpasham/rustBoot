# rustBoot [![](https://tokei.rs/b1/github/nihalpasham/rustBoot?category=code)](https://github.com/nihalpasham/rustBoot)
rustBoot is a standalone bootloader, written entirely in `Rust`, designed to run on anything from a microcontroller to a system on chip. It can be used to boot into bare-metal firmware or Linux.

![rustBoot](https://user-images.githubusercontent.com/20253082/131207587-5c0caba7-f70a-4062-bd53-5035fd6df668.png "rustBoot - Just a secure bootloader and nothing more!")

## Why build another `bootloader`? 

![Why rustBoot](https://user-images.githubusercontent.com/20253082/131207633-8fb5afc9-e879-407e-bf33-3a342f1adad3.png "Why build another bootloader")


## So, how does `rustBoot` help?

![What is rustBoot](https://user-images.githubusercontent.com/20253082/131207667-6e565963-5b6a-40a8-a541-3ca151b939e2.png "So, how does rustBoot help")


## Goals or objectives:

- **Compliance with the [IETF-SUIT](https://datatracker.ietf.org/wg/suit/about/) standard** i.e.
    - rustBoot will not require the use of specific protocols or data link interfaces to transfer `updates` to a device. 
    - transferring an `update` should be delegated to the firmware/OS to avoid size or computational limitations along with a drastic reduction in attack surface.
- **Reliable updates:**
    - rustBoot will perform a swap operation via the `A/B partitions method` to replace currently active firmware with a newly received update and at the same time store a back-up copy of it in the secondary passive partition.
- **Predictablility over Performance:** 
    - one of rustBoot's core design objectives is to keep it simple and avoid complexity. So, there will be little to no application of meta or async programming constructs. 
    - Not that we need the extra performance, rustBoot can already hit sub-second boot-times as we've stripped it down to the bare-essentials.
- **Zero-dynamic memory allocation:**
    - to make it highly portable, rustBoot uses a zero-dymnamic memory allocation architecture i.e. no heap required. 
- **Memory safety & type-state programming:** 
    - The entire bootloader is written in rust's safe-fragment with a limited set of well-defined api(s) for unsafe HW access.
    - As a result, something like parsing headers (i.e. container-formats) in rustBoot is much safer. 
    - rustBoot takes advantage of rust's powerful type-system to instantiate global singletons along with making invalid boot-states unrepresentable at compile time. 
- **Formal guarantees:** this is *aspirational* at this point but we think its doable
    - *property-based testing via symbolic execution:* for safer container-format parsing.
    - *deductive verification:* for critical sections of code.(ex: swapping contents of boot and update partitions.)

## Project layout:

This project's folder structure is divided into 2 workspaces.
- **core-bootloader:** 
     - resides in its own folder called `rustBoot`
- **hardware abstraction layer**
    - the *boards* folder contains all hardware-specific code. It houses a few other neccessary folders
        - **rustBoot-hal:** contains flash-hal (read/write/erase) impls for a specific board.
        - **rust-update:** this crate/folder contains all of the board-agnostic A/B update logic.
        - **test_firmware:** contains test firmware (i.e. blinky-led firmware) for the boot and update partitions for a specific board.
        - **test_impls:** contains a test implementation of the bootoloader for a specific board.

Additionally, the project includes a folder called `xtask` to simplify the `build-sign-flash` process involved.

For detailed instructions on usage, you can take a look at the `readme` page for each board under - `boards/test_impls/{board-name}`

## Acknowledgment: 

rustBoot's design has been heavy influenced by that of [wolfBoot](https://github.com/wolfSSL/wolfBoot). It borrows much of wolfBoot's reliable update design and implementation (its pretty much what we were looking for) easy integration of crates (such as board, HALs drivers etc.) developed by the [embedded-rust](https://crates.io/categories/embedded) community.

## Future roadmap:
- switch to `rust based KMI` that's more scalable for firmware-signing, manifest-header creation and key-generation (currently the lone available example uses `wolfboot's` python implementation for this.)
- support for `firmware transport` over end-end mutually authenticated and encrypted channels via [ockam-networking-libraries](https://github.com/ockam-network/ockam/tree/develop/documentation/use-cases/end-to-end-encryption-with-rust#readme).
- support for external flash devices
- support for ARM TrustZone-M and A and certified secure hardware elements - `microchip ATECC608a, NXP SE050, STSAFE-100`
- many more examples with test implementations for a variety of boards.

## Support:
For questions, issues, feature requests, and other changes, please file an issue in the github project.

## License:
Parts of rustBoot were derived from the C implementaion of `wolfBoot`. So, rustBoot is licensed under either of

* GNU General Public License v2.0 
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contributing:
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.