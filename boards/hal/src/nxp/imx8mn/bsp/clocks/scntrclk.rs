//! Enable or Disable system counter

use super::ccm::*;

/// Allow system counter i.e. ungate clock gate for SCTR.
pub fn enable_sctr() {
    clock_enable(CCGRIdx::CcgrSctr, true)
}