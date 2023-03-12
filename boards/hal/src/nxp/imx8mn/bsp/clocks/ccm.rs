//! Clock Control Module.
//! 
//! This module manages the on-chip module clocks. CCM receives clocks
//! from PLLs and oscillators and creates clocks for on-chip peripherals through a set of
//! multiplexers, dividers and gates. 
//! 
//! We're using the `target interface`. This programming model is a 2-step process 
//! - enable or disable `gates` corresponding to the desired clock
//! - set (or turn on) target `root` clock

use super::super::memory_map::*;
use crate::info;

/// (CCGR) is a register that controls the clock gating of a specific module or peripheral
const CCM_CCGR_N_SET: u32 = (map::mmio::CCM_START + 0x4004) as u32;
const CCM_CCGR_N_CLR: u32 = (map::mmio::CCM_START + 0x4008) as u32;
/// The `Target Register` is used to set the target `root` clock of a specific module or peripheral
const CCM_TARGET_ROOT_N_SET: u32 = (map::mmio::CCM_START + 0x8004) as u32;

/// Enable target root clock
const CLK_ROOT_ON: u32 = 1 << 28;
/// Disable target root clock
const CLK_ROOT_OFF: u32 = 0 << 28;

/// Clock source selection for each clock slice
const fn clk_root_source_sel(n: u32) -> u32 {
    (((n) & 0x7) << 24)
}

/// Clock Root Selects
/// The table below details the clock root slices. 
/// 
/// See Table 5-1. Clock Root Table
enum ClkRootIdx {
    ArmA53ClkRoot = 0,
    ArmM7ClkRoot = 1,
    Uart1ClkRoot = 94,
    Uart2ClkRoot = 95,
    Uart3ClkRoot = 96,
    Uart4ClkRoot = 97,
    Sai7ClkRoot = 134,
    ClkRootMax,
}

/// CCGRIdx identifies the clock-gate idx for a 
/// given module or peripheral
/// 
/// See Table 5-9. CCGR Mapping Table
enum CCGRIdx {
    CcgrDvfs = 0,
    CcgrCpu = 2,
    CcgrSctr = 57,
    CcgrUart1 = 73,
    CcgrUart2 = 74,
    CcgrUart3 = 75,
    CcgrUart4 = 76,
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
fn clock_enable(idx: CCGRIdx, enabled: bool) {
    let ccgr = if enabled {
        match idx {
            CCGRIdx::CcgrUart2 => CCM_CCGR_N_SET + (0x10 * 74),
            CCGRIdx::CcgrSctr => CCM_CCGR_N_SET + (0x10 * 57),
            _ => {
                unimplemented!()
            }
        }
    } else {
        match idx {
            CCGRIdx::CcgrUart2 => CCM_CCGR_N_CLR + (0x10 * 74),
            CCGRIdx::CcgrSctr => CCM_CCGR_N_CLR + (0x10 * 57),
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
fn clock_set_target_val(idx: ClkRootIdx, val: u32) {
    let target_clk = match idx {
        ClkRootIdx::Uart2ClkRoot => CCM_TARGET_ROOT_N_SET + (0x80 * 95),
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

/// Initialize UART clocks. The `i.MX8MN` has 4 UARTs in total. Although, a UART interface does not use a clock signal i.e.
/// it is an asynchronous protocol, the i.MX8MN requires the corresponding clock for a UART to be enabled
/// before you can start using the peripheral.
pub fn init_uart_clk(index: u32) {
    /*
     * set uart clock root
     * 24M OSC
     */
    match index {
        0 => {
            clock_enable(CCGRIdx::CcgrUart1, false);
            clock_set_target_val(
                ClkRootIdx::Uart1ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(0),
            );
            clock_enable(CCGRIdx::CcgrUart1, true);
        }
        1 => {
            clock_enable(CCGRIdx::CcgrUart2, false);
            clock_set_target_val(
                ClkRootIdx::Uart2ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(0),
            );
            clock_enable(CCGRIdx::CcgrUart2, true);
        }
        2 => {
            clock_enable(CCGRIdx::CcgrUart3, false);
            clock_set_target_val(
                ClkRootIdx::Uart3ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(0),
            );
            clock_enable(CCGRIdx::CcgrUart3, true);
        }
        3 => {
            clock_enable(CCGRIdx::CcgrUart4, false);
            clock_set_target_val(
                ClkRootIdx::Uart4ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(0),
            );
            clock_enable(CCGRIdx::CcgrUart4, true);
        }
        _ => {
            info!("invalid uart index\n");
        }
    }
}

/// Allow system counter i.e. ungate clock gate for SCTR.
pub fn enable_sctr() {
    clock_enable(CCGRIdx::CcgrSctr, true)
}