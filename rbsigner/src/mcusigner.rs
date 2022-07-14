use crate::curve::*;
use field::*;
use p256::ecdsa::signature::{digest::Digest, DigestSigner};
use rustBoot::rbconstants::*;
use sha2::Sha256;

use filetime::FileTime;
use std::fs;

mod field {

    use core::ops::Range;

    pub type Field = Range<usize>;
    // pub type Rest = RangeFrom<usize>;

    pub const MAGIC: Field = 0..4;
    pub const IMAGE_SIZE: Field = 4..8;

    pub const VERSION_TYPE: Field = 8..10;
    pub const VERSION_LEN: Field = 10..12;
    pub const VERSION_VALUE: Field = 12..16;

    pub const TIMESTAMP_TYPE: Field = 20..22;
    pub const TIMESTAMP_LEN: Field = 22..24;
    pub const TIMESTAMP_VALUE: Field = 24..32;

    pub const IMAGE_TYPE: Field = 32..34;
    pub const IMAGE_LEN: Field = 34..36;
    pub const IMAGE_VALUE: Field = 36..38;

    pub const DIGEST_TYPE: Field = 44..46;
    pub const DIGEST_LEN: Field = 46..48;
    pub const SHA256_DIGEST: Field = 48..80;

    pub const PUBKEY_TYPE: Field = 80..82;
    pub const PUBKEY_LEN: Field = 82..84;
    pub const PUBKEY_DIGEST_VALUE: Field = 84..116;

    pub const SIGNATURE_TYPE: Field = 116..118;
    pub const SIGNATURE_LEN: Field = 118..120;
    pub const SIGNATURE_VALUE: Field = 120..184;
}

pub trait VecExt<T>: AsMut<Vec<T>> {
    fn insert_from_slice(&mut self, index: usize, other: &[T])
    where
        T: Clone,
    {
        self.as_mut().splice(index..index, other.iter().cloned());
    }
}

impl<T> VecExt<T> for Vec<T> {}

/// Retruns a signed mcu-image, given a firmware image blob, the path to the blob, a signing key. Only supports `elliptic curve crypto`
///
/// NOTE:
/// - a valid mcu-image contains a 256-byte header.
///
pub fn sign_mcu_image(
    mut fw_blob: Vec<u8>,
    path: &str,
    sk_type: SigningKeyType,
    ver: [u8; 4],
) -> Result<Vec<u8>> {
    match sk_type {
        #[cfg(feature = "nistp256")]
        SigningKeyType::NistP256(sk) => {
            let (mut header, prehashed_digest) =
                construct_img_header::<Sha256, 32>(fw_blob.as_slice(), path, ver)
                    .map_err(|_v| RbSignerError::BadHashValue)?;
            let derived_pk = sk.verifying_key().to_encoded_point(false);
            let mut tag_len = [0u8; 4]; // tag and len each take up 2 bytes.

            // set pubkey digest type, len and value
            let pubkey_digest = Sha256::digest(&derived_pk.as_bytes()[1..]);
            let hdr_pubkey_digest_len = (PUBKEY_DIGEST_SIZE as u16).to_be_bytes();
            let pubkey_digest_tag = Tags::PubkeyDigest.get_id();
            let pubkey_digest_len = hdr_pubkey_digest_len.as_ref();
            pubkey_digest_tag
                .iter()
                .chain(pubkey_digest_len.iter())
                .enumerate()
                .for_each(|(idx, byte)| {
                    tag_len[idx] = *byte;
                });
            header.set_pubkey_tag_len(u32::from_be_bytes(tag_len));
            header.set_pubkey_digest_value(pubkey_digest.as_slice())?;

            // set signature type, len and value
            let signature = sk
                .try_sign_digest(prehashed_digest)
                .map_err(|v| RbSignerError::SignatureError(v))?;
            println!("Signing the firmware...");
            // println!("signature:\t{:?}", signature);
            println!("Done.");
            let hdr_signature_len = (ECC_SIGNATURE_SIZE as u16).to_be_bytes();
            let signature_tag = Tags::Signature.get_id();
            let signature_len = hdr_signature_len.as_ref();
            signature_tag
                .iter()
                .chain(signature_len.iter())
                .enumerate()
                .for_each(|(idx, byte)| {
                    tag_len[idx] = *byte;
                });
            header.set_signature_tag_len(u32::from_be_bytes(tag_len));
            header.set_signatue_value(signature.as_ref())?;

            //set end of header
            header.set_end_of_header(SIGNATURE_VALUE.end);
            // prepend header and return fw_blob
            let _ = fw_blob.insert_from_slice(0, header.as_slice());
            Ok(fw_blob)
        }
        #[cfg(feature = "ed25519")]
        SigningKeyType::Ed25519 => {
            todo!()
        }
        _ => return Err(RbSignerError::InvalidKeyType),
    }
}

