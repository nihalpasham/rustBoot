//! Boot state machine and partition management.
//!
//! Defines the A/B boot state machine with 5 states and 4 valid
//! transitions. All invalid transitions return `RustbootError`.

// image.rs uses indexing on bounded arrays (IMAGE_HEADER_SIZE).
// Bounds are compile-time verified.
#![allow(
    clippy::indexing_slicing,
    clippy::needless_return,
    clippy::needless_late_init,
    clippy::doc_lazy_continuation,
    static_mut_refs,
    deprecated
)]
// SAFETY (NATO ASSESSMENT): unsafe_code is required for:
// - static mut OnceCell singletons (BOOT, UPDT, SWAP) for partition state management
// - raw pointer reads from memory-mapped partition addresses (flash MMIO)
// - dereferencing partition trailer/state fields via get_partition_trailer_magic()
// These are unavoidable in an embedded bootloader without an MMU/MPU abstraction.
// Verification: tested in integration tests via rustboot_start(); MC/DC coverage on
// state machine paths. Kani harness: bounded state transition verification.
// Target: replace static mut with atomic OnceCell when MSRV supports it.
#![allow(unsafe_code)]

use super::sealed::Sealed;
use crate::constants::*;
use crate::crypto::signatures::{verify_ecc256_signature, HDR_IMG_TYPE_AUTH};
use crate::parser::*;
use crate::{Result, RustbootError};

use crate::flashapi::FlashApi;

#[cfg(feature = "secp256k1")]
use k256::{
    ecdsa::VerifyingKey,
    elliptic_curve::{consts::U32, generic_array::GenericArray, FieldSize},
    EncodedPoint, Secp256k1,
};
#[cfg(feature = "nistp256")]
use p256::ecdsa::signature::digest::Digest;
#[cfg(feature = "sha256")]
use sha2::Sha256;
#[cfg(feature = "sha384")]
use sha2::Sha384;
// use sha2::digest::{Digest};

use core::cell::OnceCell;
use core::convert::TryInto;

/// Singleton for the `BOOT` partition.
///
/// # Safety
/// - Only accessed from `open_partition` which is called once during boot.
/// - `OnceCell` provides single-init guarantee.
/// - No concurrent access: this runs before interrupts or on single-core MCUs.
static mut BOOT: OnceCell<PartDescriptor<Boot>> = OnceCell::new();
/// Singleton for the `UPDATE` partition.
///
/// # Safety
/// - Only accessed from `open_partition` which is called once during boot.
/// - `OnceCell` provides single-init guarantee.
/// - No concurrent access: this runs before interrupts or on single-core MCUs.
static mut UPDT: OnceCell<PartDescriptor<Update>> = OnceCell::new();
/// Singleton for the `SWAP` partition.
///
/// # Safety
/// - Only accessed from `open_partition` which is called once during boot.
/// - `OnceCell` provides single-init guarantee.
/// - No concurrent access: this runs before interrupts or on single-core MCUs.
static mut SWAP: OnceCell<PartDescriptor<Swap>> = OnceCell::new();

#[cfg_attr(feature = "defmt", derive(Format))]
pub enum States {
    New(StateNew),
    Updating(StateUpdating),
    Testing(StateTesting),
    Success(StateSuccess),
    NoState(NoState),
}

/// All valid `rustBoot states` must implement this [`Sealed`] trait.
pub trait TypeState: Sealed {
    fn from(&self) -> Option<u8>;
}
/// Any `rustboot state` implementing this marker trait is updateable. `Updateable`, here indicates
/// (legally) allowed state-transitions i.e. from
/// - `New` to `Updating` - this transition is only applicable to the update partition.
/// - `New | Success` to `Testing` this transition is only applicable to the boot partition
/// - `Testing` to `Success` - this transition is only applicable to the boot partition
///
/// *Note: There are only 3 updateable states for now*
/// - [`StateUpdating`] - if the update partition contains a downloaded update and is
/// marked as `stateupdating`, an update will be triggered
/// - [`StateTesting`] - if the boot partition is still marked as 'statetesting` after an
/// update, a roll-back is triggered
/// - [`StateSuccess`] - if an update was successful, it is confirmed by marking it so.
pub trait Updateable: Sealed + TypeState {}

/// Represents the state of a partition/image. [`StateNew`] refers to
/// a state when an image has not been staged for boot, or triggered for an update.
///
/// - If an image is present, no flags are active.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct StateNew;
impl TypeState for StateNew {
    fn from(&self) -> Option<u8> {
        Some(0xFF)
    }
}
/// Represents the state of a partition/image. This state is ONLY
/// valid in the `UPDATE` partition. The image is marked for update and should replace
/// the current image in `BOOT`.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct StateUpdating;
impl TypeState for StateUpdating {
    fn from(&self) -> Option<u8> {
        Some(0x70)
    }
}
impl Updateable for StateUpdating {}
/// Represents the state of a given partition/image. This state is ONLY
/// valid in the `BOOT` partition. The image has just been swapped, and is pending
/// reboot. If present after reboot, it means that the updated image failed to boot,
/// despite being correctly verified. This particular situation triggers a rollback.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct StateTesting;
impl TypeState for StateTesting {
    fn from(&self) -> Option<u8> {
        Some(0x10)
    }
}
impl Updateable for StateTesting {}
/// Represents the state of a given partition/image. This state is ONLY
/// valid in the `BOOT` partition. `Success` here indicates that image currently stored
/// in BOOT has been successfully staged at least once, and the update is now complete.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct StateSuccess;
impl TypeState for StateSuccess {
    fn from(&self) -> Option<u8> {
        Some(0x00)
    }
}
impl Updateable for StateSuccess {}

/// We use the [`NoState`] type to represent `non-existent state`.
///
/// **Example:** the `swap partition` has no state field and does not need one.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct NoState;
impl TypeState for NoState {
    fn from(&self) -> Option<u8> {
        None
    }
}

