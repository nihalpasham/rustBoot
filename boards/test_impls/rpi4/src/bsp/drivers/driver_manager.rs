// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use super::common::interface::{DeviceDriver, DriverManager};
use crate::bsp::global::{GPIO, PL011_UART};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Device Driver Manager type.
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 2],
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&GPIO, &PL011_UART],
};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the driver manager.
pub fn driver_manager() -> &'static impl DriverManager {
    &BSP_DRIVER_MANAGER
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }

    fn post_device_driver_init(&self) {
        // Configure PL011Uart's output pins.
        GPIO.map_pl011_uart();
    }
}