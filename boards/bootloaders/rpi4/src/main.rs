#![no_std]
#![no_main]
#![feature(format_args_nl, core_intrinsics, once_cell)]
#![allow(warnings)]

mod boot;
mod dtb;
mod fit;
mod log;

use boot::{boot_kernel, DTB_LOAD_ADDR, ITB_LOAD_ADDR, KERNEL_LOAD_ADDR};
use fit::{load_fit, relocate_and_patch, verify_authenticity};

use rustBoot::{
    dt::FALLBACK_TO_ACTIVE_IMG,
    fs::controller::{Controller, TestClock, VolumeIdx},
    fs::filesystem::Directory,
    RustbootError,
};
use rustBoot_hal::rpi::rpi4::bsp::{
    drivers::{common::interface::DriverManager, driver_manager::driver_manager},
    global,
    global::EMMC_CONT,
};
use rustBoot_hal::rpi::rpi4::{
    exception,
    log::{
        console,
        console::{Read, Statistics},
    },
    memory::{layout::interface::MMU, mmu::mmu, vmm},
};
use rustBoot_hal::{info, println};
use zeroize::Zeroize;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
/// - The init calls in this function must appear in the correct order.
unsafe fn kernel_init() {
    exception::exception::handling_init();
    if let Err(string) = mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }
    for i in driver_manager().all_device_drivers().iter() {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x);
        }
    }
    driver_manager().post_device_driver_init();
    // println! is usable from here on.

    // Transition from unsafe to safe.
    kernel_main()
}

fn init_logger() {
    // initialize logger, prints debug info
    match log::init() {
        Ok(_v) => {}
        Err(e) => panic!("logger error: {:?}", e),
    };
}

/// The main function running after the early init.
///
/// active_fitimage=true,image_name=xx.itb,image_version=xxx
/// is_update_available=true,image_name=xx.itb,image_version=xxx,update_status=updating
fn kernel_main() -> ! {
    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", global::board_name());

    info!("MMU online. Special regions:");
    vmm::virt_mem_layout().print_layout();

    let (_, privilege_level) = exception::exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);

    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        time_manager().resolution().as_nanos()
    );

    info!("Drivers loaded:");
    for (i, driver) in driver_manager().all_device_drivers().iter().enumerate() {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    info!("Chars written: {}", console::console().chars_written());

    // Discard any spurious received characters before going into echo mode.
    console::console().clear_rx();

    // initialize logger.
    // init_logger();

    let mut ctrlr = Controller::new(&EMMC_CONT, TestClock);
    let volume = ctrlr.get_volume(VolumeIdx(0));
    match volume {
        Ok(mut volume) => {
            let _fat_cache = match ctrlr.populate_fat_cache(&volume) {
                Ok(_val) => {
                    info!("fat cache populated ...")
                }
                Err(e) => {
                    panic!("error populating fat_cache, {:?}", e)
                }
            };
            let (itb_blob, version) = load_fit(&mut volume, &mut ctrlr);
            let res = verify_authenticity(version);

            match res {
                Ok(val) => match val {
                    true => {
                        let _ = relocate_and_patch(itb_blob); // relocate kernel, ramdisk and patch dtb
                    }
                    false => panic!("signature verification result: {}", val),
                },
                Err(e)
                    if (e == RustbootError::BadVersion
                        && unsafe { *FALLBACK_TO_ACTIVE_IMG.get().unwrap_or(&false) }) =>
                {
                    // passive image version check failed
                    // falling back to active
                    // FALLBACK_TO_ACTIVE_IMG is set to true.
                    {
                        info!("### passive-image version check failed, falling back to active...###");
                        let _ = unsafe { &mut ITB_LOAD_ADDR.0.zeroize() };
                        let (itb_blob, version) = load_fit(&mut volume, &mut ctrlr);
                        let res = verify_authenticity(version);
                        match res {
                            Ok(val) => match val {
                                true => {
                                    let _ = relocate_and_patch(itb_blob); // relocate kernel, ramdisk and patch dtb
                                }
                                false => unreachable!("this should be unreachable"), 
                            },
                            // by definition, this shouldn't be possible. An active image must have been
                            // successfully verified and booted at least once.
                            Err(e) => unreachable!("active-image boot failed, {}", e),
                        }
                    }
                }
                Err(e) => panic!("error: image verification failed, {}", e),
            }
        }
        Err(e) => {
            panic!("failed to open fat32 volume/partition, {:?}", e)
        }
    }

    println!(
        "\x1b[5m\x1b[34m*************** \
            Starting kernel \
            ***************\x1b[0m\n"
    );

    unsafe {
        mmu().disable_mmu_and_caching();
        boot_kernel(
            { &mut KERNEL_LOAD_ADDR.0 }.as_ptr() as usize,
            { &mut DTB_LOAD_ADDR.0 }.as_ptr() as usize,
        )
    }
}
