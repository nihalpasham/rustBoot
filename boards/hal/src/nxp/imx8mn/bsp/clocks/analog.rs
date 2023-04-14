use tock_registers::interfaces::ReadWriteable;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use super::super::drivers::common::MMIODerefWrapper;

register_bitfields! {
    u32,

    /// SYS PLL1 General Function Control Register
    SYS_PLL1_GEN_CTRL [
        /// PLL reference clock selecT
        ///
        /// 00 24M_REF_CLKE
        /// 01 PAD_CLKE
        /// 10 Reserved
        /// 11 Reserved
        PLL_REF_CLK_SEL OFFSET(0) NUMBITS(2) [
            Mhz24 = 0b00,
            PadClk = 0b01,
            Res1 = 0b10,
            Res2 = 0b11,
        ],
        /// PAD clock select
        ///
        /// PAD_CLKE is an alternate input reference clock for the PLL. The clock source selection for PAD_CLKE is
        /// defined below.
        ///
        /// 00 CLKIN1 XOR CLKIN2
        /// 01 CLKIN2
        /// 10 CLKIN1
        /// 11 Reserved
        PAD_CLK_SEL OFFSET(2) NUMBITS(2)[
            Clkin1X2 = 0b00,
            Clkin2 = 0b01,
            Clkin1 = 0b10,
            Res = 0b11,
        ],
        PLL_BYPASS OFFSET(4) NUMBITS(1)[],
        RESRV OFFSET(5) NUMBITS(3)[],
        PLL_RST_OVRD OFFSET(8) NUMBITS(1)[],
        PLL_RST OFFSET(9) NUMBITS(1)[],
        PLL_CLKE_OVRD OFFSET(10) NUMBITS(1)[],
        PLL_CLKE OFFSET(11) NUMBITS(1)[],
        PLL_DIV2_CLKE_OVRD OFFSET(12) NUMBITS(1)[],
        PLL_DIV2_CLKE OFFSET(13) NUMBITS(1)[],
        PLL_DIV3_CLKE_OVRD OFFSET(14) NUMBITS(1)[],
        PLL_DIV3_CLKE OFFSET(15) NUMBITS(1)[],
        PLL_DIV4_CLKE_OVRD OFFSET(16) NUMBITS(1)[],
        PLL_DIV4_CLKE OFFSET(17) NUMBITS(1)[],
        PLL_DIV5_CLKE_OVRD OFFSET(18) NUMBITS(1)[],
        PLL_DIV5_CLKE OFFSET(19) NUMBITS(1)[],
        PLL_DIV6_CLKE_OVRD OFFSET(20) NUMBITS(1)[],
        PLL_DIV6_CLKE OFFSET(21) NUMBITS(1)[],
        PLL_DIV8_CLKE_OVRD OFFSET(22) NUMBITS(1)[],
        PLL_DIV8_CLKE OFFSET(23) NUMBITS(1)[],
        PLL_DIV10_CLKE_OVRD OFFSET(24) NUMBITS(1)[],
        PLL_DIV10_CLKE OFFSET(25) NUMBITS(1)[],
        PLL_DIV20_CLKE_OVRD OFFSET(26) NUMBITS(1)[],
        PLL_DIV20_CLKE OFFSET(27) NUMBITS(1)[],
        PLL_EXT_BYPASS OFFSET(28) NUMBITS(1)[],
        PLL_LOCK_SEL OFFSET(29) NUMBITS(1)[],
        RESRV1 OFFSET(30) NUMBITS(1)[],
        PLL_LOCK OFFSET(31) NUMBITS(1)[],
    ],
    /// SYS PLL1 Divide and Fraction Data Control 0 Register
    SYS_PLL1_FDIV_CTRL [
        /// Value of the post-divider
        PLL_POST_DIV OFFSET(0) NUMBITS(3) [],
        RESRV OFFSET(3) NUMBITS(1)[],
        /// Value of the pre-divider
        PLL_PRE_DIV OFFSET(4) NUMBITS(6)[],
        RESRV1 OFFSET(10) NUMBITS(2)[],
        /// Value of the main-divider
        PLL_MAIN_DIV OFFSET(10) NUMBITS(2)[],
        RESRV2 OFFSET(22) NUMBITS(10)[],
    ],
    /// SYS PLL2 General Function Control Register
    SYS_PLL2_GEN_CTRL [
        /// PLL reference clock selecT
        ///
        /// 00 24M_REF_CLKE
        /// 01 PAD_CLKE
        /// 10 Reserved
        /// 11 Reserved
        PLL_REF_CLK_SEL OFFSET(0) NUMBITS(2) [
            Mhz24 = 0b00,
            PadClk = 0b01,
            Res1 = 0b10,
            Res2 = 0b11,
        ],
        /// PAD clock select
        ///
        /// PAD_CLKE is an alternate input reference clock for the PLL. The clock source selection for PAD_CLKE is
        /// defined below.
        ///
        /// 00 CLKIN1 XOR CLKIN2
        /// 01 CLKIN2
        /// 10 CLKIN1
        /// 11 Reserved
        PAD_CLK_SEL OFFSET(2) NUMBITS(2)[
            Clkin1X2 = 0b00,
            Clkin2 = 0b01,
            Clkin1 = 0b10,
            Res = 0b11,
        ],
        PLL_BYPASS OFFSET(4) NUMBITS(1)[],
        RESRV OFFSET(5) NUMBITS(3)[],
        PLL_RST_OVRD OFFSET(8) NUMBITS(1)[],
        PLL_RST OFFSET(9) NUMBITS(1)[],
        PLL_CLKE_OVRD OFFSET(10) NUMBITS(1)[],
        PLL_CLKE OFFSET(11) NUMBITS(1)[],
        PLL_DIV2_CLKE_OVRD OFFSET(12) NUMBITS(1)[],
        PLL_DIV2_CLKE OFFSET(13) NUMBITS(1)[],
        PLL_DIV3_CLKE_OVRD OFFSET(14) NUMBITS(1)[],
        PLL_DIV3_CLKE OFFSET(15) NUMBITS(1)[],
        PLL_DIV4_CLKE_OVRD OFFSET(16) NUMBITS(1)[],
        PLL_DIV4_CLKE OFFSET(17) NUMBITS(1)[],
        PLL_DIV5_CLKE_OVRD OFFSET(18) NUMBITS(1)[],
        PLL_DIV5_CLKE OFFSET(19) NUMBITS(1)[],
        PLL_DIV6_CLKE_OVRD OFFSET(20) NUMBITS(1)[],
        PLL_DIV6_CLKE OFFSET(21) NUMBITS(1)[],
        PLL_DIV8_CLKE_OVRD OFFSET(22) NUMBITS(1)[],
        PLL_DIV8_CLKE OFFSET(23) NUMBITS(1)[],
        PLL_DIV10_CLKE_OVRD OFFSET(24) NUMBITS(1)[],
        PLL_DIV10_CLKE OFFSET(25) NUMBITS(1)[],
        PLL_DIV20_CLKE_OVRD OFFSET(26) NUMBITS(1)[],
        PLL_DIV20_CLKE OFFSET(27) NUMBITS(1)[],
        PLL_EXT_BYPASS OFFSET(28) NUMBITS(1)[],
        PLL_LOCK_SEL OFFSET(29) NUMBITS(1)[],
        RESRV1 OFFSET(30) NUMBITS(1)[],
        PLL_LOCK OFFSET(31) NUMBITS(1)[],
    ],
    /// SYS PLL2 Divide and Fraction Data Control 0 Register
    SYS_PLL2_FDIV_CTRL [
        /// Value of the post-divider
        PLL_POST_DIV OFFSET(0) NUMBITS(3) [],
        RESRV OFFSET(3) NUMBITS(1)[],
        /// Value of the pre-divider
        PLL_PRE_DIV OFFSET(4) NUMBITS(6)[],
        RESRV1 OFFSET(10) NUMBITS(2)[],
        /// Value of the main-divider
        PLL_MAIN_DIV OFFSET(10) NUMBITS(2)[],
        RESRV2 OFFSET(22) NUMBITS(10)[],
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved0),
        (0x94 => SYS_PLL1_GEN_CTRL: ReadWrite<u32, SYS_PLL1_GEN_CTRL::Register>),
        (0x98 => SYS_PLL1_FDIV_CTRL: ReadWrite<u32, SYS_PLL1_FDIV_CTRL::Register>),
        (0x9c =>_reserved1),
        (0x104 => SYS_PLL2_GEN_CTRL: ReadWrite<u32, SYS_PLL2_GEN_CTRL::Register>),
        (0x108 => SYS_PLL2_FDIV_CTRL: ReadWrite<u32, SYS_PLL2_FDIV_CTRL::Register>),
        (0x10c => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

pub struct CCMAnalog {
    registers: Registers,
}

impl CCMAnalog {
    /// Create an instance.
    ///
    /// **Safety**
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    fn set_pll1_outputs(&self) {
        self.registers.SYS_PLL1_GEN_CTRL.modify(
            SYS_PLL1_GEN_CTRL::PLL_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV2_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV3_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV4_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV5_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV6_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV8_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV10_CLKE::SET
                + SYS_PLL1_GEN_CTRL::PLL_DIV20_CLKE::SET,
        )
    }
    fn set_pll2_outputs(&self) {
        self.registers.SYS_PLL2_GEN_CTRL.modify(
            SYS_PLL2_GEN_CTRL::PLL_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV2_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV3_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV4_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV5_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV6_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV8_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV10_CLKE::SET
                + SYS_PLL2_GEN_CTRL::PLL_DIV20_CLKE::SET,
        )
    }
    fn pll_configure(&self) {
        
    }
}
