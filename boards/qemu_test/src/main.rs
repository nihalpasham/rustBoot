#![no_std]
#![no_main]
#![allow(unsafe_code)]

#[repr(C)]
struct VT {
    sp: u32,
    reset: unsafe extern "C" fn(),
    _nmi: unsafe extern "C" fn(),
    _hardfault: unsafe extern "C" fn(),
    _memmanage: unsafe extern "C" fn(),
    _busfault: unsafe extern "C" fn(),
    _usagefault: unsafe extern "C" fn(),
    _reserved: [unsafe extern "C" fn(); 9],
    _svcall: unsafe extern "C" fn(),
    _reserved2: [unsafe extern "C" fn(); 2],
}

extern "C" fn default_handler() { loop {} }

#[link_section = "__TEXT,.isr_vector"]
#[used]
static VTABLE: VT = VT {
    sp: 0x20020000,
    reset: reset_handler,
    _nmi: default_handler,
    _hardfault: default_handler,
    _memmanage: default_handler,
    _busfault: default_handler,
    _usagefault: default_handler,
    _reserved: [default_handler; 9],
    _svcall: default_handler,
    _reserved2: [default_handler; 2],
};

extern "C" fn reset_handler() { main() }

fn putchar(c: u8) {
    unsafe {
        let func: u32 = 0x03;
        let ptr: *const u8 = &c;
        core::arch::asm!(
            "mov r0, {0}",
            "mov r1, {1}",
            ".inst 0xBEAB",
            in(reg) func,
            in(reg) ptr,
        );
    }
}
fn puts(s: &str) { for &b in s.as_bytes() { putchar(b); } }

#[no_mangle]
pub extern "C" fn main() -> ! {
    puts("rustBoot QEMU Test\n");

    use rustBoot::image::image::{StateNew, StateTesting, StateSuccess, StateUpdating, TypeState, SectFlags};

    let mut pass = true;
    if StateNew.from() != Some(0xFF) { puts("T1 FAIL\n"); pass = false; }
    if StateTesting.from() != Some(0x10) { puts("T2 FAIL\n"); pass = false; }
    if StateSuccess.from() != Some(0x00) { puts("T3 FAIL\n"); pass = false; }
    if StateUpdating.from() != Some(0x70) { puts("T4 FAIL\n"); pass = false; }
    if SectFlags::NewFlag.from() != Some(0x0F) { puts("T5 FAIL\n"); pass = false; }

    if pass { puts("\nALL PASSED\n"); } else { puts("\nFAILED\n"); }
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { puts("PANIC\n"); loop {} }