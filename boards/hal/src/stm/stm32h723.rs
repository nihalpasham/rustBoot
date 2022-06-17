//! Flash  Read, Write and Erase opration for `stm32h723zg`.

use core::convert::TryInto;
use core::slice::from_raw_parts;
use core::{ops::Add, ptr::write_volatile};

use hal::{pac, pac::FLASH};
use stm32h7xx_hal as hal;

use crate::FlashInterface;
use stm32h723zg_constants::*;

#[rustfmt::skip]
mod stm32h723zg_constants {
    pub const FLASH_SECTOR_SIZE         : u32 = 0x20000;
    // using bigger PARTITION_SIZE, since the last 128KB on each partition will be reserved for bootloader flags or states
    pub const PARTITION_SIZE            : u32 = 0x40000;
    pub const PARTITION_BOOT_ADDRESS    : u32 = 0x0802_0000;
    pub const PARTITION_UPDATE_ADDRESS  : u32 = 0x0806_0000;
    pub const STM32H7_PART_BOOT_END     : u32 = PARTITION_BOOT_ADDRESS + PARTITION_SIZE;
    pub const STM32H7_PART_UPDATE_END   : u32 = PARTITION_UPDATE_ADDRESS + PARTITION_SIZE;
    pub const STM32H7_PART_BOOT_FLAGS_PAGE_ADDRESS : u32 = ((STM32H7_PART_BOOT_END - 1) / FLASH_SECTOR_SIZE) * FLASH_SECTOR_SIZE;
    pub const STM32H7_PART_UPDATE_FLAGS_PAGE_ADDRESS : u32 = ((STM32H7_PART_UPDATE_END - 1) / FLASH_SECTOR_SIZE) * FLASH_SECTOR_SIZE;
    pub const FLASHMEM_ADDRESS_SPACE    : u32 = 0x08000000;
    pub const STACK_LOW       : u32 = 0x20_000_000;
    pub const STACK_UP        : u32 = 0x20_040_000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x08020000;
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 0x19D;

    pub const UNLOCKKEY1  : u32 = 0x45670123;
    pub const UNLOCKKEY2  : u32 = 0xCDEF89AB;
    pub const PSIZE_X8    : u8 = 0b00;
    pub const PSIZE_X16   : u8 = 0b01;
    pub const PSIZE_X32   : u8 = 0b10;
    pub const PSIZE_X64   : u8  = 0b11;
    pub const KB          : u32 = 1024;
}

