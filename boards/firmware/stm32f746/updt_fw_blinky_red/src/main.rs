#![no_main]
#![no_std]

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use crate::mcu::{pac, prelude::*};
use cortex_m_rt::entry;
use stm32f7xx_hal as mcu;

use rustBoot_hal::stm::stm32f746::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    let _p = pac::Peripherals::take().unwrap();

    let gipo = _p.GPIOB.split();
    let mut led1 = gipo.pb14.into_push_pull_output();

    let flash1 = _p.FLASH;
    let flash_writer = FlashWriterEraser { nvm: flash1 };
    let updater = FlashUpdater::new(flash_writer);

    match updater.update_success() {
        Ok(_v) => {}
        Err(e) => panic!("couldnt trigger update: {}", e),
    }

    loop {
        led1.toggle();
        cortex_m::asm::delay(8000000);
    }
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
