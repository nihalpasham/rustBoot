use super::sealed::Sealed;
use crate::constants::*;
use crate::crypto::signatures::*;
use crate::parser::*;
use crate::{Result, RustbootError};

use crate::flashapi::FlashApi;

use core::ops::Add;
#[cfg(feature = "secp256k1")]
use k256::{
    ecdsa::VerifyingKey,
    elliptic_curve::{generic_array::GenericArray, FieldSize, consts::U32},
    EncodedPoint, Secp256k1,
};
#[cfg(feature = "nistp256")]
use p256::{
    ecdsa::VerifyingKey,
    elliptic_curve::{generic_array::GenericArray, FieldSize, consts::U32},
    EncodedPoint, NistP256,
};

use sha2::{Digest, Sha256, Sha384};

use core::convert::TryInto;
use core::fmt::Display;
use core::lazy::OnceCell;

/// Singleton to ensure we only ever have one instance of the `BOOT` partition
static mut BOOT: OnceCell<PartDescriptor<Boot>> = OnceCell::new();
/// Singleton to ensure we only ever have one instance of the `UPDATE` partition
static mut UPDT: OnceCell<PartDescriptor<Update>> = OnceCell::new();
/// Singleton to ensure we only ever have one instance of the `SWAP` partition
static mut SWAP: OnceCell<PartDescriptor<Swap>> = OnceCell::new();

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
/// (legal) states that are allowed to transition from `StateTesting` to `StateUpdating` or
/// vice-versa.
///
/// *Note: Not all `rustboot states` are updateable. The only 2 updateable states are*
/// - [`StateTesting`] - if the boot partition is still marked as 'statetesting` after an
/// update, a roll-back is triggered
/// - [`StateUpdating`] - if the update partition contains a downloaded update and is
/// marked as `stateupdating`, an update will be triggered
pub trait Updateable: Sealed + TypeState {}

/// Represents the state of a partition/image. [`StateNew`] refers to
/// a state when an image has not been staged for boot, or triggered for an update.
///
/// - If an image is present, no flags are active.
#[derive(Debug)]
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
pub struct StateSuccess;
impl TypeState for StateSuccess {
    fn from(&self) -> Option<u8> {
        Some(0x00)
    }
}
/// We use the [`NoState`] type to represent `non-existent state`.
///
/// **Example:** the `swap partition` has no state field and does not need one.
#[derive(Debug)]
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
    pub fn open_partition(part: Part) -> Result<ImageType<'static>> {
        match part.part_id() {
            PartId::PartBoot => {
                let mut size = 0x0;
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

                match part_desc.get_part_status()? {
                    States::New(state) => Ok(ImageType::BootInNewState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    States::Testing(state) => Ok(ImageType::BootInTestingState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    States::Success(state) => Ok(ImageType::BootInSuccessState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(state),
                    })),
                    _ => todo!(),
                }
            }
            PartId::PartUpdate => {
                let mut size = 0x0;
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
                match part_desc.get_part_status()? {
                    States::New(state) => Ok(ImageType::UpdateInNewState(RustbootImage {
                        part_desc: unsafe {
                            UPDT.set(part_desc);
                            &mut UPDT
                        },
                        state: Some(state),
                    })),
                    States::Updating(state) => {
                        Ok(ImageType::UpdateInUpdatingState(RustbootImage {
                            part_desc: unsafe {
                                UPDT.set(part_desc);
                                &mut UPDT
                            },
                            state: Some(state),
                        }))
                    }
                    _ => todo!(),
                }
            }
            PartId::PartSwap => {
                /// Open and initialize a new partition of type `SWAP`.
                /// This is an exclusive constructor for the `swap` partition.
                let part_desc = PartDescriptor {
                    hdr: None,
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
                        SWAP.set(part_desc);
                        &mut SWAP
                    },
                    state: None,
                }))
            }
        }
    }
}

