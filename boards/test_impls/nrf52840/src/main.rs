#![no_std]
#![no_main]
#![allow(non_snake_case)]

use defmt_rtt as _; // global logger
use panic_probe as _;
use rustBoot_hal::nrf::nrf52840::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let updater = FlashUpdater::new(FlashWriterEraser::new());
    defmt::info!("start rustBoot");
    updater.rustboot_start()
}
