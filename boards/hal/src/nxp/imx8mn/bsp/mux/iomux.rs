//! IOMUX settings for Pin Muxing

use super::super::memory_map::map::mmio::IOMUXC_START;

/// MUX Mode Select Field
enum MuxMode {
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
    Alt6,
    Alt7,
}

/// Software Input On Field.
enum Sion {
    Enabled,
    Disabled,
}

/// Drive Strength Field
enum Dse {
    DseX1,
    DseX6,
    Unimplemented,
}
/// Slew Rate Field
enum Fsel {
    Slow,
    Fast,
}
/// Open Drain Enable Field
enum Ode {
    Enabled,
    Disabled,
}
/// Control IO ports PS
enum Pue {
    PullUp,
    PullDown,
}
/// Hysteresis Enable Field
enum Hys {
    Enabled,
    Disabled,
}
/// Pull Resistors Enable Field
enum Pe {
    Enabled,
    Disabled,
}

/// Pad Select
///
enum PadSelect {
    Uart2RxdAlt0,
    Uart2TxdAlt0,
    Unimplemented,
}

fn construct_pad_val(dse: Dse, fsel: Fsel, ode: Ode, pue: Pue, hys: Hys, pe: Pe) -> u32 {
    match (dse, fsel, ode, pue, hys, pe) {
        (Dse::DseX6, Fsel::Slow, Ode::Disabled, Pue::PullDown, Hys::Disabled, Pe::Disabled) => 7,
        _ => unimplemented!(),
    }
}

fn construct_pad_sel_val(pad_sel_val: PadSelect) -> u32 {
    match pad_sel_val {
        PadSelect::Uart2RxdAlt0 | PadSelect::Uart2TxdAlt0 => 0,
        _ => unimplemented!(),
    }
}

/// Pin Muxing Registers for UART2.
struct Uart2MuxRegs {
    // Pad Mux Registers
    iomuxc_sw_mux_ctl_pad_uart2_rxd: u32,
    iomuxc_sw_mux_ctl_pad_uart2_txd: u32,
    // Pad Control Registers
    iomuxc_sw_pad_ctl_pad_uart2_rxd: u32,
    iomuxc_sw_pad_ctl_pad_uart2_txd: u32,
    // Select Input Register
    iomuxc_uart2_rx_select_input: u32,
}

impl Default for Uart2MuxRegs {
    /// Defaults taken from 8.2.4 IOMUXC Memory Map/Register Definition of
    /// the i.MX 8M Nano Applications Processor Reference Manual, Rev. 2, 07/2022
    /// 
    /// Note: the device tree for i.MX 8M Nano-EVK contains these offsets as well. Look for a 
    /// a uart2 `pincfg` within the pin-controller node (i.e. pinctrl@30330000)
    fn default() -> Uart2MuxRegs {
        Uart2MuxRegs {
            iomuxc_sw_mux_ctl_pad_uart2_rxd: (IOMUXC_START + 0x23c) as u32,
            iomuxc_sw_mux_ctl_pad_uart2_txd: (IOMUXC_START + 0x240) as u32,
            iomuxc_sw_pad_ctl_pad_uart2_rxd: (IOMUXC_START + 0x4a4) as u32,
            iomuxc_sw_pad_ctl_pad_uart2_txd: (IOMUXC_START + 0x4a8) as u32,
            iomuxc_uart2_rx_select_input: (IOMUXC_START + 0x4fc) as u32,
        }
    }
}

fn set_uart2_mux_cfg(
    uart_regs: Uart2MuxRegs,
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
            ::core::ptr::write_volatile(uart_regs.iomuxc_sw_mux_ctl_pad_uart2_rxd as *mut u32, 0);
            ::core::ptr::write_volatile(uart_regs.iomuxc_sw_mux_ctl_pad_uart2_txd as *mut u32, 0);
        },
        _ => unimplemented!(),
    }

    // write to Pad Control Registers
    unsafe {
        ::core::ptr::write_volatile(
            uart_regs.iomuxc_sw_pad_ctl_pad_uart2_rxd as *mut u32,
            pad_ctrl_val,
        );
        ::core::ptr::write_volatile(
            uart_regs.iomuxc_sw_pad_ctl_pad_uart2_txd as *mut u32,
            pad_ctrl_val,
        );
        ::core::ptr::write_volatile(
            uart_regs.iomuxc_uart2_rx_select_input as *mut u32,
            input_selector,
        );
    }
}

/// Set mux-config for the UART2 peripheral.
pub fn uart2_mux_mmio_set() {
    let uart_regs = Uart2MuxRegs::default();
    let pad_ctrl_val = construct_pad_val(
        Dse::DseX6,
        Fsel::Slow,
        Ode::Disabled,
        Pue::PullDown,
        Hys::Disabled,
        Pe::Disabled,
    );
    let input_selector = construct_pad_sel_val(PadSelect::Uart2RxdAlt0);

    set_uart2_mux_cfg(
        uart_regs,
        MuxMode::Alt0,
        Sion::Disabled,
        pad_ctrl_val,
        input_selector,
    );
}
