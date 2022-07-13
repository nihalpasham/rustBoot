#![no_std]
#![no_main]

use stm32f3xx_hal as hal;
use core::ptr::write_volatile;
use cortex_m::asm;
use hal::pac::{Peripherals, FLASH};
use crate::FlashInterface;
use stm32f334r8_constants::*;
#[rustfmt::skip]
mod stm32f334r8_constants {

    pub const FLASH_PAGE_SIZE : u32 = 2048;             
    pub const STACK_LOW       : u32 = 0x2000_0000;
    pub const STACK_UP        : u32 = 0x2002_0000;
    pub const RB_HDR_SIZE     : u32 = 0x100;
    pub const BASE_ADDR       : u32 = 0x0800B800;   //  pagetor 5 starting flag
    pub const VTR_TABLE_SIZE  : u32 = 0x100;
    pub const FW_RESET_VTR    : u32 = BASE_ADDR + RB_HDR_SIZE + VTR_TABLE_SIZE + 0x89;
    pub const UNLOCKKEY1      : u32 = 0x45670123;
    pub const UNLOCKKEY2      : u32 = 0xCDEF89AB;


}
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

    /// This method is used to erase data on flash
    ///
    /// In STM32F334 only page erase is available. whatever be the length of bytes we pass to this function will erase
    /// the whole page, whichever the page the address belong to.
    ///
    /// Method arguments:
    /// -   addr: Address where data has to be erased
    /// -   len :  number of bytes to be erased
    ///
    /// Returns:
    /// -  NONE
    fn hal_flash_erase(&self, addr: usize, len: usize) {
        let mut flag: bool = true;
        let mut address = (addr & 0x0800_F800) as u32; // Finding base address of the page from the given address 
        let remaing_bytes  = len%FLASH_PAGE_SIZE as usize;
        let mut num_pages = len/FLASH_PAGE_SIZE as usize;
        if remaing_bytes != 0
        {
           num_pages = num_pages + 1; 
        }
        while num_pages > 0 {
            match address {
                (0x0800_0000..=0x0800_FFFF) => flag = true,
                _ => flag = false,
            }
            if flag {
                self.hal_flash_unlock();
                while self.nvm.sr.read().bsy().bit_is_set() {}
                self.nvm.cr.modify(|_, w| {
                    w
                    .per().set_bit() 
                });
                self.nvm.ar.write(|w| {
                    w
                    .far().bits(address)
                    });
                self.nvm.cr.modify(|_, w| {
                    w
                    .strt().set_bit()
                });
                while self.nvm.sr.read().bsy().bit_is_set() {}
                if self.nvm.sr.read().eop().bit_is_set(){
                    self.nvm.sr.modify(|_, w| { w.eop().set_bit()});
                }
                self.nvm.cr.modify(|_, w| {
                    w
                    .per().clear_bit()
                });
                self.hal_flash_lock();
            }

            address = address + FLASH_PAGE_SIZE;
            flag = false;
            num_pages = num_pages - 1;
        }
      
    }


    /// This method is to write data on flash
    ///
    /// Method arguments:
    /// -   address: It holds the address of flash where data has to be written
    /// -   data: u8 pointer holding the holding data.
    /// -   len :  number of bytes
    ///
    /// Returns:
    /// -  NONE
    /// 
    fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {

        let address = address as u32;
        let mut len = len as u32;
        let mut idx = 0u32;
        let mut src = data as *mut u16;
        let mut dst = address as *mut u16;
        self.nvm.sr.modify(|_, w| {
            w
             .pgerr().set_bit()
        });
        self.hal_flash_unlock();
        while idx < len {        
            if (len-idx) > 1 {
                while self.nvm.sr.read().bsy().bit_is_set() {}
                if self.nvm.cr.read().lock().bit_is_set() {
                    self.hal_flash_unlock();
                }
                self.nvm.cr.modify(|_, w| {
                    w
                        .pg().set_bit()
                });
                unsafe {             
                    write_volatile(dst,*src)
                };
                while self.nvm.sr.read().bsy().bit_is_set(){}
                if self.nvm.sr.read().eop().bit_is_set(){
                    self.nvm.sr.modify(|_, w| { w.eop().set_bit()});
                }
                self.nvm.cr.modify(|_, w| {
                    w
                    .pg().clear_bit()
                });
                src = ((src as u32) + 2) as *mut u16; // increment pointer by 2
                dst = ((dst as u32) + 2) as *mut u16; // increment pointer by 2     
                idx =idx+2;      
            }
            else{  
                let  src1 = src as *mut u8;
                let mut data1 = 0; 
                let mut add = 0u16;
                let mut half_words_count = 0usize;                            
                unsafe { 
                    data1 = *src1
                }
                if len == 1
                {          
                    let base_addr = 0x0800_0000;
                    let mut buffer : [u8; 2048] = [0; 2048];// Taken a buffer of one page size 2048
                    let  val = address - base_addr;
                    let  sector = val/0x800;
                    let  offset = (val % 0x800) as usize;
                    let mut addr = (base_addr + (sector * 0x800)) as *mut u8;
                    let mut dst_addr = addr as *mut u16;
                    let temp = addr as u32;
                    for byte_count in 0..2048
                    {
                        unsafe { buffer[byte_count]= *addr };
                        addr = ((addr as u32) + 1) as *mut u8;
                        asm::delay(1);              //One clock cycle delay of main clock 
                    }
                    buffer[offset] = data1;
                    self.hal_flash_erase(dst_addr as usize,1);
                    for half_words in 0..1024
                    { 
                        while self.nvm.sr.read().bsy().bit_is_set() {}
                        if self.nvm.cr.read().lock().bit_is_set() {
                            self.hal_flash_unlock();
                        }
                        self.nvm.cr.modify(|_, w| {
                            w
                            .pg().set_bit()
                        });
                        self.nvm.ar.write(|w| {
                            w
                            .far().bits(temp)
                        });
                        add = ((buffer[half_words_count+1] as u16)<<8 as u16) | buffer[half_words_count]  as u16;
                        unsafe{
                            unsafe{*dst_addr = add}
                        } 
                        asm::delay(1);
                        while self.nvm.sr.read().bsy().bit_is_set() {}
                        if self.nvm.sr.read().eop().bit_is_set(){
                                    self.nvm.sr.modify(|_, w| { w.eop().set_bit()});
                        }
                        self.nvm.cr.modify(|_, w| {
                            w
                            .pg().clear_bit()
                        });
                        dst_addr = ((dst_addr as u32) + 2) as *mut u16;
                        half_words_count = half_words_count + 2;                       
                    }
                    idx = idx +1
                }
                else{
                    let mut val = 0u16;
                    let val_bytes = ((&mut val) as *mut u16) as *mut u8;
                    let offset = (address + idx) - (((address + idx) >> 1) << 1); // offset from nearest word aligned address
                    dst = ((dst as u32) - offset) as *mut u16; // subtract offset from dst addr
                    unsafe {
                        val = *dst; // assign current val at dst to val
                        // store data byte at idx to `val`. `val_bytes` is a byte-pointer to val.
                        *val_bytes.add(offset as usize) = *data.add(idx as usize);
                    }
                    while self.nvm.sr.read().bsy().bit_is_set() {}
                    if self.nvm.cr.read().lock().bit_is_set() {
                        self.hal_flash_unlock();
                    }
                    self.nvm.cr.modify(|_, w| {
                        w
                        .pg().set_bit()
                    });
                    unsafe {         
                        write_volatile(dst, val);    
                    };
                    while self.nvm.sr.read().bsy().bit_is_set() {}
                    if self.nvm.sr.read().eop().bit_is_set(){
                        self.nvm.sr.modify(|_, w| { w.eop().set_bit()});
                    }
                    self.nvm.cr.modify(|_, w| {
                        w
                        .pg().clear_bit()
                    }); 
                    src = ((src as u32) + 1) as *mut u16; // increment pointer by 1 byte address
                    dst = ((dst as u32) + 1) as *mut u16; // increment pointer by 1 byte address
                    idx = idx+1;
                }
            }
        
        }
        self.nvm.cr.modify(|_, w| {
            w
            .pg().clear_bit()
        });
        self.hal_flash_lock();
    }

    /// This method is used to unlock the flash
    ///
    /// Flash has to be unlocked to do any operation on it.
    /// Method arguments:
    /// -   NONE
    /// Returns:
    /// -  NONE
    /// 
    fn hal_flash_unlock(&self) {
        self.nvm.keyr.write(|w| { w.fkeyr().bits(UNLOCKKEY1) });
        self.nvm.keyr.write(|w| { w.fkeyr().bits(UNLOCKKEY2) });
    }

    /// This method is used to lock the flash
    ///
    /// Once the flash is locked no operation on flash can be perfomed.
    /// Method arguments:
    /// -   NONE
    /// Returns:
    /// -  NONE
    ///
    fn hal_flash_lock(&self) {
        self.nvm.cr.modify(|_, w| w.lock().set_bit());
    }

    fn hal_init(){}

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