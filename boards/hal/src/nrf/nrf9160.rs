//! NVMC (i.e. flash) driver for the nrf52840 board, written in pure-rust.

use core::{
    ops::{Add, Sub},
    usize, cell::{OnceCell}, 
};

use nrf9160_pac as pac;

use crate::FlashInterface;
use pac::{Peripherals, NVMC_S, NVMC_NS};
use nrf9160_constants::*;
pub use pac::SPU_S as SPU;
pub const FLASH_REGION_SIZE: u32 = 32 * 1024;
pub const RAM_REGION_SIZE: u32 = 8 * 1024;
use cortex_m;

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger

#[rustfmt::skip]
mod nrf9160_constants {
    pub const FLASH_PAGE_SIZE : u32 = 4096;
    pub const STACK_LOW       : u32 = 0x20_000_000;
    pub const STACK_UP        : u32 = 0x20_040_000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x40000;
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 1;
}
// include!(concat!(env!("OUT_DIR"), "/trustzone_bindings.rs"));
// extern crate trustzone_m_nonsecure_rt;

pub struct FlashWriterEraser {
    pub nvmc_s: NVMC_S,
    pub nvmc_ns: NVMC_NS,
    pub secure: bool
}

impl FlashWriterEraser
{
    pub fn new(nvmc_s: NVMC_S, nvmc_ns:NVMC_NS, secure: bool) -> Self {
        FlashWriterEraser {
                nvmc_s: nvmc_s,
                nvmc_ns: nvmc_ns,
                secure: secure
            }
    }
}

impl FlashInterface for FlashWriterEraser {
    /// Write data at the specified address
    ///
    /// Arguments:
    /// -   address: It holds the address of flash where data has to be written
    /// -   data: u8 pointer holding the holding data.
    /// -   len :  number of bytes
    ///
    /// Return:
    /// -  NONE
    fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
        let address = address as u32;
        let len = len as u32;
        let mut idx = 0u32;
        let mut src = data as *mut u32;
        let mut dst = address as *mut u32;
        // defmt::println!("writting {:?} at address {:?} len {:?}", data.clone() ,address.clone(), len.clone() );

