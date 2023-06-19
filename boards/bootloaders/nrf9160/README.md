`rustBoot` support for [nrf9160](https://www.nordicsemi.com/Products/Development-hardware/nrf9160-dk) development board, we have one example. It has 4 leds. If you're using a different version of the board, you'll probably need to edit `firmware and hal implementations` to accomodate for differences. Just make sure you **dont change** the name of files/folders or the folder structure, as `cargo xtask` looks for these file/folder names.

- In order to test this example you'll need a couple of things - `wolfcrypt, probe-run, python3, nrf-connect Programmer installed`
- If you've managed to install all of them, you can use below commands to build and sign all 3 packages (i.e. bootloader + bootfw + updatefw) onto the board.
    - Command for build rustBoot
    `cargo nrf9160 build rustBoot-only`

    - Command for build packages
    `cargo nrf9160 build pkgs-for`

    - Command for sign packages
    `cargo nrf9160 sign pkgs-for`

- In order to flash all 3 binarise (i.e. bootloader + bootfw + updatefw) I've used `probe-rs-cli` and `probe-rs-cli`.
    - To flash bootloader use this command
    `probe-run < bootloader file name > --chip NRF9160_XXAA`
    - To flash bootfw + updatefw use following command
    'probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip nRF9160_xxAA nrf9160_bootfw_v_signed.bin'

- In order to confirm that its working, I've configured the `bootfw to turn ON LED1 and blink LED2` for a few seconds, trigger an update and then reset. Upon reset, the bootloader verifies the update and swaps the contents of boot and update partitions. If everything checks out, it boots into the update, `turn ON LED3 and blink LED4` and finally sets the confirmation flag to indicate that the update was successful.

Here's the [command line output](/boards/bootloaders/stm32h723/debug.md).

## Blinky(s):

**blinks green before image verification and swap, after trigger an update, blinks red after image verification and swap:**

[![bootfw_and_updtfw](https://user-images.githubusercontent.com/92363511/173661166-bad18bd5-8e35-4429-8852-93ea29b46ed9.png)](https://user-images.githubusercontent.com/92363511/173660773-4f4d7cbd-6d43-4418-b5b5-099619054aff.mov)