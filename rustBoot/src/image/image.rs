use super::sealed::Sealed;
use crate::crypto::signatures::*;
use crate::librustboot::*;
use crate::target::*;
use crate::{Result, RustbootError};

use k256::elliptic_curve::consts::U32;
use sha2::{Digest, Sha256, Sha384};

use core::convert::TryInto;
use core::lazy::OnceCell;

/// Singleton to ensure we only ever have one instance of the `BOOT` partition
static mut BOOT: OnceCell<PartDescriptor<Boot>> = OnceCell::new();
/// Singleton to ensure we only ever have one instance of the `UPDATE` partition
static mut UPDT: OnceCell<PartDescriptor<Update>> = OnceCell::new();
/// Singleton to ensure we only ever have one instance of the `SWAP` partition
static mut SWAP: OnceCell<PartDescriptor<Swap>> = OnceCell::new();

/// All valid `rustBoot states` must implement this (sealed) trait.
pub trait TypeState: Sealed {
    fn get_state_val(&self) -> Option<u8>;
}
/// Any `rustboot state` implementing this marker trait is updateable.
///
/// *Note: Not all `rustboot states` are updateable. For now, the only 2 updateable states are*
/// - [`StateTesting`] - if the boot partition is still marked as 'statetesting` after an
/// update, a roll-back is triggered
/// - [`StateUpdating`] - if the update partition contains a downloaded update and is
/// marked as `stateupdating`, an update will be triggered
pub trait Updateable: Sealed {}

/// Represents the state of a given partition/image. `StateNew` refers to
/// a state when an image has not been staged for boot, or triggered for an update.
///
/// - If an image is present, no flags are active.
#[derive(Debug)]
pub struct StateNew(u8);
impl TypeState for StateNew {
    fn get_state_val(&self) -> Option<u8> {
        Some(self.0)
    }
}
/// Represents the state of a given partition/image. This state is ONLY
/// valid in the `UPDATE` partition. The image is marked for update and should replace
/// the current image in `BOOT`.
#[derive(Debug)]
pub struct StateUpdating(u8);
impl TypeState for StateUpdating {
    fn get_state_val(&self) -> Option<u8> {
        Some(self.0)
    }
}
impl Updateable for StateUpdating {}
/// Represents the state of a given partition/image. This state is ONLY
/// valid in the `BOOT` partition. The image has just been swapped, and is pending
/// reboot. If present after reboot, it means that the updated image failed to boot,
/// despite being correctly verified. This particular situation triggers a rollback.
#[derive(Debug)]
pub struct StateTesting(pub u8);
impl TypeState for StateTesting {
    fn get_state_val(&self) -> Option<u8> {
        Some(self.0)
    }
}
impl Updateable for StateTesting {}
/// Represents the state of a given partition/image. This state is ONLY
/// valid in the `BOOT` partition. `Success` here indicates that image currently stored
/// in BOOT has been successfully staged at least once, and the update is now complete.
#[derive(Debug)]
pub struct StateSuccess(u8);
impl TypeState for StateSuccess {
    fn get_state_val(&self) -> Option<u8> {
        Some(self.0)
    }
}
/// We use the `NoState` type to represent `non-existent state`.
///
/// **Example:** the swap partition has no state field and does not need one.
#[derive(Debug)]
pub struct NoState;
impl TypeState for NoState {
    fn get_state_val(&self) -> Option<u8> {
        None
    }
}

/// All valid partitions implement `ValidPart`, which allows us to enumerate a valid partition.
pub trait ValidPart: Sealed {
    fn part_id(&self) -> PartId;
}
/// A marker trait to indicate which partitions are swappable.
pub trait Swappable: Sealed {}
/// Enumerated partitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartId {
    PartBoot,
    PartUpdate,
    PartSwap,
}
///  A zero-sized struct representing the `BOOT` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Boot;
impl Swappable for Boot {}
impl ValidPart for Boot {
    fn part_id(&self) -> PartId {
        PartId::PartBoot
    }
}
///  A zero-sized struct representing the `UPDATE` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Update;
impl Swappable for Update {}
impl ValidPart for Update {
    fn part_id(&self) -> PartId {
        PartId::PartUpdate
    }
}
///  A zero-sized struct representing the `SWAP` image/partition.
#[derive(Debug, PartialEq, Eq)]
pub struct Swap;
impl ValidPart for Swap {
    fn part_id(&self) -> PartId {
        PartId::PartSwap
    }
}

