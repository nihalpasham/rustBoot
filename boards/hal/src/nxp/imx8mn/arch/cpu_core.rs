//! Architectural processor code.
//!

pub use asm::nop;
use aarch64_cpu::asm;

use crate::info;
//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Pause execution on the core.
#[no_mangle]
pub fn wait_forever() -> ! {
    info!("\n");
    info!(" ... wait forever");
    loop {
        asm::wfe()
    }
}
