#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz target for the FIT image parser.
// Ensures core FIT parsing functions never panic on arbitrary input.
fuzz_target!(|data: &[u8]| {
    let _ = rustBoot::dt::parse_algo(data);
    let _ = rustBoot::dt::as_str(data);
});