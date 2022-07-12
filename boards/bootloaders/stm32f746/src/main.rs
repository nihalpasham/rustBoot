#![no_main]
#![no_std]

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use cortex_m_rt::entry;

use rustBoot_hal::stm::stm32f746::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

#[entry]
fn main() -> ! {
    let updater = FlashUpdater::new(FlashWriterEraser::new());
    updater.rustboot_start()
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
