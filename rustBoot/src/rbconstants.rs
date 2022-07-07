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
