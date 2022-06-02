anand@anand-VirtualBox:~/Desktop/dev_space/production/rustBoot-forked/my_rustBoot$ cargo stm32f446 build pkgs-for
   Compiling version_check v0.9.4
   Compiling typenum v1.15.0
   Compiling subtle v2.4.1
   Compiling rand_core v0.6.3
   Compiling proc-macro2 v1.0.39
   Compiling const-oid v0.7.1
   Compiling unicode-ident v1.0.0
   Compiling zeroize v1.4.3
   Compiling syn v1.0.95
   Compiling base16ct v0.1.1
   Compiling defmt-macros v0.3.2
   Compiling memchr v2.5.0
   Compiling cfg-if v1.0.0
   Compiling opaque-debug v0.3.0
   Compiling defmt v0.3.1
   Compiling cpufeatures v0.2.2
   Compiling defmt-parser v0.3.1
   Compiling log v0.4.17
   Compiling bitflags v1.3.2
   Compiling anyhow v1.0.57
   Compiling stable_deref_trait v1.2.0
   Compiling minimal-lexical v0.2.1
   Compiling xshell-macros v0.1.17
   Compiling byteorder v1.4.3
   Compiling ff v0.11.1
   Compiling der v0.5.1
   Compiling generic-array v0.14.5
   Compiling proc-macro-error-attr v1.0.4
   Compiling proc-macro-error v1.0.4
   Compiling as-slice v0.2.1
   Compiling group v0.11.0
   Compiling xshell v0.1.17
   Compiling quote v1.0.18
   Compiling spki v0.5.4
   Compiling nom v7.1.1
   Compiling pkcs8 v0.8.0
   Compiling digest v0.9.0
   Compiling crypto-bigint v0.3.2
   Compiling crypto-mac v0.11.1
   Compiling sec1 v0.2.1
   Compiling block-buffer v0.9.0
   Compiling hmac v0.11.0
   Compiling signature v1.3.2
   Compiling sha2 v0.9.9
   Compiling rfc6979 v0.1.0
   Compiling elliptic-curve v0.11.12
   Compiling ecdsa v0.13.4
   Compiling p256 v0.10.1
   Compiling rustBoot v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/rustBoot)
   Compiling xtask v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/xtask)
    Finished dev [unoptimized + debuginfo] target(s) in 11.06s
     Running `target/debug/xtask stm32f446 build pkgs-for`
$ cargo build --release
   Compiling panic-probe v0.3.0
   Compiling stm32f446_boot_fw v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/firmware/stm32f446/boot_fw_blinky_green)
warning: unused import: `mcu::gpio`
  --> firmware/stm32f446/boot_fw_blinky_green/src/main.rs:15:5
   |
15 | use mcu::gpio;
   |     ^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `mcu::gpio::gpiod::PD12`
  --> firmware/stm32f446/boot_fw_blinky_green/src/main.rs:16:5
   |
16 | use mcu::gpio::gpiod::PD12;
   |     ^^^^^^^^^^^^^^^^^^^^^^

warning: `stm32f446_boot_fw` (bin "stm32f446_boot_fw") generated 2 warnings
    Finished release [optimized] target(s) in 0.85s