impl<Part: ValidPart + Swappable> PartDescriptor<Part> {
    fn get_part_status(&self) -> Result<States> {
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            return Err(RustbootError::InvalidImage);
        }
        let state = unsafe { *self.get_partition_state()? };
        let state = match state {
            0xFF => Ok(States::New(StateNew)),
            0x70 => Ok(States::Updating(StateUpdating)),
            0x10 => Ok(States::Testing(StateTesting)),
            0x00 => Ok(States::Success(StateSuccess)),
            _ => Err(RustbootError::InvalidState),
        };
        state
    }

    pub fn set_state<State: TypeState + Updateable>(
        &self,
        updater: impl FlashApi,
        state: &State,
    ) -> Result<bool> {
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            self.set_partition_magic(updater);
        }
        let current_state = unsafe { *self.get_partition_state()? };
        let new_state = state.from().unwrap();
        if current_state != new_state {
            self.set_partition_state(updater, new_state);
        }
        Ok(true)
    }

    fn get_partition_magic(&self) -> Result<*const u32> {
        Ok(self.get_trailer_at_offset(0)? as *const u32)
    }

    fn set_partition_magic(&self, updater: impl FlashApi) -> Result<()> {
        let trailer_magic = (&RUSTBOOT_MAGIC_TRAIL as *const usize) as *const u8;
        Ok(updater.flash_trailer_write(self, 0, trailer_magic, MAGIC_TRAIL_LEN))
    }

    fn get_partition_state(&self) -> Result<*const u8> {
        self.get_trailer_at_offset(1)
    }

    pub fn set_partition_state(&self, updater: impl FlashApi, state: u8) -> Result<()> {
        let state = &state as *const u8;
        Ok(updater.flash_trailer_write(self, 1, state, PART_STATUS_LEN))
    }

    fn get_trailer_at_offset(&self, offset: usize) -> Result<*const u8> {
        match self.trailer {
            Some(trailer_addr) => Ok((trailer_addr as usize - (4 + offset)) as *const u8),
            None => Err(RustbootError::FieldNotSet),
        }
    }

    fn set_trailer_at(&self, updater: impl FlashApi, offset: usize, flag: u8) -> Result<()> {
        let newflag = &flag as *const u8;
        Ok(updater.flash_trailer_write(self, offset, newflag, 1))
    }
}