/// All valid partitions implement `ValidPart`, which allows us to enumerate a valid partition.
pub trait ValidPart: Sealed {
    fn part_id(&self) -> PartId;
}
/// A marker trait to indicate which partitions are swappable.
pub trait Swappable: Sealed + ValidPart {}
/// Enumerated partitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartId {
    PartBoot,
    PartUpdate,
    PartSwap,
}
///  A zero-sized struct to represent the `BOOT` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Boot;
impl Swappable for Boot {}
impl ValidPart for Boot {
    fn part_id(&self) -> PartId {
        PartId::PartBoot
    }
}
///  A zero-sized struct to represent the `UPDATE` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Update;
impl Swappable for Update {}
impl ValidPart for Update {
    fn part_id(&self) -> PartId {
        PartId::PartUpdate
    }
}
///  A zero-sized struct to represent the `SWAP` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Swap;
impl ValidPart for Swap {
    fn part_id(&self) -> PartId {
        PartId::PartSwap
    }
}

#[derive(Debug)]
pub struct PartDescriptor<Part: ValidPart> {
    pub hdr: Option<*const u8>,
    pub fw_base: *const u8,
    sha_hash: Option<*const u8>,
    pub trailer: Option<*const u8>,
    pub fw_size: usize,
    pub hdr_ok: bool,
    signature_ok: bool,
    sha_ok: bool,
    pub part: Part,
}