Second step done
$ cargo build --release
warning: unused config key `build.runner` in `/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/firmware/stm32f446/updt_fw_blinky_red/.cargo/config.toml`
   Compiling rand_core v0.6.3
   Compiling subtle v2.4.1
   Compiling const-oid v0.7.1
   Compiling nb v1.0.0
   Compiling zeroize v1.5.5
   Compiling void v1.0.2
   Compiling vcell v0.1.3
   Compiling bitfield v0.13.2
   Compiling stable_deref_trait v1.2.0
   Compiling r0 v0.2.2
   Compiling cfg-if v1.0.0
   Compiling base16ct v0.1.1
   Compiling opaque-debug v0.3.0
   Compiling bitflags v1.3.2
   Compiling bare-metal v1.0.0
   Compiling minimal-lexical v0.2.1
   Compiling cast v0.3.0
   Compiling byteorder v1.4.3
   Compiling nb v0.1.3
   Compiling volatile-register v0.2.1
   Compiling der v0.5.1
   Compiling ff v0.11.1
   Compiling embedded-dma v0.1.2
   Compiling as-slice v0.2.1
   Compiling typenum v1.15.0
   Compiling memchr v2.5.0
   Compiling log v0.4.17
   Compiling embedded-hal v0.2.7
   Compiling num-traits v0.2.15
   Compiling group v0.11.0
   Compiling bare-metal v0.2.5
   Compiling cortex-m-rt v0.6.15
   Compiling nom v7.1.1
   Compiling defmt v0.3.1
   Compiling cortex-m v0.7.4
   Compiling spki v0.5.4
   Compiling stm32f4 v0.13.0
   Compiling panic-probe v0.3.0
   Compiling pkcs8 v0.8.0
   Compiling generic-array v0.14.5
   Compiling num-integer v0.1.45
   Compiling chrono v0.4.19
   Compiling digest v0.9.0
   Compiling crypto-mac v0.11.1
   Compiling crypto-bigint v0.3.2
   Compiling sec1 v0.2.1
   Compiling block-buffer v0.9.0
   Compiling hmac v0.11.0
   Compiling signature v1.4.0
   Compiling sha2 v0.9.9
   Compiling rfc6979 v0.1.0
   Compiling elliptic-curve v0.11.12
   Compiling rtcc v0.2.1
   Compiling ecdsa v0.13.4
   Compiling p256 v0.10.1
   Compiling rustBoot v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/rustBoot)
   Compiling stm32f4xx-hal v0.10.1
   Compiling rustBoot-hal v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/hal)
   Compiling rustBoot-update v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/update)
   Compiling stm32f446_updt_fw v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/firmware/stm32f446/updt_fw_blinky_red)
    Finished release [optimized] target(s) in 16.08s
$ cargo build --release
   Compiling rand_core v0.6.3
   Compiling subtle v2.4.1
   Compiling const-oid v0.7.1
   Compiling nb v1.0.0
   Compiling zeroize v1.5.5
   Compiling vcell v0.1.3
   Compiling void v1.0.2
   Compiling cfg-if v1.0.0
   Compiling bitfield v0.13.2
   Compiling stable_deref_trait v1.2.0
   Compiling bare-metal v1.0.0
   Compiling r0 v0.2.2
   Compiling base16ct v0.1.1
   Compiling opaque-debug v0.3.0
   Compiling bitflags v1.3.2
   Compiling minimal-lexical v0.2.1
   Compiling cast v0.3.0
   Compiling byteorder v1.4.3
   Compiling volatile-register v0.2.1
   Compiling nb v0.1.3
   Compiling der v0.5.1
   Compiling ff v0.11.1
   Compiling embedded-dma v0.1.2
   Compiling as-slice v0.2.1
   Compiling typenum v1.15.0
   Compiling embedded-hal v0.2.7
   Compiling memchr v2.5.0
   Compiling log v0.4.17
   Compiling group v0.11.0
   Compiling num-traits v0.2.15
   Compiling bare-metal v0.2.5
   Compiling cortex-m-rt v0.6.15
   Compiling cortex-m v0.7.4
   Compiling defmt v0.3.1
   Compiling nom v7.1.1
   Compiling stm32f4 v0.13.0
   Compiling critical-section v0.2.7
   Compiling spki v0.5.4
   Compiling generic-array v0.14.5
   Compiling defmt-rtt v0.3.2
   Compiling pkcs8 v0.8.0
   Compiling num-integer v0.1.45
   Compiling digest v0.9.0
   Compiling crypto-mac v0.11.1
   Compiling crypto-bigint v0.3.2
   Compiling block-buffer v0.9.0
   Compiling sec1 v0.2.1
   Compiling signature v1.4.0
   Compiling hmac v0.11.0
   Compiling chrono v0.4.19
   Compiling sha2 v0.9.9
   Compiling rfc6979 v0.1.0
   Compiling elliptic-curve v0.11.12
   Compiling ecdsa v0.13.4
   Compiling rtcc v0.2.1
   Compiling p256 v0.10.1
   Compiling rustBoot v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/rustBoot)
   Compiling stm32f4xx-hal v0.10.1
   Compiling rustBoot-hal v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/hal)
   Compiling rustBoot-update v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/update)
   Compiling stm32f446 v0.1.0 (/home/anand/Desktop/dev_space/production/rustBoot-forked/my_rustBoot/boards/bootloaders/stm32f446)
warning: path statement with no effect
  --> bootloaders/stm32f446/src/main.rs:21:9
   |
21 |         cortex_m::asm::bkpt;
   |         ^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(path_statements)]` on by default

warning: `stm32f446` (bin "stm32f446") generated 1 warning
    Finished release [optimized] target(s) in 15.61s
