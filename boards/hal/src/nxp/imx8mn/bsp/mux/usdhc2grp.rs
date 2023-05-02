//! Pin Mux settings for uSDHC2

use super::super::global::{GPIO2, GPIO1};
use super::super::memory_map::map::mmio::IOMUXC_START;
use super::iomuxc::*;

/// Pad Select
enum PadSelect {
    Gpio1Io04,
    Usdhc2Vselect,
    Sdma1ExtEvent1,
}

/// GPIO regs associated with uSDHC2
struct Usdhc2GpioRegs {
    iomuxc_sw_mux_ctl_pad_gpio1_io15: u32,
    iomuxc_sw_pad_ctl_pad_gpio1_io15: u32,
}

impl Default for Usdhc2GpioRegs {
    /// Defaults taken from 8.2.4 IOMUXC Memory Map/Register Definition of
    /// the i.MX 8M Nano Applications Processor Reference Manual, Rev. 2, 07/2022
    ///
    /// Note: the device tree for i.MX 8M Nano-EVK contains these offsets as well. Look for a
    /// a usdhc2gpiogrp `pincfg` within the pin-controller node (i.e. pinctrl@30330000)
    fn default() -> Self {
        Usdhc2GpioRegs {
            iomuxc_sw_mux_ctl_pad_gpio1_io15: (IOMUXC_START + 0x64) as u32,
            iomuxc_sw_pad_ctl_pad_gpio1_io15: (IOMUXC_START + 0x2cc) as u32,
        }
    }
}

impl Usdhc2GpioRegs {
    fn set_usdhc2_gpio_mux_cfg(&self, mux_val: MuxMode, sion_val: Sion) {
        // #Safety
        //
        // Only valid register writes (vals) are used via rust pattern-matching.
        match (mux_val, sion_val) {
            (MuxMode::Alt0, Sion::Disabled) => unsafe {
                // write to Pad Mux Registers
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_gpio1_io15 as *mut u32, 0x0);
            },
            _ => unimplemented!(),
        }

        GPIO1.clear_pin(15);
        // write to Pad Control Registers
        unsafe {
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_gpio1_io15 as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX2,
                    Fsel::Slow,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
        }
        // GPIO1.clear_pin(15);

    }

    fn get_pad_ctrl_val(&self, dse: Dse, fsel: Fsel, ode: Ode, pue: Pue, hys: Hys, pe: Pe) -> u32 {
        match (dse, fsel, ode, pue, hys, pe) {
            (Dse::DseX2, Fsel::Slow, Ode::Disabled, Pue::PullUp, Hys::Enabled, Pe::Enabled) => {
                0x1c4
            }
            _ => unimplemented!(),
        }
    }
}
/// Pin Muxing Registers for uSDHC2.
struct Usdhc2MuxRegs {
    // Pad Mux Registers
    iomuxc_sw_mux_ctl_pad_sd2_cd_b: u32,
    iomuxc_sw_mux_ctl_pad_sd2_clk: u32,
    iomuxc_sw_mux_ctl_pad_sd2_cmd: u32,
    iomuxc_sw_mux_ctl_pad_sd2_data0: u32,
    iomuxc_sw_mux_ctl_pad_sd2_data1: u32,
    iomuxc_sw_mux_ctl_pad_sd2_data2: u32,
    iomuxc_sw_mux_ctl_pad_sd2_data3: u32,
    iomuxc_sw_mux_ctl_pad_sd2_reset_b: u32,
    iomuxc_sw_mux_ctl_pad_sd2_wp: u32,
    // Pad Control Registers,
    iomuxc_sw_pad_ctl_pad_sd2_cd_b: u32,
    iomuxc_sw_pad_ctl_pad_sd2_clk: u32,
    iomuxc_sw_pad_ctl_pad_sd2_cmd: u32,
    iomuxc_sw_pad_ctl_pad_sd2_data0: u32,
    iomuxc_sw_pad_ctl_pad_sd2_data1: u32,
    iomuxc_sw_pad_ctl_pad_sd2_data2: u32,
    iomuxc_sw_pad_ctl_pad_sd2_data3: u32,
    iomuxc_sw_pad_ctl_pad_sd2_reset_b: u32,
    iomuxc_sw_pad_ctl_pad_sd2_wp: u32,
    // Select Input Register
    iomuxc_sw_mux_ctl_pad_usdhc2_vselect: u32,
    iomuxc_sw_pad_ctl_pad_usdhc2_vselect: u32,
}

