/*
    IMPORTANT NOTE ABOUT RP2040 FLASH SPACE ADDRESSES:
    When you pass an address to a rp2040-hal::rom_data function it wants addresses that start at `0x0000_0000`. 
    However, when you want to read that data back you need the address space to start at `0x1000_0000` (FLASH_XIP_BASE_ADDR)
*/

use core::{convert::TryInto, *};
use cortex_m::asm;
use rp2040_hal::rom_data;
use rp2040_hal as hal;
use crate::FlashInterface;
use rp2040_constants::*;

#[rustfmt::skip]
mod rp2040_constants {
    pub const FLASH_XIP_BASE_ADDR       : usize = 0x1000_0000;
    pub const FLASH_BLOCK_SIZE          : usize = 65536;
    pub const FLASH_SECTOR_SIZE         : usize = 4096;
    pub const FLASH_PAGE_SIZE           : usize = 256;
    pub const FLASH_SECTOR_ERASE_CMD    : u8    = 0x20;
    pub const FLASH_BLOCK32_ERASE_CMD   : u8    = 0x52;
    pub const FLASH_BLOCK64_ERASE_CMD   : u8    = 0xD8;
    pub const STACK_LOW                 : u32   = 0x2000_0000;
    pub const STACK_UP                  : u32   = 0x2004_2000;
    pub const RB_HDR_SIZE               : u32   = 0x100;
    pub const FW_BASE_ADDR              : u32   = 0x1002_0000;
    pub const VTR_TABLE_SIZE            : u32   = 0x100;
    pub const FW_RESET_VTR              : u32   = FW_BASE_ADDR + RB_HDR_SIZE + 0xc1;
}

pub struct FlashWriterEraser {}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {}
    }
}

impl FlashInterface for FlashWriterEraser {

