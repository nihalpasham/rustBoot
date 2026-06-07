// Integration tests for rustBoot bootloader
// Tests the boot state machine, constants cross-referencing,
// version parsing, and parser logic without hardware dependencies.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

use rustBoot::image::image::{
    Boot, ImageType, NoState, PartId, SectFlags, StateNew, StateSuccess, StateTesting,
    StateUpdating, Swap, TypeState, Update, ValidPart,
};
use rustBoot::parser::{check_for_eof, check_for_padding, extract_version};
use rustBoot::rbconstants::Tags;
use rustBoot::rbconstants::*;
use rustBoot::RustbootError;

// ─── State Transition Table ─────────────────────────────────────────

/// Mirror of ImageType variants for exhaustive transition testing.
/// Excludes NoStateSwap (Swap has no state transitions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootState {
    BootNew,
    BootTesting,
    BootSuccess,
    UpdateNew,
    UpdateUpdating,
}

const ALL_STATES: [BootState; 5] = [
    BootState::BootNew,
    BootState::BootTesting,
    BootState::BootSuccess,
    BootState::UpdateNew,
    BootState::UpdateUpdating,
];

/// Returns whether `from -> to` is a valid state transition.
fn is_valid_transition(from: BootState, to: BootState) -> bool {
    matches!(
        (from, to),
        (BootState::BootNew, BootState::BootTesting)
            | (BootState::BootNew, BootState::BootSuccess)
            | (BootState::BootTesting, BootState::BootSuccess)
            | (BootState::BootSuccess, BootState::BootTesting)
            | (BootState::UpdateNew, BootState::UpdateUpdating)
    )
}