impl Default for Usdhc2MuxRegs {
    /// Defaults taken from 8.2.4 IOMUXC Memory Map/Register Definition of
    /// the i.MX 8M Nano Applications Processor Reference Manual, Rev. 2, 07/2022
    ///
    /// Note: the device tree for i.MX 8M Nano-EVK contains these offsets as well. Look for a
    /// a usdhc2 `pincfg` within the pin-controller node (i.e. pinctrl@30330000)
    fn default() -> Usdhc2MuxRegs {
        Usdhc2MuxRegs {
            iomuxc_sw_mux_ctl_pad_sd2_cd_b: (IOMUXC_START + 0xd0) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_clk: (IOMUXC_START + 0xd4) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_cmd: (IOMUXC_START + 0xd8) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_data0: (IOMUXC_START + 0xdc) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_data1: (IOMUXC_START + 0xe0) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_data2: (IOMUXC_START + 0xe4) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_data3: (IOMUXC_START + 0xe8) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_reset_b: (IOMUXC_START + 0xec) as u32,
            iomuxc_sw_mux_ctl_pad_sd2_wp: (IOMUXC_START + 0xf0) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_cd_b: (IOMUXC_START + 0x338) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_clk: (IOMUXC_START + 0x33c) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_cmd: (IOMUXC_START + 0x340) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_data0: (IOMUXC_START + 0x344) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_data1: (IOMUXC_START + 0x348) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_data2: (IOMUXC_START + 0x34c) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_data3: (IOMUXC_START + 0x350) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_reset_b: (IOMUXC_START + 0x354) as u32,
            iomuxc_sw_pad_ctl_pad_sd2_wp: (IOMUXC_START + 0x358) as u32,
            iomuxc_sw_mux_ctl_pad_usdhc2_vselect: (IOMUXC_START + 0x38) as u32,
            iomuxc_sw_pad_ctl_pad_usdhc2_vselect: (IOMUXC_START + 0x2a0) as u32,
        }
    }
}

impl Usdhc2MuxRegs {
    fn set_usdhc2_mux_cfg(&self, mux_val: MuxMode, sion_val: Sion, input_selector: u32) {
        // set sd2-reset pin to alternate mux-mode i.e. gpio 19
        unsafe {
            ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_reset_b as *mut u32, 0x5);
            GPIO2.set_pin(19);
        }
        // #Safety
        //
        // Only valid register writes (vals) are used via rust pattern-matching.
        match (mux_val, sion_val) {
            (MuxMode::Alt0, Sion::Disabled) => unsafe {
                // write to Pad Mux Registers
                // ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_cd_b as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_clk as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_cmd as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data0 as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data1 as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data2 as *mut u32, 0x0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data3 as *mut u32, 0x0);
                // ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_wp as *mut u32, 0x0);
            },
            _ => unimplemented!(),
        }

        // write to Pad Control Registers
        unsafe {
            // ::core::ptr::write_volatile(
            //     self.iomuxc_sw_pad_ctl_pad_sd2_cd_b as *mut u32,
            //     self.get_pad_ctrl_val(
            //         Dse::DseX1,
            //         Fsel::Slow,
            //         Ode::Disabled,
            //         Pue::PullDown,
            //         Hys::Enabled,
            //         Pe::Enabled,
            //     ),
            // );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_clk as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullDown,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_cmd as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data0 as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data1 as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data2 as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data3 as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_reset_b as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Slow,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Disabled,
                    Pe::Disabled,
                ),
            );
            // ::core::ptr::write_volatile(
            //     self.iomuxc_sw_pad_ctl_pad_sd2_wp as *mut u32,
            //     self.get_pad_ctrl_val(
            //         Dse::DseX1,
            //         Fsel::Fast,
            //         Ode::Disabled,
            //         Pue::PullDown,
            //         Hys::Enabled,
            //         Pe::Enabled,
            //     ),
            // );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_mux_ctl_pad_usdhc2_vselect as *mut u32,
                input_selector,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_usdhc2_vselect as *mut u32,
                self.get_pad_ctrl_val(
                    Dse::DseX1,
                    Fsel::Fast,
                    Ode::Disabled,
                    Pue::PullUp,
                    Hys::Enabled,
                    Pe::Enabled,
                ),
            );
            GPIO2.clear_pin(19) // clear - gpio2 pin 19 data register.
        }
    }

    fn get_pad_ctrl_val(&self, dse: Dse, fsel: Fsel, ode: Ode, pue: Pue, hys: Hys, pe: Pe) -> u32 {
        match (dse, fsel, ode, pue, hys, pe) {
            (Dse::DseX1, Fsel::Fast, Ode::Disabled, Pue::PullUp, Hys::Enabled, Pe::Enabled) => {
                0x1d0
            }
            (Dse::DseX1, Fsel::Fast, Ode::Disabled, Pue::PullDown, Hys::Enabled, Pe::Enabled) => {
                0x190
            }
            (Dse::DseX1, Fsel::Slow, Ode::Disabled, Pue::PullUp, Hys::Disabled, Pe::Disabled) => {
                0x41
            }
            (Dse::DseX1, Fsel::Slow, Ode::Disabled, Pue::PullUp, Hys::Enabled, Pe::Disabled) => {
                0xc1
            }
            _ => unimplemented!(),
        }
    }

    fn get_pad_sel_val(&self, pad_sel_val: PadSelect) -> u32 {
        match pad_sel_val {
            PadSelect::Usdhc2Vselect => 0x1,
            _ => unimplemented!(),
        }
    }
}

/// Set mux-config for the uSDHC2 peripheral.
pub fn usdhc2_mux_mmio_set() {
    // set uSDHC2 gpio mux and pincfg
    let usdhc_gpio_regs = Usdhc2GpioRegs::default();
    usdhc_gpio_regs.set_usdhc2_gpio_mux_cfg(MuxMode::Alt0, Sion::Disabled);
    // set uSDHC2 mux and pincfg
    let usdhc_regs = Usdhc2MuxRegs::default();
    let input_selector = usdhc_regs.get_pad_sel_val(PadSelect::Usdhc2Vselect);
    usdhc_regs.set_usdhc2_mux_cfg(MuxMode::Alt0, Sion::Disabled, input_selector);
}