        while idx < len {
            let data_ptr = (data as *const u32) as u32;
            // defmt::println!("hal flash write 1");
            if self.secure {
                // Check if the following holds true and do a full word write i.e. 4-byte write
                // - if `len - idx` is greater than 3 (i.e. 4 bytes).
                // - if the address is aligned on a word (i.e. 4-byte) boundary.
                // - if the data_ptr is aligned on a word (i.e. 4-byte) boundary.
                if ((len - idx > 3)
                    && ((((address + idx) & 0x03) == 0) && ((data_ptr + idx) & 0x03) == 0))
                {
                    // // defmt::println!("hal flash write 2");
                    // Enable NVM writes
                    
                    self.nvmc_s.configns.write(|w| w.wen().wen());
                    // // defmt::println!("Config nvmc enabled");
                    while self.nvmc_s.readynext.read().readynext().is_busy() {
                        // // defmt::println!("waiting for busy bit");
                    }
                    // // defmt::println!("NVMC enabled");
                    unsafe {
                        *dst = *src; // 4-byte write
                    };
                    // // defmt::println!("hal flash write 3");
                    // Wait until writing is done
                    while self.nvmc_s.ready.read().ready().is_busy() {}
                    // // defmt::println!("writting to memory done ");
                    src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                    dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                    idx += 4;
                } else {
                    // // defmt::println!("hal flash write 4");
                    // else do a single byte write i.e. 1-byte write
                    let mut val = 0u32;
                    let val_bytes = ((&mut val) as *mut u32) as *mut u8;
                    let offset = (address + idx) - (((address + idx) >> 2) << 2); // offset from nearest word aligned address
                    dst = ((dst as u32) - offset) as *mut u32; // subtract offset from dst addr
                    unsafe {
                        val = *dst; // assign current val at dst to val
                                    // store data byte at idx to `val`. `val_bytes` is a byte-pointer to val.
                        *val_bytes.add(offset as usize) = *data.add(idx as usize);
                    }
                    // // defmt::println!("hal flash write 5");
                    // Enable NVM writes
                    self.nvmc_s.configns.write(|w| w.wen().wen());
                    while self.nvmc_s.readynext.read().readynext().is_busy() {}
                    // // defmt::println!("hal flash write 6");
                    unsafe {
                        *dst = val; // Technically this is a 1-byte write ONLY
                                    // but only full 32-bit words can be written to Flash using the NVMC interface
                    };
                    // Wait until writing is done
                    while self.nvmc_s.ready.read().ready().is_busy() {}
                    // defmt::println!("hal flash write 7");
                    src = ((src as u32) + 1) as *mut u32; // increment pointer by 1
                    dst = ((dst as u32) + 1) as *mut u32; // increment pointer by 1
                    idx += 1;
                    // defmt::println!("doing single byte write");
                }
            }
            else {
                // Check if the following holds true and do a full word write i.e. 4-byte write
                // - if `len - idx` is greater than 3 (i.e. 4 bytes).
                // - if the address is aligned on a word (i.e. 4-byte) boundary.
                // - if the data_ptr is aligned on a word (i.e. 4-byte) boundary.
                if ((len - idx > 3)
                    && ((((address + idx) & 0x03) == 0) && ((data_ptr + idx) & 0x03) == 0))
                {
                    // // defmt::println!("hal flash write 2");
                    // Enable NVM writes
                    
                    self.nvmc_ns.configns.write(|w| w.wen().wen());
                    // // defmt::println!("Config nvmc enabled");
                    while self.nvmc_ns.readynext.read().readynext().is_busy() {
                        // // defmt::println!("waiting for busy bit");
                    }
                    // // defmt::println!("NVMC enabled");
                    unsafe {
                        *dst = *src; // 4-byte write
                    };
                    // // defmt::println!("hal flash write 3");
                    // Wait until writing is done
                    while self.nvmc_ns.ready.read().ready().is_busy() {}
                    // // defmt::println!("writting to memory done ");
                    src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                    dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                    idx += 4;
                } else {
                    // // defmt::println!("hal flash write 4");
                    // else do a single byte write i.e. 1-byte write
                    let mut val = 0u32;
                    let val_bytes = ((&mut val) as *mut u32) as *mut u8;
                    let offset = (address + idx) - (((address + idx) >> 2) << 2); // offset from nearest word aligned address
                    dst = ((dst as u32) - offset) as *mut u32; // subtract offset from dst addr
                    unsafe {
                        val = *dst; // assign current val at dst to val
                                    // store data byte at idx to `val`. `val_bytes` is a byte-pointer to val.
                        *val_bytes.add(offset as usize) = *data.add(idx as usize);
                    }
                    // // defmt::println!("hal flash write 5");
                    // Enable NVM writes
                    self.nvmc_ns.configns.write(|w| w.wen().wen());
                    while self.nvmc_ns.readynext.read().readynext().is_busy() {}
                    // // defmt::println!("hal flash write 6");
                    unsafe {
                        *dst = val; // Technically this is a 1-byte write ONLY
                                    // but only full 32-bit words can be written to Flash using the NVMC interface
                    };
                    // Wait until writing is done
                    while self.nvmc_ns.ready.read().ready().is_busy() {}
                    // defmt::println!("hal flash write 7");
                    src = ((src as u32) + 1) as *mut u32; // increment pointer by 1
                    dst = ((dst as u32) + 1) as *mut u32; // increment pointer by 1
                    idx += 1;
                    // defmt::println!("doing single byte write");
                }
            }
        }
    }

    /// Erase the sector of a given address
    ///
    /// Arguments:
    /// -   addr: Address where data has to be erased
    /// -   len :  number of bytes to be erased
    ///
    /// Return:
    /// -  NONE
    fn hal_flash_erase(&self, addr: usize, len: usize) {
        let starting_page = (addr/0x1000) as u32;
        let ending_page = ((addr + len)/0x1000) as u32;

        // let address = starting_page * 0x1000;
        // defmt::info!("starting_page={}, ending_page={}, len={}", starting_page, ending_page, len);
        for start_addr in (starting_page..ending_page) {
            // Enable erasing
            if self.secure{
                // Enable the erase functionality of the flash
                self.nvmc_s.configns.modify(|_, w| w.wen().een());
                // Start the erase process by writing a u32 word containing all 1's to the first word of the page
                // This is safe because the flash slice is page aligned, so a pointer to the first byte is valid as a pointer to a u32.
                unsafe {
                    let first_word = (start_addr * 0x1000) as *mut u32;
                    first_word.write_volatile(0xFFFFFFFF);
                }
                // Wait for the erase to be done
                while self.nvmc_s.ready.read().ready().is_busy() {}

                self.nvmc_s.configns.modify(|_, w| w.wen().ren());
            }
            else {
                // Enable the erase functionality of the flash
                self.nvmc_ns.configns.modify(|_, w| w.wen().een());
                // Start the erase process by writing a u32 word containing all 1's to the first word of the page
                // This is safe because the flash slice is page aligned, so a pointer to the first byte is valid as a pointer to a u32.
                unsafe {
                    let first_word = (start_addr * 0x1000) as *mut u32;
                    first_word.write_volatile(0xFFFFFFFF);
                }
                // Wait for the erase to be done
                while self.nvmc_ns.ready.read().ready().is_busy() {}

                self.nvmc_ns.configns.modify(|_, w| w.wen().ren());
            }
        }
        // Synchronize the changes
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        
    }

    fn hal_init() {}
    fn hal_flash_lock(&self) {}
    fn hal_flash_unlock(&self) {}
}

