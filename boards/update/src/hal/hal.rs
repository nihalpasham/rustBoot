use rustBoot_hal::{boot_from, preboot};

// Arch-specific code
pub fn hal_preboot() {
    preboot()
}
pub fn hal_boot_from(addr: usize) -> ! {
    boot_from(addr)
}
