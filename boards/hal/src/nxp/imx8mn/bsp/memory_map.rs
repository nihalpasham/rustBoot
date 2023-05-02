//! BSP Memory Map.

/// The board's physical memory map.
#[rustfmt::skip]
pub mod map {

    pub const GPIO1_OFFSET  :   usize = 0x0020_0000;
    pub const GPIO2_OFFSET  :   usize = 0x0021_0000;
    pub const CCM_OFFSET    :   usize = 0x0038_0000;
    pub const UART_OFFSET   :   usize = 0x0089_0000;
    pub const USDHC2_OFFSET :   usize = 0x00B5_0000;
    pub const SYSCNT_OFFSET :   usize = 0x006C_0000;
    pub const IOMUXC_OFFSET :   usize = 0x0033_0000;
    pub const ANALOG_OFFSET :   usize = 0x0036_0000;

    pub mod mmio {
        use super::*;

        pub const START:            usize =         0x3000_0000;
        pub const GPIO2_START:      usize = START + GPIO2_OFFSET;
        pub const GPIO1_START:      usize = START + GPIO1_OFFSET;
        pub const CCM_START:        usize = START + CCM_OFFSET;
        pub const UART_START:       usize = START + UART_OFFSET;
        pub const USDHC2_START:     usize = START + USDHC2_OFFSET;
        pub const SYSCNT_START:     usize = START + SYSCNT_OFFSET;
        pub const IOMUXC_START:     usize = START + IOMUXC_OFFSET;
        pub const CCM_ANALOG:       usize = START + ANALOG_OFFSET;
        pub const END_INCLUSIVE:    usize =         0x30FF_FFFF;
        
    }
}
