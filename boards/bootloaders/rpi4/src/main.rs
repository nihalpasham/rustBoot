#![feature(const_fn_fn_ptr_basics)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(asm_const)]
#![feature(asm)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
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

use crate::boot::{boot_to_kernel, DTB_LOAD_ADDR, INITRAMFS_LOAD_ADDR, KERNEL_LOAD_ADDR};

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

        // Load dtb
        info!("Get handle to `dtb` file in root_dir...");
        let mut dtb_file = ctrlr
            .open_file_in_dir(&mut volume, &root_dir, "BCM271~1.DTB", Mode::ReadOnly)
            .unwrap();
        info!("\t\tload `dtb` into RAM...");
        while !dtb_file.eof() {
            let num_read = ctrlr
                .read_multi(&volume, &mut dtb_file, unsafe { &mut DTB_LOAD_ADDR.0 })
                .unwrap();
            info!(
                "\t\tloaded dtb: {:?} bytes, starting at addr: {:p}",
                num_read,
                unsafe { &mut DTB_LOAD_ADDR.0 },
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
                .read_multi(&volume, &mut kernel_file, unsafe {
                    &mut KERNEL_LOAD_ADDR.0
                })
                .unwrap();
            info!(
                "\t\tloaded kernel: {:?} bytes, starting at addr: {:p}",
                num_read,
                unsafe { &mut KERNEL_LOAD_ADDR.0 }
            );
        }
        ctrlr.close_file(&volume, kernel_file).unwrap();

        // Load initramfs
        info!("Get handle to `initramfs` file in root_dir...");
        let mut initramfs = ctrlr
            .open_file_in_dir(&mut volume, &root_dir, "INITRA~1", Mode::ReadOnly)
            .unwrap();
        info!("\t\tload `initramfs` into RAM...");
        while !initramfs.eof() {
            let num_read = ctrlr
                .read_multi(&volume, &mut initramfs, unsafe {
                    &mut INITRAMFS_LOAD_ADDR.0
                })
                .unwrap();
            info!(
                "\t\tloaded initramfs: {:?} bytes, starting at addr: {:p}\n",
                num_read,
                unsafe { &mut INITRAMFS_LOAD_ADDR.0 }
            );
        }
        ctrlr.close_file(&volume, initramfs).unwrap();
    }

    info!(
        "***************************************** \
            Starting kernel \
            ********************************************\n"
    );
    boot_to_kernel(
        unsafe { &mut KERNEL_LOAD_ADDR.0 }.as_ptr() as usize,
        unsafe { &mut DTB_LOAD_ADDR.0 }.as_ptr() as usize,
    )

}
