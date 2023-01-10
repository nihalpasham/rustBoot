#![no_main]
#![no_std]

use stm32f4xx_hal as mcu;

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger

use cortex_m_rt::entry;
use mcu::pac;
use mcu::prelude::*;

use rustBoot_hal::stm::stm32f469::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    if let (Some(peri), Some(cortex_peri)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // GPIO Initialization
        let gpiog = peri.GPIOG.split();
        let mut green_led = gpiog.pg6.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = peri.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cortex_peri.SYST.delay(&clocks);

        let flash1 = peri.FLASH;

        let mut count = 0;
        while count < 3 {
            // On for 1s, off for 1s.
            green_led.set_high();
            delay.delay_ms(1000_u32);
            green_led.set_low();
            delay.delay_ms(1000_u32);
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
    mcu::pac::SCB::sys_reset()
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
