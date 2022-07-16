#![no_main]
#![no_std]

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

use rustBoot_hal::stm::stm32f411::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the LED.
        let gpiod = dp.GPIOD.split();
        let mut led = gpiod.pd12.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        let flash1 = dp.FLASH;
        let mut count = 0;

        while count < 6 {
            led.toggle();
            delay.delay_ms(1000_u16);
            count = count + 1;
        }

        let flash_writer = FlashWriterEraser { nvm: flash1 };
        let updater = FlashUpdater::new(flash_writer);

        match updater.update_trigger() {
            Ok(_v) => {}
            Err(e) => panic!("couldnt trigger update: {}", e),
        }
    }
    //nvic_systemreset();
    hal::pac::SCB::sys_reset()
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
