//! System console.

use crate::nxp::imx8mn::bsp::{clocks, drivers::uart0::PanicUart, global, memory_map, mux};

use core::fmt;

/// Console write functions.
pub trait Write {
    /// Write a single character.
    fn write_char(&self, c: char);

    /// Write a Rust format string.
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

    // /// Block until the last buffered character has been physically put on the TX wire.
    // fn flush(&self);
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
    let mut panic_uart = PanicUart::new(memory_map::map::mmio::UART_START);

    clocks::uartclks::enable_uart_clk(1);
    mux::uart2grp::uart2_mux_mmio_set();
    panic_uart.init_uart();
    panic_uart
}

/// Return a reference to the console.
pub fn console() -> &'static (impl Write + Read + Statistics) {
    &global::UART
}
