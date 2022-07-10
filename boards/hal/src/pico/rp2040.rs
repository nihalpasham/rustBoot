use core::{convert::TryInto, *};
use cortex_m::asm;
use rp2040_hal::rom_data;
use rp2040_pac;
use crate::FlashInterface;
pub const BLOCK_SIZE: u32 = 65536;
pub const SECTOR_SIZE: usize = 4096;
pub const SECTOR_ERASE: u8 = 0x20; // Tested and works with W25Q16JV flash chip
// pub const BLOCK64_ERASE: u8 = 0xD8;
// pub const BLOCK32_ERASE: u8 = 0x52;

pub struct FlashWriterEraser {}

impl FlashWriterEraser {
    pub fn new() -> Self {
        FlashWriterEraser {}
    }
}

impl FlashInterface for FlashWriterEraser {

    /// This method is to write data on flash 
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
        asm::delay(8000);
        if len <= 4{
            let offset = address - 0x10000000;
            let block: usize = offset / 65536;
            let sector: usize = (offset - (65536*block))/ 4096;
            let page = (offset - (65536*block) - (4096*sector))/ 256;
            let byte = offset - (65536*block) - (4096*sector) - (256*page);
            let mut page_start_addr: usize = (block*65536) + (sector*4096) + (page*256);
            let mut src = data as *mut u8;
            let mut temp_page_buf: [u8; 256] = [0; 256];
            let mut dst = (page_start_addr + 0x10000000) as *mut u8;
            for x in 0..256{
                unsafe{ temp_page_buf[x] = *dst; }
                dst = ((dst as u32) + 1 as u32) as *mut u8;
            }
            for y in 0..len{
                unsafe{ temp_page_buf[y+byte] = *src; }
                src = ((src as u32) + 1) as *mut u8;
            }
            unsafe {
                cortex_m::interrupt::free(|_cs| {
                    rom_data::connect_internal_flash();
                    rom_data::flash_exit_xip();
                    // rom_data::flash_range_erase(page_start_addr as u32, SECTOR_SIZE, BLOCK_SIZE, SECTOR_ERASE);
                    rom_data::flash_range_program(page_start_addr as u32, temp_page_buf.as_ptr(), temp_page_buf.len());
                    rom_data::flash_flush_cache(); 
                    rom_data::flash_enter_cmd_xip();
                });
            }
        }else{
            // let data_addr = data as u32;
            // defmt::println!("hal_flash_write>> addr: {:x} data: {:x} len: {}", address, data_addr, len);
            let mut addr = address - 0x10000000;
            let mut temp_page_buf: [u8; 256] = [0xff; 256];
            let starting_page = addr as u32;
            let ending_page = (addr + len) as u32;
            let mut src = data as *mut u8;

            for addr in (starting_page..ending_page).step_by(256 as usize) {
                for x in 0..256{
                    unsafe{ temp_page_buf[x] = *src; }
                    src = ((src as u32) + 1 as u32) as *mut u8;
                }
                unsafe {
                    cortex_m::interrupt::free(|_cs| {
                        rom_data::connect_internal_flash();
                        rom_data::flash_exit_xip();
                        // rom_data::flash_range_erase(page_start_addr as u32, SECTOR_SIZE, BLOCK_SIZE, SECTOR_ERASE);
                        rom_data::flash_range_program(addr as u32, temp_page_buf.as_ptr(), temp_page_buf.len());
                        rom_data::flash_flush_cache();
                        rom_data::flash_enter_cmd_xip();
                    });
                }
                temp_page_buf = [0xff; 256];
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
        let addres = (addr - 0x10000000) as u32;
        let starting_page = (addres ) as u32;
        let ending_page = (addres + len as u32) as u32;
        for addr in (starting_page..ending_page).step_by(SECTOR_SIZE as usize) {
            unsafe {
                cortex_m::interrupt::free(|_cs| {
                    rom_data::connect_internal_flash();
                    rom_data::flash_exit_xip();
                    rom_data::flash_range_erase(addr, SECTOR_SIZE, BLOCK_SIZE, SECTOR_ERASE);
                    rom_data::flash_flush_cache();
                    rom_data::flash_enter_cmd_xip();
                });
            }
        }
    }
    fn hal_init() {}
    fn hal_flash_lock(&self) {}
    fn hal_flash_unlock(&self) {}
}


/// This method is used to boot the firmware from a particular address
///
/// Method arguments:
/// -   fw_base_address  : address of the firmware
/// Returns:
/// -  NONE
#[rustfmt::skip]
pub fn boot_from(fw_base_address: usize) {
    static mut USER_RESET: Option<extern "C" fn()> = None;
    let scb = rp2040_pac::SCB::PTR;
    let address = fw_base_address as u32;
    unsafe {
        let sp = *(address as *const u32);
        let rv = *((address + 4) as *const u32);
        USER_RESET = Some(core::mem::transmute(rv));
        (*scb).vtor.write(address);
        cortex_m::register::msp::write(sp);
        (USER_RESET.unwrap())();
    }
}