#[derive(Debug)]
pub(crate) struct PartDescriptor<Part: ValidPart> {
    pub hdr: Option<*const u8>,
    fw_base: *const u8,
    sha_hash: Option<*const u8>,
    trailer: Option<*const u8>,
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
    /// create `RustbootImage` instances.
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

                match part_desc.get_state() {
                    Ok(0xFF) => Ok(ImageType::BootInNewState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(StateNew(0xFF)),
                    })),
                    Ok(0x10) => Ok(ImageType::BootInTestingState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(StateTesting(0x10)),
                    })),
                    Ok(0x00) => Ok(ImageType::BootInSuccessState(RustbootImage {
                        part_desc: unsafe {
                            BOOT.set(part_desc);
                            &mut BOOT
                        },
                        state: Some(StateSuccess(0x00)),
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
                match part_desc.get_state() {
                    Ok(0xFF) => Ok(ImageType::UpdateInNewState(RustbootImage {
                        part_desc: unsafe {
                            UPDT.set(part_desc);
                            &mut UPDT
                        },
                        state: Some(StateNew(0xFF)),
                    })),
                    Ok(0x70) => Ok(ImageType::UpdateInUpdatingState(RustbootImage {
                        part_desc: unsafe {
                            UPDT.set(part_desc);
                            &mut UPDT
                        },
                        state: Some(StateUpdating(0x70)),
                    })),
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
    fn get_state(&self) -> Result<u8> {
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            return Err(RustbootError::InvalidImage);
        }
        let state = unsafe { *self.get_partition_state()? };
        Ok(state)
    }

    pub fn set_state<State: TypeState + Updateable>(&self, state: &State) -> Result<bool> {
        let magic_trailer = unsafe { *self.get_partition_magic()? };
        if (magic_trailer != RUSTBOOT_MAGIC_TRAIL as u32) {
            self.set_partition_magic();
        }
        let current_state = unsafe { *self.get_partition_state()? };
        let new_state = state.get_state_val().unwrap();
        if current_state != new_state {
            self.set_partition_state(new_state);
        }
        Ok(true)
    }

    fn get_partition_magic(&self) -> Result<*const u32> {
        Ok(self.get_trailer_at_offset(0)? as *const u32)
    }

    fn set_partition_magic(&self) {
        todo!()
    }

    fn get_partition_state(&self) -> Result<*const u8> {
        self.get_trailer_at_offset(1)
    }

    fn set_partition_state(&self, state: u8) {
        todo!()
    }

    fn get_trailer_at_offset(&self, offset: usize) -> Result<*const u8> {
        match self.trailer {
            Some(trailer_addr) => Ok((trailer_addr as usize - (4 + offset)) as *const u8),
            None => Err(RustbootError::FieldNotSet),
        }
    }

    fn set_trailer_at(&self, offset: usize, val: u8) -> Result<bool> {
        todo!()
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
    pub fn set_flags(&self, sector: usize, flag: SectFlags) {
        todo!()
    }

    fn set_update_sector_flags(&self, pos: usize, flag: u8) {
        self.set_trailer_at(2 + pos, flag);
        todo!()
    }
}

/// A struct to describe the layout and contents of a given image/partition.
/// The 2 generic type parameters indicate `partition type` and `partition state`.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct RustbootImage<'a, Part: ValidPart, State: TypeState> {
    pub part_desc: &'a mut OnceCell<PartDescriptor<Part>>,
    state: Option<State>,
}

/// An enum to hold all valid (i.e. legal) image-types or [`RustbootImage`]s.
#[derive(Debug)]
pub(crate) enum ImageType<'a> {
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
            state: Some(StateTesting(0x10)),
        }
    }
}

impl<'a> RustbootImage<'a, Boot, StateSuccess> {
    pub fn into_testing_state(self) -> RustbootImage<'a, Boot, StateTesting> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateTesting(0x10)),
        }
    }
}

impl<'a> RustbootImage<'a, Boot, StateTesting> {
    fn into_success_state(self) -> RustbootImage<'a, Boot, StateSuccess> {
        RustbootImage {
            part_desc: self.part_desc,
            state: Some(StateSuccess(0x00)),
        }
    }
}

impl<'a, Part: ValidPart + Swappable, State: TypeState> RustbootImage<'a, Part, State> {
        pub fn get_firmware_version(&self) -> Result<u32> {
        let (val, _, _) = parse_image_header(self, 0x03)?;
        let fw_version =
            u32::from_be_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
        Ok(fw_version)
    }
}