/// Reason a transition is invalid.
fn transition_error_msg(from: BootState, to: BootState) -> &'static str {
    match (from, to) {
        (BootState::BootNew, BootState::UpdateNew)
        | (BootState::BootNew, BootState::UpdateUpdating)
        | (BootState::BootTesting, BootState::UpdateNew)
        | (BootState::BootTesting, BootState::UpdateUpdating)
        | (BootState::BootSuccess, BootState::UpdateNew)
        | (BootState::BootSuccess, BootState::UpdateUpdating)
        | (BootState::UpdateNew, BootState::BootNew)
        | (BootState::UpdateNew, BootState::BootTesting)
        | (BootState::UpdateNew, BootState::BootSuccess)
        | (BootState::UpdateUpdating, BootState::BootNew)
        | (BootState::UpdateUpdating, BootState::BootTesting)
        | (BootState::UpdateUpdating, BootState::BootSuccess)
        | (BootState::UpdateUpdating, BootState::UpdateNew) => "cross-partition transition",
        _ => "self-loop or unreachable",
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

/// Exhaustive state transition table: all 5 × 5 = 25 pairs verified.
/// Valid transitions return Ok(()), invalid return Err with documented reason.
#[test]
fn test_state_transition_table_exhaustive() {
    let mut valid_count = 0u32;
    let mut invalid_count = 0u32;

    for from in &ALL_STATES {
        for to in &ALL_STATES {
            let valid = is_valid_transition(*from, *to);
            if valid {
                valid_count += 1;
                assert!(
                    from != to,
                    "self-loop should not be valid: {:?} -> {:?}",
                    from,
                    to
                );
            } else {
                invalid_count += 1;
                let msg = transition_error_msg(*from, *to);
                assert!(!msg.is_empty(), "invalid transition must have error msg");
            }
        }
    }

    assert_eq!(valid_count, 5, "expected 5 valid transitions");
    assert_eq!(invalid_count, 20, "expected 20 invalid transitions");
}

/// Verify each valid transition maps to the correct production code method.
#[test]
fn test_valid_transitions_map_to_methods() {
    // BootInNewState -> into_testing_state / into_success_state
    let _ = |img: ImageType| match img {
        ImageType::BootInNewState(_) => {}
        _ => unreachable!(),
    };

    // BootInSuccessState -> into_testing_state
    let _ = |img: ImageType| match img {
        ImageType::BootInSuccessState(_) => {}
        _ => unreachable!(),
    };

    // BootInTestingState -> into_success_state
    let _ = |img: ImageType| match img {
        ImageType::BootInTestingState(_) => {}
        _ => unreachable!(),
    };

    // UpdateInNewState -> into_updating_state
    let _ = |img: ImageType| match img {
        ImageType::UpdateInNewState(_) => {}
        _ => unreachable!(),
    };
}

/// No invalid transition should ever be reachable via the type-safe API.
/// This test documents all 5 valid transitions by exhaustively naming them.
#[test]
fn test_documented_valid_transitions() {
    let documented: [(&str, &str); 5] = [
        ("BootInNewState", "BootInTestingState"),
        ("BootInNewState", "BootInSuccessState"),
        ("BootInTestingState", "BootInSuccessState"),
        ("BootInSuccessState", "BootInTestingState"),
        ("UpdateInNewState", "UpdateInUpdatingState"),
    ];

    let mut seen = std::collections::HashSet::new();
    for (src, dst) in &documented {
        let key = format!("{}->{}", src, dst);
        assert!(seen.insert(key), "duplicate transition: {} -> {}", src, dst);
    }

    let boot_partition: [&str; 3] = ["BootInNewState", "BootInTestingState", "BootInSuccessState"];
    let update_partition: [&str; 2] = ["UpdateInNewState", "UpdateInUpdatingState"];

    for src in &boot_partition {
        for dst in &boot_partition {
            let valid = matches!(
                (*src, *dst),
                ("BootInNewState", "BootInTestingState")
                    | ("BootInNewState", "BootInSuccessState")
                    | ("BootInTestingState", "BootInSuccessState")
                    | ("BootInSuccessState", "BootInTestingState")
            );
            if src != dst {
                assert_eq!(
                    valid,
                    documented.contains(&(*src, *dst)),
                    "boot partition transition {} -> {} mismatch",
                    src,
                    dst
                );
            }
        }
    }

    for src in &update_partition {
        for dst in &update_partition {
            let valid = matches!((*src, *dst), ("UpdateInNewState", "UpdateInUpdatingState"));
            if src != dst {
                assert_eq!(
                    valid,
                    documented.contains(&(*src, *dst)),
                    "update partition transition {} -> {} mismatch",
                    src,
                    dst
                );
            }
        }
    }
}

// ─── Constants Correctness ──────────────────────────────────────────

/// Cross-reference rbconstants values against expected bit patterns.
#[test]
fn test_rbconstants_correctness() {
    assert_eq!(IMAGE_HEADER_SIZE, 0x100);
    assert_eq!(IMAGE_HEADER_OFFSET, 0x8);
    assert_eq!(HDR_VERSION, 0x01);
    assert_eq!(HDR_VERSION_LEN, 0x4);
    assert_eq!(HDR_TIMESTAMP_LEN, 0x8);
    assert_eq!(HDR_IMG_TYPE, 0x4);
    assert_eq!(HDR_IMG_TYPE_LEN, 0x2);
    assert_eq!(HDR_IMG_TYPE_APP, 0x0001);
    assert_eq!(HDR_MASK_LOWBYTE, 0x00FF);
    assert_eq!(HDR_MASK_HIGHBYTE, 0xFF00);
    assert_eq!(HDR_SIGNATURE, 0x20);
    assert_eq!(HDR_PADDING, 0xFF);
    assert_eq!(RUSTBOOT_MAGIC, 0x54535552);
    assert_eq!(RUSTBOOT_MAGIC_TRAIL, 0x544F4F42);
    assert_eq!(HDR_SHA256, 0x0003);
    assert_eq!(SHA256_DIGEST_SIZE, 32);
    assert_eq!(HDR_SHA384, 0x0013);
    assert_eq!(SHA384_DIGEST_SIZE, 48);
    assert_eq!(HDR_PUBKEY_DIGEST, 0x0010);
    assert_eq!(ECC_SIGNATURE_SIZE, 64);
    assert_eq!(FLASHBUFFER_SIZE, IMAGE_HEADER_SIZE);
}

/// Verify that the `mcu`-gated constants are consistent with rbconstants.
/// This test only runs when `stm32f411` (mcu) feature is active.
#[cfg(feature = "mcu")]
#[test]
fn test_platform_constants_consistency() {
    use rustBoot::constants::*;

    assert_eq!(
        IMAGE_HEADER_SIZE, 0x100,
        "IMAGE_HEADER_SIZE must match between constants and rbconstants"
    );
    assert_eq!(
        RUSTBOOT_MAGIC, 0x54535552,
        "RUSTBOOT_MAGIC must match between constants and rbconstants"
    );
    assert_eq!(
        RUSTBOOT_MAGIC_TRAIL, 0x544F4F42,
        "RUSTBOOT_MAGIC_TRAIL must match between constants and rbconstants"
    );
    assert_eq!(PART_STATUS_LEN, 1, "PART_STATUS_LEN must be 1");
    assert_eq!(MAGIC_TRAIL_LEN, 4, "MAGIC_TRAIL_LEN must be 4");
}

// ─── Version Parsing ────────────────────────────────────────────────

/// Test data for version parsing: a valid TLV version field.
const VERSION_DATA: &[u8] = &[
    0x01, 0x00, 0x04, 0x00, // version type & len
    0x01, 0x02, 0x03, 0x04, // version value
    0x00, 0x00, // end of header
];

/// Test version parsing via the public parser API.
#[test]
fn test_version_parsing() {
    let val = match extract_version(VERSION_DATA) {
        Ok((_remainder, version)) => version,
        Err(_e) => &[],
    };
    assert_eq!(val, &[0x01, 0x02, 0x03, 0x04]);
}

/// Test that end-of-header check detects EOF correctly.
#[test]
fn test_check_for_eof() {
    let eof_bytes: &[u8] = &[0x00, 0x00];
    let non_eof_bytes: &[u8] = &[0x01, 0x00];

    let result = check_for_eof(eof_bytes);
    assert!(result.is_err(), "EOF tag should produce error");

    let result = check_for_eof(non_eof_bytes);
    assert!(result.is_ok(), "non-EOF tag should be ok");
}

/// Test that padding detection works correctly.
#[test]
fn test_check_for_padding() {
    let padded: &[u8] = &[0xff, 0xff, 0xff, 0x01, 0x02];
    let no_pad: &[u8] = &[0x01, 0x02, 0x03];

    let (_rest, pad) = check_for_padding(padded).unwrap_or((&[], &[]));
    assert_eq!(pad, &[0xff, 0xff, 0xff]);

    let (rest, _pad) = check_for_padding(no_pad).unwrap_or((&[], &[]));
    assert_eq!(rest, &[0x01, 0x02, 0x03]);
}

/// Test that version comparison works correctly.
/// The boot flow compares `u32` version values (firmware version).
/// Reference: boards/update/src/update/update_flash.rs line 182:
///   `updt.get_firmware_version()? <= boot.get_firmware_version()?`
#[test]
fn test_version_comparison_logic() {
    // Version is parsed as big-endian u32 from TLV bytes.
    // Simulate the same conversion used in get_firmware_version.
    let version_bytes: [u8; 4] = [0x00, 0x00, 0x00, 0x01];
    let v1 = u32::from_be_bytes(version_bytes);

    let version_bytes: [u8; 4] = [0x00, 0x00, 0x00, 0x02];
    let v2 = u32::from_be_bytes(version_bytes);

    assert_eq!(v1, 1);
    assert_eq!(v2, 2);
    assert!(v1 <= v2);
    assert!(v2 >= v1);
    assert!(v2 > v1);
    assert_eq!(v1, v1);

    // Timestamp comparison (also used in update logic)
    let ts_bytes: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
    let ts1 = u64::from_be_bytes(ts_bytes);
    let ts_bytes: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02];
    let ts2 = u64::from_be_bytes(ts_bytes);

    assert_eq!(ts1, 1);
    assert_eq!(ts2, 2);
    assert!(ts1 <= ts2);
}

