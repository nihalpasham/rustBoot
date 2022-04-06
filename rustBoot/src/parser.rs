use core::usize;

use crate::constants::*;
use crate::image::image::{RustbootImage, Swappable, TypeState, ValidPart};
use crate::{Result, RustbootError};

/// A function to parse the image-header contained in a `boot or update` partition, for a given `TLV`. It
/// takes as input a ref to [`RustbootImage`] and a [`Tags`] variant.
///
/// Returns a slice containing the value
pub(crate) fn parse_tlv<'a, Part: ValidPart + Swappable, State: TypeState>(
    img: &RustbootImage<Part, State>,
    type_field: Tags,
) -> Result<&'a [u8]> {
    let part_desc = img.part_desc.get().unwrap();
    if let Some(val) = part_desc.hdr {
        let mut header_bytes: &[u8] = (unsafe { (val as *const [u8; IMAGE_HEADER_SIZE]).as_ref() })
            .ok_or(RustbootError::__Nonexhaustive)?;
        // we've checked `magic` and `size` fields of the header during init
        // start parsing from the 8th byte of the header
        header_bytes = &header_bytes[8..];
        let value = match type_field {
            Tags::Version => {
                let (_, version) =
                    extract_version(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                version
            }
            Tags::TimeStamp => {
                let (_, timestamp) =
                    extract_timestamp(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                timestamp
            }
            Tags::ImgType => {
                let (_, img_type) =
                    extract_img_type(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                img_type
            }
            Tags::Digest256 => {
                let (_, digest256) =
                    extract_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                digest256
            }
            Tags::Digest384 => {
                let (_, digest384) =
                    extract_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                digest384
            }
            Tags::PubkeyDigest => {
                let (_, pubkey_digest) =
                    extract_pubkey_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                pubkey_digest
            }
            Tags::Signature => {
                let (_, signature) =
                    extract_signature(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                signature
            }
            Tags::EndOfHeader => todo!(),
        };
        Ok(value)
    } else {
        Err(RustbootError::__Nonexhaustive)
    }
}

/// Returns an offset value for the supplied [`Tags`] variant.
///
/// *Note: offset represents the index/byte-position of a `TLV` from `start of image-header`.*
pub(crate) fn get_tlv_offset<'a, Part: ValidPart + Swappable, State: TypeState>(
    img: &RustbootImage<Part, State>,
    type_field: Tags,
) -> Result<usize> {
    let part_desc = img.part_desc.get().unwrap();
    if let Some(val) = part_desc.hdr {
        let mut header_bytes: &[u8] = (unsafe { (val as *const [u8; IMAGE_HEADER_SIZE]).as_ref() })
            .ok_or(RustbootError::__Nonexhaustive)?;
        header_bytes = &header_bytes[8..]; // skip magic & size fields
        match type_field {
            Tags::Version => {
                let (remaining, _) =
                    extract_version(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + HDR_VERSION_LEN);
                Ok(offset)
            }
            Tags::TimeStamp => {
                let (remaining, _) =
                    extract_timestamp(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + HDR_TIMESTAMP_LEN);
                Ok(offset)
            }
            Tags::ImgType => {
                let (remaining, _) =
                    extract_img_type(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + HDR_IMG_TYPE_LEN);
                Ok(offset)
            }
            Tags::Digest256 => {
                let (remaining, _) =
                    extract_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + SHA256_DIGEST_SIZE);
                Ok(offset)
            }
            Tags::Digest384 => {
                let (remaining, _) =
                    extract_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + SHA384_DIGEST_SIZE);
                Ok(offset)
            }
            Tags::PubkeyDigest => {
                let (remaining, _) =
                    extract_pubkey_digest(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + PUBKEY_DIGEST_SIZE);
                Ok(offset)
            }
            Tags::Signature => {
                let (remaining, _) =
                    extract_signature(header_bytes).map_err(|_| RustbootError::InvalidValue)?;
                let offset = IMAGE_HEADER_SIZE - remaining.len() - (4 + ECC_SIGNATURE_SIZE);
                Ok(offset)
            }
            Tags::EndOfHeader => todo!(),
        }
    } else {
        Err(RustbootError::__Nonexhaustive)
    }
}

#[derive(Clone, Copy)]
/// Each variant in [`Tags`] represents a field in the image-header.
///
/// *Note: [`EndOfHeader`] is a pseudo-Tag, i.e. doesnt come
/// with an associated length-value pair*
pub enum Tags {
    Version,
    TimeStamp,
    ImgType,
    Digest256,
    Digest384,
    PubkeyDigest,
    Signature,
    EndOfHeader,
}