fn construct_img_header<'a, D, const H: usize>(
    fw_blob: &'a [u8],
    path: &str,
    version: [u8; 4],
) -> Result<(McuImageHeader<[u8; 256]>, D)>
where
    D: Digest + Clone,
{
    // Construct an McuImageHeader
    let mut header = McuImageHeader::new_checked([0; 256])?;
    let mut tag_len = [0u8; 4]; // tag and len each take up 2 bytes.

    // set magic value and firmware size
    header.set_magic();
    header.set_image_size(fw_blob.len() as u32);

    // set version type, len and value
    let hdr_version_len = (HDR_VERSION_LEN as u16).to_be_bytes();
    let version_tag = Tags::Version.get_id();
    let version_len = hdr_version_len.as_ref();
    version_tag
        .iter()
        .chain(version_len.iter())
        .enumerate()
        .for_each(|(idx, byte)| {
            tag_len[idx] = *byte;
        });
    header.set_version_tag_len(u32::from_be_bytes(tag_len));
    header.set_version_value(&version)?;

    // set timestamp type, len and value
    let metadata =
        fs::metadata(path).expect("something's wrong with your file path for your image");

    let mtime = FileTime::from_last_modification_time(&metadata);
    // println!("\nimage timestamp: {}", mtime.unix_seconds()); // unix seconds values can be interpreted across platforms
    let atime = FileTime::from_last_access_time(&metadata);
    assert!(mtime < atime);

    let hdr_timestamp_len = (HDR_TIMESTAMP_LEN as u16).to_be_bytes();
    let timestamp_tag = Tags::TimeStamp.get_id();
    let timestamp_len = hdr_timestamp_len.as_ref();
    timestamp_tag
        .iter()
        .chain(timestamp_len.iter())
        .enumerate()
        .for_each(|(idx, byte)| {
            tag_len[idx] = *byte;
        });
    header.set_timestamp_tag_len(u32::from_be_bytes(tag_len));
    header.set_timestamp_value(&mtime.unix_seconds().to_le_bytes())?;

    // set image type, len and value
    let hdr_img_tag_len = (HDR_IMG_TYPE_LEN as u16).to_be_bytes();
    let img_tag = Tags::ImgType.get_id();
    let img_len = hdr_img_tag_len.as_ref();
    img_tag
        .iter()
        .chain(img_len.iter())
        .enumerate()
        .for_each(|(idx, byte)| {
            tag_len[idx] = *byte;
        });
    header.set_image_tag_len(u32::from_be_bytes(tag_len));
    header.set_image_value(&[0x01, 0x02])?;

    let mut hasher = D::new();
    hasher.update(&header.inner_ref()[..DIGEST_TYPE.start]);
    hasher.update(fw_blob);
    let digest = hasher.clone().finalize();

    match H {
        32 => {
            // set digest type, len and value
            let hdr_digest_len = (SHA256_DIGEST_SIZE as u16).to_le_bytes();
            let digest_tag = Tags::Digest256.get_id();
            let digest_len = hdr_digest_len.as_ref();
            digest_tag
                .iter()
                .chain(digest_len.iter())
                .enumerate()
                .for_each(|(idx, byte)| {
                    tag_len[idx] = *byte;
                });
            println!("Calculating sha256 digest...");
            header.set_digest_tag_len(u32::from_be_bytes(tag_len));
            header.set_sha256_digest_value(digest.as_slice())?;
        }
        _ => unimplemented!(),
    }

    Ok((header, hasher))
}

