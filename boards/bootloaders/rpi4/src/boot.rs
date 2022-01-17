

const MAX_INITRAMFS_SIZE: usize = 16066 * 4 * 512;
const MAX_KERNEL_SIZE: usize = 14624 * 4 * 512;
const MAX_DTB_SIZE: usize = 100 * 512;

pub struct InitRamfs(pub [u8; MAX_INITRAMFS_SIZE]);
pub struct KernelEntry(pub [u8; MAX_KERNEL_SIZE]);
pub struct DtbEntry(pub [u8; MAX_DTB_SIZE]);

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

impl InitRamfs {
    /// Get an entry point to the `initramfs`. 
    pub const fn new() -> Self {
        Self([0u8; MAX_INITRAMFS_SIZE])
    }
}

#[link_section = ".initramfs_load_addr._initramfs_start"]
pub static mut INITRAMFS_LOAD_ADDR: InitRamfs = InitRamfs::new();

#[link_section = ".kernel_load_addr._kernel_start"]
pub static mut KERNEL_LOAD_ADDR: KernelEntry = KernelEntry::new();

#[link_section = ".dtb_load_addr._dtb_start"]
pub static mut DTB_LOAD_ADDR: DtbEntry = DtbEntry::new();

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


