//! NVMC (i.e. flash) driver for the nrf52840 board, written in pure-rust.

use core::{
    ops::{Add, Sub},
    usize,
};

use nrf52840_hal as hal;

use crate::FlashInterface;
use hal::pac::{Peripherals, NVMC};
use nrf52840_constants::*;

#[rustfmt::skip]
mod nrf52840_constants {
    pub const FLASH_PAGE_SIZE : u32 = 4096;
    pub const STACK_LOW       : u32 = 0x20_000_000;
    pub const STACK_UP        : u32 = 0x20_040_000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x2f000;
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 1;
}

pub struct FlashWriterEraser {
    pub nvmc: NVMC,
}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {
            nvmc: Peripherals::take().unwrap().NVMC,
        }
    }
}

impl FlashInterface for FlashWriterEraser {
    fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
        let address = address as u32;
        let len = len as u32;

        let mut idx = 0u32;
        let mut src = data as *mut u32;
        let mut dst = address as *mut u32;

        while idx < len {
            let data_ptr = (data as *const u32) as u32;
            // Check if the following holds true and do a full word write i.e. 4-byte write
            // - if `len - idx` is greater than 3 (i.e. 4 bytes).
            // - if the address is aligned on a word (i.e. 4-byte) boundary.
            // - if the data_ptr is aligned on a word (i.e. 4-byte) boundary.
            if ((len - idx > 3)
                && ((((address + idx) & 0x03) == 0) && ((data_ptr + idx) & 0x03) == 0))
            {
                // Enable NVM writes
                self.nvmc.config.write(|w| w.wen().wen());
                while self.nvmc.readynext.read().readynext().is_busy() {}
                unsafe {
                    *dst = *src; // 4-byte write
                };
                // Wait until writing is done
                while self.nvmc.ready.read().ready().is_busy() {}
                src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                idx += 4;
            } else {
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

                // Enable NVM writes
                self.nvmc.config.write(|w| w.wen().wen());
                while self.nvmc.readynext.read().readynext().is_busy() {}
                unsafe {
                    *dst = val; // Technically this is a 1-byte write ONLY
                                // but only full 32-bit words can be written to Flash using the NVMC interface
                };
                // Wait until writing is done
                while self.nvmc.ready.read().ready().is_busy() {}
                src = ((src as u32) + 1) as *mut u32; // increment pointer by 1
                dst = ((dst as u32) + 1) as *mut u32; // increment pointer by 1
                idx += 1;
            }
        }
    }

    fn hal_flash_erase(&self, addr: usize, len: usize) {
        let starting_page = addr as u32;
        let ending_page = (addr + len) as u32;
        // defmt::info!("starting_page={}, ending_page={}, len={}", starting_page, ending_page, len);
        for addr in (starting_page..ending_page).step_by(FLASH_PAGE_SIZE as usize) {
            // Enable erasing
            self.nvmc.config.write(|w| w.wen().een());
            // Wait until writing is done
            while self.nvmc.readynext.read().readynext().is_busy() {}
            // Erase page starting at addr
            self.nvmc
                .erasepage()
                .write(|w| unsafe { w.erasepage().bits(addr) });
            // Wait until erasing is done
            while self.nvmc.ready.read().ready().is_busy() {}
        }
    }

    fn hal_init() {}
    fn hal_flash_lock(&self) {}
    fn hal_flash_unlock(&self) {}
}

pub fn preboot() {}

struct RefinedUsize<const MIN: u32, const MAX: u32, const VAL: u32>(u32);

impl<const MIN: u32, const MAX: u32, const VAL: u32> RefinedUsize<MIN, MAX, VAL> {
    pub fn bounded_int(i: u32) -> Self {
        assert!(i >= MIN && i <= MAX);
        RefinedUsize(i)
    }
    pub fn single_valued_int(i: u32) -> Self {
        assert!(i == VAL);
        RefinedUsize(i)
    }
}

#[rustfmt::skip]
pub fn boot_from(fw_base_address: usize) -> ! {
    let mut core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let mut scb = core_peripherals.SCB;
    unsafe {
        let base_img_addr = fw_base_address as u32;
        let stack_pointer = RefinedUsize::<STACK_LOW, STACK_UP, 0>::bounded_int(
            *(fw_base_address as *const u32)).0;
        let reset_vector = RefinedUsize::<0, 0, FW_RESET_VTR>::single_valued_int(
            *((fw_base_address + 4) as *const u32)).0;
        let jump_vector = core::mem::transmute::<usize, extern "C" fn() -> !>(reset_vector as usize);

        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        scb.vtor.write(base_img_addr);
        cortex_m::register::msp::write(stack_pointer);
        jump_vector()
    }
}
