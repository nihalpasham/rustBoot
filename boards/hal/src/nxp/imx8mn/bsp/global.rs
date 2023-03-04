//! BSP Processor code. Top-level BSP file for the i.MX8MN.

use super::drivers::{uart0::Uart, gpio::Gpio};
use super::memory_map;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------
#[no_mangle]
pub static UART: Uart = unsafe { Uart::new(memory_map::map::mmio::UART_START) };
pub static GPIO: Gpio = unsafe { Gpio::new(memory_map::map::mmio::GPIO_START) };

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Board identification.
pub fn board_name() -> &'static str {
    {
        "i.MX8MN"
    }
}
