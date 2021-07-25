#![no_std]
#![allow(warnings)]

use core::mem::size_of;
use core::usize;

use crate::image::image::{PartId, RustbootImage, Swappable, TypeState, ValidPart};
use crate::target::*;
use crate::{Result, RustbootError};

pub const IMAGE_HEADER_SIZE: usize = 0x100;
pub const IMAGE_HEADER_OFFSET: usize = 0x8;

pub const HDR_IMG_TYPE: u16 = 0x4;
pub const HDR_IMG_TYPE_APP: u16 = 0x0001;
pub const HDR_MASK_LOWBYTE: u16 = 0x00FF;
pub const HDR_MASK_HIGHBYTE: u16 = 0xFF00;
pub const HDR_SIGNATURE: u16 = 0x20;
pub const HDR_PADDING: u8 = 0xFF;

pub const SECT_FLAG_NEW: u8 = 0x0F;

/// Enumerated BOOT partition
pub const PART_BOOT: u8 = 0x0;
pub const BOOT_TRAILER_ADDRESS: usize = BOOT_PARTITION_ADDRESS + PARTITION_SIZE;
pub const BOOT_FWBASE: usize = BOOT_PARTITION_ADDRESS + IMAGE_HEADER_SIZE;
/// Enumerated UPDATE partition
pub const PART_UPDATE: u8 = 0x1;
pub const UPDATE_TRAILER_ADDRESS: usize = UPDATE_PARTITION_ADDRESS + PARTITION_SIZE;
pub const UPDATE_FWBASE: usize = UPDATE_PARTITION_ADDRESS + IMAGE_HEADER_SIZE;
/// Enumerated SWAP partition
pub const PART_SWAP: u8 = 0x2;
pub const SWAP_BASE: usize = SWAP_PARTITION_ADDRESS;

pub const RUSTBOOT_MAGIC: usize = 0x54535552; // RUST
pub const RUSTBOOT_MAGIC_TRAIL: usize = 0x544F4F42; // BOOT

/*  Hash Config */

// SHA256 constants
pub const HDR_SHA256: u16 = 0x0003;
pub const SHA256_DIGEST_SIZE: usize = 32;
// SHA384 constants
pub const HDR_SHA384: u16 = 0x0013;
pub const SHA384_DIGEST_SIZE: usize = 48;

// NVM_FLASH_WRITEONCE
#[cfg(feature = "ext_flash")]
pub const FLASHBUFFER_SIZE: usize = SECTOR_SIZE;
pub const FLASHBUFFER_SIZE: usize = IMAGE_HEADER_SIZE;

/* Signature Config */
pub const ECC_SIGNATURE_SIZE: usize = 64;

// EC256 constants
#[cfg(feature = "secp256k1")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0200;
// pub const N: 
// ED25519 constants
#[cfg(feature = "ed25519")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0100;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SectFlags {
    NewFlag,
    SwappingFlag,
    BackupFlag,
    UpdatedFlag,
    None
}

impl SectFlags {

    pub fn has_new_flag(&self) -> bool {
        self == &SectFlags::NewFlag
    }

    pub fn has_swapping_flag(&self) -> bool {
        self == &SectFlags::SwappingFlag
    }

    pub fn has_backup_flag(&self) -> bool {
        self == &SectFlags::BackupFlag
    }

    pub fn has_updated_flag(&self) -> bool {
        self == &SectFlags::UpdatedFlag
    }

    pub fn set_swapping_flag(mut self) -> Self {
        self = SectFlags::SwappingFlag;
        self
    }

    pub fn set_backup_flag(mut self) -> Self {
        self = SectFlags::BackupFlag;
        self
    }

    pub fn set_updated_flag(mut self) -> Self {
        self = SectFlags::UpdatedFlag;
        self
    }
}
/// A function to parse a valid `boot or update` image header for a given `TLV`. It
/// takes as input a ref to [`RustbootImage`] i.e. a valid image/partition and `type_field`.
///
/// Returns a tuple containing 
/// - the ref to the val, 
/// - the length and 
/// - a counter which represents the byte-position of the TLV from `start of header`.
pub(crate) fn parse_image_header<'a, Part: ValidPart + Swappable, State: TypeState>(
    img: &RustbootImage<Part, State>,
    type_field: u16,
) -> Result<(&'a [u8], u16, usize)> {
    let mut err = RustbootError::__Nonexhaustive;
    let part_desc = img.part_desc.get().unwrap();
    if let Some(val) = part_desc.hdr {
        let hdr_val = (unsafe { (val as *const [u8; IMAGE_HEADER_SIZE]).as_ref() })
            .ok_or(RustbootError::NullValue)?;
        // start parsing from the start after the 8th byte of the header
        let mut counter: usize = 8;
        while (counter + 4) < hdr_val.len() {
            if (hdr_val[counter] == 0 && hdr_val[counter + 1] == 0) {
                err = RustbootError::TLVNotFound;
                break;
            }
            if (hdr_val[counter] == HDR_PADDING) {
                counter += 1;
            }
            if ((counter & 0x01) != 0) {
                counter += 1;
            }
            let len = hdr_val[counter + 2] as u16 | (hdr_val[counter + 3] as u16) << 8;
            if ((4 + len) > ((IMAGE_HEADER_SIZE - IMAGE_HEADER_OFFSET) as u16)) {
                err = RustbootError::InvalidHdrFieldLength;
                break;
            }
            if (counter + 4 + len as usize) > hdr_val.len() {
                err = RustbootError::InvalidHdrFieldLength;
                break;
            }
            if hdr_val[counter] as u16 | (hdr_val[counter + 1] as u16) << 8 == type_field {
                return Ok((
                    &hdr_val[(counter + 4)..(counter + 4 + len as usize)],
                    len,
                    counter,
                ));
            }
            counter += (4 + len as usize);
        }
    }
    Err(err)
}


