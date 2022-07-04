#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt;

//#[cfg(feature = "defmt")]
//use defmt_rtt as _; // global logger

extern crate stm32f3xx_hal as mcu;
use cortex_m::peripheral::Peripherals;
use cortex_m::asm;
use cortex_m_rt::entry;
use mcu::{pac, prelude::*};

use rustBoot_hal::stm::stm32f334::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    if let (Some(peri), Some(_cortex_peri)) = (pac::Peripherals::take(), Peripherals::take()){
      
      let mut rcc = peri.RCC.constrain();
      let mut gpioa = peri.GPIOA.split(&mut rcc.ahb);
      let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

      let flash1 = peri.FLASH;
      let mut count = 0;

      while count < 6 {
            led.toggle().unwrap();
            asm::delay(8000_000);
            count +=  1;
        }
      
       let flash_writer = FlashWriterEraser { nvm: flash1 };
       let updater = FlashUpdater::new(flash_writer);
        
       match updater.update_trigger() {
            Ok(_v) => {}
            Err(e) => panic!("couldnt trigger update: {}", e)
        } 

    }
    mcu::pac::SCB::sys_reset()
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}