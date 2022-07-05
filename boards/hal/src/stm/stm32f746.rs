//! Flash  Read, Write and Erase opration for `stm32f746zg`.

use stm32f7xx_hal as hal;

use crate::FlashInterface;
use core::ptr::write_volatile;
use core::slice::from_raw_parts;

use hal::pac::{Peripherals, FLASH};
use stm32f746rc_constants::*;

#[rustfmt::skip]
mod stm32f746rc_constants {
    pub const FLASH_PAGE_SIZE : u32 = 0x40000;   // 1 sector size = 256KB   
    pub const STACK_LOW       : u32 = 0x2000_0000;
    pub const STACK_UP        : u32 = 0x2002_0000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x08040000;   //  sector 5 starting address
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 0xC9;
    pub const UNLOCKKEY1      : u32 = 0x45670123;
    pub const UNLOCKKEY2      : u32 = 0xCDEF89AB;
}

/// Constrained FLASH peripheral
pub struct FlashWriterEraser {
    pub nvm: FLASH,
}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {
            nvm: Peripherals::take().unwrap().FLASH,
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
        let mut data1 = unsafe { from_raw_parts((data as *mut u8), len) };

        // Ensure no effective write, erase or option byte change operation is ongoing
        while self.nvm.sr.read().bsy().bit() {}

        // Unlock the FLASH_CR register.
        self.hal_flash_unlock();

        let addr = address as *mut u32;

        // Set parallelism to write in 8 bit chunks, and enable programming.
        self.nvm
            .cr
            .write(|w| w.lock().unlocked().psize().psize8().pg().program());

        for idx in 0..(len + 1) {
            if idx == len {
                let mut offset = idx - 1;

                let word: u8 = (data1[offset]);

                let write_address = ((address as u32) + offset as u32) as *mut u8;

                unsafe { core::ptr::write_volatile(write_address, word) };
                cortex_m::asm::delay(4);
            } else {
                let offset = idx;

                let word: u8 = (data1[offset]);

                let write_address = ((address as u32) + offset as u32) as *mut u8;

                unsafe { core::ptr::write_volatile(write_address, word) };
                cortex_m::asm::dmb();
                cortex_m::asm::delay(4);
            }

            let sr = self.nvm.sr.read();
        }
        // Cleanup by clearing the PG bit
        self.nvm.cr.modify(|_, w| w.pg().clear_bit());
        // Lock the FLASH_CR register
        self.hal_flash_lock();
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
        let mut sec: u8 = 0;
        let mut flag: bool = true;
        let address = addr as u32;
        match address {
            (0x0800_0000..=0x0800_7FFF) => sec = 0,
            (0x0800_8000..=0x0800_FFFF) => sec = 1,
            (0x0801_0000..=0x0801_7FFF) => sec = 2,
            (0x0801_8000..=0x0801_FFFF) => sec = 3,
            (0x0802_0000..=0x0803_FFFF) => sec = 4,
            (0x0804_0000..=0x0807_FFFF) => sec = 5,
            (0x0808_0000..=0x080B_FFFF) => sec = 6,
            (0x080C_0000..=0x080F_FFFF) => sec = 7,
            _ => flag = false,
        }

        if flag {
            self.hal_flash_unlock();

            cortex_m::asm::delay(8000000);
            #[rustfmt::skip]
            self.nvm.cr.modify(|_, w| unsafe {
                w
                    // start
                    .strt().set_bit()
                    .psize().psize8()
                    // sector number
                    .snb().bits(sec)
                    // sectore erase
                    .ser().set_bit()
                    // no programming
                    .pg().clear_bit()
            });

            self.nvm.cr.modify(|_, w| w.strt().start());
            cortex_m::asm::delay(8000000);
            // Wait until erasing is done
            while self.nvm.sr.read().bsy().bit_is_set() {}
            let sr = self.nvm.sr.read();
            if sr.wrperr().bit_is_set() {
                self.nvm.sr.modify(|_, w| w.wrperr().clear_bit());
            }
            self.nvm.cr.modify(|_, w| w.ser().clear_bit());
            //Lock the FLASH
            self.hal_flash_lock();
        }
    }

    /// Locks the flash memory.
    ///
    /// Once the flash is locked no operation on flash can be perfomed.
    ///
    /// Arguments:
    /// -  NONE
    ///
    /// Return:
    /// -  NONE
    fn hal_flash_lock(&self) {
        self.nvm.cr.modify(|_, w| w.lock().set_bit());
    }

    /// Unlocks the flash memory.
    ///
    /// Flash has to be unlocked to do any operation on it.
    ///
    /// Arguments:
    /// -   NONE
    ///
    /// Return:
    /// -  NONE
    fn hal_flash_unlock(&self) {
        self.nvm.keyr.write(|w| unsafe { w.key().bits(UNLOCKKEY1) });
        self.nvm.keyr.write(|w| unsafe { w.key().bits(UNLOCKKEY2) });
    }
    fn hal_init() {}
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
       let address = fw_base_address as u32;
       let scb = hal::pac::SCB::ptr();
       unsafe {
       let sp = RefinedUsize::<STACK_LOW, STACK_UP, 0>::bounded_int(
        *(fw_base_address as *const u32)).0;
       let rv = RefinedUsize::<0, 0, FW_RESET_VTR>::single_valued_int(
        *((fw_base_address + 4) as *const u32)).0;
       let jump_vector = core::mem::transmute::<usize, extern "C" fn() -> !>(rv as usize);
       (*scb).vtor.write(address);
       cortex_m::register::msp::write(sp);
       jump_vector();
    
       }
       loop{}
}
