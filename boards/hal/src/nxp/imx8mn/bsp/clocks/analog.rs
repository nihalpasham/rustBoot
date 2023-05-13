//! PLL configuration - TODO - implementation not ready yet

use tock_registers::interfaces::ReadWriteable;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use super::super::drivers::common::MMIODerefWrapper;
use super::super::drivers::usdhc::timer_wait_micro;
use super::ccm::*;

const INTPLL_CLKE_MASK: u32 = 1 << 11;

pub enum PllClocks {
    ArmPll,
    VpuPll,
    GpuPll,
    SystemPll1,
    SystemPll2,
    SystemPll3,
    AudioPll1,
    AudioPll2,
    VideoPll,
    DramPll,
}

register_bitfields! {
    u32,

    /// ARM PLL General Function Control Register
    ARM_GEN_CTRL [
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
        RESRV0 OFFSET(5) NUMBITS(3)[],
        PLL_RST_OVRD OFFSET(8) NUMBITS(1)[],
        PLL_RST OFFSET(9) NUMBITS(1)[],
        PLL_CLKE_OVRD OFFSET(10) NUMBITS(1)[],
        PLL_CLKE OFFSET(11) NUMBITS(1)[],
        RESRV1 OFFSET(12) NUMBITS(16)[],
        PLL_EXT_BYPASS OFFSET(28) NUMBITS(1)[],
        PLL_LOCK_SEL OFFSET(29) NUMBITS(1)[],
        RESRV2 OFFSET(30) NUMBITS(1)[],
        PLL_LOCK OFFSET(31) NUMBITS(1)[],
    ],
    /// ARM PLL Divide and Fraction Data Control 0 Register
    ARM_FDIV_CTRL [
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
    /// SYS PLL3 General Function Control Register
    SYS_PLL3_GEN_CTRL [
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
    /// SYS PLL3 Divide and Fraction Data Control 0 Register
    SYS_PLL3_FDIV_CTRL [
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
        (0x84 => ARM_PLL_GEN_CTRL: ReadWrite<u32, ARM_GEN_CTRL::Register>),
        (0x88 => ARM_PLL_FDIV_CTRL: ReadWrite<u32, ARM_FDIV_CTRL::Register>),
        (0x8c => _reserved1),
        (0x94 => SYS_PLL1_GEN_CTRL: ReadWrite<u32, SYS_PLL1_GEN_CTRL::Register>),
        (0x98 => SYS_PLL1_FDIV_CTRL: ReadWrite<u32, SYS_PLL1_FDIV_CTRL::Register>),
        (0x9c =>_reserved2),
        (0x104 => SYS_PLL2_GEN_CTRL: ReadWrite<u32, SYS_PLL2_GEN_CTRL::Register>),
        (0x108 => SYS_PLL2_FDIV_CTRL: ReadWrite<u32, SYS_PLL2_FDIV_CTRL::Register>),
        (0x10c => _reserved3),
        (0x114 => SYS_PLL3_GEN_CTRL: ReadWrite<u32, SYS_PLL3_GEN_CTRL::Register>),
        (0x118 => SYS_PLL3_FDIV_CTRL: ReadWrite<u32, SYS_PLL3_FDIV_CTRL::Register>),
        (0x11c => @END),

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

    pub fn set_pll1_outputs(&self) {
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
    pub fn set_pll2_outputs(&self) {
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
    /// TODO: implementation not complete. Still needs to be tested
    pub fn pll_configure(&self, pll: PllClocks, freq: u32) {
        let pll_clke_masks = INTPLL_CLKE_MASK;
        // Bypass clock and set lock to pll output lock
        match pll {
            PllClocks::ArmPll => {
                self.registers
                    .ARM_PLL_GEN_CTRL
                    .modify(ARM_GEN_CTRL::PLL_BYPASS::SET + ARM_GEN_CTRL::PLL_LOCK_SEL::SET);
                // Enable reset
                self.registers
                    .ARM_PLL_GEN_CTRL
                    .modify(ARM_GEN_CTRL::PLL_RST::CLEAR);
                // configure
                match freq {
                    1200 => self.registers.ARM_PLL_FDIV_CTRL.modify(
                        ARM_FDIV_CTRL::PLL_MAIN_DIV.val(0x12c)
                            + ARM_FDIV_CTRL::PLL_PRE_DIV.val(3)
                            + ARM_FDIV_CTRL::PLL_POST_DIV.val(1),
                    ),
                    _ => {}
                }
                // delay
                timer_wait_micro(100);
                // Disable reset
                self.registers
                    .ARM_PLL_GEN_CTRL
                    .modify(ARM_GEN_CTRL::PLL_RST::SET);
                // wait for pll lock
                while !self
                    .registers
                    .ARM_PLL_GEN_CTRL
                    .is_set(ARM_GEN_CTRL::PLL_LOCK)
                {}
                // Clear bypass clock
                self.registers
                    .ARM_PLL_GEN_CTRL
                    .modify(ARM_GEN_CTRL::PLL_BYPASS::CLEAR);
                self.registers
                    .ARM_PLL_GEN_CTRL
                    .modify(ARM_GEN_CTRL::PLL_CLKE::SET);
            }
            PllClocks::SystemPll3 => {
                self.registers.SYS_PLL3_GEN_CTRL.modify(
                    SYS_PLL3_GEN_CTRL::PLL_BYPASS::SET + SYS_PLL3_GEN_CTRL::PLL_LOCK_SEL::SET,
                );
                // Enable reset
                self.registers
                    .SYS_PLL3_GEN_CTRL
                    .modify(SYS_PLL3_GEN_CTRL::PLL_RST::CLEAR);
                // configure
                match freq {
                    600 => self.registers.SYS_PLL3_FDIV_CTRL.modify(
                        SYS_PLL3_FDIV_CTRL::PLL_MAIN_DIV.val(0x12c)
                            + SYS_PLL3_FDIV_CTRL::PLL_PRE_DIV.val(3)
                            + SYS_PLL3_FDIV_CTRL::PLL_POST_DIV.val(2),
                    ),
                    _ => {}
                }
                // delay
                timer_wait_micro(100);
                // Disable reset
                self.registers
                    .SYS_PLL3_GEN_CTRL
                    .modify(SYS_PLL3_GEN_CTRL::PLL_RST::SET);
                // wait for pll lock
                while !self
                    .registers
                    .SYS_PLL3_GEN_CTRL
                    .is_set(SYS_PLL3_GEN_CTRL::PLL_LOCK)
                {}
                // Clear bypass 
                self.registers
                    .SYS_PLL3_GEN_CTRL
                    .modify(SYS_PLL3_GEN_CTRL::PLL_BYPASS::CLEAR);
                self.registers
                    .SYS_PLL3_GEN_CTRL
                    .modify(SYS_PLL3_GEN_CTRL::PLL_CLKE::SET);
            }
            PllClocks::SystemPll1 => {}
            PllClocks::SystemPll2 => {}
            _ => {}
        }
    }
    /// TODO: implementation not complete. Still needs to be tested
    /// Configure system Plls and set clock-gates, root-clocks for GIC, DRAM, NAND, WDG etc.
    pub fn clock_init(&self) {
        self.set_pll1_outputs();
        self.set_pll2_outputs();
        // Configure ARM at 1.2GHz
        clock_set_target_val(
            ClkRootIdx::ArmA53ClkRoot,
            CLK_ROOT_ON | clk_root_source_sel(2),
        );
        self.pll_configure(PllClocks::ArmPll, 1200);

        // Bypass CCM A53 ROOT, Switch to ARM PLL -> MUX-> CPU
        clock_set_target_val(ClkRootIdx::CoreSelCfg, clk_root_source_sel(1));

        self.pll_configure(PllClocks::SystemPll3, 600);

        clock_set_target_val(ClkRootIdx::NocClkRoot, CLK_ROOT_ON | clk_root_source_sel(2));

        // config GIC to sys_pll2_100m
        clock_enable(CCGRIdx::CcgrGic, false);
        clock_set_target_val(ClkRootIdx::GicClkRoot, CLK_ROOT_ON | clk_root_source_sel(3));
        clock_enable(CCGRIdx::CcgrGic, true);

        clock_set_target_val(
            ClkRootIdx::NandUsdhcBusClkRoot,
            CLK_ROOT_ON | clk_root_source_sel(1),
        );

        clock_enable(CCGRIdx::CcgrDdr1, false);
        clock_set_target_val(
            ClkRootIdx::DramAltClkRoot,
            CLK_ROOT_ON | clk_root_source_sel(1),
        );
        clock_set_target_val(
            ClkRootIdx::DramApbClkRoot,
            CLK_ROOT_ON | clk_root_source_sel(1),
        );
        clock_enable(CCGRIdx::CcgrDdr1, true);

        // init watchdog clocks
        clock_enable(CCGRIdx::CcgrWdog1, false);
        clock_enable(CCGRIdx::CcgrWdog2, false);
        clock_enable(CCGRIdx::CcgrWdog3, false);
        clock_set_target_val(
            ClkRootIdx::WdogClkRoot,
            CLK_ROOT_ON | clk_root_source_sel(0),
        );
        clock_enable(CCGRIdx::CcgrWdog1, true);
        clock_enable(CCGRIdx::CcgrWdog2, true);
        clock_enable(CCGRIdx::CcgrWdog3, true);

        clock_enable(CCGRIdx::CcgrTempSensor, true);

        clock_enable(CCGRIdx::CcgrSecDebug, true);
    }
}
