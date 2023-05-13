#![no_std]
#![no_main]
#![feature(format_args_nl)]

mod boot;

use rustBoot_hal::info;
use rustBoot_hal::nxp::imx8mn::arch::cpu_core::*;
use rustBoot_hal::nxp::imx8mn::bsp::drivers::usdhc::SdResult;
use rustBoot_hal::nxp::imx8mn::bsp::global::SDHC2;
use rustBoot_hal::nxp::imx8mn::bsp::{
    clocks,
    drivers::{
        common::interface::DriverManager,
        driver_manager::{driver_manager, start_system_counter},
    },
    global, mux,
};
use rustBoot_hal::nxp::imx8mn::{
    memory,
    exception,
    log::{console, console::Statistics},
};

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
#[no_mangle]
unsafe fn kernel_init() -> ! {
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

/// The main function running after early initialization.
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

    // init uSDHC
    match SDHC2.init_usdhc() {
        SdResult::SdOk => info!("uSDHC driver initialized..."),
        _ => info!("failed to initialize"),
    }

    // info!("");
    // info!("Trying to read from non-existent OCRAM addresss 0x980000...");
    // unsafe { core::ptr::read_volatile(0x980000 as *mut u64) };

    wait_forever()
}
