//! System Counter.
//!
//! # Resources
//!
//! Descriptions taken from
//! i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

use super::drivers::common::MMIODerefWrapper;
use aarch64_cpu::registers::*;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// System Counter registers.
//
// i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

register_bitfields! {
    u32,
    /// Counter Control Register
    SYS_CNTCR [
        /// Enable Counting
        EN OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Enable Debug
        HDBG OFFSET(1) NUMBITS(1) [
            Ignore = 0,
            Halt = 1,   // 1 - The assertion of the debug input causes the System Counter to halt.
        ],
        /// Frequency Change Request, ID 1
        FCR0 OFFSET(8) NUMBITS(1) [
            NoChange = 0,
            SelEntry0 = 1,   // 1 Select frequency modes table entry 0.
        ],
        /// Frequency Change Request, ID 2
        FCR1 OFFSET(9) NUMBITS(1) [
            NoChange = 0,
            SelEntry1 = 1,   // 1 Select frequency modes table entry 1
        ],
    ],
    /// Counter Status Register
    ///
    /// The system counter status register provides information concerning the clock frequency
    /// and debug state.
    SYS_CNTSR [
        /// Counter is either halted or not halted by debug.
        DBGH OFFSET(0) NUMBITS(1) [
            NotHalted = 0,
            Halted = 1,
        ],
        /// Frequency Change Acknowledge, ID 0
        FCR0 OFFSET(8) NUMBITS(1) [
            NotBase = 0,    // 0 Base frequency is not selected.
            BaseSlctd = 1,   // 1 Base frequency is selected.
        ],
        /// Frequency Change Acknowledge, ID 1
        FCR1 OFFSET(9) NUMBITS(1) [
            NotBase = 0,    // 0 Base frequency is not selected.
            BaseSlctd = 1,   // 1 Base frequency is selected.
        ],
    ],
    /// Counter Count Value Low Register - (part of CNTControlBase)
    ///
    /// The Counter Count Value Low register indicates the current count value bits 31-0.
    SYS_CNTCV0 [
        /// Counter Count Value bits [31:0]
        ///
        /// Writes to the CNTCV registers must be performed while
        /// operating on the base frequency only. Writes to these registers
        /// while running on the alternate frequency may have
        /// unpredictable results.
        CNTCV0 OFFSET(0) NUMBITS(31) []
    ],
    /// Counter Count Value High Register - (part of CNTControlBase)
    ///
    /// The Counter Count Value High register indicates the current count value bits 63-32.
    SYS_CNTCV1 [
        /// Counter Count Value bits [32:55]
        ///
        /// Writes to the CNTCV registers must be performed while
        /// operating on the base frequency only. Writes to these registers
        /// while running on the alternate frequency may have unpredictable results.
        CNTCV1 OFFSET(0) NUMBITS(25) []
    ],
    /// Frequency Modes Table 0 Register. For i.MX8M, this entry is the base frequency is fixed at (24 MHz /3 = 8 MHz)
    ///
    /// The Counter Frequency ID registers is the frequency modes table starting at offset 0x020.
    SYS_CNTFID0 [
        /// Table entries are 32-bits, and each entry specifies a system counter update frequency, in
        /// Hz. The first entry in the table specifies the base frequency of the system counter.
        CNTFID0 OFFSET(0) NUMBITS(31) []
    ],
    /// Frequency Modes Table 0 Register. For i.MX8M, this entry is the alternate frequency is fixed at (32 kHz /64 = 512 Hz)
    ///
    /// The Counter Frequency ID registers is the frequency modes table starting at offset 0x020.
    SYS_CNTFID1 [
        /// Table entries are 32-bits, and each entry specifies a system counter update frequency, in
        /// Hz. The first entry in the table specifies the base frequency of the system counter.
        CNTFID1 OFFSET(0) NUMBITS(31) []
    ],
    /// Frequency Modes Table 0 Register. For i.MX8M, this is the end-marker in the table.
    ///
    /// The Counter Frequency ID registers is the frequency modes table starting at offset 0x020.
    SYS_CNTFID2 [
        /// Table entries are 32-bits, and each entry specifies a system counter update frequency, in
        /// Hz. The first entry in the table specifies the base frequency of the system counter.
        CNTFID2 OFFSET(0) NUMBITS(31) []
    ],
    /// Counter ID Register
    ///
    /// The Counter ID register indicates the architecture version 0.
    SYS_CNTID0 [
        CNTID0 OFFSET(0) NUMBITS(31) []
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => SYS_CNTCR: ReadWrite<u32, SYS_CNTCR::Register>),
        (0x04 => SYS_CNTSR: ReadOnly<u32, SYS_CNTSR::Register>),
        (0x08 => SYS_CNTCV0: ReadWrite<u32, SYS_CNTCV0::Register> ),
        (0x0C => SYS_CNTCV1: ReadWrite<u32, SYS_CNTCV1::Register>),
        (0x10 => _reserved0),
        (0x20 => SYS_CNTFID0: ReadOnly<u32, SYS_CNTFID0::Register>),
        (0x24 => SYS_CNTFID1: ReadOnly<u32, SYS_CNTFID1::Register>),
        (0x28 => SYS_CNTFID2: ReadOnly<u32, SYS_CNTFID2::Register>),
        (0x2c => _reserved1),
        (0xfd0 => SYS_CNTID0: ReadOnly<u32, SYS_CNTID0::Register>),
        (0xfd4 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

/// System Counter.
pub struct SystemCounter {
    registers: Registers,
}

impl SystemCounter {
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

    /// Get frequency id 0 - the base frequency
    pub fn get_cntfid0(&self) -> u32 {
        self.registers.SYS_CNTFID0.get()
    }
    /// Get the contents of counter control register.
    pub fn get_cntcr(&self) -> u32 {
        self.registers.SYS_CNTCR.get()
    }

    /// Start system counter.
    pub fn start_counter(&self) {
        let freq = self.get_cntfid0();
        // Update with accurate clock frequency
        CNTFRQ_EL0.set(freq as u64);
        self.registers
            .SYS_CNTCR
            .write(SYS_CNTCR::FCR0::SelEntry0 + SYS_CNTCR::EN::Enable + SYS_CNTCR::HDBG::Halt)
    }
}
