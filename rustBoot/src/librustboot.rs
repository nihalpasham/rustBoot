#![no_std]
#![allow(warnings)]

use core::convert::TryInto;
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
    None,
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

#[derive(Clone, Copy)]
pub enum Tags {
    Version,
    TimeStamp,
    ImgType,
    Digest256,
    Digest384,
    Signature,
    EndOfHeader,
}

impl Tags {
    #[rustfmt::skip]
    fn get_id(self) -> &'static [u8] {
        match self {
            Self::Version       => &[0x00, 0x01],
            Self::TimeStamp     => &[0x00, 0x02],
            Self::ImgType       => &[0x00, 0x04],
            Self::Digest256     => &[0x00, 0x03],
            Self::Digest384     => &[0x00, 0x13],
            Self::Signature     => &[0x00, 0x20],
            Self::EndOfHeader   => &[0x00, 0x00],
        }
    }
}

use nom::bytes::complete::take_while;
use nom::bytes::complete::{tag, take};
use nom::lib;
use nom::{
    error::{Error, ErrorKind},
    Err, IResult,
};

// use libc_print::libc_println;

fn check_for_eof(input: &[u8]) -> IResult<&[u8], &[u8]> {
    match tag::<_, _, Error<&[u8]>>(Tags::EndOfHeader.get_id())(input) {
        Ok((tail, eof)) => Err(Err::Error(Error::new(input, ErrorKind::Eof))),
        Err(_e) => Ok((input, &[])),
    }
}

fn check_for_padding(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let res = take_while::<_, _, Error<&[u8]>>(|pad_byte| pad_byte == 0xff)(input)?;
    Ok(res)
}

fn extract_version<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (input, _) = check_for_eof(input)?;
    let (input, _) = check_for_padding(input)?;
    let (tail, version) = take(8u32)(input)?;
    let (_, version_check) = take(2u32)(version)?;
    if version_check == Tags::Version.get_id() {
        Ok((tail, &version[4..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_timestamp<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (tail, _) = extract_version(input)?;
    let (tail, _) = check_for_eof(tail)?;
    let (tail, _) = check_for_padding(tail)?;
    let (tail, timestamp) = take(12u32)(tail)?;
    let (_, timestamp_check) = take(2u32)(timestamp)?;
    if timestamp_check == Tags::TimeStamp.get_id() {
        Ok((tail, &timestamp[4..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_img_type<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (tail, _) = extract_timestamp(input)?;
    let (tail, _) = check_for_eof(tail)?;
    let (tail, _) = check_for_padding(tail)?;
    let (tail, img_type) = take(6u32)(tail)?;
    let (_, img_type_check) = take(2u32)(img_type)?;
    if img_type_check == Tags::ImgType.get_id() {
        Ok((tail, &img_type[4..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_digest<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (tail, _) = extract_img_type(input)?;
    let (tail, _) = check_for_eof(tail)?;
    let (tail, _) = check_for_padding(tail)?;
    let (tail, typelen) = take(4u32)(tail)?;
    let len = (typelen[3] as u16 | (typelen[2] as u16) << 8) as u32;
    let (tail, digest) = take(len)(tail)?;
    let (_, digest_check) = take(2u32)(typelen)?;
    if digest_check == Tags::Digest256.get_id() || digest_check == Tags::Digest384.get_id() {
        Ok((tail, &digest[..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_signature<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (tail, _) = extract_digest(input)?;
    let (tail, _) = check_for_eof(tail)?;
    let (tail, _) = check_for_padding(tail)?;
    let (tail, typelen) = take(4u32)(tail)?;
    let len = (typelen[3] as u16 | (typelen[2] as u16) << 8) as u32;
    let (tail, signature) = take(len)(tail)?;
    let (_, signature_check) = take(2u32)(typelen)?;
    if signature_check == Tags::Signature.get_id() {
        Ok((tail, &signature[..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

    const PAD1: &[u8] = &[0x20, 0x01, 0xff, 0x02, 0x03];
    const PAD2: &[u8] = &[0xff, 0xff, 0xff, 0x02, 0x03];

    #[rustfmt::skip]
    const DATA: &[u8] = &[
        // 0x54, 0x53, 0x55, 0x52, // magic
        // 0x65, 0x51, 0x48, 0x54, // size
        0x00, 0x01, 0x00, 0x04, // version type & len
        0x01, 0x02, 0x03, 0x04, // version value

        0xff, 0xff, 0xff, 0xff, // padding bytes

        0x00, 0x02, 0x00, 0x08, // timestamp type & len
        0x11, 0x11, 0x11, 0x11, // timestamp value
        0x22, 0x22, 0x22, 0x22, 

        0x00, 0x04, 0x00, 0x02, // img type and len
        0x02, 0x00, 

        0xff, 0xff, 0xff, 0xff, // padding bytes
        0xff, 0xff,

        // 32 byte digest type and len
        0x00, 0x03, 0x00, 0x20, 
        // digest value
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        
        // signature type and len
        0x00, 0x20, 0x00, 0x40, 
        // signature value
        0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
        0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
        0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
        0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
        0x44, 0x44, 0x44, 0x44, 

        // end of header
        0x00, 0x00, 
    ];

    #[test]
    fn padding_test() {
        let val = match check_for_padding(PAD1) {
            Ok((tail, val)) => {
                libc_println!("incorrect padding: {:?}", tail);
                tail
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x20, 0x01, 0xff, 0x02, 0x03]);

        let val = match check_for_padding(PAD2) {
            Ok((tail, val)) => {
                libc_println!("padding: {:?}", val);
                val
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0xff, 0xff, 0xff])
    }

    #[test]
    fn parse_version() {
        let val = match extract_version(DATA) {
            Ok((tail, version)) => {
                libc_println!("version: {:?}", version);
                version
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x01, 0x02, 0x03, 0x04])
    }

    #[test]
    fn parse_timestamp() {
        let val = match extract_timestamp(DATA) {
            Ok((tail, timestamp)) => {
                libc_println!("timestamp: {:?}", timestamp);
                timestamp
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22])
    }

    #[test]
    fn parse_signature() {
        let val = match extract_signature(DATA) {
            Ok((tail, signature)) => {
                libc_println!("signature: {:?}", signature);
                signature
            }
            Err(_e) => &[],
        };
        assert_eq!(
            val,
            &[
                0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44,
            ]
        )
    }
}
