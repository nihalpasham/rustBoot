//! Clock Control Module.
//!
//! This module manages the on-chip module clocks. CCM receives clocks
//! from PLLs and oscillators and creates clocks for on-chip peripherals through a set of
//! multiplexers, dividers and gates.
//!
//! We're using the `target interface` api. This programming model is a 2-step process
//! - enable or disable `gates` corresponding to the desired clock
//! - set (or turn on) target `root` clock

use super::super::memory_map::*;

/// (CCGR) is a register that controls the clock gating of a specific module or peripheral
const CCM_CCGR_N_SET: u32 = (map::mmio::CCM_START + 0x4004) as u32;
const CCM_CCGR_N_CLR: u32 = (map::mmio::CCM_START + 0x4008) as u32;
/// The `Target Register` is used to set the target `root` clock of a specific module or peripheral
const CCM_TARGET_ROOT_N_SET: u32 = (map::mmio::CCM_START + 0x8004) as u32;

/// Enable target root clock
pub const CLK_ROOT_ON: u32 = 1 << 28;
/// Disable target root clock
pub const CLK_ROOT_OFF: u32 = 0 << 28;

/// Clock source selection for each clock slice
pub const fn clk_root_source_sel(n: u32) -> u32 {
    (((n) & 0x7) << 24)
}

/// Clock Root Selects
/// The table below details the clock root slices.
///
/// See Table 5-1. Clock Root Table
pub enum ClkRootIdx {
    ArmA53ClkRoot = 0,
    ArmM7ClkRoot = 1,
    NandUsdhcBusClkRoot = 18,
    NocClkRoot = 26,
    CoreSelCfg = 49,
    DramAltClkRoot = 64,
    DramApbClkRoot = 65,
    Usdhc1ClkRoot = 88,
    Usdhc2ClkRoot = 89,
    Uart1ClkRoot = 94,
    Uart2ClkRoot = 95,
    Uart3ClkRoot = 96,
    Uart4ClkRoot = 97,
    GicClkRoot = 100,
    WdogClkRoot = 114,
    Usdhc3ClkRoot = 121,
    Sai7ClkRoot = 134,
    ClkRootMax,
}

/// CCGRIdx identifies the clock-gate idx for a
/// given module or peripheral
///
/// See Table 5-9. CCGR Mapping Table
pub enum CCGRIdx {
    CcgrDvfs = 0,
    CcgrCpu = 2,
    CcgrDdr1 = 5,
    CcgrSctr = 57,
    CcgrSecDebug = 60,
    CcgrUart1 = 73,
    CcgrUart2 = 74,
    CcgrUart3 = 75,
    CcgrUart4 = 76,
    CcgrUsdhc1 = 81,
    CcgrUsdhc2 = 82,
    CcgrWdog1 = 83,
    CcgrWdog2 = 84,
    CcgrWdog3 = 85,
    CcgrGic = 92,
    CcgrUsdhc3 = 94,
    CcgrTempSensor = 98,
}

/// Before a clock root goes to on–chip peripherals, the clock root is distributed through low
/// power clock gates (LPCG). CCGR registers allow us to set `clock gate control` settings.
///
/// Valid values are
/// - 00 Domain clocks not needed
/// - 01 Domain clocks needed when in RUN
/// - 10 Domain clocks needed when in RUN and WAIT
/// - 11 Domain clocks needed all the time
///
/// TODO! - figure out what domains (0-3) actually mean here.
pub fn clock_enable(idx: CCGRIdx, enabled: bool) {
    let ccgr = if enabled {
        match idx {
            CCGRIdx::CcgrDdr1 => CCM_CCGR_N_SET + (0x10 * 5),
            CCGRIdx::CcgrUart2 => CCM_CCGR_N_SET + (0x10 * 74),
            CCGRIdx::CcgrSctr => CCM_CCGR_N_SET + (0x10 * 57),
            CCGRIdx::CcgrSecDebug => CCM_CCGR_N_SET + (0x10 * 60),
            CCGRIdx::CcgrUsdhc2 => CCM_CCGR_N_SET + (0x10 * 82),
            CCGRIdx::CcgrWdog1 => CCM_CCGR_N_SET + (0x10 * 83),
            CCGRIdx::CcgrWdog2 => CCM_CCGR_N_SET + (0x10 * 84),
            CCGRIdx::CcgrWdog3 => CCM_CCGR_N_SET + (0x10 * 85),
            CCGRIdx::CcgrGic => CCM_CCGR_N_SET + (0x10 * 92),
            CCGRIdx::CcgrTempSensor => CCM_CCGR_N_SET + (0x10 * 98),
            _ => {
                unimplemented!()
            }
        }
    } else {
        match idx {
            CCGRIdx::CcgrDdr1 => CCM_CCGR_N_CLR + (0x10 * 5),
            CCGRIdx::CcgrUart2 => CCM_CCGR_N_CLR + (0x10 * 74),
            CCGRIdx::CcgrSctr => CCM_CCGR_N_CLR + (0x10 * 57),
            CCGRIdx::CcgrSecDebug => CCM_CCGR_N_CLR + (0x10 * 60),
            CCGRIdx::CcgrUsdhc2 => CCM_CCGR_N_CLR + (0x10 * 82),
            CCGRIdx::CcgrWdog1 => CCM_CCGR_N_CLR + (0x10 * 83),
            CCGRIdx::CcgrWdog2 => CCM_CCGR_N_CLR + (0x10 * 84),
            CCGRIdx::CcgrWdog3 => CCM_CCGR_N_CLR + (0x10 * 85),
            CCGRIdx::CcgrGic => CCM_CCGR_N_CLR + (0x10 * 92),
            CCGRIdx::CcgrTempSensor => CCM_CCGR_N_CLR + (0x10 * 98),
            _ => {
                unimplemented!()
            }
        }
    };
    // #Safety
    //
    // casting ccgr produces an address in CCM's memory-map. The address is valid assuming
    // the offsets are set as per i.MX8MN reference manual
    unsafe { ::core::ptr::write_volatile(ccgr as *mut u32, 0x3) }
}

/// Set the `target clock` for a specific module or peripheral (i.e. enable root clock for a given module)
///
/// Apart from enabling clocks for peripherals, the `Target Register` also allows us to set
/// - the clock source number to be selected,
/// - pre-divide value, and
/// - post-divide value.
///
/// If a clock slice does not support a setting, that setting is simply ignored, and will not effect the supported
/// fields.
///
/// This function only enables or disables the clock and does not change any other settings.
pub fn clock_set_target_val(idx: ClkRootIdx, val: u32) {
    let target_clk = match idx {
        ClkRootIdx::ArmA53ClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 0),
        ClkRootIdx::NandUsdhcBusClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 18),
        ClkRootIdx::NocClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 26),
        ClkRootIdx::CoreSelCfg => CCM_TARGET_ROOT_N_SET + (0x80 * 49),
        ClkRootIdx::DramAltClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 64),
        ClkRootIdx::DramApbClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 65),
        ClkRootIdx::Usdhc2ClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 89),
        ClkRootIdx::Uart2ClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 95),
        ClkRootIdx::GicClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 100),
        ClkRootIdx::WdogClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 114),
        _ => {
            unimplemented!()
        }
    };
    // #Safety
    //
    // casting target_clk produces an address in CCM's memory-map. The address is valid assuming
    // the offsets are set as per i.MX8MN reference manual
    unsafe { ::core::ptr::write_volatile(target_clk as *mut u32, val) }
}
