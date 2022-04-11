// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! System console.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

use crate::rpi::rpi4::bsp::drivers::{gpio::PanicGPIO, uart0::PanicUart};
use crate::rpi::rpi4::bsp::global;
use crate::rpi::rpi4::bsp::memory_map;

use core::fmt;

/// Console write functions.
pub trait Write {
    /// Write a single character.
    fn write_char(&self, c: char);

    /// Write a Rust format string.
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

    /// Block until the last buffered character has been physically put on the TX wire.
    fn flush(&self);
}

/// Console read functions.
pub trait Read {
    /// Read a single character.
    fn read_char(&self) -> char {
        ' '
    }

    /// Clear RX buffers, if any.
    fn clear_rx(&self);
}

/// Console statistics.
pub trait Statistics {
    /// Return the number of characters written.
    fn chars_written(&self) -> usize {
        0
    }

    /// Return the number of characters read.
    fn chars_read(&self) -> usize {
        0
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// In case of a panic, the panic handler uses this function to take a last shot at printing
/// something before the system is halted.
///
/// We try to init panic-versions of the GPIO and the UART. The panic versions are not protected
/// with synchronization primitives, which increases chances that we get to print something, even
/// when the kernel's default GPIO or UART instances happen to be locked at the time of the panic.
///
/// # Safety
///
/// - Use only for printing during a panic.
pub unsafe fn panic_console_out() -> impl fmt::Write {
    let mut panic_gpio = PanicGPIO::new(memory_map::map::mmio::GPIO_START);
    let mut panic_uart = PanicUart::new(memory_map::map::mmio::PL011_UART_START);

    panic_gpio.map_pl011_uart();
    panic_uart.init();
    panic_uart
}

/// Return a reference to the console.
pub fn console() -> &'static (impl Write + Read + Statistics) {
    &global::PL011_UART
}