impl Tags {
    #[rustfmt::skip]
    /// The ids are reversed to account for endianess
    fn get_id(self) -> &'static [u8] {
        match self {
            Self::Version       => &[0x01, 0x00],
            Self::TimeStamp     => &[0x02, 0x00],
            Self::ImgType       => &[0x04, 0x00],
            Self::Digest256     => &[0x03, 0x00],
            Self::Digest384     => &[0x13, 0x00],
            Self::PubkeyDigest  => &[0x10, 0x00],
            Self::Signature     => &[0x20, 0x00],
            Self::EndOfHeader   => &[0x00, 0x00],
        }
    }
}

use nom::bytes::complete::take_while;
use nom::bytes::complete::{tag, take};
use nom::{
    error::{Error, ErrorKind},
    Err, IResult,
};

// use libc_print::libc_println;

fn check_for_eof(input: &[u8]) -> IResult<&[u8], &[u8]> {
    match tag::<_, _, Error<&[u8]>>(Tags::EndOfHeader.get_id())(input) {
        Ok((_remainder, _eof)) => Err(Err::Error(Error::new(input, ErrorKind::Eof))),
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
    let (remainder, version) = take(8u32)(input)?;
    let (lengthvalue, version_check) = take(2u32)(version)?;
    let (value, version_len) = take(2u32)(lengthvalue)?;
    let len = (version_len[0] as u16 | (version_len[1] as u16) << 8) as usize;
    if version_check == Tags::Version.get_id() && len == HDR_VERSION_LEN {
        Ok((remainder, value))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_timestamp<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (remainder, _) = extract_version(input)?;
    let (remainder, _) = check_for_eof(remainder)?;
    let (remainder, _) = check_for_padding(remainder)?;
    let (remainder, timestamp) = take(12u32)(remainder)?;
    let (lengthvalue, timestamp_check) = take(2u32)(timestamp)?;
    let (value, timestamp_len) = take(2u32)(lengthvalue)?;
    let len = (timestamp_len[0] as u16 | (timestamp_len[1] as u16) << 8) as usize;
    if timestamp_check == Tags::TimeStamp.get_id() && len == HDR_TIMESTAMP_LEN {
        Ok((remainder, value))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_img_type<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (remainder, _) = extract_timestamp(input)?;
    let (remainder, _) = check_for_eof(remainder)?;
    let (remainder, _) = check_for_padding(remainder)?;
    let (remainder, img_type) = take(6u32)(remainder)?;
    let (lengthvalue, img_type_check) = take(2u32)(img_type)?;
    let (value, timestamp_len) = take(2u32)(lengthvalue)?;
    let len = (timestamp_len[0] as u16 | (timestamp_len[1] as u16) << 8) as usize;
    if img_type_check == Tags::ImgType.get_id() && len == HDR_IMG_TYPE_LEN {
        Ok((remainder, value))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_digest<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (remainder, _) = extract_img_type(input)?;
    let (remainder, _) = check_for_eof(remainder)?;
    let (remainder, _) = check_for_padding(remainder)?;
    let (remainder, typelen) = take(4u32)(remainder)?;
    let len = (typelen[2] as u16 | (typelen[3] as u16) << 8) as usize;
    let (remainder, digest) = take(len)(remainder)?;
    let (_, digest_check) = take(2u32)(typelen)?;
    if (digest_check == Tags::Digest256.get_id() && len == SHA256_DIGEST_SIZE)
        || (digest_check == Tags::Digest384.get_id() && len == SHA384_DIGEST_SIZE)
    {
        Ok((remainder, &digest[..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_pubkey_digest<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (remainder, _) = extract_digest(input)?;
    let (remainder, _) = check_for_eof(remainder)?;
    let (remainder, _) = check_for_padding(remainder)?;
    let (remainder, typelen) = take(4u32)(remainder)?;
    let len = (typelen[2] as u16 | (typelen[3] as u16) << 8) as usize;
    let (remainder, digest) = take(len)(remainder)?;
    let (_, digest_check) = take(2u32)(typelen)?;
    if (digest_check == Tags::PubkeyDigest.get_id() && len == SHA256_DIGEST_SIZE)
        || (digest_check == Tags::PubkeyDigest.get_id() && len == SHA384_DIGEST_SIZE)
    {
        Ok((remainder, &digest[..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn extract_signature<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (remainder, _) = extract_pubkey_digest(input)?;
    let (remainder, _) = check_for_eof(remainder)?;
    let (remainder, _) = check_for_padding(remainder)?;
    let (remainder, typelen) = take(4u32)(remainder)?;
    let len = (typelen[2] as u16 | (typelen[3] as u16) << 8) as usize;
    let (remainder, signature) = take(len)(remainder)?;
    let (_, signature_check) = take(2u32)(typelen)?;
    if signature_check == Tags::Signature.get_id() && len == ECC_SIGNATURE_SIZE {
        Ok((remainder, &signature[..]))
    } else {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

#[cfg(test)]
mod tests {
    // use libc_print::libc_println;
    use super::*;

    const PAD1: &[u8] = &[0x20, 0x01, 0xff, 0x02, 0x03];
    const PAD2: &[u8] = &[0xff, 0xff, 0xff, 0x02, 0x03];

    #[rustfmt::skip]
    const DATA: &[u8] = &[
        // 0x54, 0x53, 0x55, 0x52, // magic
        // 0x65, 0x51, 0x48, 0x54, // size
        0x01, 0x00, 0x04, 0x00, // version type & len
        0x01, 0x02, 0x03, 0x04, // version value

        0xff, 0xff, 0xff, 0xff, // padding bytes

        0x02, 0x00, 0x08, 0x00, // timestamp type & len
        0x11, 0x11, 0x11, 0x11, // timestamp value
        0x22, 0x22, 0x22, 0x22, 

        0x04, 0x00, 0x02, 0x00, // img type and len
        0x02, 0x00,             // img value

        0xff, 0xff, 0xff, 0xff, // padding bytes
        0xff, 0xff,

        // 32 byte digest type and len
        0x03, 0x00, 0x20, 0x00, 
        // digest value
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 
        // 32-byte pubkey digest type and len
        0x10, 0x00, 0x20, 0x00, 
        // pubkey digest value
        0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 
        0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 
        0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 
        0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 
        // signature type and len
        0x20, 0x00, 0x40, 0x00, 
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
            Ok((remainder, _val)) => {
                // libc_println!("incorrect padding: {:?}", remainder);
                remainder
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x20, 0x01, 0xff, 0x02, 0x03]);

        let val = match check_for_padding(PAD2) {
            Ok((_remainder, val)) => {
                // libc_println!("padding: {:?}", val);
                val
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0xff, 0xff, 0xff]);
    }

    #[test]
    fn parse_version() {
        let val = match extract_version(DATA) {
            Ok((_remainder, version)) => {
                // libc_println!("version: {:?}", version);
                version
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x01, 0x02, 0x03, 0x04])
    }

    #[test]
    fn parse_timestamp() {
        let val = match extract_timestamp(DATA) {
            Ok((_remainder, timestamp)) => {
                // libc_println!("timestamp: {:?}", timestamp);
                timestamp
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22])
    }

    #[test]
    fn parse_img_type() {
        let val = match extract_img_type(DATA) {
            Ok((_remainder, img_type)) => {
                // libc_println!("img_type: {:?}", img_type);
                img_type
            }
            Err(_e) => &[],
        };
        assert_eq!(val, &[0x02, 0x00])
    }

    #[test]
    fn parse_digest() {
        let val = match extract_digest(DATA) {
            Ok((_remainder, digest)) => {
                // libc_println!("digest: {:?}", digest);
                digest
            }
            Err(_e) => &[],
        };
        assert_eq!(
            val,
            &[
                0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
                0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33,
                0x33, 0x33, 0x33, 0x33,
            ]
        )
    }

    #[test]
    fn parse_pubkey_digest() {
        let val = match extract_pubkey_digest(DATA) {
            Ok((_remainder, digest)) => {
                // libc_println!("pubkey digest: {:?}", digest);
                digest
            }
            Err(_e) => &[],
        };
        assert_eq!(
            val,
            &[
                0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
                0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
                0x55, 0x55, 0x55, 0x55,
            ]
        )
    }

    #[test]
    fn parse_signature() {
        let val = match extract_signature(DATA) {
            Ok((_remainder, signature)) => {
                // libc_println!("signature: {:?}", signature);
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

    #[test]
    fn get_tlv_digest256() {
        let remaining = match extract_digest(DATA) {
            Ok((remainder, _digest)) => remainder,
            Err(_e) => &[],
        };
        let offset = DATA.len() - remaining.len() - (4 + SHA256_DIGEST_SIZE);
        assert_eq!(offset, 8 + 4 + 12 + 6 + 6)
    }

    #[test]
    fn get_tlv_pubkey_digest() {
        let remaining = match extract_pubkey_digest(DATA) {
            Ok((remainder, _digest)) => remainder,
            Err(_e) => &[],
        };
        let offset = DATA.len() - remaining.len() - (4 + PUBKEY_DIGEST_SIZE);
        assert_eq!(offset, 8 + 4 + 12 + 6 + 6 + 36)
    }
}
