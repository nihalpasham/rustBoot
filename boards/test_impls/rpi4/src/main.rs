#![feature(const_fn_fn_ptr_basics)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![no_main]
#![no_std]

pub mod arch;
pub mod bsp;
pub mod log;
mod panic_wait;
mod sync;

use bsp::drivers::common::interface::DriverManager;
use bsp::drivers::driver_manager::driver_manager;
use bsp::global;
use console::{Read, Statistics, Write};
use log::console;

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
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("[1] Booting on: {}", global::board_name());

    println!("[2] Drivers loaded:");
    for (i, driver) in driver_manager().all_device_drivers().iter().enumerate() {
        println!("      {}. {}", i + 1, driver.compatible());
    }

    println!("[3] Chars written: {}", console::console().chars_written());
    println!("[4] Echoing input now");

    // Discard any spurious received characters before going into echo mode.
    console::console().clear_rx();
    loop {
        let c = console::console().read_char();
        console::console().write_char(c);
    }
}
