#![allow(dead_code)]
#![allow(non_snake_case)]
#![deny(unused_must_use)]

use rustBoot::constants::{BOOT_PARTITION_ADDRESS, PARTITION_SIZE, UPDATE_PARTITION_ADDRESS};
use std::path::Path;
use std::{env, path::PathBuf};

use xshell::cmd;

#[rustfmt::skip]
fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();
    
    match &args[..] {
        ["test", "rustBoot"] => test_rustBoot(),
        ["build", "pkgs-for", board]    => build_rustBoot(board),
        ["sign" , "pkgs-for", board]    => sign_packages(board),
        ["flash", "signed-pkg", board]  => flash_signed_fwimages(board),
        ["flash", "rustBoot", board]    => flash_rustBoot(board),
        ["build", "rustBoot-only", board] => build_rustBoot_only(board),
        ["build-sign-flash", "rustBoot", board] => full_image_flash(board),
        ["erase-and-flash-trailer-magic", board] => erase_and_flash_trailer_magic(board),
        _ => {
            println!("USAGE: cargo xtask test rustBoot");
            println!("OR");
            println!("USAGE: cargo xtask [build|sign|flash] [pkgs-for|signed-pkg] [board]");
            println!("OR");
            println!("USAGE: cargo xtask [build-sign-flash] [rustBoot] [board]");
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
            cmd!("cargo build --release").run()?; // for logging add `--features log`
            if Path::new("kernel8.img").exists() {
                cmd!("powershell -command \"del kernel8.img\"").run()?;
            }
            cmd!("rust-objcopy --strip-all -O binary ..\\..\\target\\aarch64-unknown-none-softfloat\\release\\kernel kernel8.img").run()?;
        }
        &"nrf52840" => {
            cmd!("cargo build --release").run()?;
        }
        &"stm32f411" => {
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

fn sign_packages(target: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
            cmd!("py convert2bin.py").run()?;
            // python script has a linux dependency - `wolfcrypt`
            cmd!("wsl python3 signer.py").run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
            //  cmd!("python3 --version").run()?;
            cmd!("python3 convert2bin.py").run()?;
            // python script has a linux dependency - `wolfcrypt`
            cmd!("wsl python3 signer.py").run()?;
            Ok(())
        }
        _ => todo!(),
    }
}

fn flash_signed_fwimages(target: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("pyocd flash -t nrf52840 --base-address {boot_part_addr} nrf52840_bootfw_v1234_signed.bin").run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("pyocd flash -t nrf52840 --base-address {updt_part_addr} nrf52840_updtfw_v1235_signed.bin").run()?;
            Ok(())
        }
        "stm32f411" => {
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
            let boot_part_addr = format!("0x{:x}", BOOT_PARTITION_ADDRESS);
            cmd!("pyocd flash --base-address {boot_part_addr} stm32f411_bootfw_v1235_signed.bin")
                .run()?;

            let updt_part_addr = format!("0x{:x}", UPDATE_PARTITION_ADDRESS);
            cmd!("pyocd flash -t stm32f411 --base-address {updt_part_addr} stm32f411_updtfw_v1235_signed.bin").run()?;
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
        _ => todo!(),
    }
}

fn full_image_flash(target: &&str) -> Result<(), anyhow::Error> {
    build_rustBoot(target)?;
    sign_packages(target)?;
    cmd!("pyocd erase -t nrf52 --mass-erase").run()?;
    flash_signed_fwimages(target)?;
    flash_rustBoot(target)?;
    Ok(())
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}

/// to be used ONLY for testing.
fn erase_and_flash_trailer_magic(target: &&str) -> Result<(), anyhow::Error> {
    match *target {
        "nrf52840" => {
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
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
            let _p = xshell::pushd(root_dir().join("boards/signing_tools/signed_images"))?;
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

        _ => todo!(),
    }
}
