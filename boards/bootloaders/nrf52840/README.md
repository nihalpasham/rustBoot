
We have one example for the [nrf52840-mdk](https://wiki.makerdiary.com/nrf52840-mdk/). This is a maker-diary board. It has a custom led configuration. If you're using a different version of the board, you'll probably need to edit `test-firmware implementations` to accomodate for differences. Just make sure you **dont change** the name of files/folders or the folder structure, as `cargo xtask` looks for these file/folder names.

- In order to test this example you'll need a couple of things - `windows with WSL2, pyocd, python3 installed`
- If you've managed to install all of them, you can simply call `cargo xtask build-sign-flash rustBoot nrf52840`. This will build, sign and flash all 3 packages (i.e. bootloader + bootfw + updatefw) onto the board.
- In order to confirm that its working, I've configured the `bootfw to blink green` for a few seconds, trigger an update and then reset. Upon reset, the bootloader verifies the update and swaps the contents of boot and update partitions. If everything checks out, it boots into the update, `blinks a red led` and finally sets the confirmation flag to indicate that the update was successful. 

*Note:* 
- *just a test-example, it does not contain a valid root-of-trust, the embedded public-key is placed in the signing_tools folder.* 
- *as the bootfw in this example does not include a networking stack, we flash both partitions with blinky firmware (i.e. bootfw-green and updfw-red) and have the bootfw manually trigger a reset to start the update process.*

Here's the command line output that should be produced.

```sh
PS C:\Users\Nil\devspace\rust\projects\rb> cargo xtask build-sign-flash rustBoot nrf52840
   Compiling xtask v0.1.0 (C:\Users\Nil\devspace\rust\projects\rb\xtask)
    Finished dev [unoptimized + debuginfo] target(s) in 1.06s
     Running `target\debug\xtask.exe build-sign-flash rustBoot nrf52840`
$ cargo build --release
   Compiling rand_core v0.6.3
   Compiling nb v1.0.0
   ...
   ...
   Compiling nrf52840_bootfw v0.1.0 (C:\Users\Nil\devspace\rust\projects\rb\boards\test_firmware\nrf52840\boot_fw_blinky_blue)
    Finished release [optimized + debuginfo] target(s) in 46.70s
$ cargo build --release
   Compiling nrf52840_updtfw v0.1.0 (C:\Users\Nil\devspace\rust\projects\rb\boards\test_firmware\nrf52840\updt_fw_blinky_red)
    Finished release [optimized + debuginfo] target(s) in 2.22s
$ cargo build --release
   Compiling defmt-rtt v0.2.0
   Compiling nrf52840 v0.1.0 (C:\Users\Nil\devspace\rust\projects\rb\boards\test_impls\nrf52840)
    Finished release [optimized + debuginfo] target(s) in 5.52s
$ python3 convert2bin.py
$ wsl python3 signer.py
['sign.py', '--ecc256', '--sha256', 'nrf52840_bootfw.bin', 'ecc256.der', '1234']
Update type:          Firmware
Input image:          nrf52840_bootfw.bin
Selected cipher:      ecc256
Public key:           ecc256.der
Output image:         nrf52840_bootfw_v1234_signed.bin
Not Encrypted
Calculating sha256 digest...
Signing the firmware...
Done.
Output image successfully created.
Update type:          Firmware
Input image:          nrf52840_updtfw.bin
Selected cipher:      ecc256
Public key:           ecc256.der
Output image:         nrf52840_updtfw_v1235_signed.bin
Not Encrypted
Calculating sha256 digest...
Signing the firmware...
Done.
Output image successfully created.
$ pyocd erase -t nrf52 --mass-erase
0001530:INFO:eraser:Successfully erased.
$ pyocd flash -t nrf52840 --base-address 0x2f000 nrf52840_bootfw_v1234_signed.bin
[====================] 100%
0001848:INFO:loader:Erased 4096 bytes (1 sector), programmed 4096 bytes (1 page), skipped 0 bytes (0 pages) at 4.84 kB/s
$ pyocd flash -t nrf52840 --base-address 0x56ffc trailer_magic.bin
[====================] 100%
0002045:INFO:loader:Erased 4096 bytes (1 sector), programmed 4096 bytes (1 page), skipped 0 bytes (0 pages) at 4.23 kB/s
$ pyocd flash -t nrf52840 --base-address 0x58000 nrf52840_updtfw_v1235_signed.bin
[====================] 100%
[====================] 100%
0001983:INFO:loader:Erased 4096 bytes (1 sector), programmed 4096 bytes (1 page), skipped 0 bytes (0 pages) at 4.22 kB/s
    Finished release [optimized + debuginfo] target(s) in 0.15s
    Flashing C:\Users\Nil\devspace\rust\projects\rb\boards\target\thumbv7em-none-eabihf\release\nrf52840
     Erasing sectors ✔ [00:00:01] [############################################################################] 44.00KiB/44.00KiB @ 24.39KiB/s (eta 0s )
 Programming pages   ✔ [00:00:03] [############################################################################] 44.00KiB/44.00KiB @  5.43KiB/s (eta 0s )
    Finished in 4.995s
PS C:\Users\Nil\devspace\rust\projects\rb>
```

## Blinky(s):

**blinks green before image verification and swap:**

[![blinky_bootfw_green](https://user-images.githubusercontent.com/20253082/131297750-05136516-c7e6-428d-807d-5a574eda5c3e.png)
](https://user-images.githubusercontent.com/20253082/131297185-8e93a741-f23a-492e-bcab-26c4c0b7efe4.mp4)

**blinks red after image verification and swap:**

[![blinky_updtfw_red](https://user-images.githubusercontent.com/20253082/131297971-b59506f5-8940-4798-a959-876d82965e5b.png)](https://user-images.githubusercontent.com/20253082/131295835-2941dd5e-775a-4798-9e46-a6225b0d9e02.mp4)

