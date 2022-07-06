#![allow(dead_code)]
#![allow(non_snake_case)]
#![deny(unused_must_use)]

#[cfg(feature = "mcu")]
use rustBoot::constants::{BOOT_PARTITION_ADDRESS, PARTITION_SIZE, UPDATE_PARTITION_ADDRESS};
use std::{env, path::PathBuf};
// use std::path::Path;

use xshell::cmd;

#[rustfmt::skip]
fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["test", "rustBoot"] => test_rustBoot(),
        [board, "build", "pkgs-for",]    => build_rustBoot(board),
        [board, "sign" , "pkgs-for", boot_ver, updt_ver]    => sign_packages(board, boot_ver, updt_ver),
        #[cfg(feature = "mcu")]
        [board, "flash", "signed-pkg", boot_ver, updt_ver]  => flash_signed_fwimages(board, boot_ver, updt_ver),
        [board, "flash", "rustBoot",]    => flash_rustBoot(board),
        [board, "build", "rustBoot-only",] => build_rustBoot_only(board),
        #[cfg(feature = "mcu")]
        [board, "build-sign-flash", "rustBoot", boot_ver, updt_ver] => full_image_flash(board, boot_ver, updt_ver),
        #[cfg(feature = "mcu")]
        [board, "erase-and-flash-trailer-magic",] => erase_and_flash_trailer_magic(board),
        _ => {
            println!("USAGE: cargo [board] test rustBoot");
            println!("OR");
            println!("USAGE: cargo [board] [build|sign|flash] [pkgs-for]|signed-pkg] [boot-ver] [updt-ver]");
            println!("OR");
            println!("USAGE: cargo [board] [build-sign-flash] [rustBoot] [boot-ver] [updt-ver]");
            Ok(())
        }
    }
}

fn test_rustBoot() -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir())?;
    cmd!("cargo test --workspace").run()?;
    Ok(())
}

fn build_rustBoot_only(target: &&str) -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
    match target {
        &"rpi4" => {
            cmd!("cargo build --release").run()?; // `
                                                  // if Path::new("kernel8.img").exists() {
                                                  //     cmd!("powershell -command \"del kernel8.img\"").run()?;
                                                  // }
            #[cfg(feature = "windows")]
            cmd!("rust-objcopy --strip-all -O binary ..\\..\\target\\aarch64-unknown-none-softfloat\\release\\kernel rustBoot.bin").run()?;
            #[cfg(not(feature = "windows"))]
            cmd!("rust-objcopy --strip-all -O binary ../../target/aarch64-unknown-none-softfloat/release/kernel rustBoot.bin").run()?;
        }
        &"nrf52840" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32f411" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32f446" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32h723" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32f746" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32f334" => {
            cmd!("cargo build --release").run()?;
        }
        _ => {
            println!("board not supported");
        }
    }

    Ok(())
}

fn build_rustBoot(target: &&str) -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(
        root_dir()
            .join("boards/firmware")
            .join(target)
            .join("boot_fw_blinky_green"),
    )?;
    cmd!("cargo build --release").run()?;
    let _p = xshell::pushd(
        root_dir()
            .join("boards/firmware")
            .join(target)
            .join("updt_fw_blinky_red"),
    )?;
    cmd!("cargo build --release").run()?;
    build_rustBoot_only(target)?;
    Ok(())
}

fn sign_packages(target: &&str, boot_ver: &&str, updt_ver: &&str) -> Result<(), anyhow::Error> {
    // let boot_ver = target[3].to_string();
    // let updt_ver = target[4].to_string();

    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/nrf52840_bootfw -O binary nrf52840_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/nrf52840_updtfw -O binary nrf52840_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/nrf52840_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/nrf52840_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f411_bootfw -O binary stm32f411_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f411_updtfw -O binary stm32f411_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f411_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f411_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }
        "stm32f446" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f446_bootfw -O binary stm32f446_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f446_updtfw -O binary stm32f446_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f446_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f446_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }
        "stm32h723" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32h723_bootfw -O binary stm32h723_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32h723_updtfw -O binary stm32h723_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32h723_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32h723_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }
        "stm32f746" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f746_bootfw -O binary stm32f746_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f746_updtfw -O binary stm32f746_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f746_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f746_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }
        "stm32f334" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f334_bootfw -O binary stm32f334_bootfw.bin").run()?;
            cmd!("rust-objcopy -I elf32-littlearm ../../target/thumbv7em-none-eabihf/release/stm32f334_updtfw -O binary stm32f334_updtfw.bin").run()?;

            let _p = xshell::pushd(root_dir().join("rbsigner"))?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f334_bootfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {boot_ver}").run()?;
            cmd!("cargo run mcu-image ../boards/sign_images/signed_images/stm32f334_updtfw.bin nistp256 ../boards/sign_images/keygen/ecc256.der {updt_ver}").run()?;
            Ok(())
        }

        _ => todo!(),
    }
}