pub fn preboot() {}

struct RefinedUsize<const MIN: u32, const MAX: u32, const VAL: u32>(u32);

impl<const MIN: u32, const MAX: u32, const VAL: u32> RefinedUsize<MIN, MAX, VAL> {
    /// This method is used to check the address bound of stack pointer
    ///
    /// Method arguments:
    /// -   i : starting address of stack  
    /// Returns:
    /// -  It returns u32 address of stack pointer
    pub fn bounded_int(i: u32) -> Self {
        assert!(i >= MIN && i <= MAX);
        RefinedUsize(i)
    }
    /// This method is used to check the address of reset pointer
    ///
    /// Method arguments:
    /// -   i : starting address of reset  
    /// Returns:
    /// -  It returns u32 address of reset pointer
    pub fn single_valued_int(i: u32) -> Self {
        // defmt::println!("i {:?} == VAL {:?}", i, VAL);
        assert!(i == VAL);
        RefinedUsize(i)
    }
}

/// This method is used to boot the firmware from a particular address
///
/// Method arguments:
/// -   fw_base_address  : address of the firmware
/// Returns:
/// -  NONE
#[rustfmt::skip]
pub fn boot_from(fw_base_address: usize) -> ! {
    unsafe {
        let ns_vector_table_addr = fw_base_address as u32;
        // Write the Non-Secure Main Stack Pointer before switching state. Its value is the first
        // entry of the Non Secure Vector Table.
        cortex_m::register::msp::write_ns(*(ns_vector_table_addr as *const u32));
        // Create a Non-Secure function pointer to the address of the second entry of the Non
        // Secure Vector Table.
        let ns_reset_vector: extern "C-cmse-nonsecure-call" fn() -> ! =
            core::mem::transmute::<u32, _>(ns_vector_table_addr + 4);
        ns_reset_vector()
    }
}

