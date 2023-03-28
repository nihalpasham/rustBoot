//! The `i.MX8MN` has 4 UARTs in total. Although, a UART interface does not use a clock signal i.e.
//! it is an asynchronous protocol, the i.MX8MN requires the corresponding clock for a UART to be enabled
//! before you can start using the peripheral.

use super::ccm::*;
use crate::info;

/// Enable UART clocks. 
pub fn enable_uart_clk(index: u32) {
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
            info!("invalid uart selection \n");
        }
    }
}