
//! Architectural boot code.

use core::arch::global_asm;
use rustBoot_hal::info;

use crate::kernel_init;

// Assembly counterpart to this file.
global_asm!(include_str!("entry.S"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The Rust entry of the `kernel` binary.
///
/// The function is called from the assembly `_start` function.
///
/// # Safety
///
/// - Exception return from EL2 must must continue execution in EL1 with `kernel_init()`.
#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    kernel_init()
}


//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub fn halt() -> ! {
    info!("halting ... ");
    loop {
        unsafe { core::arch::asm!("wfe") }
    }
}

// /// Prints verbose information about the exception and then panics.
// #[no_mangle]
// #[link_section = ".vectors._exception"]
// pub fn exception(exc: u8) {
//     panic!(
//         "Unhandled CPU Exception!\n\n\
//         {}",
//         exc
//     );
// }

// #[no_mangle]
// #[link_section = ".vectors._exception_sync"]
// pub fn exception_sync(exc: u8) {
//     panic!(
//         "Unhandled CPU Exception!\n\n\
//         {}",
//         exc
//     );
// }