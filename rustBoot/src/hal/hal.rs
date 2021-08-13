
use rustBoot_hal::{preboot, boot_from};

// Arch-specific code
pub fn hal_preboot() {
    preboot()
}
pub fn hal_boot_from(addr: usize) -> ! {
    boot_from(addr)
}
