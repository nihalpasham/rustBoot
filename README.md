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
- **Predicatblility over Performance:** 
    - one of the rustBoot's core design objectives is to keep it simple and avoid complexity. So, there will be little to no application of meta or async programming constructs. 
    - Not that we need extra performance as rustBoot can already hit sub-second boot-times as we've stripped it down to the bare-essentials.
- **Zero-dynamic memory allocation:**
    - to make it highly portable, rustBoot uses a zero-dymnamic memory allocation architecture i.e. no heap required. 
- **Memory safety & type-state programming:** 
    - The entire bootloader is written in rust's safe-fragment with a select set of well-defined api(s) around unsafe HW access.
    - As a result, something like parsing headers (i.e. container-formats) in rustBoot is much safer. 
    - rustBoot takes advantage of rust's powerful type-system to instantiate global singletons along with making invalid boot-states unrepresentable at compile time. 
- **Formal guarantees:** - this is *aspirational* at this point but we think its doable
    - property-based testing via symbolic execution: for safer container-format parsing.
    - deductive verification: for critical sections of code.(ex: swapping contents of boot and update partitions.)

## Usage:

## Acknowledgments: 

rustBoot's design has been heavy influenced by that of [wolfBoot](https://github.com/wolfSSL/wolfBoot). It borrows much of wolfBoot's reliable update design and implementation (didn't need to re-invent what we were looking for). However, rustBoot builds on top of it with rust's memory safety guarantees, safer parsing logic and  easy integration of crates (such as board, HALs drivers etc.) developed by the [embedded-rust](https://crates.io/categories/embedded) community.

### Future roadmap:
- switch to `rust based KMI` that's more scalable for firmware-signing, manifest-header creation and key-generation (currently the lone available example uses `wolfboot's` python implementation for this.)
- support for external flash devices
- support for ARM TrustZone-M and A and certified secure hardware elements - `microchip ATECC608a, NXP SE050, STSAFE-100`
- many more examples with test implementations for a variety of baords.