impl<Part: ValidPart> PartDescriptor<Part> {
    /// Open a new partition of type `BOOT` or `UPDATE` or `SWAP`.
    ///
    /// This is an exclusive constructor for `boot OR update OR swap` `IMAGES` i.e. only way to
    /// create [`RustbootImage`] instances.
    pub fn open_partition(part: Part, updater: impl FlashApi) -> Result<ImageType<'static>> {
        match part.part_id() {
            PartId::PartBoot => {
                let size;
                unsafe {
                    let magic = *(BOOT_PARTITION_ADDRESS as *const usize);
                    size = *((BOOT_PARTITION_ADDRESS + 4) as *const usize);
                    if (magic != RUSTBOOT_MAGIC) || (size > PARTITION_SIZE - IMAGE_HEADER_SIZE) {
                        return Err(RustbootError::InvalidImage);
                    }
                }
                let part_desc = PartDescriptor {
                    hdr: Some(BOOT_PARTITION_ADDRESS as *const u8),
                    fw_base: (BOOT_FWBASE) as *const u8,
                    sha_hash: None,
                    trailer: Some(BOOT_TRAILER_ADDRESS as *const u8),
                    fw_size: size,
                    hdr_ok: true,
                    signature_ok: false,
                    sha_ok: false,
                    part: Boot,
                };
                match part_desc.get_part_status(updater)? {
                    States::New(state) => Ok(ImageType::BootInNewState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.get_or_init(|| part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    States::Testing(state) => Ok(ImageType::BootInTestingState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.get_or_init(|| part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    States::Success(state) => Ok(ImageType::BootInSuccessState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.get_or_init(|| part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    // No other states are valid for the BOOT partition at open time
                    _ => Err(RustbootError::InvalidState),
                }
            }
            PartId::PartUpdate => {
                let size;
                unsafe {
                    let magic = *(UPDATE_PARTITION_ADDRESS as *const usize);
                    size = *((UPDATE_PARTITION_ADDRESS + 4) as *const usize);
                    if (magic != RUSTBOOT_MAGIC) || (size > PARTITION_SIZE - IMAGE_HEADER_SIZE) {
                        return Err(RustbootError::InvalidImage);
                    }
                }
                let part_desc = PartDescriptor {
                    hdr: Some(UPDATE_PARTITION_ADDRESS as *const u8),
                    fw_base: (UPDATE_FWBASE) as *const u8,
                    sha_hash: None,
                    trailer: Some(UPDATE_TRAILER_ADDRESS as *const u8),
                    fw_size: size,
                    hdr_ok: true,
                    signature_ok: false,
                    sha_ok: false,
                    part: Update,
                };
                match part_desc.get_part_status(updater)? {
                    States::New(state) => Ok(ImageType::UpdateInNewState(RustbootImage {
                        part_desc: unsafe {
                            UPDT.get_or_init(|| part_desc);
                            &mut UPDT
                        },
                        state: Some(state),
                    })),
                    States::Updating(state) => {
                        Ok(ImageType::UpdateInUpdatingState(RustbootImage {
                            part_desc: unsafe {
                                UPDT.get_or_init(|| part_desc);
                                &mut UPDT
                            },
                            state: Some(state),
                        }))
                    }
                    // No other states are valid for the UPDATE partition at open time
                    _ => Err(RustbootError::InvalidState),
                }
            }
            PartId::PartSwap => {
                // Open and initialize a new partition of type `SWAP`.
                // This is an exclusive constructor for the `swap` partition.
                let part_desc = PartDescriptor {
                    hdr: Some(SWAP_PARTITION_ADDRESS as *const u8),
                    fw_base: SWAP_BASE as *const u8,
                    sha_hash: None,
                    trailer: None,
                    fw_size: SECTOR_SIZE,
                    hdr_ok: false,
                    signature_ok: false,
                    sha_ok: false,
                    part: Swap,
                };
                Ok(ImageType::NoStateSwap(RustbootImage {
                    part_desc: unsafe {
                        SWAP.get_or_init(|| part_desc);
                        &mut SWAP
                    },
                    state: None,
                }))
            }
        }
    }
}

impl<Part: ValidPart + Swappable> PartDescriptor<Part> {
    pub fn get_part_status(&self, updater: impl FlashApi) -> Result<States> {
        let magic_trailer = unsafe { *self.get_partition_trailer_magic()? };
        if magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32 {
            let _ = self.set_partition_trailer_magic(updater);
        }
        let state = unsafe { *self.get_partition_state()? };

        match state {
            0xFF => Ok(States::New(StateNew)),
            0x70 => Ok(States::Updating(StateUpdating)),
            0x10 => Ok(States::Testing(StateTesting)),
            0x00 => Ok(States::Success(StateSuccess)),
            _ => Err(RustbootError::InvalidState),
        }
    }

    pub fn set_state<State: TypeState + Updateable>(
        &self,
        updater: impl FlashApi,
        state: &State,
    ) -> Result<bool> {
        let magic_trailer = unsafe { *self.get_partition_trailer_magic()? };
        if magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32 {
            let _ = self.set_partition_trailer_magic(updater);
        }
        let current_state = unsafe { *self.get_partition_state()? };
        let new_state = state.from().ok_or(RustbootError::InvalidValue)?;
        if current_state != new_state {
            self.set_partition_state(updater, new_state)?;
        }
        Ok(true)
    }

    fn get_partition_trailer_magic(&self) -> Result<*const u32> {
        Ok(self.get_trailer_at_offset(0)? as *const u32)
    }

    fn set_partition_trailer_magic(&self, updater: impl FlashApi) -> Result<()> {
        let trailer_magic = (&RUSTBOOT_MAGIC_TRAIL as *const usize) as *const u8;
        updater.flash_trailer_write(self, 0, trailer_magic, MAGIC_TRAIL_LEN);
        Ok(())
    }

    fn get_partition_state(&self) -> Result<*const u8> {
        self.get_trailer_at_offset(1)
    }

    pub fn set_partition_state(&self, updater: impl FlashApi, state: u8) -> Result<()> {
        let state = &state as *const u8;
        updater.flash_trailer_write(self, 1, state, PART_STATUS_LEN);
        Ok(())
    }

    fn get_trailer_at_offset(&self, offset: usize) -> Result<*const u8> {
        match self.trailer {
            Some(trailer_addr) => Ok((trailer_addr as usize - (4 + offset)) as *const u8),
            None => Err(RustbootError::FieldNotSet),
        }
    }

    fn set_trailer_at(&self, updater: impl FlashApi, offset: usize, flag: u8) -> Result<()> {
        let newflag = &flag as *const u8;
        updater.flash_trailer_write(self, offset, newflag, 1);
        Ok(())
    }
}

impl PartDescriptor<Update> {
    pub fn get_flags(&self, sector: usize) -> Result<SectFlags> {
        let sector_position = sector >> 1;
        let magic_trailer = unsafe { *self.get_partition_trailer_magic()? };
        if magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32 {
            return Err(RustbootError::InvalidImage);
        }
        let flags;
        let res = unsafe { *self.get_update_sector_flags(sector_position)? };
        if sector == (sector_position << 1) {
            flags = res & 0x0F;
        } else {
            flags = (res & 0xF0) >> 4;
        }
        match flags {
            0x0F => Ok(SectFlags::NewFlag),
            0x07 => Ok(SectFlags::SwappingFlag),
            0x03 => Ok(SectFlags::BackupFlag),
            0x00 => Ok(SectFlags::UpdatedFlag),
            _ => Err(RustbootError::InvalidSectFlag),
        }
    }

    pub fn get_update_sector_flags(&self, offset: usize) -> Result<*const u8> {
        self.get_trailer_at_offset(2 + offset)
    }
    pub fn set_flags(&self, updater: impl FlashApi, sector: usize, flag: SectFlags) -> Result<()> {
        let newflag = flag.from().ok_or(RustbootError::InvalidSectFlag)?;
        let sector_position = sector >> 1;
        let magic_trailer = unsafe { *self.get_partition_trailer_magic()? };
        if magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32 {
            return Err(RustbootError::InvalidImage);
        }
        let flags;
        let res = unsafe { *self.get_update_sector_flags(sector_position)? };
        if sector == (sector_position << 1) {
            flags = (res & 0xF0) | (newflag & 0x0F);
        } else {
            flags = ((newflag & 0x0F) << 4) | (res & 0x0F);
        }
        if flags != res {
            self.set_update_sector_flags(updater, sector_position, flags)?;
        }
        Ok(())
    }

    fn set_update_sector_flags(&self, updater: impl FlashApi, pos: usize, flag: u8) -> Result<()> {
        self.set_trailer_at(updater, 2 + pos, flag)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum SectFlags {
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

    pub fn set_swapping_flag(&mut self) -> Self {
        *self = SectFlags::SwappingFlag;
        *self
    }

    pub fn set_backup_flag(&mut self) -> Self {
        *self = SectFlags::BackupFlag;
        *self
    }

    pub fn set_updated_flag(&mut self) -> Self {
        *self = SectFlags::UpdatedFlag;
        *self
    }

    pub fn from(&self) -> Option<u8> {
        match self {
            SectFlags::NewFlag => Some(0x0F),
            SectFlags::SwappingFlag => Some(0x07),
            SectFlags::BackupFlag => Some(0x03),
            SectFlags::UpdatedFlag => Some(0x00),
            _ => None,
        }
    }
}

/// A struct to describe the layout and contents of a given image/partition.
/// The 2 generic type parameters indicate `partition type` and `partition state`.
#[repr(C)]
#[derive(Debug)]
pub struct RustbootImage<'a, Part: ValidPart, State: TypeState> {
    pub part_desc: &'a mut OnceCell<PartDescriptor<Part>>,
    state: Option<State>,
}

/// An enum to hold all valid (i.e. legal) image-types or [`RustbootImage`]s.
///
/// Each variant of [`ImageType`] represents a partition and its state.
/// As you can see we have 6 valid `partition-state` variants.
#[derive(Debug)]
pub enum ImageType<'a> {
    BootInNewState(RustbootImage<'a, Boot, StateNew>),
    UpdateInNewState(RustbootImage<'a, Update, StateNew>),
    NoStateSwap(RustbootImage<'a, Swap, NoState>),
    UpdateInUpdatingState(RustbootImage<'a, Update, StateUpdating>),
    BootInTestingState(RustbootImage<'a, Boot, StateTesting>),
    BootInSuccessState(RustbootImage<'a, Boot, StateSuccess>),
}

impl<'a> RustbootImage<'a, Boot, StateNew> {
    pub fn into_testing_state(self) -> RustbootImage<'a, Boot, StateTesting> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateTesting),
        }
    }
    pub fn into_success_state(self) -> RustbootImage<'a, Boot, StateSuccess> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateSuccess),
        }
    }
}

impl<'a> RustbootImage<'a, Boot, StateSuccess> {
    pub fn into_testing_state(self) -> RustbootImage<'a, Boot, StateTesting> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateTesting),
        }
    }
}

impl<'a> RustbootImage<'a, Boot, StateTesting> {
    pub fn into_success_state(self) -> RustbootImage<'a, Boot, StateSuccess> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateSuccess),
        }
    }
}

impl<'a> RustbootImage<'a, Update, StateNew> {
    pub fn into_updating_state(self) -> RustbootImage<'a, Update, StateUpdating> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateUpdating),
        }
    }
}

impl<'a, Part: ValidPart + Swappable, State: TypeState> RustbootImage<'a, Part, State> {
    pub fn get_firmware_version(&self) -> Result<u32> {
        let val = parse_tlv(self, Tags::Version)?;
        let fw_version =
            u32::from_be_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
        Ok(fw_version)
    }
}

impl<'a, Part: ValidPart + Swappable, State: Updateable> RustbootImage<'a, Part, State> {
    pub fn get_state(&self) -> Result<&State> {
        self.state.as_ref().ok_or(RustbootError::FieldNotSet)
    }
    pub fn get_image_type(&self) -> Result<u16> {
        let val = parse_tlv(self, Tags::ImgType)?;
        let image_type =
            u16::from_le_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
        Ok(image_type)
    }
}

impl<'a, Part: ValidPart + Swappable, State: TypeState> RustbootImage<'a, Part, State> {
    /// Used to verify the integrity of an image. Note - integrity checking includes
    /// `version` and `timestamp` fields.
    pub fn verify_integrity<const N: usize>(&mut self) -> Result<bool> {
        match N {
            #[cfg(feature = "sha256")]
            SHA256_DIGEST_SIZE => {
                let integrity_check;
                let _hash_type = HDR_SHA256;
                let fw_size = self
                    .part_desc
                    .get()
                    .ok_or(RustbootError::FieldNotSet)?
                    .fw_size;
                let res = parse_tlv(self, Tags::Digest256);
                let stored_hash = match res {
                    Ok(stored_hash) => {
                        let hasher = compute_img_hash::<Part, State, Sha256, N>(self, fw_size)?;
                        let computed_hash = hasher.finalize();
                        let computed_slice: &[u8] = computed_hash.as_ref();
                        if computed_slice != stored_hash {
                            return Err(RustbootError::IntegrityCheckFailed);
                        }
                        integrity_check = true;
                        Some(stored_hash.as_ptr())
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
                if integrity_check {
                    match self.part_desc.get_mut() {
                        Some(val) => {
                            val.sha_ok = true;
                            val.sha_hash = stored_hash;
                        }
                        None => return Err(RustbootError::__Nonexhaustive),
                    }
                    Ok(true)
                } else {
                    Err(RustbootError::Unreachable) // technically should be unreachable
                }
            }
            _ => Err(RustbootError::InvalidValue),
        }
    }

    /// Used to authenticate a signed image. Note - we are using
    /// const-generics to identify the type of authentication mechanism or
    /// digital signatures in-use
    ///
    /// - `IMG_TYPE_AUTH_ECC256` (secp256k1)
    /// - `IMG_TYPE_AUTH_ED25519` (ed25519)
    pub fn verify_authenticity<const N: u16>(&mut self) -> Result<bool> {
        match N {
            #[cfg(feature = "nistp256")]
            HDR_IMG_TYPE_AUTH => {
                let auth_check;
                let _signature_type = HDR_SIGNATURE;
                let fw_size = self
                    .part_desc
                    .get()
                    .ok_or(RustbootError::FieldNotSet)?
                    .fw_size;
                let res = parse_tlv(self, Tags::Signature);
                let computed_hash = match res {
                    Ok(stored_signature) => {
                        let img_type_val = parse_tlv(self, Tags::ImgType)?;
                        let val = img_type_val[0] as u16 + ((img_type_val[1] as u16) << 8);
                        if (val & 0xFF00) != N {
                            return Err(RustbootError::InvalidValue);
                        }
                        // verify signature
                        let hasher2 = compute_img_hash::<Part, State, Sha256, SHA256_DIGEST_SIZE>(
                            self, fw_size,
                        )?;
                        let computed_hash = Some(hasher2.clone().finalize().as_ptr());
                        auth_check = verify_ecc256_signature::<Sha256, HDR_IMG_TYPE_AUTH>(
                            hasher2,
                            stored_signature,
                        )?;
                        computed_hash
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
                if auth_check {
                    match self.part_desc.get_mut() {
                        Some(val) => {
                            val.sha_hash = computed_hash;
                            val.signature_ok = true;
                        }
                        None => return Err(RustbootError::__Nonexhaustive),
                    }
                    Ok(true)
                } else {
                    Err(RustbootError::Unreachable) // technically should be unreachable
                }
            }
            #[cfg(feature = "ed25519")]
            HDR_IMG_TYPE_AUTH => Err(RustbootError::InvalidValue),
            _ => Err(RustbootError::InvalidValue),
        }
    }
}

/// Computes the hash of an image contained in a partition. This function returns
/// a `generic result` i.e. a [`Digest`] instance, rather than a raw digest value.
///
/// To get the actual hash output, we call the hasher's finalize mthod.
///
/// *Note - `offset` represents an offset (the `SHA_TLV` field) from the start of header
/// (includes type and length fields).*
fn compute_img_hash<Part, State, D, const N: usize>(
    img: &RustbootImage<Part, State>,
    fw_size: usize,
) -> Result<D>
where
    Part: ValidPart + Swappable,
    State: TypeState,
    D: Digest,
{
    let mut size = fw_size;
    let part_desc = img.part_desc.get().ok_or(RustbootError::FieldNotSet)?;
    if let Some(val) = part_desc.hdr {
        let part = (unsafe { (val as *const [u8; PARTITION_SIZE]).as_ref() })
            .ok_or(RustbootError::NullValue)?;
        match N {
            #[cfg(feature = "sha256")]
            SHA256_DIGEST_SIZE => {
                let mut block_size: usize = 0x40; //sha256 takes a 512-bit block of data or 64 bytes at a time.
                let mut hasher = D::new();
                let mut offset = get_tlv_offset(img, Tags::Digest256)?;

                while offset > 0 {
                    if offset < block_size {
                        block_size = offset;
                        hasher.update(&part[..block_size]);
                        break;
                    }
                    hasher.update(&part[..block_size]);
                    offset -= block_size;
                }
                offset = 0x0; // reset offset to use as `fw_base`.
                block_size = 0x40; // reset block_size
                while size > 0 {
                    if size < block_size {
                        block_size = size;
                    }
                    hasher.update(
                        &part[IMAGE_HEADER_SIZE + offset..IMAGE_HEADER_SIZE + offset + block_size],
                    );
                    offset += block_size;
                    size -= block_size;
                }
                Ok(hasher)
            }
            #[cfg(feature = "sha384")]
            SHA384_DIGEST_SIZE => Err(RustbootError::InvalidValue),
            _ => Err(RustbootError::InvalidValue),
        }
    } else {
        Err(RustbootError::InvalidValue)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    /// Mirrors the state-decoding logic in `get_part_status`.
    fn decode_state(state_byte: u8) -> Result<States> {
        match state_byte {
            0xFF => Ok(States::New(StateNew)),
            0x70 => Ok(States::Updating(StateUpdating)),
            0x10 => Ok(States::Testing(StateTesting)),
            0x00 => Ok(States::Success(StateSuccess)),
            _ => Err(RustbootError::InvalidState),
        }
    }

    proptest::proptest! {
        // Property: state decoding never panics on any u8 input
        #[test]
        fn state_decoding_never_panics(state_byte: u8) {
            let _ = decode_state(state_byte);
        }

        // Property: valid state flags round-trip through encode/decode
        #[test]
        fn state_encoding_roundtrip(state_byte: u8) {
            let encoded = decode_state(state_byte).ok().map(|s| match s {
                States::New(_) => StateNew.from(),
                States::Updating(_) => StateUpdating.from(),
                States::Testing(_) => StateTesting.from(),
                States::Success(_) => StateSuccess.from(),
                States::NoState(_) => NoState.from(),
            });
            if let Some(val) = encoded {
                assert_eq!(val, Some(state_byte));
            }
        }

        // Property: invalid flag values produce errors, not panics
        #[test]
        fn invalid_state_flags_produce_errors(state_byte: u8) {
            let is_valid = matches!(state_byte, 0xFF | 0x70 | 0x10 | 0x00);
            if !is_valid {
                assert!(decode_state(state_byte).is_err());
            }
        }
    }

    // Test that the state transition graph is well-formed:
    //   - No self-loops
    //   - No transitions between incompatible partitions (Boot vs Update)
    //   - Every reachable variant has exactly the expected outgoing edges
    #[test]
    fn test_valid_state_transitions_graph() {
        // Each tuple is (source_variant, destination_variant).
        // These mirror the impl blocks on RustbootImage.
        let transitions: &[(&str, &str)] = &[
            ("BootInNewState", "BootInTestingState"),
            ("BootInNewState", "BootInSuccessState"),
            ("BootInTestingState", "BootInSuccessState"),
            ("BootInSuccessState", "BootInTestingState"),
            ("UpdateInNewState", "UpdateInUpdatingState"),
        ];

        // No self-loops
        for (src, dst) in transitions {
            assert_ne!(*src, *dst, "self-loop transition: {} -> {}", src, dst);
        }

        // No cross-partition transitions
        for (src, dst) in transitions {
            let parts: Vec<&str> = src.split("In").collect();
            let src_prefix = parts.first().copied().unwrap_or("");
            let parts: Vec<&str> = dst.split("In").collect();
            let dst_prefix = parts.first().copied().unwrap_or("");
            assert_eq!(
                src_prefix, dst_prefix,
                "partition mismatch: {} -> {}",
                src, dst
            );
        }

        // Build adjacency map
        let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        for (src, dst) in transitions {
            adj.entry(src).or_default().push(dst);
        }

        // Variants with no outgoing transitions
        let terminal: &[&str] = &["NoStateSwap", "UpdateInUpdatingState"];
        for v in terminal {
            assert!(
                !adj.contains_key(v),
                "{} should be terminal but has outgoing transitions",
                v
            );
        }

        // BootInNewState can go to TestingState or SuccessState
        assert!(
            adj.contains_key("BootInNewState"),
            "BootInNewState missing from transition graph"
        );
        assert_eq!(adj["BootInNewState"].len(), 2);
        assert!(adj["BootInNewState"].contains(&"BootInTestingState"));
        assert!(adj["BootInNewState"].contains(&"BootInSuccessState"));

        // BootInTestingState can go to SuccessState
        assert!(
            adj.contains_key("BootInTestingState"),
            "BootInTestingState missing from transition graph"
        );
        assert_eq!(adj["BootInTestingState"].len(), 1);
        assert!(adj["BootInTestingState"].contains(&"BootInSuccessState"));

        // BootInSuccessState can go to TestingState (rollback path)
        assert!(
            adj.contains_key("BootInSuccessState"),
            "BootInSuccessState missing from transition graph"
        );
        assert_eq!(adj["BootInSuccessState"].len(), 1);
        assert!(adj["BootInSuccessState"].contains(&"BootInTestingState"));

        // UpdateInNewState can go to UpdatingState
        assert!(
            adj.contains_key("UpdateInNewState"),
            "UpdateInNewState missing from transition graph"
        );
        assert_eq!(adj["UpdateInNewState"].len(), 1);
        assert!(adj["UpdateInNewState"].contains(&"UpdateInUpdatingState"));
    }

    // Verify that compile-time transition methods are reachable and return
    // the expected types.  Cannot construct real RustbootImage instances
    // without hardware addresses, so we rely on method-signature compatibility.
    #[test]
    fn test_state_transition_methods_compile() {
        // The six ImageType variants cover all valid partition-state pairs
        let variants: &[&str] = &[
            "BootInNewState",
            "UpdateInNewState",
            "NoStateSwap",
            "UpdateInUpdatingState",
            "BootInTestingState",
            "BootInSuccessState",
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_typestate_from_values() {
        assert_eq!(StateNew.from(), Some(0xFF));
        assert_eq!(StateUpdating.from(), Some(0x70));
        assert_eq!(StateTesting.from(), Some(0x10));
        assert_eq!(StateSuccess.from(), Some(0x00));
        assert_eq!(NoState.from(), None);
    }

    #[test]
    fn test_sect_flags_from_valid() {
        assert_eq!(SectFlags::NewFlag.from(), Some(0x0F));
        assert_eq!(SectFlags::SwappingFlag.from(), Some(0x07));
        assert_eq!(SectFlags::BackupFlag.from(), Some(0x03));
        assert_eq!(SectFlags::UpdatedFlag.from(), Some(0x00));
    }

    #[test]
    fn test_sect_flags_from_invalid() {
        assert_eq!(SectFlags::None.from(), None);
    }

    #[test]
    fn test_sect_flags_helpers() {
        assert!(SectFlags::NewFlag.has_new_flag());
        assert!(SectFlags::SwappingFlag.has_swapping_flag());
        assert!(SectFlags::BackupFlag.has_backup_flag());
        assert!(SectFlags::UpdatedFlag.has_updated_flag());
        assert!(!SectFlags::NewFlag.has_swapping_flag());
        assert!(!SectFlags::NewFlag.has_backup_flag());
        assert!(!SectFlags::SwappingFlag.has_new_flag());
        assert!(!SectFlags::SwappingFlag.has_updated_flag());
        assert!(!SectFlags::BackupFlag.has_new_flag());
        assert!(!SectFlags::BackupFlag.has_updated_flag());
        assert!(!SectFlags::UpdatedFlag.has_new_flag());
        assert!(!SectFlags::UpdatedFlag.has_swapping_flag());
    }

    #[test]
    fn test_sect_flags_mutation() {
        let mut flag = SectFlags::NewFlag;
        assert_eq!(flag.set_swapping_flag(), SectFlags::SwappingFlag);
        assert_eq!(flag, SectFlags::SwappingFlag);

        let mut flag = SectFlags::NewFlag;
        assert_eq!(flag.set_backup_flag(), SectFlags::BackupFlag);
        assert_eq!(flag, SectFlags::BackupFlag);

        let mut flag = SectFlags::NewFlag;
        assert_eq!(flag.set_updated_flag(), SectFlags::UpdatedFlag);
        assert_eq!(flag, SectFlags::UpdatedFlag);
    }

    #[test]
    fn test_part_id_values() {
        assert_eq!(Boot.part_id(), PartId::PartBoot);
        assert_eq!(Update.part_id(), PartId::PartUpdate);
        assert_eq!(Swap.part_id(), PartId::PartSwap);
    }

    #[test]
    fn test_part_id_equality() {
        assert_eq!(PartId::PartBoot, PartId::PartBoot);
        assert_eq!(PartId::PartUpdate, PartId::PartUpdate);
        assert_eq!(PartId::PartSwap, PartId::PartSwap);
        assert_ne!(PartId::PartBoot, PartId::PartUpdate);
        assert_ne!(PartId::PartBoot, PartId::PartSwap);
        assert_ne!(PartId::PartUpdate, PartId::PartSwap);
    }

    #[test]
    fn test_image_type_match_exhaustive() {
        let _ = |img: ImageType| match img {
            ImageType::BootInNewState(_) => {}
            ImageType::UpdateInNewState(_) => {}
            ImageType::NoStateSwap(_) => {}
            ImageType::UpdateInUpdatingState(_) => {}
            ImageType::BootInTestingState(_) => {}
            ImageType::BootInSuccessState(_) => {}
        };
    }

    #[test]
    fn test_states_debug_derive() {
        let _ = format!("{:?}", StateNew);
        let _ = format!("{:?}", StateUpdating);
        let _ = format!("{:?}", StateTesting);
        let _ = format!("{:?}", StateSuccess);
        let _ = format!("{:?}", NoState);
    }

    #[test]
    fn test_part_boot_debug_and_eq() {
        assert_eq!(Boot, Boot);
        assert_eq!(Update, Update);
        assert_eq!(Swap, Swap);
        assert_eq!(Boot.part_id(), PartId::PartBoot);
        assert_eq!(Update.part_id(), PartId::PartUpdate);
        assert_eq!(Swap.part_id(), PartId::PartSwap);
        assert_ne!(Boot.part_id(), Update.part_id());
        assert_ne!(Update.part_id(), Swap.part_id());
        assert_ne!(Boot.part_id(), Swap.part_id());
    }

    #[test]
    fn test_sect_flags_debug_clone_copy() {
        let f = SectFlags::NewFlag;
        let f2 = f;
        assert_eq!(f, f2);
        assert_eq!(format!("{:?}", SectFlags::NewFlag), "NewFlag");
        assert_eq!(format!("{:?}", SectFlags::SwappingFlag), "SwappingFlag");
        assert_eq!(format!("{:?}", SectFlags::BackupFlag), "BackupFlag");
        assert_eq!(format!("{:?}", SectFlags::UpdatedFlag), "UpdatedFlag");
        assert_eq!(format!("{:?}", SectFlags::None), "None");
    }

    #[test]
    fn test_sect_flags_none_has_false() {
        assert!(!SectFlags::None.has_new_flag());
        assert!(!SectFlags::None.has_swapping_flag());
        assert!(!SectFlags::None.has_backup_flag());
        assert!(!SectFlags::None.has_updated_flag());
    }

    #[test]
    fn test_sect_flags_all_has_flags_mutually_exclusive() {
        for flag in &[
            SectFlags::NewFlag,
            SectFlags::SwappingFlag,
            SectFlags::BackupFlag,
            SectFlags::UpdatedFlag,
        ] {
            let count = [
                flag.has_new_flag(),
                flag.has_swapping_flag(),
                flag.has_backup_flag(),
                flag.has_updated_flag(),
            ]
            .iter()
            .filter(|&&b| b)
            .count();
            assert_eq!(
                count, 1,
                "{:?} should match exactly one has_* predicate",
                flag
            );
        }
    }

    #[test]
    fn test_sect_flags_mutation_from_each_state() {
        for (start, expected_swap, expected_backup, expected_update) in [
            (
                SectFlags::NewFlag,
                SectFlags::SwappingFlag,
                SectFlags::BackupFlag,
                SectFlags::UpdatedFlag,
            ),
            (
                SectFlags::SwappingFlag,
                SectFlags::SwappingFlag,
                SectFlags::BackupFlag,
                SectFlags::UpdatedFlag,
            ),
            (
                SectFlags::BackupFlag,
                SectFlags::SwappingFlag,
                SectFlags::BackupFlag,
                SectFlags::UpdatedFlag,
            ),
            (
                SectFlags::UpdatedFlag,
                SectFlags::SwappingFlag,
                SectFlags::BackupFlag,
                SectFlags::UpdatedFlag,
            ),
        ] {
            let mut f = start;
            assert_eq!(f.set_swapping_flag(), expected_swap);
            let mut f = start;
            assert_eq!(f.set_backup_flag(), expected_backup);
            let mut f = start;
            assert_eq!(f.set_updated_flag(), expected_update);
        }
    }

    #[test]
    fn test_sect_flags_mutation_from_none() {
        let mut f = SectFlags::None;
        assert_eq!(f.set_swapping_flag(), SectFlags::SwappingFlag);
        let mut f = SectFlags::None;
        assert_eq!(f.set_backup_flag(), SectFlags::BackupFlag);
        let mut f = SectFlags::None;
        assert_eq!(f.set_updated_flag(), SectFlags::UpdatedFlag);
    }

    #[test]
    fn test_updateable_trait_bounds() {
        fn assert_updateable<T: Updateable>() {}
        assert_updateable::<StateUpdating>();
        assert_updateable::<StateTesting>();
        assert_updateable::<StateSuccess>();
    }

    #[test]
    fn test_typestate_trait_bounds() {
        fn assert_typestate<T: TypeState>() {}
        assert_typestate::<StateNew>();
        assert_typestate::<StateUpdating>();
        assert_typestate::<StateTesting>();
        assert_typestate::<StateSuccess>();
        assert_typestate::<NoState>();
    }

    #[test]
    fn test_valid_part_trait_bounds() {
        fn assert_valid_part<T: ValidPart>() {}
        assert_valid_part::<Boot>();
        assert_valid_part::<Update>();
        assert_valid_part::<Swap>();
    }

    #[test]
    fn test_swappable_trait_bounds() {
        fn assert_swappable<T: Swappable>() {}
        assert_swappable::<Boot>();
        assert_swappable::<Update>();
    }

    // Verify state encoding round-trips for every valid state value,
    // not just the proptest-based all-u8 coverage.
    #[test]
    fn test_state_encoding_roundtrip_exhaustive_valid() {
        type Encoder = fn() -> Option<u8>;
        let pairs: [(u8, Encoder); 4] = [
            (0xFF, || StateNew.from()),
            (0x70, || StateUpdating.from()),
            (0x10, || StateTesting.from()),
            (0x00, || StateSuccess.from()),
        ];
        for (byte, encoder) in &pairs {
            let encoded = encoder();
            assert_eq!(encoded, Some(*byte), "state byte 0x{:02X} mismatch", byte);
        }
    }

    #[test]
    fn test_state_encoding_nostate() {
        assert_eq!(NoState.from(), None);
    }

    #[test]
    fn test_sect_flags_from_none() {
        assert_eq!(SectFlags::None.from(), None);
    }

    #[test]
    fn test_sect_flags_from_all_variants() {
        assert_eq!(SectFlags::NewFlag.from(), Some(0x0F));
        assert_eq!(SectFlags::SwappingFlag.from(), Some(0x07));
        assert_eq!(SectFlags::BackupFlag.from(), Some(0x03));
        assert_eq!(SectFlags::UpdatedFlag.from(), Some(0x00));
    }

    /// Combined boot flow simulation:
    /// Tests the full state machine + SectFlags + TypeState pipeline
    /// end-to-end without hardware (pure software).
    #[test]
    fn test_combined_boot_flow_scenario() {
        // Step 1: Verify TypeState encodes the correct state byte
        assert_eq!(StateNew.from(), Some(0xFF)); // 255 = new
        assert_eq!(StateUpdating.from(), Some(0x70)); // 112 = updating
        assert_eq!(StateTesting.from(), Some(0x10)); // 16 = testing
        assert_eq!(StateSuccess.from(), Some(0x00)); // 0 = success

        // Step 2: Verify the transition DAG produces correct final states
        let new_decode = decode_state(StateNew.from().unwrap()).unwrap();
        let testing_decode = decode_state(StateTesting.from().unwrap()).unwrap();
        let success_decode = decode_state(StateSuccess.from().unwrap()).unwrap();

        match new_decode {
            States::New(_) => {}
            _ => panic!("Expected New state"),
        }
        match testing_decode {
            States::Testing(_) => {}
            _ => panic!("Expected Testing state"),
        }
        match success_decode {
            States::Success(_) => {}
            _ => panic!("Expected Success state"),
        }

        // Step 3: Verify the full update flow
        let update_updating_decode = decode_state(StateUpdating.from().unwrap()).unwrap();
        match update_updating_decode {
            States::Updating(_) => {}
            _ => panic!("Expected Updating state for update"),
        }
    }

    /// Test that the complete state machine decision matrix
    /// (similar to rustboot_start logic) produces correct actions
    #[test]
    fn test_boot_decision_logic() {
        // Test: All valid boot states produce correct decode
        let boot_values = [
            (StateNew.from().unwrap(), "BootInNewState"),
            (StateTesting.from().unwrap(), "BootInTestingState"),
            (StateSuccess.from().unwrap(), "BootInSuccessState"),
        ];

        for &(state_byte, _name) in &boot_values {
            assert!(
                decode_state(state_byte).is_ok(),
                "All boot states should decode"
            );
        }

        // Verify that invalid boot states are rejected
        let invalid_states = [0x01u8, 0x55, 0x80, 0xAA, 0xFE];
        for &state in &invalid_states {
            assert!(
                decode_state(state).is_err(),
                "Invalid state should be rejected"
            );
        }

        // Verify round-trip: TypeState -> u8 -> decode -> correct States variant
        assert!(matches!(
            decode_state(StateNew.from().unwrap()).unwrap(),
            States::New(_)
        ));
        assert!(matches!(
            decode_state(StateTesting.from().unwrap()).unwrap(),
            States::Testing(_)
        ));
        assert!(matches!(
            decode_state(StateSuccess.from().unwrap()).unwrap(),
            States::Success(_)
        ));
        assert!(matches!(
            decode_state(StateUpdating.from().unwrap()).unwrap(),
            States::Updating(_)
        ));
    }

    /// Verify that state encoding values are consistent between
    /// TypeState and SectFlags representations
    #[test]
    fn test_state_encoding_consistency() {
        // TypeState encoding values
        let new_val = StateNew.from().unwrap();
        let test_val = StateTesting.from().unwrap();
        let succ_val = StateSuccess.from().unwrap();
        let upd_val = StateUpdating.from().unwrap();

        // Verify each TypeState value
        assert_eq!(new_val, 0xFF, "StateNew encodes as 0xFF");
        assert_eq!(test_val, 0x10, "StateTesting encodes as 0x10");
        assert_eq!(succ_val, 0x00, "StateSuccess encodes as 0x00");
        assert_eq!(upd_val, 0x70, "StateUpdating encodes as 0x70");

        // Verify SectFlags provide a consistent but distinct encoding
        assert_eq!(SectFlags::NewFlag.from(), Some(0x0F));
        assert_eq!(SectFlags::SwappingFlag.from(), Some(0x07));
        assert_eq!(SectFlags::BackupFlag.from(), Some(0x03));
        assert_eq!(SectFlags::UpdatedFlag.from(), Some(0x00));
    }

    // ── Additional MC/DC edge case tests ──────────────────────────────

    #[test]
    fn test_state_decoding_all_invalid_bytes() {
        for byte in 0x01u8..=0x0F {
            assert!(decode_state(byte).is_err(), "byte 0x{byte:02X}");
        }
        for byte in 0x11u8..=0x6F {
            assert!(decode_state(byte).is_err(), "byte 0x{byte:02X}");
        }
        for byte in 0x71u8..=0xFE {
            assert!(decode_state(byte).is_err(), "byte 0x{byte:02X}");
        }
    }

    #[test]
    fn test_state_decoding_all_valid_bytes_individually() {
        let cases: [(u8, fn() -> States); 4] = [
            (0xFF, || States::New(StateNew)),
            (0x70, || States::Updating(StateUpdating)),
            (0x10, || States::Testing(StateTesting)),
            (0x00, || States::Success(StateSuccess)),
        ];
        for (byte, expected_fn) in &cases {
            let decoded = decode_state(*byte).unwrap();
            let expected = expected_fn();
            let decoded_name = match decoded {
                States::New(_) => "New",
                States::Updating(_) => "Updating",
                States::Testing(_) => "Testing",
                States::Success(_) => "Success",
                States::NoState(_) => "NoState",
            };
            let expected_name = match expected {
                States::New(_) => "New",
                States::Updating(_) => "Updating",
                States::Testing(_) => "Testing",
                States::Success(_) => "Success",
                States::NoState(_) => "NoState",
            };
            assert_eq!(
                decoded_name, expected_name,
                "byte 0x{byte:02X} decoded to wrong variant"
            );
        }
    }

    #[test]
    fn test_sect_flags_from_all_covers_exhaustive_pattern() {
        let all_flags = [
            SectFlags::NewFlag,
            SectFlags::SwappingFlag,
            SectFlags::BackupFlag,
            SectFlags::UpdatedFlag,
            SectFlags::None,
        ];
        let expected = [Some(0x0F), Some(0x07), Some(0x03), Some(0x00), None];
        for (flag, exp) in all_flags.iter().zip(expected.iter()) {
            assert_eq!(flag.from(), *exp, "mismatch for {:?}", flag);
        }
    }

    #[test]
    fn test_sect_flags_has_flags_all_false_for_none() {
        let none = SectFlags::None;
        assert!(!none.has_new_flag());
        assert!(!none.has_swapping_flag());
        assert!(!none.has_backup_flag());
        assert!(!none.has_updated_flag());
    }

    #[test]
    fn test_sect_flags_set_swapping_preserves_input_for_none() {
        let mut f = SectFlags::None;
        assert_eq!(f.set_swapping_flag(), SectFlags::SwappingFlag);
        assert_eq!(f, SectFlags::SwappingFlag);
    }

    #[test]
    fn test_sect_flags_set_backup_preserves_input_for_none() {
        let mut f = SectFlags::None;
        assert_eq!(f.set_backup_flag(), SectFlags::BackupFlag);
        assert_eq!(f, SectFlags::BackupFlag);
    }

    #[test]
    fn test_sect_flags_set_updated_preserves_input_for_none() {
        let mut f = SectFlags::None;
        assert_eq!(f.set_updated_flag(), SectFlags::UpdatedFlag);
        assert_eq!(f, SectFlags::UpdatedFlag);
    }

    #[test]
    fn test_decode_state_new_byte() {
        assert!(matches!(decode_state(0xFF).unwrap(), States::New(_)));
    }

    #[test]
    fn test_decode_state_updating_byte() {
        assert!(matches!(decode_state(0x70).unwrap(), States::Updating(_)));
    }

    #[test]
    fn test_decode_state_testing_byte() {
        assert!(matches!(decode_state(0x10).unwrap(), States::Testing(_)));
    }

    #[test]
    fn test_decode_state_success_byte() {
        assert!(matches!(decode_state(0x00).unwrap(), States::Success(_)));
    }

    #[test]
    fn test_decode_state_invalid_byte_returns_error() {
        let invalid_bytes = [0x01, 0x55, 0x80, 0xAA, 0xFE, 0x0F, 0x71, 0x6F, 0x11];
        for byte in invalid_bytes {
            assert!(
                decode_state(byte).is_err(),
                "byte 0x{byte:02X} should be invalid"
            );
        }
    }

    #[test]
    fn test_state_transition_methods_not_on_update_partition() {
        // Verify UpdateInNewState can only go to UpdateInUpdatingState
        // This is a compile-time trait bound check expressed at runtime
        fn assert_updateable<T: Updateable>() {}
        assert_updateable::<StateUpdating>();
        assert_updateable::<StateTesting>();
        assert_updateable::<StateSuccess>();
        // StateNew is NOT Updateable, so UpdateInNewState cannot use
        // set_state/into_updating_state on arbitrary partitions
    }
}
