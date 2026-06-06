#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz target for the update config file parser.
// Ensures the config parser never panics on arbitrary string input.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = core::str::from_utf8(data) {
        let _ = rustBoot::cfgparser::parse_config(s);
    }
});