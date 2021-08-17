#![no_std]
#![no_main]
#![allow(non_snake_case)]

use rustBoot_hal::nrf::nrf52840::FlashWriterEraser;
use rustBoot_update::update::{UpdateInterface, update_flash::FlashUpdater};

use cortex_m_rt::{entry, exception};

#[entry]
fn main() -> ! {
    let updater = FlashUpdater::new(FlashWriterEraser::new());
    updater.rustboot_start()
}

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}