//! GPIO Driver.

use super::common::MMIODerefWrapper;
use crate::nxp::imx8mn::sync::{interface::Mutex, NullLock};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

// GPIO registers.
//
// Descriptions taken from
// i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

register_bitfields! {
    u32,

    /// GPIO Data Register
    ///
    /// The 32-bit GPIO_DR register stores data that is ready to be driven to the output lines
    GPIO_DR [
        /// Holds data to be written to output lines.
        DR OFFSET(0) NUMBITS(32) []
    ],

    /// GPIO Direction Register
    ///
    /// GPIO_GDIR functions as direction control. Each bit specifies the direction of a one-bit signal.
    GPIO_GDIR [
        /// Controls the direction for PINS of the imx8mn
        PIN_DIR OFFSET(0) NUMBITS(32) [],
    ],

    /// GPIO Pad Status Register
    ///
    /// GPIO_PSR is a read-only register. Each bit stores the value of the corresponding input
    /// signal
    GPIO_PSR [
        /// Input state of pin(s).
        PIN_PSR OFFSET(0) NUMBITS(32) [],
    ],

    /// GPIO Interrupt Configuration Register 1
    ///
    /// GPIO_ICR1 contains 16 two-bit fields, where each field specifies the interrupt
    /// configuration for a different input signal.
    GPIO_ICR1 [
        /// ICR15
        ICR15 OFFSET(30) NUMBITS(2) [
            LowLevel = 0b00,    // Interrupt n is low-level sensitive.
            HighLevel = 0b01,   // Interrupt n is high-level sensitive.
            RisingEdge = 0b10,  // Interrupt n is rising-edge sensitive.
            FallingEdge = 0b11, // Interrupt n is falling-edge sensitive.
        ],

        /// ICR 14
        ICR14 OFFSET(28) NUMBITS(2) [
            LowLevel = 0b00,    // Interrupt n is low-level sensitive.
            HighLevel = 0b01,   // Interrupt n is high-level sensitive.
            RisingEdge = 0b10,  // Interrupt n is rising-edge sensitive.
            FallingEdge = 0b11, // Interrupt n is falling-edge sensitive.
        ]
    ],

    /// GPIO Interrupt Configuration Register 2
    ///
    /// GPIO_ICR2 contains 16 two-bit fields, where each field specifies the interrupt
    /// configuration for a different input signal.
    GPIO_ICR2 [
        /// ICR 31
        ICR31 OFFSET(30) NUMBITS(2) [
            LowLevel = 0b00,    // Interrupt n is low-level sensitive.
            HighLevel = 0b01,   // Interrupt n is high-level sensitive.
            RisingEdge = 0b10,  // Interrupt n is rising-edge sensitive.
            FallingEdge = 0b11, // Interrupt n is falling-edge sensitive.
        ],

        /// ICR 30
        ICR30 OFFSET(28) NUMBITS(2) [
            LowLevel = 0b00,    // Interrupt n is low-level sensitive.
            HighLevel = 0b01,   // Interrupt n is high-level sensitive.
            RisingEdge = 0b10,  // Interrupt n is rising-edge sensitive.
            FallingEdge = 0b11, // Interrupt n is falling-edge sensitive.
        ]
    ],

    /// GPIO Interrupt Mask Register
    ///
    /// GPIO_IMR contains masking bits for each interrupt line.
    GPIO_IMR [
        /// IMR 31
        IMR31 OFFSET(31) NUMBITS(1) [
            Masked = 0,     // Interrupt n is disabled.
            Unmasked = 1    // Interrupt n is enabled.
        ],
        /// IMR 30
        IMR30 OFFSET(30) NUMBITS(1) [
            Masked = 0,     // Interrupt n is disabled.
            Unmasked = 1    // Interrupt n is enabled.
        ]
    ],

    /// GPIO Interrupt Status Register
    ///
    /// The GPIO_ISR functions as an interrupt status indicator. Each bit indicates whether an
    /// interrupt condition has been met for the corresponding input signal.
    ///
    /// Bit n of this register is asserted (active high) when the active condition (as
    /// determined by the corresponding ICR register field) is detected on the GPIO input and is waiting for service. The
    /// value of this register is independent of the value in GPIO_IMR. When the active condition has been
    /// detected, the corresponding bit remains set until cleared by software.
    /// Status flags are cleared by writing a 1 to the corresponding bit position.
    GPIO_ISR [
        /// ISR 30
        ISR30 OFFSET(30) NUMBITS(1) [
            Set = 1,
            NotSet = 0,
        ],
        /// ISR 31
        ISR31 OFFSET(31) NUMBITS(1) [
            Set = 1,
            NotSet = 0,
        ]
    ],

    /// GPIO Edge Select Register
    ///
    /// If the GPIO_EDGE_SEL bit is set, then a rising edge or falling edge in the corresponding
    /// signal generates an interrupt. This register provides backward compatibility. On reset all
    /// bits are cleared (ICR is not overridden).
    GPIO_EDGE_SEL   [
        EDGE_SEL OFFSET(0) NUMBITS(31) []
    ]


}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => GPIO_DR: ReadWrite<u32, GPIO_DR::Register>),
        (0x04 => GPIO_GDIR: ReadWrite<u32, GPIO_GDIR::Register>),
        (0x08 => GPIO_PSR: ReadOnly<u32, GPIO_PSR::Register>),
        (0x0C => ICR1: ReadWrite<u32, GPIO_ICR1::Register>),
        (0x10 => ICR2: ReadWrite<u32, GPIO_ICR2::Register>),
        (0x14 => IMR: ReadWrite<u32, GPIO_IMR::Register>),
        (0x18 => ISR: ReadOnly<u32, GPIO_ISR::Register>),
        (0x1C => EDGE_SEL: ReadWrite<u32, GPIO_EDGE_SEL::Register>),
        (0x20 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

pub struct GpioInner {
    registers: Registers,
}

// Export the inner struct so that BSPs can use it for the panic handler.
pub use GpioInner as PanicGPIO;

/// Representation of the GPIO HW.
pub struct Gpio {
    inner: NullLock<GpioInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl GpioInner {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    /// Set GPIO pin
    fn set_gpio_pin(&mut self, pin: u8, val: bool) {
        // set or clear DR bit for corresponding pin
        match val {
            true => {
                // Set pin-direction to output
                self.registers
                    .GPIO_GDIR
                    .modify(GPIO_GDIR::PIN_DIR.val(1 << pin));
                self.registers.GPIO_DR.modify(GPIO_DR::DR.val(1 << pin));
            }
            false => {
                self.registers.GPIO_DR.modify(GPIO_DR::DR.val(0 << pin));
            }
        }
    }

    /// Get GPIO pin state
    pub fn get_gpio_pin(&mut self, pin: u8) -> u8 {
        // Set pin-direction to input
        self.registers
            .GPIO_GDIR
            .modify(GPIO_GDIR::PIN_DIR.val(0 << pin));

        // read pad status
        let psr = self.registers.GPIO_PSR.read(GPIO_PSR::PIN_PSR);
        ((psr >> pin) & 0x1) as u8
    }
}

impl Gpio {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(GpioInner::new(mmio_start_addr)),
        }
    }
    /// Sets the supplied Gpio pin's direction and state to output
    pub fn set_pin(&self, pin: u8) {
        self.inner.lock(|gpio| gpio.set_gpio_pin(pin, true));
    }
    /// Clears the supplied Gpio pin's output-mode status
    pub fn clear_pin(&self, pin: u8) {
        self.inner.lock(|gpio| gpio.set_gpio_pin(pin, false));
    }
    /// Reads the the supplied Gpio pin's state.
    pub fn read_pin(&self, pin: u8) -> u8 {
        let res = self.inner.lock(|gpio| gpio.get_gpio_pin(pin));
        res
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
impl super::common::interface::DeviceDriver for Gpio {
    fn compatible(&self) -> &'static str {
        "i.MX 8M Nano Gpio"
    }
}
