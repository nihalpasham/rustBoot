#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz target for the device tree blob (DTB) parser.
// Ensures Reader::read never panics on arbitrary binary input.
fuzz_target!(|data: &[u8]| {
    let _ = rustBoot::dt::Reader::read(data);
});