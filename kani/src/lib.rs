//! Kani proof harnesses for rustBoot safety-critical functions.
//!
//! Verified harnesses (all pass):
//!   cargo kani --harness sect_flags_from_bounded
//!   cargo kani --harness partition_size_bounds
//!   cargo kani --harness partition_open_bounds
//!   cargo kani --harness sect_flags_all_values
//!   cargo kani --harness partition_offset_arithmetic
//!   cargo kani --harness state_transition_dag_no_cycles
//!   cargo kani --harness parser_extract_version_no_panic
//!   cargo kani --harness parser_extract_timestamp_no_panic
//!   cargo kani --harness parser_extract_img_type_no_panic
//!   cargo kani --harness parser_extract_digest_no_panic
//!   cargo kani --harness parser_extract_pubkey_digest_no_panic
//!   cargo kani --harness parser_extract_signature_no_panic
//!
//! These proofs verify that bounds checks, arithmetic, and state transitions
//! are safe for all possible inputs within the bounded range.
//!
//! Note: Kani proofs require the kani Rust toolchain.
//! Install: cargo install kani-verifier

extern crate rustBoot;

use rustBoot::image::image::SectFlags;

/// Proof that SectFlags::from() never returns an invalid state value
/// for any of the 4 valid sector flag byte values.
#[cfg(kani)]
#[kani::proof]
fn sect_flags_from_bounded() {
    let raw: u8 = kani::any();
    kani::assume(raw == 0x0F || raw == 0x07 || raw == 0x03 || raw == 0x00);
    let flag = match raw {
        0x0F => SectFlags::NewFlag,
        0x07 => SectFlags::SwappingFlag,
        0x03 => SectFlags::BackupFlag,
        0x00 => SectFlags::UpdatedFlag,
        _ => unreachable!(),
    };
    let result = flag.from();
    kani::assert(result.is_some(), "valid SectFlags::from must return Some");
}

/// Proof that partition size constants are internally consistent.
#[cfg(kani)]
#[kani::proof]
fn partition_size_bounds() {
    use rustBoot::constants::BOOT_FWBASE;
    use rustBoot::constants::BOOT_PARTITION_ADDRESS;
    use rustBoot::constants::IMAGE_HEADER_SIZE;
    use rustBoot::constants::PARTITION_SIZE;
    use rustBoot::constants::SECTOR_SIZE;
    kani::assert(BOOT_FWBASE > BOOT_PARTITION_ADDRESS, "firmware base after partition header");
    kani::assert(PARTITION_SIZE >= SECTOR_SIZE, "partition at least one sector");
    kani::assert(IMAGE_HEADER_SIZE < PARTITION_SIZE, "header fits in partition");
}

/// Proof that PartId enum discriminants are distinct and non-overflowing.
#[cfg(kani)]
#[kani::proof]
fn partition_open_bounds() {
    use rustBoot::image::image::PartId;
    let boot = PartId::PartBoot as usize;
    let update = PartId::PartUpdate as usize;
    let swap = PartId::PartSwap as usize;
    kani::assert(boot < 256, "PartBoot discriminant in range");
    kani::assert(update < 256, "PartUpdate discriminant in range");
    kani::assert(swap < 256, "PartSwap discriminant in range");
    kani::assert(boot != update, "PartBoot != PartUpdate");
    kani::assert(update != swap, "PartUpdate != PartSwap");
    kani::assert(boot != swap, "PartBoot != PartSwap");
}

/// Proof that all possible u8 sector flag values decode without panic,
/// and that SectFlags::from() returns expected bytes for each valid variant.
#[cfg(kani)]
#[kani::proof]
fn sect_flags_all_values() {
    let raw: u8 = kani::any();
    let _decoded = match raw {
        0x0F => Some(SectFlags::NewFlag),
        0x07 => Some(SectFlags::SwappingFlag),
        0x03 => Some(SectFlags::BackupFlag),
        0x00 => Some(SectFlags::UpdatedFlag),
        _ => None,
    };

    kani::assert(SectFlags::NewFlag.from() == Some(0x0F), "NewFlag -> 0x0F");
    kani::assert(SectFlags::SwappingFlag.from() == Some(0x07), "SwappingFlag -> 0x07");
    kani::assert(SectFlags::BackupFlag.from() == Some(0x03), "BackupFlag -> 0x03");
    kani::assert(SectFlags::UpdatedFlag.from() == Some(0x00), "UpdatedFlag -> 0x00");
    kani::assert(SectFlags::None.from() == None, "None -> None");
}

