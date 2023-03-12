//! BSP Processor code. Global peripherals file for the i.MX8MN.

use super::drivers::{gpio::Gpio, uart0::Uart};
use super::counter::SystemCounter;
use super::mux::iomux::*;
use super::memory_map;

pub static UART: Uart = unsafe { Uart::new(memory_map::map::mmio::UART_START) };
pub static GPIO: Gpio = unsafe { Gpio::new(memory_map::map::mmio::GPIO_START) };
pub static CNTR: SystemCounter = unsafe { SystemCounter::new(memory_map::map::mmio::SYSCNT_START) };

/// Board identification.
pub fn board_name() -> &'static str {
    {
        "i.MX 8M Nano EVK"
    }
}