/// This method is used to initialize the trustzone memory region, Peripheral, Pins, 
/// DPPI to Secure/Non-Secure/Non-SecureCallable accordingly. The method checks the provided inpupts range 
/// with the one provided in memory.x file and accordingly initializes the trustzone. 
/// The method uses SPM register interface to initialize the trustZone. 
///
/// Method arguments:
/// -   Non-Secure Peripherals  : Array of peripherals which are to initialized as Non-secure
/// -   Non-Secure Pins  : Array of pins which are to initialized as Non-secure
/// -   Non-Secure DPPI  : Array of Dppi which are to initialized as Non-secure
/// Returns:
/// -  NONE
pub fn initialize<const PERIPHERALS_LEN: usize, const PINS_LEN: usize, const DPPI_LEN: usize>(
    nonsecure_peripherals: [NonSecurePeripheral; PERIPHERALS_LEN],
    nonsecure_pins: [(usize, u32); PINS_LEN],
    nonsecure_dppi: [(usize, u32); DPPI_LEN],
) {
    extern "C" {
        static _s_flash_start: u32;
        static _s_flash_end: u32;

        static _nsc_flash_start: u32;
        static _nsc_flash_end: u32;

        static _ns_flash_start: u32;
        static _ns_flash_end: u32;

        static _s_ram_start: u32;
        static _s_ram_end: u32;

        static _ns_ram_start: u32;
        static _ns_ram_end: u32;
    }
    
    let s_flash_start = unsafe { core::ptr::addr_of!(_s_flash_start) as u32 };
    let s_flash_end = unsafe { core::ptr::addr_of!(_s_flash_end) as u32 };
    let s_flash = s_flash_start..s_flash_end;

    let nsc_flash_start = unsafe { core::ptr::addr_of!(_nsc_flash_start) as u32 };
    let nsc_flash_end = unsafe { core::ptr::addr_of!(_nsc_flash_end) as u32 };
    let nsc_flash = nsc_flash_start..nsc_flash_end;
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(s_flash_start % FLASH_REGION_SIZE, 0, "The start of the flash region must be on a region boundary: val % {FLASH_REGION_SIZE:#X} must be 0");
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(nsc_flash_end % FLASH_REGION_SIZE, 0, "The end of the nsc_flash region must be on a region boundary: val % {FLASH_REGION_SIZE:#X} must be 0");
    // defmt::println!("in init function");

    let ns_flash_start = unsafe { core::ptr::addr_of!(_ns_flash_start) as u32 };
    let ns_flash_end = unsafe { core::ptr::addr_of!(_ns_flash_end) as u32 };
    // defmt::println!("ns_flash_start: {:?}", ns_flash_start);
    // defmt::println!("ns_flash_end: {:?}", ns_flash_end);
    let ns_flash = ns_flash_start..ns_flash_end;
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(ns_flash_start % FLASH_REGION_SIZE, 0, "The start of the ns flash region must be on a region boundary: val % {FLASH_REGION_SIZE:#X} must be 0");
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(ns_flash_end % FLASH_REGION_SIZE, 0, "The end of the ns flash region must be on a region boundary: val % {FLASH_REGION_SIZE:#X} must be 0");
    
    let s_ram_start = unsafe { core::ptr::addr_of!(_s_ram_start) as u32 };
    let s_ram_end = unsafe { core::ptr::addr_of!(_s_ram_end) as u32 }; 
    let s_ram = s_ram_start..s_ram_end;
    
    // #[cfg(feature = "memory_region_assertions")]
    // assert_eq!(s_ram_start % RAM_REGION_SIZE, 0, "The start of the ram region must be on a region boundary: val % {RAM_REGION_SIZE:#X} must be 0");
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(s_ram_end % RAM_REGION_SIZE, 0, "The end of the ram region must be on a region boundary: val % {RAM_REGION_SIZE:#X} must be 0");
    
    let ns_ram_start = unsafe { core::ptr::addr_of!(_ns_ram_start) as u32 };
    let ns_ram_end = unsafe { core::ptr::addr_of!(_ns_ram_end) as u32 }; 
    let ns_ram = ns_ram_start..ns_ram_end;
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(ns_ram_start % RAM_REGION_SIZE, 0, "The start of the ns ram region must be on a region boundary: val % {RAM_REGION_SIZE:#X} must be 0");
    #[cfg(feature = "memory_region_assertions")]
    assert_eq!(ns_ram_end % RAM_REGION_SIZE, 0, "The end of the ns ram region must be on a region boundary: val % {RAM_REGION_SIZE:#X} must be 0");
    
    let spu = unsafe { core::mem::transmute::<_, SPU>(()) };
    // // defmt::println!("in init function 5");
    for (address, region) in spu
        .flashregion
        .iter()
        .enumerate()
        .map(|(index, region)| (index as u32 * FLASH_REGION_SIZE, region))
    {
        if s_flash.contains(&address) || nsc_flash.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .secure()
            });
        }
        else if ns_flash.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .non_secure()
            });
        }
    }
    
    set_nsc_region(&spu, nsc_flash_start..nsc_flash_end);

    for (address, region) in spu
        .ramregion
        .iter()
        .enumerate()
        .map(|(index, region)| (0x20000000 + index as u32 * RAM_REGION_SIZE, region))
    {
        if s_ram.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .secure()
            });
        }
        else if ns_ram.contains(&address) {
            region.perm.write(|w| {
                w.execute()
                    .enable()
                    .read()
                    .enable()
                    .write()
                    .enable()
                    .secattr()
                    .non_secure()
            });
        }
    }
    // defmt::println!("ns ram initialized");
    // Set all given peripherals to nonsecure
    for peripheral in nonsecure_peripherals {
        spu.periphid[peripheral.id]
            .perm
            .write(|w| w.secattr().non_secure().dmasec().non_secure());
    }

    // Set all given pins to nonsecure
    for (pin_port, pin) in nonsecure_pins {
        spu.gpioport[pin_port]
            .perm
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << pin)) })
    }

    // Set all given dppi channels to nonsecure
    for (port, channel) in nonsecure_dppi {
        spu.dppi[port]
            .perm
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << channel)) })
    }
    // We're using Nordic's SPU instead of the default SAU. To do that we must disable the SAU and
    // set the ALLNS (All Non-secure) bit.
    let sau = unsafe { core::mem::transmute::<_, cortex_m::peripheral::SAU>(()) };
    unsafe {
        sau.ctrl.modify(|mut ctrl| {
            ctrl.0 = 0b10;
            ctrl
        });

        // Also set the stack pointer of nonsecure
        cortex_m::register::msp::write_ns(ns_ram_end);
    }
}

