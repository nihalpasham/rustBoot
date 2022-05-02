// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.

use core::arch::global_asm;
use cortex_a::{asm, registers::*};
use tock_registers::interfaces::Writeable;

// Assembly counterpart to this file.
global_asm!(include_str!("boot.s"));

/// Prepares the transition from EL2 to EL1.
///
/// # Safety
///
/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
/// - The HW state of EL1 must be prepared in a sound way.
#[inline(always)]
unsafe fn el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
    // Enable timer counter registers for EL1.
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters.
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    //
    // First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
    // stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to kernel_init().
    ELR_EL2.set(crate::kernel_init as *const () as u64);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
    // are no plans to ever return to EL2, just re-use the same stack.
    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
}

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
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret()
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Used by `arch` code to find the early boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

const MAX_INITRAMFS_SIZE: usize = 16066 * 4 * 512;
const MAX_KERNEL_SIZE: usize = 14624 * 4 * 512;
pub(crate) const MAX_DTB_SIZE: usize = 100 * 512;
const MAX_ITB_SIZE: usize = 32000 * 4 * 512;

pub struct InitRamfsEntry(pub [u8; MAX_INITRAMFS_SIZE]);
#[repr(align(2097152))]
pub struct KernelEntry(pub [u8; MAX_KERNEL_SIZE]);
#[repr(align(2097152))]
pub struct DtbEntry(pub [u8; MAX_DTB_SIZE]);

pub struct ImageTreeEntry(pub [u8; MAX_ITB_SIZE]);

impl ImageTreeEntry {
    /// Get an entry point to the ITB.
    pub const fn new() -> Self {
        Self([0u8; MAX_ITB_SIZE])
    }
}

impl KernelEntry {
    /// Get the kernel's entry point. We assume all Aarch64 kernels use a 2MB aligned base.
    /// i.e. this impl wont work for kernels that aren't 2MB aligned.  
    ///
    /// The flags field (introduced in v3.17) is a little-endian 64-bit field.
    /// Bit 3 of the flags field specifies `Kernel physical placement`
    /// - 0 - 2MB aligned base should be as close as possible to the base of DRAM, since memory
    /// below it is not accessible via the linear mapping
    /// - 1 - 2MB aligned base may be anywhere in physical memory
    pub const fn new() -> Self {
        Self([0u8; MAX_KERNEL_SIZE])
    }
}

impl DtbEntry {
    /// Get a 2MB aligned entry point to the DTB.
    pub const fn new() -> Self {
        Self([0u8; MAX_DTB_SIZE])
    }
}

impl InitRamfsEntry {
    /// Get an entry point to the `initramfs`.
    pub const fn new() -> Self {
        Self([0u8; MAX_INITRAMFS_SIZE])
    }
}

pub static mut INITRAMFS_LOAD_ADDR: InitRamfsEntry = InitRamfsEntry::new();
pub static mut KERNEL_LOAD_ADDR: KernelEntry = KernelEntry::new();
pub static mut DTB_LOAD_ADDR: DtbEntry = DtbEntry::new();
pub static mut ITB_LOAD_ADDR: ImageTreeEntry = ImageTreeEntry::new();

type EntryPoint = unsafe extern "C" fn(dtb: usize, rsv0: usize, rsv1: usize, rsv2: usize);

#[no_mangle]
#[inline(never)]
/// Jump to kernel. I like this method better as it has a safe abstraction around the `unsafe jump`
pub fn boot_kernel(kernel_entry: usize, dtb_addr: usize) -> ! {
    unsafe {
        let f = core::mem::transmute::<usize, EntryPoint>(kernel_entry);
        f(dtb_addr, 0, 0, 0);
    }
    halt()
}

pub fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("wfe") }
    }
}

// #[no_mangle]
// #[inline(never)]
// /// Unconditionally jump to the kernel. This method uses `inline assembly`. I'd much rather avoid this.
// pub unsafe extern "C" fn boot_into_kernel(img: usize, dtb: usize) -> ! {
//     asm!(
//         "mov x4, {img}",     // move linux kernel pointer into register x4
//         "mov x5, {dtb}",     // move dtb pointer into register x5
//         img = in(reg) img,
//         dtb = in(reg) dtb,
//         options(nomem, nostack, preserves_flags)
//     );

//     asm!(
//         "mov x3, xzr", // zero-out registers x1, x2, x3
//         "mov x2, xzr",
//         "mov x1, xzr",
//         "mov x0, x5", // move the dtb pointer to x0 (as first argument)
//         "br x4",      // unconditionally jump to kernel entry at x4
//         options(nomem, nostack, preserves_flags)
//     );

//     // we dont intend to return, i.e. `boot_into_kernel` diverges.
//     halt()
// }
