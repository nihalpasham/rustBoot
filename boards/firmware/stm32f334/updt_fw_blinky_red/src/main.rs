#![no_std]
#![no_main]


extern crate cortex_m;
extern crate cortex_m_rt;

extern crate stm32f3xx_hal as mcu;
use cortex_m::asm;
use cortex_m_rt::entry;
use crate::mcu::{
    pac,
    prelude::*,
};
use panic_probe as _;

use rustBoot_hal::stm::stm32f334::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpio = dp.GPIOA.split(&mut rcc.ahb);
    let mut led = gpio.pa5.into_push_pull_output(&mut gpio.moder, &mut gpio.otyper);
    
    let flash1 = dp.FLASH;
    let flash_writer = FlashWriterEraser { nvm: flash1 };
    let updater = FlashUpdater::new(flash_writer);
   
    match updater.update_success() {
        Ok(_v) => {}
        Err(e) => panic!("couldnt trigger update: {}", e),
    }
    
    loop {
            led.toggle().unwrap();
            asm::delay(4000_00);
       }
    
}