/// This method is used to set the Non-SecureCallable region of memory.
///
/// Method arguments:
/// -   &SPU  : Reference of Secure partition unit.
/// -   region : Memory region which is to be set ans Non-SecureCallable
/// Returns:
/// -  NONE
fn set_nsc_region(spu: &SPU, region: core::ops::Range<u32>) {
    let sg_start = region.start;
    let nsc_size = FLASH_REGION_SIZE - (sg_start % FLASH_REGION_SIZE);
    let size_reg = (31 - nsc_size.leading_zeros()) - 4;
    let region_reg = (sg_start as u32 / FLASH_REGION_SIZE) & 0x3F; // x << SPU_FLASHNSC_REGION_REGION_Pos & SPU_FLASHNSC_REGION_REGION_Msk
    spu.flashnsc[0].size.write(|w| {
        unsafe {
            w.bits(size_reg);
        }
        w
    });
    spu.flashnsc[0].region.write(|w| {
        unsafe {
            w.bits(region_reg);
        }
        w
    });
}

pub struct NonSecurePeripheral {
    id: usize,
}

macro_rules! impl_ns_peripheral {
    ($peripheral:ty, $id:expr) => {
        impl From<$peripheral> for NonSecurePeripheral {
            fn from(_: $peripheral) -> Self {
                Self { id: $id }
            }
        }
    };
}

