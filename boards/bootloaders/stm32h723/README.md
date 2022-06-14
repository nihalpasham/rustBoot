`rustBoot` support for [stm32h723zg](https://www.st.com/en/evaluation-tools/nucleo-h723zg.html) nucleo board, we have one example. It has a custom led configuration. If you're using a different version of the board, you'll probably need to edit `firmware and hal implementations` to accomodate for differences. Just make sure you **dont change** the name of files/folders or the folder structure, as `cargo xtask` looks for these file/folder names.

- In order to test this example you'll need a couple of things - `wolfcrypt, probe-run, python3, stm32cube-Programmer installed`
- If you've managed to install all of them, you can use below commands to build and sign all 3 packages (i.e. bootloader + bootfw + updatefw) onto the board.
    - Command for build rustBoot
    `cargo stm32h723zg build rustBoot-only`

    - Command for build packages
    `cargo stm32h723zg build pkgs-for`

    - Command for sign packages
    `cargo stm32h723zg sign pkgs-for`

- In order to flash all 3 binarise (i.e. bootloader + bootfw + updatefw) I've used `probe-run` and `stm32cube-programmer`.
    - To flash bootloader use this command
    `probe-run < bootloader file name > --chip stm32h723zgtx`
    - To flash bootfw + updatefw use `stm32cube programer` 

- In order to confirm that its working, I've configured the `bootfw to blink green` for a few seconds, trigger an update and then reset. Upon reset, the bootloader verifies the update and swaps the contents of boot and update partitions. If everything checks out, it boots into the update, `blinks a red led` and finally sets the confirmation flag to indicate that the update was successful.

Here's the [command line output](/boards/bootloaders/stm32h723/debug.md).

## Blinky(s):

**blinks green before image verification and swap, after trigger an update, blinks red after image verification and swap:**

[![bootfw_and_updtfw](https://user-images.githubusercontent.com/92363511/173661166-bad18bd5-8e35-4429-8852-93ea29b46ed9.png)](https://user-images.githubusercontent.com/92363511/173660773-4f4d7cbd-6d43-4418-b5b5-099619054aff.mov)