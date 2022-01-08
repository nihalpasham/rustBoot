// use crate::info;

// use crate::bsp::global::IRQ_CNTLR;
// use crate::bsp::drivers::common::interface::DeviceDriver;

#[no_mangle]
#[inline(never)]
/// Unconditionally jump to the kernel. This method uses `inline assembly`. I'd much rather avoid this.
pub unsafe extern "C" fn boot_into_kernel(img: usize, dtb: usize) -> ! {
    asm!(
        "mov x4, {img}",     // move linux kernel pointer into register x4
        "mov x5, {dtb}",     // move dtb pointer into register x5
        img = in(reg) img,
        dtb = in(reg) dtb,
        options(nomem, nostack, preserves_flags)
    );

    // initialize GICv2 for kernel.
    // let _ = IRQ_CNTLR.init();

    asm!(
        "mov x3, xzr", // zero-out registers x1, x2, x3
        "mov x2, xzr",
        "mov x1, xzr",
        "mov x0, x5", // move the dtb pointer to x0 (as first argument)
        "br x4",      // unconditionally jump to kernel entry at x4
        options(nomem, nostack, preserves_flags)
    );

    // we dont intend to return, i.e. `boot_into_kernel` diverges.
    halt()
}

type EntryPoint = unsafe extern "C" fn(dtb: usize, rsv0: usize, rsv1: usize, rsv2: usize);

#[no_mangle]
#[inline(never)]
/// Jump to kernel. I like this method better as it has a safe abstraction around the `unsafe jump`
pub fn boot_to_kernel(kernel_entry: usize, dtb_addr: usize) -> ! {
    unsafe {
        let f = core::mem::transmute::<usize, EntryPoint>(kernel_entry);
        f(dtb_addr, 0, 0, 0);
    }
    halt()
}

pub fn halt() -> ! {
    loop {
        unsafe { asm!("wfe") }
    }
}