impl PartDescriptor<Update> {
    pub fn get_flags(&self, sector: usize) -> Result<SectFlags> {
        let sector_position = sector >> 1;
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            return Err(RustbootError::InvalidImage);
        }
        let mut flags = 0u8;
        let res = unsafe { *self.get_update_sector_flags(sector_position)? };
        if (sector == (sector_position << 1)) {
            flags = res & 0x0F;
        } else {
            flags = (res & 0x0F) >> 4;
        }
        match flags {
            0x0F => Ok(SectFlags::NewFlag),
            0x07 => Ok(SectFlags::SwappingFlag),
            0x03 => Ok(SectFlags::BackupFlag),
            0x00 => Ok(SectFlags::UpdatedFlag),
            _ => return Err(RustbootError::InvalidSectFlag),
        }
    }

    pub fn get_update_sector_flags(&self, offset: usize) -> Result<*const u8> {
        self.get_trailer_at_offset(2 + offset)
    }
    pub fn set_flags(&self, updater: impl FlashApi, sector: usize, flag: SectFlags) -> Result<()> {
        let newflag = flag.from().ok_or(RustbootError::InvalidSectFlag)?;
        let sector_position = sector >> 1;
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            return Err(RustbootError::InvalidImage);
        }
        let mut flags = 0u8;
        let res = unsafe { *self.get_update_sector_flags(sector_position)? };
        if (sector == (sector_position << 1)) {
            flags = (res & 0xF0) | (newflag & 0x0F);
        } else {
            flags = ((newflag & 0x0F) << 4) | (res & 0x0F);
        }
        if flags != res {
            self.set_update_sector_flags(updater, sector_position, flags);
        }
        Ok(())
    }

    fn set_update_sector_flags(&self, updater: impl FlashApi, pos: usize, flag: u8) -> Result<()> {
        self.set_trailer_at(updater, (2 + pos), flag)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    pub fn from(&self) -> Option<u8> {
        match self {
            NewFlag => Some(0x0F),
            SwappingFlag => Some(0x07),
            BackupFlag => Some(0x03),
            UpdatedFlag => Some(0x00),
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
        let (val) = parse_tlv(self, Tags::Version)?;
        let fw_version =
            u32::from_be_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
        Ok(fw_version)
    }
}

impl<'a, Part: ValidPart + Swappable, State: Updateable> RustbootImage<'a, Part, State> {
    pub fn get_state(&self) -> &State {
        let state = self.state.as_ref().unwrap();
        state
    }
    pub fn get_image_type(&self) -> Result<u16> {
        let (val) = parse_tlv(self, Tags::ImgType)?;
        let image_type =
            u16::from_be_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
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
                let mut integrity_check = false;
                let hash_type = HDR_SHA256;
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
                        if (computed_hash.as_slice() != stored_hash) {
                            return Err(RustbootError::IntegrityCheckFailed);
                        }
                        integrity_check = true;
                        Some(stored_hash.as_ptr())
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
                if integrity_check.eq(&true) {
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
            _ => todo!(),
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
                let mut auth_check = false;
                let signature_type = HDR_SIGNATURE;
                let fw_size = self
                    .part_desc
                    .get()
                    .ok_or(RustbootError::FieldNotSet)?
                    .fw_size;
                let res = parse_tlv(self, Tags::Signature);
                let computed_hash = match res {
                    Ok(stored_signature) => {
                        let (img_type_val) = parse_tlv(self, Tags::ImgType)?;
                        let val = img_type_val[0] as u16 + ((img_type_val[1] as u16) << 8);
                        if ((val & 0xFF00) != N) {
                            return Err(RustbootError::InvalidValue);
                        }
                        // verify signature
                        let hasher2 = compute_img_hash::<Part, State, Sha256, SHA256_DIGEST_SIZE>(
                            self, fw_size,
                        )?;
                        let computed_hash = Some(hasher2.clone().finalize().as_ptr());
                        auth_check = verify_ecc256_signature::<Sha256, HDR_IMG_TYPE_AUTH>(
                            hasher2,
                            &stored_signature,
                        )?;
                        computed_hash
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
                if auth_check.eq(&true) {
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
            HDR_IMG_TYPE_AUTH => todo!(),
            _ => todo!(),
        }
    }
}

/// Computes the hash of an image contained in a partition. This function returns
/// a `generic result` i.e. a [`Digest`] instance, rather than a raw digest value.
///
/// To get the actual hash output, we call the hasher's finalize mthod.
///
/// *Note - `Offset` represents an offset (the `SHA_TLV` field) from the start of header
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
    let part_desc = img.part_desc.get().unwrap();
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
            SHA384_DIGEST_SIZE => todo!(),
            _ => todo!(),
        }
    } else {
        return Err(RustbootError::InvalidValue);
    }
}

/// Performs the signature verification; take as argument, a pre-updated
/// [`Digest`] instance thats needs to be finalized and the associated signature
/// to be verified.
fn verify_ecc256_signature<D: Digest<OutputSize = U32>, const N: u16>(
    digest: D,
    signature: &[u8],
) -> Result<bool> {
    match N {
        #[cfg(feature = "nistp256")]
        IMG_TYPE_AUTH_ECC256 => {
            if let VerifyingKeyTypes::VKeyNistP256(vk) = import_pubkey(PubkeyTypes::NistP256)? {
                let ecc256_verifier = NistP256Signature { verify_key: vk };
                let res = ecc256_verifier.verify(digest, signature)?;
                match res {
                    true => Ok(true),
                    false => Err(RustbootError::FwAuthFailed),
                }
            } else {
                Err(RustbootError::Unreachable)
            }
        }
        #[cfg(feature = "secp256k1")]
        IMG_TYPE_AUTH_ECC256 => {
            let ecc256_verifier = Secp256k1Signature {
                verify_key: import_pubkey(PubkeyTypes::Secp256k1)?,
            };
            let res = ecc256_verifier.verify(digest, signature)?;
            match res {
                true => {
                    defmt::info!("verify_ecc256_success");
                    Ok(true)
                }
                false => {
                    defmt::info!("verify_ecc256_failed");
                    Err(RustbootError::FwAuthFailed)
                }
            }
        }
        #[cfg(feature = "ed25519")]
        IMG_TYPE_AUTH_ED25519 => todo!(),
        _ => todo!(),
    }
}

enum PubkeyTypes {
    Secp256k1,
    Ed25519,
    NistP256,
    NistP384,
}

enum VerifyingKeyTypes {
    #[cfg(feature = "secp256k1")]
    VKey256k1(VerifyingKey),
    #[cfg(feature = "nistp256")]
    VKeyNistP256(VerifyingKey),
    VKeyEd25519,
    VKeyNistP384,
}


/// Imports a raw public key embedded in the bootloader.
///
/// *Note: this function can be extended to add support for HW
/// secure elements*
fn import_pubkey(pk: PubkeyTypes) -> Result<VerifyingKeyTypes> {
    match pk {
        #[cfg(feature = "secp256k1")]
        PubkeyTypes::Secp256k1 => {
            let embedded_pubkey = [
                0x74, 0xBF, 0x5D, 0xE9, 0xF8, 0x69, 0x69, 0x44, 0x35, 0xAE, 0xB7, 0x39, 0x6F, 0xA1,
                0x40, 0x11, 0xB6, 0xA1, 0x7F, 0x2D, 0x8A, 0x86, 0xB9, 0x58, 0xBC, 0x4A, 0x51, 0xF7,
                0xF3, 0x0F, 0x23, 0x77, 0x78, 0x0E, 0x11, 0x46, 0x95, 0x3A, 0x1D, 0xDF, 0x69, 0xCD,
                0x34, 0x23, 0xFE, 0x63, 0x05, 0x15, 0x30, 0x43, 0xBB, 0x9E, 0x75, 0x63, 0xE0, 0x41,
                0x6A, 0x70, 0xCE, 0x16, 0x0A, 0x60, 0x2A, 0x38,
            ];
            let untagged_bytes: &GenericArray<u8, <FieldSize<Secp256k1> as Add>::Output> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
            // `try_from` is fallible i.e. it will check to see if the point is on the curve.
            let secp256k1_vk = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
                .map_err(|_| RustbootError::ECCError);
            Ok(VerifyingKeyTypes::VKey256k1(secp256k1_vk?))
        }
        #[cfg(feature = "nistp256")]
        PubkeyTypes::NistP256 => {
            let embedded_pubkey = [
                0x74, 0xBF, 0x5D, 0xE9, 0xF8, 0x69, 0x69, 0x44, 0x35, 0xAE, 0xB7, 0x39, 0x6F, 0xA1,
                0x40, 0x11, 0xB6, 0xA1, 0x7F, 0x2D, 0x8A, 0x86, 0xB9, 0x58, 0xBC, 0x4A, 0x51, 0xF7,
                0xF3, 0x0F, 0x23, 0x77, 0x78, 0x0E, 0x11, 0x46, 0x95, 0x3A, 0x1D, 0xDF, 0x69, 0xCD,
                0x34, 0x23, 0xFE, 0x63, 0x05, 0x15, 0x30, 0x43, 0xBB, 0x9E, 0x75, 0x63, 0xE0, 0x41,
                0x6A, 0x70, 0xCE, 0x16, 0x0A, 0x60, 0x2A, 0x38,
            ];
            let untagged_bytes: &GenericArray<u8, <FieldSize<NistP256> as Add>::Output> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
            // `try_from` is fallible i.e. it will check to see if the point is on the curve.
            let p256_vk = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
                .map_err(|_| RustbootError::ECCError);
            Ok(VerifyingKeyTypes::VKeyNistP256(p256_vk?))
        }
        _ => todo!(),
    }
}
