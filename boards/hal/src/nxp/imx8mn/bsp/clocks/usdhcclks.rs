//! The i.MX8 has 3 uSDHC(s). Select root clock SYSTEM_PLL1_DIV2_CLK i.e. 800Mhz/2 for uSDHC2  

use super::ccm::*;
use crate::info;

/// Enable uSDHC clocks.
pub fn enable_usdhc_clk(index: u32) {
    match index {
        1 => {
            clock_enable(CCGRIdx::CcgrUsdhc1, false);
            clock_set_target_val(
                ClkRootIdx::Usdhc1ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(1),
            );
            clock_enable(CCGRIdx::CcgrUsdhc1, true);
        }
        2 => {
            clock_enable(CCGRIdx::CcgrUsdhc2, false);
            clock_set_target_val(
                ClkRootIdx::Usdhc2ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(1),
            );
            clock_enable(CCGRIdx::CcgrUsdhc2, true);
        }
        3 => {
            clock_enable(CCGRIdx::CcgrUsdhc3, false);
            clock_set_target_val(
                ClkRootIdx::Usdhc3ClkRoot,
                CLK_ROOT_ON | clk_root_source_sel(1),
            );
            clock_enable(CCGRIdx::CcgrUsdhc3, true);
        }
        _ => {
            info!("invalid uSDHC selection \n");
        }
    }
}
