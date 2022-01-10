#![feature(const_fn_fn_ptr_basics)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(asm_const)]
#![feature(asm)]
#![cfg_attr(not(test), no_std)]
#![no_main]
#![allow(warnings)]

pub mod arch;
pub mod bsp;
mod exception;
pub mod fs;
pub mod log;
mod panic_wait;
mod state;
mod sync;

mod boot;

use arch::time::*;
use bsp::drivers::common::interface::DriverManager;
use bsp::drivers::driver_manager::driver_manager;
use bsp::global;
use bsp::global::EMMC_CONT;
use console::{Read, Statistics};
use core::time::Duration;
use log::console;

use crate::fs::emmcfat::{Controller, TestClock, VolumeIdx};
use crate::fs::filesystem::Mode;

use crate::boot::{boot_to_kernel, DTB_LOAD_ADDR, KERNEL_LOAD_ADDR};

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
unsafe fn kernel_init() -> ! {
    for i in driver_manager().all_device_drivers().iter() {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
        }
    }
    driver_manager().post_device_driver_init();
    // println! is usable from here on.

    // Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
fn kernel_main() -> ! {
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", global::board_name());

    let (_, privilege_level) = exception::exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);

    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        time_manager().resolution().as_nanos()
    );

    info!("Drivers loaded:");
    for (i, driver) in driver_manager().all_device_drivers().iter().enumerate() {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    info!("Chars written: {}", console::console().chars_written());

    // Discard any spurious received characters before going into echo mode.
    console::console().clear_rx();

    // Test a failing timer case.
    time_manager().wait_for(Duration::from_nanos(1));

    // let mut buff = [0u8; 512 * 2];
    // let _ = &EMMC_CONT.emmc_transfer_blocks(0x2000, 2, &mut buff, false);
    // info!("read 2 blocks: {:?}", buff);

    let mut ctrlr = Controller::new(&EMMC_CONT, TestClock);
    let volume = ctrlr.get_volume(VolumeIdx(0));

    if let Ok(mut volume) = volume {
        let root_dir = ctrlr.open_root_dir(&volume).unwrap();
        info!("\tListing root directory:\n");
        ctrlr
            .iterate_dir(&volume, &root_dir, |x| {
                info!("\t\tFound: {:?}", x);
            })
            .unwrap();
        // info!("\tRetrieve handle to `config.txt` file present in root_dir...");
        // let mut file = ctrlr
        //     .open_file_in_dir(&mut volume, &root_dir, "CONFIG.TXT", Mode::ReadOnly)
        //     .unwrap();
        // info!("\tRead `config.txt` from sd-card, output to terminal...");
        // info!("FILE STARTS:");
        // while !file.eof() {
        //     let mut buffer = [0u8; 4*512];
        //     let num_read = ctrlr.read(&volume, &mut file, &mut buffer).unwrap();
        //     let file_contents = core::str::from_utf8(&buffer).unwrap();
        //     info!("\n{}", file_contents);
        // }
        // info!("EOF");
        // ctrlr.close_file(&volume, file).unwrap();

        // Load dtb
        info!("Get handle to `dtb` file in root_dir...");
        let mut dtb_file = ctrlr
            .open_file_in_dir(&mut volume, &root_dir, "BCM271~1.DTB", Mode::ReadOnly)
            .unwrap();
        info!("\t\tload `dtb` into RAM...");
        while !dtb_file.eof() {
            let num_read = ctrlr
                .read(&volume, &mut dtb_file, unsafe { &mut DTB_LOAD_ADDR.0 })
                .unwrap();
            info!(
                "\t\tloaded dtb: {:?} bytes, starting at addr: {:p}",
                num_read,
                unsafe { &mut DTB_LOAD_ADDR.0 }
            );
        }
        ctrlr.close_file(&volume, dtb_file).unwrap();

        // Load kernel
        info!("Get handle to `kernel` file in root_dir...");
        let mut kernel_file = ctrlr
            .open_file_in_dir(&mut volume, &root_dir, "VMLINUZ", Mode::ReadOnly)
            .unwrap();
        info!("\t\tload `kernel` into RAM...");
        while !kernel_file.eof() {
            let num_read = ctrlr
                .read(&volume, &mut kernel_file, unsafe {
                    &mut KERNEL_LOAD_ADDR.0
                })
                .unwrap();
            info!(
                "\t\tloaded kernel: {:?} bytes, starting at addr: {:p}",
                num_read,
                unsafe { &mut KERNEL_LOAD_ADDR.0 }
            );
        }
        info!(
            "\n***************************************** \
            Starting kernel \
            ********************************************\n"
        );
        ctrlr.close_file(&volume, kernel_file).unwrap();
    }

    boot_to_kernel(
        unsafe { &mut KERNEL_LOAD_ADDR.0 }.as_ptr() as usize,
        unsafe { &mut DTB_LOAD_ADDR.0 }.as_ptr() as usize,
    )

    // loop {
    //     // let c = console::console().read_char();
    //     // console::console().write_char(c);

    //     info!("waiting for 1 second");
    //     time_manager().wait_for(Duration::from_secs(1));
    // }
}
