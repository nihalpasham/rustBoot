#![no_std]
#![no_main]

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger
// use panic_probe as _; // global logger
// use defmt::*;

use cortex_m_rt::entry;
use cortex_m::asm;

use embedded_hal::digital::v2::OutputPin;
use rp2040_hal as hal;

use rustBoot_hal::pico::rp2040::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {

    let mut pac = hal::pac::Peripherals::take().unwrap();
    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new (
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led_pin = pins.gpio25.into_push_pull_output();

    let flash_writer = FlashWriterEraser {};
    let updater = FlashUpdater::new(flash_writer);

    match updater.update_success() {
        Ok(_v) => {}
        Err(e) => panic!("couldnt trigger update: {}", e),
    }

    loop {
        led_pin.set_high().unwrap();
        asm::delay(4_00_000);       // 125 mSec
        led_pin.set_low().unwrap();
        asm::delay(4_00_000);       // 125 mSec
    }
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
// End of file
