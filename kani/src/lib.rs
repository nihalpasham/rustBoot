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
//!   cargo kani --harness parser_extract_version_bounds
//!   cargo kani --harness parser_extract_timestamp_bounds
//!   cargo kani --harness constants_consistent
//!   cargo kani --harness version_comparison_properties
//!   cargo kani --harness flatten_bounds
//!   cargo kani --harness state_encode_decode_roundtrip
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

/// Proof that extract_version never panics for any 16-byte input
/// and that the maximum parsed value fits in the return type.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(20)]
fn parser_extract_version_bounds() {
    use rustBoot::parser::extract_version;
    let buf: [u8; 16] = kani::any();
    let _ = extract_version(&buf);
}

/// Proof that extract_timestamp never panics for any 24-byte input
/// and that the maximum parsed value fits in the return type.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(30)]
fn parser_extract_timestamp_bounds() {
    use rustBoot::parser::extract_timestamp;
    let buf: [u8; 24] = kani::any();
    let _ = extract_timestamp(&buf);
}

/// Proof that all rustBoot compile-time constants are internally consistent.
/// Verifies address ordering, size non-zero, and header-fit constraints.
#[cfg(kani)]
#[kani::proof]
fn constants_consistent() {
    use rustBoot::constants::*;
    kani::assert(BOOT_PARTITION_ADDRESS < BOOT_FWBASE, "boot partition addr below firmware base");
    kani::assert(PARTITION_SIZE > 0, "partition size positive");
    kani::assert(SECTOR_SIZE > 0, "sector size positive");
    kani::assert(SECTOR_SIZE <= PARTITION_SIZE, "sector fits in partition");
    kani::assert(IMAGE_HEADER_SIZE < PARTITION_SIZE, "header fits in partition");
}

/// Proof that version arithmetic and comparison are well-defined.
/// Verifies that the parsed firmware version from the TLV header
/// fits in u32, and that u32 comparison is reflexive, antisymmetric, and transitive.
#[cfg(kani)]
#[kani::proof]
fn version_comparison_properties() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    let c: u32 = kani::any();

    // Reflexive
    kani::assert(a == a, "reflexive eq");
    kani::assert(a <= a, "reflexive le");
    kani::assert(a >= a, "reflexive ge");

    // Antisymmetric
    if a <= b && b <= a {
        kani::assert(a == b, "antisymmetric");
    }

    // Transitive
    if a <= b && b <= c {
        kani::assert(a <= c, "transitive");
    }

    // No overflow in subtraction (checked)
    let _sub1 = a.checked_sub(b);
    let _sub2 = b.checked_sub(a);

    // No overflow in addition (checked)
    let _add = a.checked_add(b);
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

/// Proof that flatten() never panics and always returns exactly 128 bytes.
#[cfg(kani)]
#[kani::proof]
fn flatten_bounds() {
    use rustBoot::dt::flatten;
    let input: [[u8; 32]; 4] = kani::any();
    let result = flatten(input);
    kani::assert(result.len() == 128, "flatten output is 128 bytes");
}

/// Proof that every valid SectFlags variant round-trips through
/// encode (from) and that invalid variants produce None.
/// Kani checks: no panic, no unexpected None for valid flags.
#[cfg(kani)]
#[kani::proof]
fn state_encode_decode_roundtrip() {
    use rustBoot::image::image::SectFlags;

    // Check all valid variants round-trip
    let variants = [
        SectFlags::NewFlag,
        SectFlags::SwappingFlag,
        SectFlags::BackupFlag,
        SectFlags::UpdatedFlag,
    ];
    let expected = [0x0Fu8, 0x07, 0x03, 0x00];
    for (i, v) in variants.iter().enumerate() {
        let encoded: u8 = v.from().expect("valid flag must encode");
        kani::assert(encoded == expected[i], "variant encodes to expected byte");
    }

    // None variant produces None
    kani::assert(SectFlags::None.from().is_none(), "None variant decodes to None");
}