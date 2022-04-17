// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSP Memory Map.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// The board's physical memory map.
#[rustfmt::skip]
pub mod map {
    pub const END_INCLUSIVE: usize = 0xFFFF_FFFF;

    pub const GPIO_OFFSET:   usize = 0x0020_0000;
    pub const UART_OFFSET:   usize = 0x0020_1000;
    pub const EMMC_OFFSET:   usize = 0x0034_0000;

    pub mod mmio {
        use super::*;

        pub const START:            usize =         0xFE00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
        pub const EMMC_START:       usize = START + EMMC_OFFSET;
        pub const END_INCLUSIVE:    usize =         0xFF84_FFFF;
        
    }
}


