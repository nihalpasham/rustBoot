#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;

extern crate stm32f4xx_hal as mcu;

use crate::mcu::{prelude::*, stm32};
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use mcu::delay::Delay;
use panic_probe as _;

use rustBoot_hal::stm::stm32f446::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();
    let gpio = dp.GPIOA.split();
    let mut led = gpio.pa5.into_push_pull_output();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();
    let mut delay = Delay::new(cp.SYST, &clocks);

    let mut count: i32 = 0;
    while count < 6 {
        led.toggle();
        delay.delay_ms(1000_u16);
        count = count + 1;
    }

    let flash1 = dp.FLASH;

    let flash_writer = FlashWriterEraser { nvm: flash1 };
    let updater = FlashUpdater::new(flash_writer);
    match updater.update_success() {
        Ok(_v) => {}
        Err(e) => panic!("couldnt trigger update: {}", e),
    }

    loop {
        led.toggle();
        delay.delay_ms(1000_u16);
    }
}