    /// This method is to write data on flash. 
    /// 
    /// RP2040 uses external QSPI Flash chip. Before sending write commands over QSPI, XIP engine and interrupts are disabled.
    /// For flash writing it is using bootrom functions therefore, this function needs to be placed in RAM
    /// 
    /// Method arguments:
    /// -   address: It holds the address of flash where data has to be written
    /// -   data: u8 pointer holding the holding data.
    /// -   len :  number of bytes
    ///
    /// Returns:
    /// -  NONE
    #[inline(never)]
    #[link_section = ".data.ram_func"]
    fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
        asm::delay(8000);   // delay before writing data to flash
        if len <= 4 { 
            // for single byte or 4byte write

            // to find the page no. where address belongs and the offset of bytes to be written
            let offset_addr = address - FLASH_XIP_BASE_ADDR;
            let block_num: usize = offset_addr / FLASH_BLOCK_SIZE;
            let sector_num: usize = (offset_addr - (FLASH_BLOCK_SIZE * block_num)) / FLASH_SECTOR_SIZE;
            let page_num = (offset_addr - (FLASH_BLOCK_SIZE * block_num) - (FLASH_SECTOR_SIZE * sector_num)) / FLASH_PAGE_SIZE;
            let byte_num = offset_addr - (FLASH_BLOCK_SIZE * block_num) - (FLASH_SECTOR_SIZE * sector_num) - (FLASH_PAGE_SIZE * page_num);
            let mut page_start_addr: usize = (block_num * FLASH_BLOCK_SIZE) + (sector_num * FLASH_SECTOR_SIZE) + (page_num * FLASH_PAGE_SIZE);
            let mut src = data as *mut u8;
            let mut temp_page_buf: [u8; FLASH_PAGE_SIZE] = [0; FLASH_PAGE_SIZE];
            let mut dst = (page_start_addr + FLASH_XIP_BASE_ADDR) as *mut u8;

            // Caching the entire page the address belongs to as flash_range_program() wants the count in multiple of 256 bytes
            for idx in 0..FLASH_PAGE_SIZE {
                unsafe { 
                    temp_page_buf[idx] = *dst; 
                }
                dst = ((dst as u32) + 1 as u32) as *mut u8;
            }
            // Insert the bytes to be written in cached buffer 
            for idx in 0..len {
                unsafe { 
                    temp_page_buf[idx+byte_num] = *src; 
                }
                src = ((src as u32) + 1) as *mut u8;
            }

            unsafe {
                cortex_m::interrupt::free(|_cs| {
                    rom_data::connect_internal_flash();     // Restore all QSPI controls to their default state and connects SSI to QSPI
                    rom_data::flash_exit_xip();             // Initiates XIP exit sequence
                    rom_data::flash_range_program(page_start_addr as u32, temp_page_buf.as_ptr(), temp_page_buf.len());
                    rom_data::flash_flush_cache();          // Get the XIP working again
                    rom_data::flash_enter_cmd_xip();        // Start XIP back up
                });
            }
        } else {
            // for writing entire sector page by page for swapping

            let mut addr = address - FLASH_XIP_BASE_ADDR;
            let mut temp_page_buf: [u8; FLASH_PAGE_SIZE] = [0xff; FLASH_PAGE_SIZE];
            let starting_page = addr as u32;
            let ending_page = (addr + len) as u32;
            let mut src = data as *mut u8;

            // Caching the entire page the address belongs to as flash_range_program() wants count in multiple of 256 bytes 
            for addr in (starting_page..ending_page).step_by(FLASH_PAGE_SIZE) {
                for idx in 0..FLASH_PAGE_SIZE {
                    unsafe{ temp_page_buf[idx] = *src; }
                    src = ((src as u32) + 1 as u32) as *mut u8;
                }
                unsafe {
                    cortex_m::interrupt::free(|_cs| {
                        rom_data::connect_internal_flash(); // Restore all QSPI controls to their default state and connects SSI to QSPI
                        rom_data::flash_exit_xip();         // Initiates XIP exit sequence
                        rom_data::flash_range_program(addr as u32, temp_page_buf.as_ptr(), temp_page_buf.len());
                        rom_data::flash_flush_cache();      // Get the XIP working again
                        rom_data::flash_enter_cmd_xip();    // Start XIP back up
                    });
                }
                temp_page_buf = [0xff; FLASH_PAGE_SIZE];
            }
        }
    }


    /// This method is used to erase data on flash
    ///
    /// In RP2040 only sector erase is available. whatever be the length of bytes we pass to this function will erase
    /// the whole sector, whichever the sector the address belong to.
    ///
    /// Method arguments:
    /// -   addr: Address where data has to be erased
    /// -   len :  number of bytes to be erased
    ///
    /// Returns:
    /// -  NONE
    #[inline(never)]
    #[link_section = ".data.ram_func"]
    fn hal_flash_erase(&self, addr: usize, len: usize) {
        asm::delay(8000);
        let addres = (addr - FLASH_XIP_BASE_ADDR) as u32;
        let starting_page = (addres ) as u32;
        let ending_page = (addres + len as u32) as u32;
        // flash is erased a 4K sector at a time
        for addr in (starting_page..ending_page).step_by(FLASH_SECTOR_SIZE) {
            unsafe {
                cortex_m::interrupt::free(|_cs| {
                    rom_data::connect_internal_flash();
                    rom_data::flash_exit_xip();             // Initiates XIP exit sequence
                    rom_data::flash_range_erase(addr, FLASH_SECTOR_SIZE, FLASH_BLOCK_SIZE as u32, FLASH_SECTOR_ERASE_CMD);
                    rom_data::flash_flush_cache();          // Get the XIP working again
                    rom_data::flash_enter_cmd_xip();        // Start XIP back up
                });
            }
        }
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
    let scb = hal::pac::SCB::PTR;
    let address = fw_base_address as u32;
    unsafe {
        let stack_pointer = RefinedUsize::<STACK_LOW, STACK_UP, 0>::bounded_int(
            *(fw_base_address as *const u32)).0;
        let reset_vector = RefinedUsize::<0, 0, FW_RESET_VTR>::single_valued_int(
            *((fw_base_address + 4) as *const u32)).0;
        (*scb).vtor.write(address);
        cortex_m::asm::bootstrap(stack_pointer as *const u32, reset_vector as *const u32);
    }
}