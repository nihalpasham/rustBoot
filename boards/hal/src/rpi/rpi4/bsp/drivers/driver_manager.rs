// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP driver support.

use super::common::interface::{DeviceDriver, DriverManager};
use crate::info;
use crate::rpi::rpi4::bsp::global::{EMMC_CONT, GPIO, PL011_UART};

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
        // initialize EMMC controller (i.e. sd card driver).
        // Note: emmc HW is to be initialized only after we fully initialize the uart instance,
        // we'll need the ability to `print` debug/error info prior to emmc initialization.
        match &EMMC_CONT.emmc_init_card() {
            &super::emmc::SdResult::EMMC_OK => {
                info!("EMMC2 driver initialized...\n")
            }
            _ => {
                info!("failed to initialize EMMC2...\n")
            }
        }
    }
}
