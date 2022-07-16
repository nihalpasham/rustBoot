// #![deny(warnings)]
#![no_main]
#![no_std]

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use stm32h7xx_hal::{pac, prelude::*};

use rustBoot_hal::stm::stm32h723::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    //GPIO init
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);

    // Configure PE1 as output.
    let mut led1 = gpiob.pb0.into_push_pull_output();

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    let flsh = dp.FLASH;

    let mut count = 0;

    while count < 4 {
        led1.set_high();
        delay.delay_ms(500_u16);
        led1.set_low();
        delay.delay_ms(500_u16);
        count = count + 1;
    }

    let flash_writer = FlashWriterEraser { nvm: flsh };
    let updater = FlashUpdater::new(flash_writer);

    match updater.update_trigger() {
        Ok(_v) => {}
        Err(e) => panic!("couldnt trigger update: {}", e),
    }

    stm32h7xx_hal::pac::SCB::sys_reset();
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}