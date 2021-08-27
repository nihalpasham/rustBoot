#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use panic_probe as _;
use rustBoot_hal::nrf::nrf52840::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let updater = FlashUpdater::new(FlashWriterEraser::new());
    defmt::info!("trigger update");
    match updater.update_trigger() {
        Ok(_v) => {
            defmt::info!("start rustBoot");
            updater.rustboot_start()
        }
        Err(e) => {
            defmt::info!("failed to trigger update");
            panic!("failed to trigger update {}", e)
        }
    }
}
