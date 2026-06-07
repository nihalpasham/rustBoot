#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz target for the image header TLV parser.
// Ensures the parser never panics on arbitrary input.
fuzz_target!(|data: &[u8]| {
    let _ = rustBoot::parser::extract_version(data);
    let _ = rustBoot::parser::extract_timestamp(data);
    let _ = rustBoot::parser::extract_img_type(data);
    let _ = rustBoot::parser::extract_digest(data);
    let _ = rustBoot::parser::extract_pubkey_digest(data);
    let _ = rustBoot::parser::extract_signature(data);
});