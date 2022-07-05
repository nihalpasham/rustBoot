#![allow(non_snake_case)]

// **** TARGET PLATFORM - FLASH PARTIONINING ****

#[cfg(feature = "nrf52840")]
pub const SECTOR_SIZE: usize = 0x1000;
#[cfg(feature = "nrf52840")]
pub const PARTITION_SIZE: usize = 0x28000;
#[cfg(feature = "nrf52840")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x2f000;
#[cfg(feature = "nrf52840")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x57000;
#[cfg(feature = "nrf52840")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x58000;

#[cfg(feature = "stm32f411")]
pub const SECTOR_SIZE: usize = 0x20000;
#[cfg(feature = "stm32f411")]
pub const PARTITION_SIZE: usize = 0x20000;
#[cfg(feature = "stm32f411")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x08020000;
#[cfg(feature = "stm32f411")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x08060000;
#[cfg(feature = "stm32f411")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x08040000;

#[cfg(feature = "stm32f446")]
pub const SECTOR_SIZE: usize = 0x20000;
#[cfg(feature = "stm32f446")]
pub const PARTITION_SIZE: usize = 0x20000;
#[cfg(feature = "stm32f446")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x08020000;
#[cfg(feature = "stm32f446")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x08060000;
#[cfg(feature = "stm32f446")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x08040000;

#[cfg(feature = "stm32h723")]
pub const SECTOR_SIZE: usize = 0x20000;
#[cfg(feature = "stm32h723")]
pub const PARTITION_SIZE: usize = 0x40000;
#[cfg(feature = "stm32h723")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x08020000;
#[cfg(feature = "stm32h723")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x080A0000;
#[cfg(feature = "stm32h723")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x08060000;

#[cfg(feature = "stm32f746")]
pub const SECTOR_SIZE: usize = 0x40000; // 256kb
#[cfg(feature = "stm32f746")]
pub const PARTITION_SIZE: usize = 0x40000;
#[cfg(feature = "stm32f746")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x08040000;
#[cfg(feature = "stm32f746")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x080C0000;
#[cfg(feature = "stm32f746")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x08080000;

#[cfg(feature = "stm32f334")]
pub const SECTOR_SIZE: usize = 0x1800;
#[cfg(feature = "stm32f334")]
pub const PARTITION_SIZE: usize = 0x1800;
#[cfg(feature = "stm32f334")]
pub const BOOT_PARTITION_ADDRESS: usize = 0x0800b800;
#[cfg(feature = "stm32f334")]
pub const SWAP_PARTITION_ADDRESS: usize = 0x0800e800;
#[cfg(feature = "stm32f334")]
pub const UPDATE_PARTITION_ADDRESS: usize = 0x0800d000;

// **** RAM BOOT options for staged OS (update_ram only) ****
pub const DTS_BOOT_ADDRESS: usize = 0xa0000;
pub const DTS_UPDATE_ADDRESS: usize = 0x10a0000;
pub const RAM_LOAD_ADDRESS: usize = 0x3000000;
pub const LOAD_DTS_ADDRESS: usize = 0x4000000;

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
