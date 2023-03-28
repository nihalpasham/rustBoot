//! Pin Mux settings for uSDHC2

use super::super::memory_map::map::mmio::IOMUXC_START;
use super::iomuxc::*;

/// Pad Select
enum PadSelect {
    Gpio1Io04,
    Usdhc2Vselect,
    Sdma1ExtEvent1,
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
    fn set_usdhc2_mux_cfg(
        &self,
        mux_val: MuxMode,
        sion_val: Sion,
        pad_ctrl_val: u32,
        input_selector: u32,
    ) {
        // #Safety
        //
        // Only valid register writes vals are used via rust pattern-matching.
        match (mux_val, sion_val) {
            (MuxMode::Alt0, Sion::Disabled) => unsafe {
                // write to Pad Mux Registers
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_cd_b as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_clk as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_cmd as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data0 as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data1 as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data2 as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_data3 as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_wp as *mut u32, 0);
                ::core::ptr::write_volatile(self.iomuxc_sw_mux_ctl_pad_sd2_reset_b as *mut u32, 0);
            },
            _ => unimplemented!(),
        }

        // write to Pad Control Registers
        unsafe {
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_cd_b as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_clk as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_cmd as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data0 as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data1 as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data2 as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_data3 as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_reset_b as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_sd2_wp as *mut u32,
                pad_ctrl_val,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_mux_ctl_pad_usdhc2_vselect as *mut u32,
                input_selector,
            );
            ::core::ptr::write_volatile(
                self.iomuxc_sw_pad_ctl_pad_usdhc2_vselect as *mut u32,
                pad_ctrl_val,
            );
        }
    }

    fn get_pad_ctrl_val(&self, dse: Dse, fsel: Fsel, ode: Ode, pue: Pue, hys: Hys, pe: Pe) -> u32 {
        match (dse, fsel, ode, pue, hys, pe) {
            (Dse::DseX6, Fsel::Slow, Ode::Disabled, Pue::PullDown, Hys::Disabled, Pe::Disabled) => {
                0x6
            }
            _ => unimplemented!(),
        }
    }

    fn get_pad_sel_val(&self, pad_sel_val: PadSelect) -> u32 {
        match pad_sel_val {
            PadSelect::Usdhc2Vselect => 1,
            _ => unimplemented!(),
        }
    }
}

/// Set mux-config for the uSDHC2 peripheral.
pub fn usdhc2_mux_mmio_set() {
    let usdhc_regs = Usdhc2MuxRegs::default();
    let pad_ctrl_val = usdhc_regs.get_pad_ctrl_val(
        Dse::DseX6,
        Fsel::Slow,
        Ode::Disabled,
        Pue::PullDown,
        Hys::Disabled,
        Pe::Disabled,
    );
    let input_selector = usdhc_regs.get_pad_sel_val(PadSelect::Usdhc2Vselect);

    usdhc_regs.set_usdhc2_mux_cfg(MuxMode::Alt0, Sion::Disabled, pad_ctrl_val, input_selector);
}