/// Constrained FLASH peripheral
pub struct FlashWriterEraser {
    pub nvm: FLASH,
}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {
            nvm: pac::Peripherals::take().unwrap().FLASH,
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
    fn hal_flash_write(&self, addr: usize, data: *const u8, len: usize) {
        let mut i = 0u32;
        let mut ii = 0u32;

        let mut src = data as *mut u32;
        let mut data1 = unsafe { from_raw_parts(data, len) };
        let mut dst = addr as *mut u32;
        let mut stm32h7_cache = [0u32; 32];
        let vbytes = (&mut stm32h7_cache[0] as *mut u32) as *mut u8;
        let off = ((addr as u32) + i) - ((((addr as u32) + i) >> 5) << 5);
        let base_address = ((addr as u32) + i) & (!0x1F);

        while i < len as u32 {
            if (len > 32)
            // && (((((addr as u32) + i) & 0x1F) == 0) && (((data as u32) + i) & 0x1F) == 0))
            {
                // Ensure no effective write, erase or option byte change operation is ongoing
                while self.nvm.bank1().sr.read().bsy().bit_is_set() {}

                // Unlock the FLASH_CR register.
                self.hal_flash_unlock();

                // Flash clear errors
                self.nvm.bank1().sr.modify(|_, w| w.wrperr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.pgserr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.strberr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.incerr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.operr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.rdperr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.rdserr().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.sneccerr1().clear_bit());
                self.nvm.bank1().sr.modify(|_, w| w.dbeccerr().clear_bit());

                // Enable write operations by setting the PG bit in the FLASH_CR register.
                self.nvm.bank1().cr.modify(|_, w| unsafe {
                    w.psize().bits(PSIZE_X8).pg().set_bit().fw().set_bit()
                });

                for ii in 0..8 {
                    unsafe {
                        *dst = *src;
                        src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                        dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                    }
                }
                i += 32;
            } else {
                let mut off = ((addr as u32) + i) - ((((addr as u32) + i) >> 5) << 5);
                let base_address = ((addr as u32) + i) & (!0x1F);
                dst = base_address as *mut u32;

                // Unlock the FLASH_CR register.
                self.hal_flash_unlock();

                // Flash clear errors
                self.nvm.bank1().sr.modify(|_, w| w.pgserr().clear_bit());

                // Enable write operations by setting the PG1 bit in the FLASH_CR1/2 register.
                self.nvm
                    .bank1()
                    .cr
                    .modify(|_, w| unsafe { w.pg().set_bit().fw().set_bit() });

                for ii in 0..8 {
                    unsafe {
                        stm32h7_cache[ii] = *dst;
                    }
                    dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                }

                // Checks if flags page.
                // STM32H7: Due to ECC functionality, it is not possible to write partition/sector
                // flags and signature more than once. This flags_cache is used to intercept write operations and
                // ensures that the sector is always erased before each write.
                if stm32h7_boot_flag_page(addr as u32) {
                    self.hal_flash_lock();
                    self.hal_flash_erase((STM32H7_PART_BOOT_FLAGS_PAGE_ADDRESS as usize), 1);
                    self.hal_flash_unlock();
                } else if stm32h7_update_flag_page(addr as u32) {
                    self.hal_flash_lock();
                    self.hal_flash_erase((STM32H7_PART_UPDATE_FLAGS_PAGE_ADDRESS as usize), 1);
                    self.hal_flash_unlock();
                }

                while (off < 32) && (i < len as u32) {
                    unsafe {
                        let x = (i as usize);
                        *vbytes.add((off).try_into().unwrap()) = data1[x];
                        off = off + 1;
                        i = i + 1;
                    }
                }

                cortex_m::asm::isb();
                cortex_m::asm::dsb();

                dst = base_address as *mut u32;

                for ii in 0..8 {
                    unsafe {
                        *dst = stm32h7_cache[ii];
                        dst = ((dst as u32) + 4) as *mut u32;
                    }
                }

                cortex_m::asm::isb();
                cortex_m::asm::dsb();
                i += 1;
            }

            // Check that QW1 (respectively QW2) has been raised and wait until it is reset to 0.
            while self.nvm.bank1().sr.read().qw().bit_is_set() {}
            // Additionally wait for the busy flag to clear
            while self.nvm.bank1().sr.read().bsy().bit_is_set() {}

            // Check that EOP flag is set in the FLASH_SR register (meaning that the programming
            // operation has succeed), and clear it by software.
            if self.nvm.bank1().sr.read().eop().bit_is_set() {
                self.nvm.bank1().sr.modify(|_, w| w.eop().set_bit()); // Clear
            }

            // Cleanup by clearing the PG bit
            self.nvm.bank1().cr.modify(|_, w| w.pg().clear_bit());

            // Lock the FLASH_CR register
            self.hal_flash_lock();
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
        let mut sec: u8 = 0;
        let mut flag: bool = true;
        let address = addr as u32;

        match address {
            (0x0800_0000..=0x0801_FFFF) => sec = 0,
            (0x0802_0000..=0x0803_FFFF) => sec = 1,
            (0x0804_0000..=0x0805_FFFF) => sec = 2,
            (0x0806_0000..=0x0807_FFFF) => sec = 3,
            (0x0808_0000..=0x0809_FFFF) => sec = 4,
            (0x080A_0000..=0x080B_FFFF) => sec = 5,
            (0x080C_0000..=0x080D_FFFF) => sec = 6,
            (0x080E_0000..=0x080F_FFFF) => sec = 7,
            _ => flag = false,
        }

        if flag {
            while self.nvm.bank1().sr.read().bsy().bit_is_set() {}

            //Lock the FLASH_CR register
            self.hal_flash_unlock();

            // Erase page starting at addr
            #[rustfmt::skip]
            self.nvm.bank1().cr.modify(|_, w| unsafe {
                w
                    .psize().bits(PSIZE_X32)
                    // sector number
                    .snb().bits(sec)
            });
            self.nvm.bank1().cr.modify(|_, w| w.ser().set_bit());

            // Set the START bit in the FLASH_CR register.
            self.nvm.bank1().cr.modify(|_, w| w.start().bit(true));

            while self.nvm.bank1().sr.read().qw().bit() {}

            // Wait until erasing is done
            while self.nvm.bank1().sr.read().bsy().bit() {}

            //Unlock the FLASH_CR register
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
        self.nvm.bank1().cr.modify(|_, w| w.lock().set_bit());
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
        const FLASH_KEY1: u32 = 0x4567_0123;
        const FLASH_KEY2: u32 = 0xCDEF_89AB;

        self.nvm
            .bank1()
            .keyr
            .write(|w| unsafe { w.bits(FLASH_KEY1) });
        self.nvm
            .bank1()
            .keyr
            .write(|w| unsafe { w.bits(FLASH_KEY2) });
    }

    /// Hal initialization.
    fn hal_init() {}
}

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
/// Checks boot partition page
fn stm32h7_boot_flag_page(addr: u32) -> bool {
    ((addr >= STM32H7_PART_BOOT_FLAGS_PAGE_ADDRESS) && (addr < STM32H7_PART_BOOT_END))
}

/// Checks update partition page
fn stm32h7_update_flag_page(addr: u32) -> bool {
    ((addr >= STM32H7_PART_UPDATE_FLAGS_PAGE_ADDRESS) && (addr < STM32H7_PART_UPDATE_END))
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
