#![no_std]
#![no_main]
#![allow(unsafe_code)]

static mut SHARED_BYTE: u8 = 0;

core::arch::global_asm!(
    ".section .vector_table,\"ax\"",
    ".word 0x20020000",
    ".word _reset + 1",
    ".word _halt + 1",
    ".word _halt + 1",
    ".word _halt + 1",
    ".word _halt + 1",
    ".word _halt + 1",
    ".space 36",
    ".section .text,\"ax\"",
    ".balign 4",
    "_halt: b _halt",
    ".balign 4",
    "_reset: bl main; b _reset",
);

fn putchar(c: u8) {
    unsafe {
        SHARED_BYTE = c;
        let p: u32 = &SHARED_BYTE as *const u8 as u32;
        core::arch::asm!("mov r0, #0x03", "mov r1, {0}", "bkpt #0xab", in(reg) p);
    }
}

fn puts(s: &str) { for &b in s.as_bytes() { putchar(b); } }

#[no_mangle]
pub extern "C" fn main() -> ! {
    use rustBoot::image::image::{StateNew, StateTesting, StateSuccess, StateUpdating, TypeState, SectFlags};
    use rustBoot::parser::{check_for_eof, check_for_padding};

    puts("rustBoot QEMU Test\n");
    let mut pass = true;
    if StateNew.from() != Some(0xFF) { puts("T1 FAIL\n"); pass = false; }
    if StateTesting.from() != Some(0x10) { puts("T2 FAIL\n"); pass = false; }
    if StateSuccess.from() != Some(0x00) { puts("T3 FAIL\n"); pass = false; }
    if StateUpdating.from() != Some(0x70) { puts("T4 FAIL\n"); pass = false; }
    if SectFlags::NewFlag.from() != Some(0x0F) { puts("T5 FAIL\n"); pass = false; }
    let data = [0x01u8, 0x02];
    if check_for_eof(&data).is_err() { puts("T6 FAIL\n"); pass = false; }
    let pad = [0xFFu8, 0xFF];
    if check_for_padding(&pad).map(|(r,_)| r.is_empty()) != Ok(true) { puts("T7 FAIL\n"); pass = false; }
    if pass { puts("ALL PASSED\n"); } else { puts("FAILED\n"); }
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }