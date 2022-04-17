// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Processor code. Top-level BSP file for the Raspberry Pi 4.

use super::drivers::{emmc::EMMCController, gpio::GPIO, uart0::PL011Uart};
use super::memory_map;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------
pub static GPIO: GPIO = unsafe { GPIO::new(memory_map::map::mmio::GPIO_START) };

pub static PL011_UART: PL011Uart =
    unsafe { PL011Uart::new(memory_map::map::mmio::PL011_UART_START) };

pub static EMMC_CONT: EMMCController =
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