#[cfg(feature = "nrf9160")]
mod nrf9160_peripheral_impl {
    use super::*;

    impl_ns_peripheral!(nrf9160_pac::REGULATORS_S, 4);
    impl_ns_peripheral!((nrf9160_pac::CLOCK_S, nrf9160_pac::POWER_S), 5);
    impl_ns_peripheral!(
        (
            nrf9160_pac::SPIM0_S,
            nrf9160_pac::SPIS0_S,
            nrf9160_pac::TWIM0_S,
            nrf9160_pac::TWIS0_S,
            nrf9160_pac::UARTE0_S
        ),
        8
    );
    impl_ns_peripheral!(
        (
            nrf9160_pac::SPIM1_S,
            nrf9160_pac::SPIS1_S,
            nrf9160_pac::TWIM1_S,
            nrf9160_pac::TWIS1_S,
            nrf9160_pac::UARTE1_S
        ),
        9
    );
    impl_ns_peripheral!(
        (
            nrf9160_pac::SPIM2_S,
            nrf9160_pac::SPIS2_S,
            nrf9160_pac::TWIM2_S,
            nrf9160_pac::TWIS2_S,
            nrf9160_pac::UARTE2_S
        ),
        10
    );
    impl_ns_peripheral!(
        (
            nrf9160_pac::SPIM3_S,
            nrf9160_pac::SPIS3_S,
            nrf9160_pac::TWIM3_S,
            nrf9160_pac::TWIS3_S,
            nrf9160_pac::UARTE3_S
        ),
        11
    );
    impl_ns_peripheral!(nrf9160_pac::SAADC_S, 14);
    impl_ns_peripheral!(nrf9160_pac::TIMER0_S, 15);
    impl_ns_peripheral!(nrf9160_pac::TIMER1_S, 16);
    impl_ns_peripheral!(nrf9160_pac::TIMER2_S, 17);
    impl_ns_peripheral!(nrf9160_pac::RTC0_S, 20);
    impl_ns_peripheral!(nrf9160_pac::RTC1_S, 21);
    impl_ns_peripheral!(&nrf9160_pac::DPPIC_S, 23);
    impl_ns_peripheral!(nrf9160_pac::WDT_S, 24);
    impl_ns_peripheral!(nrf9160_pac::EGU0_S, 27);
    impl_ns_peripheral!(nrf9160_pac::EGU1_S, 28);
    impl_ns_peripheral!(nrf9160_pac::EGU2_S, 29);
    impl_ns_peripheral!(nrf9160_pac::EGU3_S, 30);
    impl_ns_peripheral!(nrf9160_pac::EGU4_S, 31);
    impl_ns_peripheral!(nrf9160_pac::EGU5_S, 32);
    impl_ns_peripheral!(nrf9160_pac::PWM0_S, 34);
    impl_ns_peripheral!(nrf9160_pac::PWM1_S, 35);
    impl_ns_peripheral!(nrf9160_pac::PWM2_S, 36);
    impl_ns_peripheral!(nrf9160_pac::PDM_S, 38);
    impl_ns_peripheral!(nrf9160_pac::I2S_S, 40);
    impl_ns_peripheral!(nrf9160_pac::IPC_S, 42);
    impl_ns_peripheral!(nrf9160_pac::FPU_S, 44);
    impl_ns_peripheral!((&nrf9160_pac::KMU_S, 
                         &nrf9160_pac::NVMC_S), 57);
    impl_ns_peripheral!(nrf9160_pac::VMC_S, 58);
    impl_ns_peripheral!(&nrf9160_pac::P0_S, 66);
}

