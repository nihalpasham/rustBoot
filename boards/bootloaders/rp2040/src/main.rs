#![no_std]
#![no_main]

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger

use cortex_m_rt::entry;
use rustBoot_hal::pico::rp2040::FlashWriterEraser;
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};


/// This is the second-Stage bootloader. RP2040 is a stateless device, 
/// with support for cached execute-in-place from external QSPI memory.   
/// This will put 256 byte header right at the start of the eventual program binary, 
/// required to configure external QSPI flash memory (W25Q080) for high speed, efficient, code access.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

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
