// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Processor code. Top-level BSP file for the Raspberry Pi 4.

use super::drivers::{gpio::GPIO, sdhost::EMMCController, uart0::PL011Uart};
use super::memory_map;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Used by `arch` code to find the early boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------
pub(crate) static GPIO: GPIO = unsafe { GPIO::new(memory_map::map::mmio::GPIO_START) };

pub(crate) static PL011_UART: PL011Uart =
    unsafe { PL011Uart::new(memory_map::map::mmio::PL011_UART_START) };

pub(crate) static EMMC2_CONT: EMMCController =
    unsafe { EMMCController::new(memory_map::map::mmio::EMMC_START) };

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Board identification.
pub fn board_name() -> &'static str {
    {
        "Raspberry Pi 4"
    }
}