impl<'a, Part: ValidPart + Swappable, State: TypeState + Updateable>
    RustbootImage<'a, Part, State>
{
    pub fn get_state(&self) -> &State {
        let state = self.state.as_ref().unwrap();
        state
    }
    pub fn get_image_type(&self) -> Result<u16> {
        let (val, _, _) = parse_image_header(self, HDR_IMG_TYPE)?;
        let image_type =
            u16::from_be_bytes(val.try_into().map_err(|_| RustbootError::InvalidValue)?);
        Ok(image_type)
    }
}

impl<'a, Part: ValidPart + Swappable, State: TypeState>
    RustbootImage<'a, Part, State>
{
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
                let res = parse_image_header(self, hash_type);
                let stored_hash = match res {
                    Ok((stored_hash, hash_len, _)) => {
                        if N != hash_len as usize {
                            return Err(RustbootError::InvalidHdrFieldLength);
                        };
                        let hasher = compute_img_hash::<Part, State, Sha256, N>(self, fw_size)?;
                        let computed_hash = hasher.finalize();
                        if (computed_hash.as_slice() != stored_hash) {
                            return Err(RustbootError::IntegrityCheckFailed);
                        }
                        integrity_check = true;
                        Some(stored_hash.as_ptr())
                    }
                    Err(e) => return Err(e),
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
            #[cfg(feature = "secp256k1")]
            HDR_IMG_TYPE_AUTH => {
                let mut auth_check = false;
                let signature_type = HDR_SIGNATURE;
                let fw_size = self
                    .part_desc
                    .get()
                    .ok_or(RustbootError::FieldNotSet)?
                    .fw_size;
                let res = parse_image_header(self, signature_type);
                let computed_hash = match res {
                    Ok((stored_signature, signature_len, _)) => {
                        if ECC_SIGNATURE_SIZE != signature_len as usize {
                            return Err(RustbootError::InvalidHdrFieldLength);
                        };
                        let type_of_img = HDR_IMG_TYPE;
                        let (img_type_val, img_type_len, _) =
                            parse_image_header(self, type_of_img)?;
                        // Image type field length is 2 bytes.
                        if img_type_len != 2 {
                            return Err(RustbootError::InvalidHdrFieldLength);
                        };
                        let val = img_type_val[0] as u16 + (img_type_val[1] as u16) << 8;
                        if ((val & 0xFF00) != N) {
                            return Err(RustbootError::InvalidValue);
                        }
                        // verify signature
                        let hasher2 = compute_img_hash::<Part, State, Sha256, SHA256_DIGEST_SIZE>(
                            self, fw_size,
                        )?;
                        let computed_hash = Some(hasher2.clone().finalize().as_ptr());
                        auth_check = verify_ec256_signature::<Sha256, HDR_IMG_TYPE_AUTH>(
                            hasher2,
                            &stored_signature,
                        )?;
                        computed_hash
                    }
                    Err(e) => return Err(e),
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
                // In actuality, this is a fixed offset but rather than hard-code it,
                // I've chosen to compute it by parsing the img_header (for the SHA_TLV field) to accomodate
                // the insertion of possible new header fields.
                let (_, _, mut offset) = parse_image_header(img, HDR_SHA256)?;

                while offset > 0 {
                    if offset < block_size {
                        block_size = offset
                    }
                    hasher.update(&part[..block_size]);
                    offset -= block_size;
                }
                offset = 0x0; // reset offset to use as `fw_base`.
                while size > 0 {
                    if offset > size {
                        block_size = offset - size;
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
fn verify_ec256_signature<D: Digest<OutputSize = U32>, const N: u16>(
    digest: D,
    signature: &[u8],
) -> Result<bool> {
    match N {
        #[cfg(feature = "secp256k1")]
        IMG_TYPE_AUTH_ECC256 => {
            let ecc256_verifier = Secp256k1Signature(import_pubkey::<64>()?);
            let res = ecc256_verifier.verify(digest, signature)?;
            match res {
                true => Ok(true),
                false => Err(RustbootError::FwAuthFailed),
            }
        }
        #[cfg(feature = "ed25519")]
        IMG_TYPE_AUTH_ED25519 => todo!(),
        _ => todo!(),
    }
}

/// todo
fn import_pubkey<const N: usize>() -> Result<[u8; N]> {
    todo!()
}