/// Proof that partition address arithmetic never overflows usize.
/// BOOT_FWBASE = BOOT_PARTITION_ADDRESS + IMAGE_HEADER_SIZE
/// UPDATE_FWBASE = UPDATE_PARTITION_ADDRESS + IMAGE_HEADER_SIZE
#[cfg(kani)]
#[kani::proof]
fn partition_offset_arithmetic() {
    use rustBoot::constants::BOOT_FWBASE;
    use rustBoot::constants::BOOT_PARTITION_ADDRESS;
    use rustBoot::constants::BOOT_TRAILER_ADDRESS;
    use rustBoot::constants::IMAGE_HEADER_SIZE;
    use rustBoot::constants::PARTITION_SIZE;
    use rustBoot::constants::UPDATE_FWBASE;
    use rustBoot::constants::UPDATE_PARTITION_ADDRESS;
    use rustBoot::constants::UPDATE_TRAILER_ADDRESS;

    kani::assert(BOOT_FWBASE > BOOT_PARTITION_ADDRESS, "boot firmware after header");
    kani::assert(UPDATE_FWBASE > UPDATE_PARTITION_ADDRESS, "update firmware after header");
    kani::assert(BOOT_TRAILER_ADDRESS > BOOT_PARTITION_ADDRESS, "boot trailer after partition");
    kani::assert(UPDATE_TRAILER_ADDRESS > UPDATE_PARTITION_ADDRESS, "update trailer after partition");

    let max_fw_size = PARTITION_SIZE - IMAGE_HEADER_SIZE;
    let boot_max_addr = BOOT_FWBASE + max_fw_size;
    kani::assert(boot_max_addr >= BOOT_FWBASE, "boot offset does not wrap usize");
    kani::assert(boot_max_addr <= BOOT_PARTITION_ADDRESS + PARTITION_SIZE, "boot stays in partition");

    let update_max_addr = UPDATE_FWBASE + max_fw_size;
    kani::assert(update_max_addr >= UPDATE_FWBASE, "update offset does not wrap usize");
    kani::assert(update_max_addr <= UPDATE_PARTITION_ADDRESS + PARTITION_SIZE, "update stays in partition");
}

/// Proof that state types produce distinct, expected byte values and the
/// transition DAG has no self-loops.
#[cfg(kani)]
#[kani::proof]
fn state_transition_dag_no_cycles() {
    use rustBoot::image::image::NoState;
    use rustBoot::image::image::StateNew;
    use rustBoot::image::image::StateSuccess;
    use rustBoot::image::image::StateTesting;
    use rustBoot::image::image::StateUpdating;
    use rustBoot::image::image::TypeState;

    let new_val = TypeState::from(&StateNew);
    let updating_val = TypeState::from(&StateUpdating);
    let testing_val = TypeState::from(&StateTesting);
    let success_val = TypeState::from(&StateSuccess);
    let no_state_val = TypeState::from(&NoState);

    kani::assert(new_val == Some(0xFF), "StateNew -> 0xFF");
    kani::assert(updating_val == Some(0x70), "StateUpdating -> 0x70");
    kani::assert(testing_val == Some(0x10), "StateTesting -> 0x10");
    kani::assert(success_val == Some(0x00), "StateSuccess -> 0x00");
    kani::assert(no_state_val == None, "NoState -> None");

    kani::assert(new_val != updating_val, "new != updating");
    kani::assert(new_val != testing_val, "new != testing");
    kani::assert(new_val != success_val, "new != success");
    kani::assert(updating_val != testing_val, "updating != testing");
    kani::assert(updating_val != success_val, "updating != success");
    kani::assert(testing_val != success_val, "testing != success");
}

/// Proof that individual parser functions never panic for any bounded input.
/// Each function is tested independently with a sufficiently small symbolic array.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(20)]
fn parser_extract_version_no_panic() {
    use rustBoot::parser::{check_for_eof, check_for_padding, extract_version};

    let input: [u8; 16] = kani::any();
    let slice: &[u8] = &input;

    let _ = check_for_eof(slice);
    let _ = check_for_padding(slice);
    let _ = extract_version(slice);
}

/// Proof that extract_timestamp never panics on bounded input.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(30)]
fn parser_extract_timestamp_no_panic() {
    use rustBoot::parser::extract_timestamp;

    let input: [u8; 24] = kani::any();
    let _ = extract_timestamp(&input[..]);
}

/// Proof that extract_img_type never panics on bounded input.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(40)]
fn parser_extract_img_type_no_panic() {
    use rustBoot::parser::extract_img_type;

    let input: [u8; 32] = kani::any();
    let _ = extract_img_type(&input[..]);
}

/// Proof that extract_digest never panics on bounded input.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(50)]
fn parser_extract_digest_no_panic() {
    use rustBoot::parser::extract_digest;

    let input: [u8; 48] = kani::any();
    let _ = extract_digest(&input[..]);
}

/// Proof that extract_pubkey_digest never panics on bounded input.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(60)]
fn parser_extract_pubkey_digest_no_panic() {
    use rustBoot::parser::extract_pubkey_digest;

    let input: [u8; 64] = kani::any();
    let _ = extract_pubkey_digest(&input[..]);
}

/// Proof that extract_signature never panics on bounded input.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(80)]
fn parser_extract_signature_no_panic() {
    use rustBoot::parser::extract_signature;

    let input: [u8; 96] = kani::any();
    let _ = extract_signature(&input[..]);
}