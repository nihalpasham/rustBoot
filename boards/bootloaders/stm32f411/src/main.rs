#![no_std]
#![no_main]

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger
// use panic_probe as _;

use rustBoot_hal::stm::stm32f411::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

use cortex_m_rt::entry;

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
