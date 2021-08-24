
// **** TARGET PLATFORM - FLASH PARTIONINING **** 

pub const SECTOR_SIZE: usize = 4096;
pub const PARTITION_SIZE: usize = 0x28000;

pub const BOOT_PARTITION_ADDRESS: usize = 0x2f000; 
pub const SWAP_PARTITION_ADDRESS: usize = 0x57000;
pub const UPDATE_PARTITION_ADDRESS: usize = 0x58000;


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

pub const SECT_FLAG_NEW: u8 = 0x0F;

/// Enumerated BOOT partition
pub const BOOT_TRAILER_ADDRESS: usize = BOOT_PARTITION_ADDRESS + PARTITION_SIZE;
pub const BOOT_FWBASE: usize = BOOT_PARTITION_ADDRESS + IMAGE_HEADER_SIZE;
/// Enumerated UPDATE partition
pub const UPDATE_TRAILER_ADDRESS: usize = UPDATE_PARTITION_ADDRESS + PARTITION_SIZE;
pub const UPDATE_FWBASE: usize = UPDATE_PARTITION_ADDRESS + IMAGE_HEADER_SIZE;
/// Enumerated SWAP partition
pub const SWAP_BASE: usize = SWAP_PARTITION_ADDRESS;

pub const RUSTBOOT_MAGIC: usize = 0x54535552; // RUST
pub const RUSTBOOT_MAGIC_TRAIL: usize = 0x544F4F42; // BOOT

pub const PART_STATUS_LEN: usize = 1;
pub const MAGIC_TRAIL_LEN: usize = 4;

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

// NIST-P256 constants
#[cfg(feature = "nistp256")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0200;
// ECC-SECPK1 constants
#[cfg(feature = "secp256k1")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0000;
// ED25519 constants
#[cfg(feature = "ed25519")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0100;

