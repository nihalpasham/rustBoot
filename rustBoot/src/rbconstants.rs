// **** rustBoot constants ****
pub const IMAGE_HEADER_SIZE: usize = 0x100;
pub const IMAGE_HEADER_OFFSET: usize = 0x8;

pub const HDR_VERSION: u16 = 0x01;
pub const HDR_VERSION_LEN: usize = 0x4;
pub const HDR_TIMESTAMP_LEN: usize = 0x8;
pub const HDR_IMG_TYPE: u16 = 0x4;
pub const HDR_IMG_TYPE_LEN: usize = 0x2;
pub const HDR_IMG_TYPE_APP: u16 = 0x0001;
pub const HDR_MASK_LOWBYTE: u16 = 0x00FF;
pub const HDR_MASK_HIGHBYTE: u16 = 0xFF00;
pub const HDR_SIGNATURE: u16 = 0x20;
pub const HDR_PADDING: u8 = 0xFF;

pub const RUSTBOOT_MAGIC: usize = 0x54535552; // RUST
pub const RUSTBOOT_MAGIC_TRAIL: usize = 0x544F4F42; // BOOT

/*  Hash Config */
// SHA256 constants
pub const HDR_SHA256: u16 = 0x0003;
pub const SHA256_DIGEST_SIZE: usize = 32;
// SHA384 constants
pub const HDR_SHA384: u16 = 0x0013;
pub const SHA384_DIGEST_SIZE: usize = 48;

// SHA384 constants
pub const HDR_PUBKEY_DIGEST: u16 = 0x0010;
#[cfg(feature = "sha256")]
pub const PUBKEY_DIGEST_SIZE: usize = 32;
#[cfg(feature = "sha384")]
pub const PUBKEY_DIGEST_SIZE: usize = 48;

// NVM_FLASH_WRITEONCE
#[cfg(feature = "ext_flash")]
pub const FLASHBUFFER_SIZE: usize = SECTOR_SIZE;
pub const FLASHBUFFER_SIZE: usize = IMAGE_HEADER_SIZE;

/* Signature Config */
pub const ECC_SIGNATURE_SIZE: usize = 64;

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
    pub fn get_id(self) -> &'static [u8] {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_values() {
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
    }

    #[test]
    fn test_tags_get_id() {
        assert_eq!(Tags::Version.get_id(), &[0x01, 0x00]);
        assert_eq!(Tags::TimeStamp.get_id(), &[0x02, 0x00]);
        assert_eq!(Tags::ImgType.get_id(), &[0x04, 0x00]);
        assert_eq!(Tags::Digest256.get_id(), &[0x03, 0x00]);
        assert_eq!(Tags::Digest384.get_id(), &[0x13, 0x00]);
        assert_eq!(Tags::PubkeyDigest.get_id(), &[0x10, 0x00]);
        assert_eq!(Tags::Signature.get_id(), &[0x20, 0x00]);
        assert_eq!(Tags::EndOfHeader.get_id(), &[0x00, 0x00]);
    }

    #[test]
    fn test_tags_match_exhaustive() {
        let _ = |t: Tags| match t {
            Tags::Version => {}
            Tags::TimeStamp => {}
            Tags::ImgType => {}
            Tags::Digest256 => {}
            Tags::Digest384 => {}
            Tags::PubkeyDigest => {}
            Tags::Signature => {}
            Tags::EndOfHeader => {}
        };
    }
}
