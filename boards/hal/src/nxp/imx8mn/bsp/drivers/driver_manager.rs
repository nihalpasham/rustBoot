//! BSP driver support.

use super::common::interface::{DeviceDriver, DriverManager};
use crate::info;
use crate::nxp::imx8mn::bsp::global::{UART, CNTR};

/// Device Driver Manager type.
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 1],
}

// Global instances

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&UART],
};

/// Return a reference to the driver manager.
pub fn driver_manager() -> &'static impl DriverManager {
    &BSP_DRIVER_MANAGER
}

/// Turn on system counter.
pub fn start_system_counter() {
    &CNTR.start_counter();
}

impl DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }
}
