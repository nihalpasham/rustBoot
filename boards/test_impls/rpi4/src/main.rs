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

use arch::time::*;
use bsp::drivers::common::interface::DriverManager;
use bsp::drivers::driver_manager::driver_manager;
use bsp::global;
use bsp::global::EMMC_CONT;
use console::{Read, Statistics};
use core::time::Duration;
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
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", global::board_name());

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

    let mut buff = [0u8; 512*2 + 512];
    let _ = &EMMC_CONT.emmc_transfer_blocks(0, 2, &mut buff, false);
    info!("read 2 blocks: {:?}", buff);

    loop {
        // let c = console::console().read_char();
        // console::console().write_char(c);

        info!("waiting for 1 second");
        time_manager().wait_for(Duration::from_secs(1));
    }
}