#[derive(Debug, PartialEq, Clone)]
pub struct McuImageHeader<T> {
    buffer: T,
}

impl<T: AsRef<[u8]>> McuImageHeader<T> {
    /// Imbue a raw octet buffer with `McuImageHeader` structure.
    pub fn new_unchecked(buffer: T) -> McuImageHeader<T> {
        McuImageHeader { buffer }
    }

    /// Shorthand for a combination of [new_unchecked].
    ///
    /// [new_unchecked]: #method.new_unchecked
    pub fn new_checked(buffer: T) -> Result<McuImageHeader<T>> {
        let hdr = Self::new_unchecked(buffer);
        if hdr.inner_ref().as_ref().len() != IMAGE_HEADER_SIZE {
            panic!("rustBoot header error: rustBoot-images must have a 256-byte header.")
        }
        Ok(hdr)
    }

    /// Returns a ref to the underlying buffer.
    pub fn inner_ref(&self) -> &T {
        &self.buffer
    }

    /// Returns a ref to the underlying buffer.
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    #[allow(dead_code)]
    /// Returns a 32-byte digest value. It includes the first 44-bytes of the header and the firmware image.
    pub fn get_sha256_digest_value(&self) -> Result<&[u8]> {
        let header = self.buffer.as_ref();
        Ok(&header[SHA256_DIGEST])
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> McuImageHeader<T> {
    /// Sets a 4-byte magic value - `constants::RUSTBOOT_MAGIC`.
    #[inline]
    pub fn set_magic(&mut self) {
        let header = self.buffer.as_mut();
        header[MAGIC].copy_from_slice((RUSTBOOT_MAGIC as u32).to_le_bytes().as_slice());
    }

    /// Sets the firmware's size which is a 4-byte field.
    #[inline]
    pub fn set_image_size(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[IMAGE_SIZE].copy_from_slice(value.to_le_bytes().as_slice());
    }

    /// Sets the tag and length for `image-version` field.
    #[inline]
    pub fn set_version_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[VERSION_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[VERSION_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_le_bytes().as_ref());
    }

    /// Sets the image version. The image-version value is a 4 byte field.
    #[inline]
    pub fn set_version_value(&mut self, value: &[u8]) -> Result<()> {
        let len = value.len();
        if len != HDR_VERSION_LEN {
            panic!("invalid image-version: length of image-version is a 4 byte value.")
        }

        let header = self.buffer.as_mut();
        header[VERSION_VALUE].copy_from_slice(value);

        // add a 4-byte padding constant - reserved bytes for the future
        match len % 4 {
            0 => {
                // add 4-bytes of constant padding.
                const PAD_LEN: usize = 4;
                let padding = [0xff; PAD_LEN];
                let padding_offset = field::VERSION_VALUE.end;
                header[padding_offset..padding_offset + PAD_LEN]
                    .copy_from_slice(&padding[..PAD_LEN]);
            }
            _ => {
                panic!("image-version are 4-byte values")
            }
        }
        Ok(())
    }

    /// Sets the tag and length for `image-timestamp` field.
    #[inline]
    pub fn set_timestamp_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[TIMESTAMP_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[TIMESTAMP_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_le_bytes().as_ref());
    }

    /// Set the `image-timestamp` value.
    #[inline]
    pub fn set_timestamp_value(&mut self, value: &[u8]) -> Result<()> {
        let len = value.len();
        if len != HDR_TIMESTAMP_LEN {
            panic!("invalid image-timestamp: length of image-timestamp is an 8 byte value.")
        }
        let header = self.buffer.as_mut();
        Ok(header[TIMESTAMP_VALUE].copy_from_slice(value))
    }

    /// Sets the tag and length for `image` field.
    #[inline]
    pub fn set_image_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[IMAGE_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[IMAGE_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_le_bytes().as_ref());
    }

    /// Sets the type of signing algorithm used to sign the `image`.
    ///
    /// Ex:
    /// 0x0200 - is NISTP256, with SHA256 for hashing
    #[inline]
    pub fn set_image_value(&mut self, value: &[u8]) -> Result<()> {
        let len = value.len();
        if len != HDR_IMG_TYPE_LEN {
            panic!("invalid image-type: image-type is a 2 byte value.")
        }
        let header = self.buffer.as_mut();
        header[IMAGE_VALUE].copy_from_slice(value);

        match len % 4 {
            2 => {
                // padding must be at least 4-bytes long
                // and 4-bytes aligned. So, add 6-bytes of constant padding.
                const PAD_LEN: usize = 6;
                let padding = [0xff; PAD_LEN];
                let padding_offset = field::IMAGE_VALUE.end;
                header[padding_offset..padding_offset + PAD_LEN]
                    .copy_from_slice(&padding[..PAD_LEN]);
            }
            _ => {
                panic!("image-type is a 2-byte value")
            }
        }
        Ok(())
    }

    /// Sets the tag and length for the `digest` field.
    #[inline]
    pub fn set_digest_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[DIGEST_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[DIGEST_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_be_bytes().as_ref());
    }

    /// Set the image-digest value.
    #[inline]
    pub fn set_sha256_digest_value(&mut self, value: &[u8]) -> Result<()> {
        if value.len() != SHA256_DIGEST_SIZE {
            panic!("invalid sha256 digest length")
        };
        let header = self.buffer.as_mut();
        Ok(header[SHA256_DIGEST].copy_from_slice(value))
    }

    /// Sets the tag and length for the `pubkey` field.
    #[inline]
    pub fn set_pubkey_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[PUBKEY_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[PUBKEY_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_le_bytes().as_ref());
    }

    /// Sets the pubkey-digest value
    #[inline]
    pub fn set_pubkey_digest_value(&mut self, value: &[u8]) -> Result<()> {
        if value.len() != PUBKEY_DIGEST_SIZE {
            panic!("invalid sha256 digest length")
        };
        let header = self.buffer.as_mut();
        Ok(header[PUBKEY_DIGEST_VALUE].copy_from_slice(value))
    }

    /// Sets the tag and length for the `signature` field.
    #[inline]
    pub fn set_signature_tag_len(&mut self, value: u32) {
        let header = self.buffer.as_mut();
        header[SIGNATURE_TYPE]
            .copy_from_slice((((value >> 16) & 0xFFFF) as u16).to_be_bytes().as_ref());
        header[SIGNATURE_LEN].copy_from_slice(((value & 0xFFFF) as u16).to_le_bytes().as_ref());
    }

    /// Sets the signature value.
    #[inline]
    pub fn set_signatue_value(&mut self, value: &[u8]) -> Result<()> {
        // let len = value.len();

        let header = self.buffer.as_mut();
        header[SIGNATURE_VALUE].copy_from_slice(value);

        // pad the remaining bytes barring the last 2.
        // match len % 4 {
        //     0 => {
        //         let padding_offset = field::SIGNATURE_VALUE.end;
        //         for byte in padding_offset..(IMAGE_HEADER_SIZE - 2) {
        //             header[byte] = 0xff;
        //         }
        //     }
        //     _ => {
        //         panic!("image-signatures are 4-byte multiple")
        //     }
        // }
        Ok(())
    }

    /// Sets the end-of-header value. Takes as input the end of the last field.
    #[inline]
    pub fn set_end_of_header(&mut self, end_of_last_field: usize) {
        let header = self.buffer.as_mut();
        header[end_of_last_field] = 0x00;
        header[end_of_last_field + 1] = 0x00;
    }
}

#[cfg(test)]
mod tests {
    use p256::{elliptic_curve::sec1::EncodedPoint, NistP256};
    use rustBoot::crypto::signatures::{import_pubkey, PubkeyTypes, VerifyingKeyTypes};

    use super::*;

    #[test]
    fn magic_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_magic();
                println!("magic_value: {:?}", &hdr.inner_ref()[MAGIC]);
                assert_eq!(&hdr.inner_ref()[MAGIC], &[0x52, 0x55, 0x53, 0x54]);
            }
            Err(_e) => {}
        };
    }
    #[test]
    fn image_size_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_image_size(8192);
                println!("image_size: {:?}", &hdr.inner_ref()[IMAGE_SIZE]);
                assert_eq!(&hdr.inner_ref()[IMAGE_SIZE], &[0x00, 0x20, 0x00, 0x00]);
            }
            Err(_e) => {}
        };
    }
    #[test]
    fn version_tag_len_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_version_tag_len(65540);
                println!("version_tag: {:?}", &hdr.inner_ref()[VERSION_TYPE]);
                println!("version_len: {:?}", &hdr.inner_ref()[VERSION_LEN]);
                assert_eq!(
                    &hdr.inner_ref()[VERSION_TYPE.start..VERSION_LEN.end],
                    &[0x01, 0x00, 0x04, 0x00]
                );
            }
            Err(_e) => {}
        };
    }
    #[test]
    fn version_tag_value() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_version_value(&[0x01, 0x02, 0x03, 0x04]);
                println!("version_value: {:?}", &hdr.inner_ref()[VERSION_VALUE]);
                println!(
                    "padding bytes after version: {:?}",
                    &hdr.inner_ref()[VERSION_VALUE.end..VERSION_VALUE.end + 4]
                );
                assert_eq!(&hdr.inner_ref()[VERSION_VALUE], &[0x01, 0x02, 0x03, 0x04]);
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn pk_bytes_test() {
        let sk_bytes: [u8; 32] = [
            0x53, 0xce, 0x7e, 0x5d, 0x40, 0xa8, 0xbe, 0xca, 0xe3, 0xdf, 0x7f, 0x9f, 0xb3, 0x07,
            0x1a, 0x93, 0xf9, 0x52, 0x47, 0x30, 0xcc, 0x30, 0xe6, 0x07, 0x1c, 0xe7, 0xfc, 0x90,
            0x7d, 0x5e, 0x58, 0xa0,
        ];
        let sk_type = import_signing_key(CurveType::NistP256, &sk_bytes[..]).unwrap();
        let derived_pk: EncodedPoint<NistP256>;
        match sk_type {
            SigningKeyType::NistP256(sk) => {
                derived_pk = sk.verifying_key().to_encoded_point(false);
                print!(
                    "derived_pk_bytes: {:?}, derived_pk_len: {:?}",
                    derived_pk.as_bytes(),
                    derived_pk.as_bytes().len()
                );
            }
            _ => {
                unimplemented!()
            }
        }
        let pk_type = import_pubkey(PubkeyTypes::NistP256).unwrap();
        match pk_type {
            VerifyingKeyTypes::VKeyNistP256(pk) => {
                let imported_pk = pk.to_encoded_point(false);
                assert_eq!(derived_pk, imported_pk);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn timestamp_tag_len_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_timestamp_tag_len(65535);
                println!("timestamp_type: {:?}", &hdr.inner_ref()[TIMESTAMP_TYPE]);
                println!("timestamp_len: {:?}", &hdr.inner_ref()[TIMESTAMP_LEN]);
                assert_eq!(
                    &hdr.inner_ref()[TIMESTAMP_TYPE.start..TIMESTAMP_LEN.end],
                    &[0x00, 0x00, 0xFF, 0xFF]
                );
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn set_timestamp_value_test() {
        use filetime::FileTime;
        use std::fs;

        let metadata = fs::metadata("../").unwrap();
        let mtime = FileTime::from_last_modification_time(&metadata);
        println!("image timestamp {}", mtime.unix_seconds()); // unix seconds values can be interpreted across platforms
        let atime = FileTime::from_last_access_time(&metadata);
        assert!(mtime < atime);

        let timestamp_bytes = mtime.unix_seconds().to_le_bytes();
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_timestamp_value(&timestamp_bytes);
                println!("timestamp_value: {:?}", &hdr.inner_ref()[TIMESTAMP_VALUE]);
                assert_eq!(
                    i64::from_le_bytes(hdr.inner_ref()[TIMESTAMP_VALUE].try_into().unwrap()),
                    mtime.unix_seconds()
                );
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn sha256_digest_value_test() {
        let sha256_digest_bytes: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_sha256_digest_value(&sha256_digest_bytes);
                println!(
                    "sha256_digest_value: {:?}",
                    &hdr.inner_ref()[SHA256_DIGEST]
                );
                assert_eq!(
                    &hdr.inner_ref()[SHA256_DIGEST.start..SHA256_DIGEST.end],
                    &sha256_digest_bytes
                );
            }
            Err(_e) => {}
        };
    }
    
    #[test]
    fn pubkey_digest_value_test() {
        let pubkey_digest_bytes: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_pubkey_digest_value(&pubkey_digest_bytes);
                println!(
                    "pubkey_digest_value: {:?}",
                    &hdr.inner_ref()[PUBKEY_DIGEST_VALUE]
                );
                assert_eq!(
                    &hdr.inner_ref()[PUBKEY_DIGEST_VALUE.start..PUBKEY_DIGEST_VALUE.end],
                    &pubkey_digest_bytes
                );
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn signatue_value_test() {
        let signature_value_bytes: [u8; 64] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a,
            0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
            0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40,
        ];

        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_signatue_value(&signature_value_bytes);
                println!("signatue_value: {:?}", &hdr.inner_ref()[SIGNATURE_VALUE]);
                assert_eq!(
                    &hdr.inner_ref()[SIGNATURE_VALUE.start..SIGNATURE_VALUE.end],
                    &signature_value_bytes
                );
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn end_of_header_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_end_of_header(SIGNATURE_VALUE.end);
                println!("end_of_header: {:?}", &hdr.inner_ref()[SIGNATURE_VALUE.end]);
                assert_eq!(
                    &hdr.inner_ref()[SIGNATURE_VALUE.end..=SIGNATURE_VALUE.end + 1],
                    &[0x00, 0x00]
                );
            }
            Err(_e) => {}
        };
    }

    #[test]
    fn digest_tag_len_test(){
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_digest_tag_len(65540);
                println!("digest_tag: {:?}", &hdr.inner_ref()[DIGEST_TYPE]);
                println!("digest_len: {:?}", &hdr.inner_ref()[DIGEST_LEN]);
                assert_eq!(
                    &hdr.inner_ref()[DIGEST_TYPE.start..DIGEST_LEN.end],
                    &[0x00, 0x01, 0x00, 0x04,]
                );
            }
            Err(_e) => {}
        };
    }
    
    #[test]
    fn signature_tag_len_test() {
        let header = McuImageHeader::new_checked([0; 256]);
        let _val = match header {
            Ok(mut hdr) => {
                let _ = hdr.set_signature_tag_len(65535);
                println!("signature_tag: {:?}", &hdr.inner_ref()[SIGNATURE_TYPE]);
                println!("signaure_len: {:?}", &hdr.inner_ref()[SIGNATURE_LEN]);
                assert_eq!(
                    &hdr.inner_ref()[SIGNATURE_TYPE.start..SIGNATURE_LEN.end],
                    &[0x00, 0x00, 0xff, 0xff]
                );
            }
            Err(_e) => {}
        };
    }
}