
use rustBoot_hal::nrf::nrf52840::FlashWriterEraser;
use rustboot::update::{Updater, update_flash::FlashUpdater};


fn main() {
    let updater = FlashUpdater::new(FlashWriterEraser::new());
    &updater.rustboot_start();
}