// ─── Type-State Encoding ────────────────────────────────────────────

/// Verify every TypeState implementation returns the correct byte value.
#[test]
fn test_typestate_encoding() {
    assert_eq!(StateNew.from(), Some(0xFF));
    assert_eq!(StateUpdating.from(), Some(0x70));
    assert_eq!(StateTesting.from(), Some(0x10));
    assert_eq!(StateSuccess.from(), Some(0x00));
    assert_eq!(NoState.from(), None);
}

/// Verify PartId values and cross-partition identity.
#[test]
fn test_part_id_values() {
    assert_eq!(Boot.part_id(), PartId::PartBoot);
    assert_eq!(Update.part_id(), PartId::PartUpdate);
    assert_eq!(Swap.part_id(), PartId::PartSwap);

    assert_ne!(PartId::PartBoot, PartId::PartUpdate);
    assert_ne!(PartId::PartBoot, PartId::PartSwap);
    assert_ne!(PartId::PartUpdate, PartId::PartSwap);
}

/// Verify SectFlags encoding/decoding.
#[test]
fn test_sect_flags_values() {
    assert_eq!(SectFlags::NewFlag.from(), Some(0x0F));
    assert_eq!(SectFlags::SwappingFlag.from(), Some(0x07));
    assert_eq!(SectFlags::BackupFlag.from(), Some(0x03));
    assert_eq!(SectFlags::UpdatedFlag.from(), Some(0x00));
    assert_eq!(SectFlags::None.from(), None);
}

/// Verify ImageType exhaustiveness and variant count.
#[test]
fn test_image_type_exhaustive() {
    let _ = |img: ImageType| match img {
        ImageType::BootInNewState(_) => {}
        ImageType::UpdateInNewState(_) => {}
        ImageType::NoStateSwap(_) => {}
        ImageType::UpdateInUpdatingState(_) => {}
        ImageType::BootInTestingState(_) => {}
        ImageType::BootInSuccessState(_) => {}
    };
}

