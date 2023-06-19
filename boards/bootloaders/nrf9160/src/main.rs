#![no_std]
#![no_main]
#![feature(abi_c_cmse_nonsecure_call)]
#![feature(cmse_nonsecure_entry)]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger

use rustBoot_hal::nrf::nrf9160::{FlashWriterEraser, initialize};
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};

use cortex_m_rt::entry;


#[entry]
fn main() -> ! {
    let dp = nrf9160_pac::Peripherals::take().unwrap();
    
    unsafe {
        (*cortex_m::peripheral::SCB::PTR)
            .shcsr
            .write((1 << 19) | (1 << 18) | (1 << 17) | (1 << 16))
    };
    
    initialize(
        [
            (dp.SPIM0_S, dp.SPIS0_S, dp.TWIM0_S, dp.TWIS0_S, dp.UARTE0_S).into(),
            (dp.SPIM1_S, dp.SPIS1_S, dp.TWIM1_S, dp.TWIS1_S, dp.UARTE1_S).into(),
            (dp.SPIM2_S, dp.SPIS2_S, dp.TWIM2_S, dp.TWIS2_S, dp.UARTE2_S).into(),
            (dp.SPIM3_S, dp.SPIS3_S, dp.TWIM3_S, dp.TWIS3_S, dp.UARTE3_S).into(),
            (&dp.P0_S).into(),
            (&dp.KMU_S, &dp.NVMC_S).into(),
            (dp.CLOCK_S, dp.POWER_S).into(),
            (dp.RTC0_S).into(),
            (dp.RTC1_S).into(),
        ],
        [
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 9),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
            (0, 14),
            (0, 15),
            (0, 16),
            (0, 17),
            (0, 18),
            (0, 19),
            (0, 20),
            (0, 21),
            (0, 22),
            (0, 23),
            (0, 24),
            (0, 25),
            (0, 26),
            (0, 27),
            // (0, 28),
            // (0, 29),
            (0, 30),
            // (0, 31),
        ],
        [
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 9),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
            (0, 14),
            (0, 15),
        ],
    );
    
    // defmt::println!("Non secure memory initialized");
    
    let updater = FlashUpdater::new(FlashWriterEraser::new(dp.NVMC_S, dp.NVMC_NS, true));
    updater.rustboot_start()
}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
