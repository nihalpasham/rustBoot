```zsh terminal
    ~/Github/rustBoot  on    main !17 ?3 
❯ cargo stm32h723 build rustBoot-only
   Compiling typenum v1.15.0
   Compiling version_check v0.9.4
   Compiling subtle v2.4.1
   ..
   ..
   Compiling p256 v0.10.1
   Compiling rustBoot v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/rustBoot)
   Compiling xtask v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/xtask)
    Finished dev [unoptimized + debuginfo] target(s) in 8.60s
     Running `target/debug/xtask stm32h723 build rustBoot-only`
$ cargo build --release
   Compiling version_check v0.9.4
   Compiling typenum v1.15.0
   Compiling proc-macro2 v1.0.39
   ..
   ..
   Compiling stm32h7xx-hal v0.12.2
   Compiling rustBoot-hal v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/boards/hal)
   Compiling rustBoot-update v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/boards/update)
    Finished release [optimized] target(s) in 56.87s

    ~/Github/rustBoot  on    main !17 ?3 
❯ cargo stm32h723 build pkgs-for     
    Finished dev [unoptimized + debuginfo] target(s) in 0.10s
     Running `target/debug/xtask stm32h723 build pkgs-for`
$ cargo build --release
   Compiling panic-probe v0.3.0
   Compiling stm32h723_bootfw v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/boards/firmware/stm32h723/boot_fw_blinky_green)
    Finished release [optimized] target(s) in 2.14s
$ cargo build --release
   Compiling stm32h723_updtfw v0.1.0 (/Users/imrankhaleelsab/Github/rustBoot/boards/firmware/stm32h723/updt_fw_blinky_red)
    Finished release [optimized] target(s) in 1.81s
$ cargo build --release
    Finished release [optimized] target(s) in 0.09s
    
    ~/Github/rustBoot  on    main !17 ?3 
❯ cargo stm32h723 sign pkgs-for
    Finished dev [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/xtask stm32h723 sign pkgs-for`
$ python3 convert2bin.py
$ python3 signer.py
['sign.py', '--ecc256', '--sha256', 'stm32h723_updtfw.bin', 'ecc256.der', '1235']
Update type:          Firmware
Input image:          stm32h723_updtfw.bin
Selected cipher:      ecc256
Public key:           ecc256.der
Output image:         stm32h723_updtfw_v1235_signed.bin
Not Encrypted
Calculating sha256 digest...
Signing the firmware...
Done.
Output image successfully created.
['sign.py', '--ecc256', '--sha256', 'stm32h723_bootfw.bin', 'ecc256.der', '1234']
Update type:          Firmware
Input image:          stm32h723_bootfw.bin
Selected cipher:      ecc256
Public key:           ecc256.der
Output image:         stm32h723_bootfw_v1234_signed.bin
Not Encrypted
Calculating sha256 digest...
Signing the firmware...
Done.
Output image successfully created.

    ~/Github/rustBoot/boards/target/thumbv7em-none-eabihf/release  on    main !17 ?5 
❯ probe-run stm32h723 --chip stm32h723ZGTx
(HOST) WARN  insufficient DWARF info; compile your program with `debug = 2` to enable location info
(HOST) INFO  flashing program (45 pages / 45.00 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
```
