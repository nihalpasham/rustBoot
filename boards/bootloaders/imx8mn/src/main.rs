#![no_std]
#![no_main]
#![feature(format_args_nl, core_intrinsics, once_cell)]

mod boot;
// mod log;

use rustBoot_hal::info;
use rustBoot_hal::nxp::imx8mn::bsp::{
    drivers::{common::interface::DriverManager, driver_manager::driver_manager},
    global,
};
use rustBoot_hal::nxp::imx8mn::{
    exception,
    log::{console, console::Statistics},
};

use crate::boot::halt;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
#[no_mangle]
unsafe fn kernel_init() -> ! {
    // set up vector base address register handlers
    exception::exception::handling_init();
    // initialize drivers
    for i in driver_manager().all_device_drivers().iter() {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
        }
    }
    // we should be able print with `info!` from here on.

    // Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
#[no_mangle]
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

    info!("Drivers loaded:");
    for (i, driver) in driver_manager().all_device_drivers().iter().enumerate() {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    info!("Chars written: {}", console::console().chars_written());

    halt()
}