#[cfg(feature = "mcu")]
#[rustfmt::skip]
fn flash_signed_fwimages(target: &&str, boot_ver: &&str, updt_ver: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip nRF52840_xxAA nrf52840_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip nRF52840_xxAA nrf52840_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip stm32f411vetx stm32f411_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip stm32f411vetx stm32f411_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }
        "stm32f446" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip stm32f446retx stm32f446_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip stm32f446retx stm32f446_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }
        "stm32h723" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip STM32H723ZGTx stm32h723_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip STM32H723ZGTx stm32h723_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }
        "stm32f746" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip stm32f746zgtx stm32f746_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip stm32f746zgtx stm32f746_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }
        "stm32f334" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {boot_part_addr} --chip stm32f334r8tx stm32f334_bootfw_v{boot_ver}_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("probe-rs-cli download --format Bin --base-address {updt_part_addr} --chip stm32f334r8tx stm32f334_updtfw_v{updt_ver}_signed.bin").run()?;
            Ok(())
        }

        _ => todo!(),
    }
}

fn flash_rustBoot(target: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip nRF52840_xxAA --release").run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip stm32f411vetx --release").run()?;
            Ok(())
        }
        "stm32f446" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip stm32f446vetx --release").run()?;
            Ok(())
        }
        "stm32h723" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip STM32H723ZGTx --release").run()?;
            Ok(())
        }
        "stm32f746" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip stm32f746zgtx --release").run()?;
            Ok(())
        }
        "stm32f334" => {
            let _p = xshell::pushd(root_dir().join("boards/bootloaders").join(target))?;
            cmd!("cargo flash --chip stm32f334r8tx --release").run()?;
            Ok(())
        }

        _ => todo!(),
    }
}

#[cfg(feature = "mcu")]
fn full_image_flash(target: &&str, boot_ver: &&str, updt_ver: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip nRF52840_xxAA").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }
        "stm32f411" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip stm32f411vetx").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }
        "stm32f446" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip stm32f446retx").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }
        "stm32h723" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip STM32H723ZGTx").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }
        "stm32f746" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip stm32f746zgtx").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }
        "stm32f334" => {
            build_rustBoot(target)?;
            sign_packages(target, boot_ver, updt_ver)?;
            cmd!("probe-rs-cli erase --chip stm32f334r8tx").run()?;
            flash_signed_fwimages(target, boot_ver, updt_ver)?;
            flash_rustBoot(target)?;
            Ok(())
        }

        _ => todo!(),
    }
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}

#[cfg(feature = "mcu")]
/// to be used ONLY for testing.
fn erase_and_flash_trailer_magic(target: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t nrf52840 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t nrf52840 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t nrf52840 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t nrf52840 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t nrf52840 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t stm32f411 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f411 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f411 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f411 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f411 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }
        "stm32f446" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t stm32f446 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f446 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f446 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f446 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f446 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }
        "stm32h723" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t stm32h723 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32h723 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32h723 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32h723 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32h723 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }
        "stm32f746" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t stm32f746 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f746 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f746 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f746 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f746 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }
        "stm32f334" => {
            let _p = xshell::pushd(root_dir().join("boards/sign_images/signed_images"))?;
            // just to ensure that an existing bootloader doesnt start to boot automatically - during a test
            cmd!("pyocd erase -t stm32f334 -s 0x0").run()?;
            let boot_trailer_magic = format!("0x{:x}", BOOT_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f334 -s {boot_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f334 --base-address {boot_trailer_magic} trailer_magic.bin")
                .run()?;

            let updt_trailer_magic =
                format!("0x{:x}", UPDATE_PARTITION_ADDRESS + PARTITION_SIZE - 4);
            cmd!("pyocd erase -t stm32f334 -s {updt_trailer_magic}").run()?;
            cmd!("pyocd flash -t stm32f334 --base-address {updt_trailer_magic} trailer_magic.bin")
                .run()?;
            Ok(())
        }

        _ => todo!(),
    }
}
