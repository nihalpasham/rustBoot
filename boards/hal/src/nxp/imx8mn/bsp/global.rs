//! BSP Processor code. Global peripherals file for the i.MX8MN.

use super::counter::SystemCounter;
use super::drivers::{gpio::Gpio, uart0::Uart, usdhc::UsdhController};
use super::memory_map;
use super::mux::uart2grp::*;

pub static UART: Uart = unsafe { Uart::new(memory_map::map::mmio::UART_START) };
pub static GPIO2: Gpio = unsafe { Gpio::new(memory_map::map::mmio::GPIO2_START) };
pub static CNTR: SystemCounter = unsafe { SystemCounter::new(memory_map::map::mmio::SYSCNT_START) };
pub static SDHC2: UsdhController =
    unsafe { UsdhController::new(memory_map::map::mmio::USDHC2_START) };

/// Board identification.
pub fn board_name() -> &'static str {
    {
        "i.MX 8M Nano EVK"
    }
}