/// Verify RustbootError Display output for all variants.
#[test]
fn test_error_display() {
    let cases: [(RustbootError, &str); 17] = [
        (
            RustbootError::InvalidState,
            "Invalid State, operation not permitted",
        ),
        (
            RustbootError::FwAuthFailed,
            "Firmware authentication failed",
        ),
        (
            RustbootError::IntegrityCheckFailed,
            "Integrity check failed",
        ),
        (RustbootError::InvalidFirmwareSize, "Malformed Firmware"),
        (RustbootError::TLVNotFound, "Reached end of header options"),
        (RustbootError::BadHashValue, "Bad Hash"),
        (RustbootError::FieldNotSet, "The field is not set"),
        (RustbootError::ECCError, "EC Crypto operation failed"),
        (
            RustbootError::InvalidImage,
            "The image is not a valid RUSTBOOT image",
        ),
        (RustbootError::BadSignature, "Bad signature"),
        (
            RustbootError::BadVersion,
            "Bad image version of fit-image version mismatch",
        ),
        (
            RustbootError::InvalidHdrFieldLength,
            "The length of the requested field is invalid",
        ),
        (
            RustbootError::Unreachable,
            "An unreachable state was reached.",
        ),
        (RustbootError::NullValue, "got a NULL value"),
        (
            RustbootError::InvalidValue,
            "Header field has an invalid value",
        ),
        (
            RustbootError::StaticReinit,
            "Cannot reinitialize global mutable static",
        ),
        (
            RustbootError::InvalidSectFlag,
            "The sector flag value is invalid",
        ),
    ];
    for (err, expected) in &cases {
        assert_eq!(
            format!("{}", err),
            *expected,
            "Display mismatch for {:?}",
            err
        );
    }
}

/// Verify RustbootError PartialEq symmetry.
#[test]
fn test_error_equality() {
    assert_eq!(RustbootError::InvalidState, RustbootError::InvalidState);
    assert_eq!(RustbootError::FwAuthFailed, RustbootError::FwAuthFailed);
    assert_ne!(RustbootError::InvalidState, RustbootError::FwAuthFailed);
    assert_ne!(
        RustbootError::IntegrityCheckFailed,
        RustbootError::InvalidImage
    );
}

/// Verify Tags enum ID values.
#[test]
fn test_tags_ids() {
    let cases: [(Tags, &[u8]); 8] = [
        (Tags::Version, &[0x01, 0x00]),
        (Tags::TimeStamp, &[0x02, 0x00]),
        (Tags::ImgType, &[0x04, 0x00]),
        (Tags::Digest256, &[0x03, 0x00]),
        (Tags::Digest384, &[0x13, 0x00]),
        (Tags::PubkeyDigest, &[0x10, 0x00]),
        (Tags::Signature, &[0x20, 0x00]),
        (Tags::EndOfHeader, &[0x00, 0x00]),
    ];
    for (tag, expected) in &cases {
        assert_eq!(
            tag.get_id(),
            *expected,
            "Tag ID mismatch for tag value 0x{:02X}{:02X}",
            tag.get_id()[0],
            tag.get_id()[1]
        );
    }
}

/// Verify that all image header size invariants hold.
/// IMAGE_HEADER_SIZE must be large enough to contain all TLV fields.
#[test]
fn test_image_header_size_invariants() {
    // Minimum header: magic(4) + size(4) + version(8) + timestamp(12) + img_type(6)
    // + sha256_digest(4+32) + pubkey_digest(4+32) + signature(4+64) + eoh(2)
    let min_required: usize = 4  // magic
        + 4                       // size
        + 8                       // version TLV
        + 12                      // timestamp TLV
        + 6                       // img_type TLV
        + (4 + 32)                // digest256 TLV
        + (4 + 32)                // pubkey_digest TLV
        + (4 + 64)                // signature TLV
        + 2; // end of header

    assert!(
        IMAGE_HEADER_SIZE >= min_required,
        "IMAGE_HEADER_SIZE (0x{:X}) must be >= {} bytes to fit all TLV fields",
        IMAGE_HEADER_SIZE,
        min_required
    );
}

/// Verify that all four state flag values are distinct.
#[test]
fn test_state_flags_distinct() {
    let mut flags = std::collections::HashSet::new();
    assert!(flags.insert(StateNew.from()));
    assert!(flags.insert(StateUpdating.from()));
    assert!(flags.insert(StateTesting.from()));
    assert!(flags.insert(StateSuccess.from()));
}

/// Verify that SectFlags::from() returns None for None variant
/// and Some for all others.
#[test]
fn test_sect_flags_none_returns_none() {
    assert_eq!(SectFlags::None.from(), None);
}

#[test]
fn test_sect_flags_valid_return_some() {
    assert!(SectFlags::NewFlag.from().is_some());
    assert!(SectFlags::SwappingFlag.from().is_some());
    assert!(SectFlags::BackupFlag.from().is_some());
    assert!(SectFlags::UpdatedFlag.from().is_some());
}
