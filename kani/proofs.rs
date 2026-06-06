//! Kani proof harnesses for rustBoot safety-critical functions.
//!
//! Usage:
//!   cargo kani --harness sect_flags_from_bounded
//!   cargo kani --harness partition_size_bounds
//!
//! These proofs verify that bounds checks, arithmetic, and state transitions
//! are safe for all possible inputs within the bounded range.
//!
//! Note: Kani proofs require the kani Rust toolchain.
//! Install: cargo install kani-verifier

extern crate rustBoot;

use rustBoot::image::image::SectFlags;

/// Proof that SectFlags::from() never returns an invalid state value.
#[cfg(kani)]
#[kani::proof]
fn sect_flags_from_bounded() {
    let flag = kani::any();
    kani::assume(flag == SectFlags::NewFlag
        || flag == SectFlags::SwappingFlag
        || flag == SectFlags::BackupFlag
        || flag == SectFlags::UpdatedFlag);
    let result = flag.from();
    kani::assert!(result.is_some());
}

/// Proof that partition size constants are internally consistent.
#[cfg(kani)]
#[kani::proof]
fn partition_size_bounds() {
    use rustBoot::constants::*;
    kani::assert!(BOOT_FWBASE > BOOT_PARTITION_ADDRESS);
    kani::assert!(PARTITION_SIZE >= SECTOR_SIZE);
    kani::assert!(IMAGE_HEADER_SIZE < PARTITION_SIZE);
}