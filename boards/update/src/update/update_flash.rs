use core::marker::PhantomData;

use crate::hal::hal::*;
use rustBoot::constants::*;
use rustBoot::crypto::signatures::HDR_IMG_TYPE_AUTH;
use rustBoot::image::image::*;
use rustBoot::parser::*;
use rustBoot::{Result, RustbootError};

use super::UpdateInterface;
use rustBoot::flashapi::FlashApi;
use rustBoot_hal::FlashInterface;

struct RefinedUsize<const MIN: usize, const MAX: usize, const VAL: usize>(usize);

impl<const MIN: usize, const MAX: usize, const VAL: usize> RefinedUsize<MIN, MAX, VAL> {
    pub fn bounded_int(i: usize) -> Self {
        assert!(i >= MIN && i <= MAX);
        RefinedUsize(i)
    }
    pub fn single_valued_int(i: usize) -> Self {
        assert!(i == VAL);
        RefinedUsize(i)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlashUpdater<Interface> {
    iface: Interface,
}

impl<Interface> FlashUpdater<Interface>
where
    Interface: FlashInterface,
{
    pub fn new(iface: Interface) -> Self {
        FlashUpdater { iface }
    }
}
impl<Interface> FlashApi for &FlashUpdater<Interface>
where
    Interface: FlashInterface,
{
    fn flash_write<Part: ValidPart>(
        self,
        part: &PartDescriptor<Part>,
        offset: usize,
        data: *const u8,
        len: usize,
    ) {
        let addr = part.hdr.unwrap() as usize + offset;
        self.iface.hal_flash_write(addr, data, len)
    }
    fn flash_erase<Part: ValidPart>(self, part: &PartDescriptor<Part>, offset: usize, len: usize) {
        let addr = part.hdr.unwrap() as usize + offset;
        self.iface.hal_flash_erase(addr, len);
    }

    fn flash_trailer_write<Part: ValidPart + Swappable>(
        self,
        part: &PartDescriptor<Part>,
        offset: usize,
        data: *const u8,
        len: usize,
    ) {
        let addr = part.trailer.unwrap() as usize - (4 + offset);
        self.iface.hal_flash_write(addr, data, len)
    }

    fn flash_init() {}
    fn flash_unlock() {}
    fn flash_lock() {}
}

impl<Interface> FlashUpdater<Interface>
where
    Interface: FlashInterface,
{
    fn copy_sector<SrcPart: ValidPart, DstPart: ValidPart>(
        &self,
        src_part: &PartDescriptor<SrcPart>,
        dst_part: &PartDescriptor<DstPart>,
        sector: usize,
    ) -> Result<usize> {
        let mut pos = 0usize;
        let mut src_sector_offset = sector * SECTOR_SIZE;
        let mut dst_sector_offset = sector * SECTOR_SIZE;

        if (src_part.part.part_id() == PartId::PartSwap) {
            src_sector_offset = 0;
        }
        if (dst_part.part.part_id() == PartId::PartSwap) {
            dst_sector_offset = 0;
        }
        self.flash_erase(dst_part, dst_sector_offset, SECTOR_SIZE);
        while (pos < SECTOR_SIZE) {
            if (src_sector_offset + pos < (src_part.fw_size + IMAGE_HEADER_SIZE + FLASHBUFFER_SIZE))
            {
                let data =
                    ((src_part.hdr.unwrap() as usize) + src_sector_offset + pos) as *const u8;
                self.flash_write(dst_part, dst_sector_offset + pos, data, FLASHBUFFER_SIZE);
            }
            pos += FLASHBUFFER_SIZE;
        }
        Ok(pos)
    }

    fn rustboot_update<'a>(&self, rollback: bool) -> Result<RustbootImage<'a, Boot, StateTesting>> {
        let boot = PartDescriptor::open_partition(Boot, self)?;
        let updt = PartDescriptor::open_partition(Update, self)?;
        let swap = PartDescriptor::open_partition(Swap, self)?;

        let mut new_boot_img = None;

        match (updt, swap) {
            (ImageType::UpdateInUpdatingState(mut updt), ImageType::NoStateSwap(swap)) => {
                /* use largest size for the swap */
                let mut total_size = 0usize;
                let mut sector = 0usize;
                let mut flag = SectFlags::None;
                {
                    // This scope is to satisfy the borrow checker
                    let updt_part = updt.part_desc.get().unwrap();
                    let boot_part = match boot {
                        // Explicitly check all possible Boot states
                        ImageType::BootInNewState(ref boot) => {
                            let boot_fw_size = boot.part_desc.get().unwrap().fw_size; // can be unwrapped as it was checked during init.
                            let update_fw_size = updt_part.fw_size;
                            total_size = boot_fw_size + IMAGE_HEADER_SIZE;
                            if ((update_fw_size + IMAGE_HEADER_SIZE) > total_size) {
                                total_size = update_fw_size + IMAGE_HEADER_SIZE;
                            }
                            boot.part_desc.get()
                        }
                        ImageType::BootInSuccessState(ref boot) => {
                            let boot_fw_size = boot.part_desc.get().unwrap().fw_size; // can be unwrapped as it was checked during init.
                            let update_fw_size = updt_part.fw_size;
                            total_size = boot_fw_size + IMAGE_HEADER_SIZE;
                            if ((update_fw_size + IMAGE_HEADER_SIZE) > total_size) {
                                total_size = update_fw_size + IMAGE_HEADER_SIZE;
                            }
                            boot.part_desc.get()
                        }
                        // in case of a rollback
                        ImageType::BootInTestingState(ref boot) => {
                            let boot_fw_size = boot.part_desc.get().unwrap().fw_size; // can be unwrapped as it was checked during init.
                            let update_fw_size = updt_part.fw_size;
                            total_size = boot_fw_size + IMAGE_HEADER_SIZE;
                            if ((update_fw_size + IMAGE_HEADER_SIZE) > total_size) {
                                total_size = update_fw_size + IMAGE_HEADER_SIZE;
                            }
                            boot.part_desc.get()
                        }
                        _ => {
                            return Err(RustbootError::InvalidState);
                        }
                    };
                    if total_size <= IMAGE_HEADER_SIZE {
                        return Err(RustbootError::InvalidImage);
                    }
                    // Check the first sector to detect an interrupted update.
                    if updt_part.get_flags(0).is_err() || updt_part.get_flags(0)?.has_new_flag() {
                        let update_type = updt.get_image_type()?;
                        // In the event that this is a new update, perform the required checks on the update
                        // before starting the swap.
                        if ((update_type & HDR_MASK_LOWBYTE) != HDR_IMG_TYPE_APP)
                            || ((update_type & HDR_MASK_HIGHBYTE) != HDR_IMG_TYPE_AUTH)
                        {
                            return Err(RustbootError::ECCError);
                        }
                        if (!updt_part.hdr_ok
                            || updt.verify_integrity::<SHA256_DIGEST_SIZE>().is_err()
                            || updt.verify_authenticity::<HDR_IMG_TYPE_AUTH>().is_err())
                        {
                            panic!("firmware authentication failed");
                        }
                    }
                    // disallow downgrades
                    match boot {
                        ImageType::BootInNewState(ref boot) => {
                            if (!rollback
                                && (updt.get_firmware_version()? <= boot.get_firmware_version()?))
                            {
                                return Err(RustbootError::FwAuthFailed);
                            }
                        }
                        ImageType::BootInSuccessState(ref boot) => {
                            if (!rollback
                                && (updt.get_firmware_version()? <= boot.get_firmware_version()?))
                            {
                                return Err(RustbootError::FwAuthFailed);
                            }
                        }
                        ImageType::BootInTestingState(ref boot) => {
                            // do nothing as we actually want to rollback
                        }
                        _ => {
                            return Err(RustbootError::InvalidState);
                        }
                    }

                    /* Interruptible swap
                     * The status is saved in the sector flags of the update partition.
                     * If something goes wrong, the operation will be resumed upon reboot.
                     */
                    let boot_part = boot_part.unwrap();
                    let updt_part = updt.part_desc.get().unwrap();
                    let swap_part = swap.part_desc.get().unwrap();
                    while ((sector * SECTOR_SIZE) < total_size) {
                        if updt_part.get_flags(sector).is_err()
                            || updt_part.get_flags(sector)?.has_new_flag()
                        {
                            flag = flag.set_swapping_flag();
                            self.copy_sector(updt_part, swap_part, sector);
                            if (((sector + 1) * SECTOR_SIZE) < PARTITION_SIZE) {
                                updt_part.set_flags(self, sector, flag)?;
                            }
                        }
                        if flag.has_swapping_flag() {
                            flag = flag.set_backup_flag();
                            self.copy_sector(boot_part, updt_part, sector);
                            if (((sector + 1) * SECTOR_SIZE) < PARTITION_SIZE) {
                                updt_part.set_flags(self, sector, flag)?;
                            }
                        }
                        if flag.has_backup_flag() {
                            flag = flag.set_updated_flag();
                            self.copy_sector(swap_part, boot_part, sector);
                            if (((sector + 1) * SECTOR_SIZE) < PARTITION_SIZE) {
                                updt_part.set_flags(self, sector, flag)?;
                            }
                        }
                        sector += 1;
                    }

                    while ((sector * SECTOR_SIZE) < PARTITION_SIZE) {
                        self.flash_erase(boot_part, sector * SECTOR_SIZE, SECTOR_SIZE);
                        self.flash_erase(updt_part, sector * SECTOR_SIZE, SECTOR_SIZE);
                        sector += 1;
                    }
                    self.flash_erase(swap_part, 0, SECTOR_SIZE);
                }
                // Re-open the `Boot` partition after swap.
                // Note: A successful swap moves the image in the update partition to the boot partition.
                // TODO: As we're using singletons (i.e. BOOT, UPDT), swap the following `rustBoot header` fields -
                //       size, sha_hash, signature_ok, sha_ok, hdr_ok.
                let boot = PartDescriptor::open_partition(Boot, self).unwrap();
                // the only valid state for the boot partition after a swap is `newState` as all state
                // info is erased post the swap.
                let new_img = match boot {
                    // Transition from current boot state to `StateTesting`. This step consumes the old
                    // bootImage (i.e. struct) and returns the new bootImage with the new state.
                    ImageType::BootInNewState(img) => img.into_testing_state(),
                    _ => return Err(RustbootError::InvalidState),
                };
                // Set new status byte in the boot partition.
                new_img
                    .part_desc
                    .get()
                    .unwrap()
                    .set_state(self, new_img.get_state());
                new_boot_img = Some(new_img);
            }
            _ => return Err(RustbootError::InvalidState),
        }
        Ok(new_boot_img.unwrap())
    }
}

impl<Interface> UpdateInterface for &FlashUpdater<Interface>
where
    Interface: FlashInterface,
{
    fn rustboot_start(self) -> ! {
        let mut boot = PartDescriptor::open_partition(Boot, self).unwrap();
        let updt = PartDescriptor::open_partition(Update, self).unwrap();

        // Check the BOOT partition for state - if it is still in TESTING, trigger rollback.
        if let ImageType::BootInTestingState(_v) = boot {
            self.update_trigger();
            match self.rustboot_update(true) {
                Ok(_v) => {}
                Err(_e) => {
                    panic!("rollback failed.")
                }
            }
        // Check the UPDATE partition for state - if it is marked as UPDATING, trigger update.
        } else if let ImageType::UpdateInUpdatingState(_v) = updt {
            match self.rustboot_update(false) {
                Ok(_v) => {}
                Err(_e) => {
                    panic!("update-swap failed.")
                }
            }
        } else {
            match boot {
                ImageType::BootInNewState(ref mut img) => {
                    if (img.verify_integrity::<SHA256_DIGEST_SIZE>().is_err()
                        || img.verify_authenticity::<HDR_IMG_TYPE_AUTH>().is_err())
                    {
                        match self.rustboot_update(true) {
                            Err(_v) => {
                                #[cfg(feature = "defmt")]
                                panic!("all boot options exhausted")
                            } // all boot options exhausted
                            Ok(ref mut img) => {
                                // Emergency update successful, try to re-authenticate boot image.
                                if (img.verify_integrity::<SHA256_DIGEST_SIZE>().is_err()
                                    || img.verify_authenticity::<HDR_IMG_TYPE_AUTH>().is_err())
                                {
                                    panic!("something went wrong after the emergency update")
                                    // something went wrong after the emergency update
                                }
                            }
                        }
                    }
                }
                ImageType::BootInSuccessState(ref mut img) => {
                    if (img.verify_integrity::<SHA256_DIGEST_SIZE>().is_err()
                        || img.verify_authenticity::<HDR_IMG_TYPE_AUTH>().is_err())
                    {
                        match self.rustboot_update(true) {
                            Err(_v) => {
                                #[cfg(feature = "defmt")]
                                panic!("all boot options exhausted")
                            } // all boot options exhausted
                            Ok(ref mut img) => {
                                // Emergency update successful, try to re-authenticate boot image.
                                if (img.verify_integrity::<SHA256_DIGEST_SIZE>().is_err()
                                    || img.verify_authenticity::<HDR_IMG_TYPE_AUTH>().is_err())
                                {
                                    panic!("something went wrong after the emergency update")
                                    // something went wrong after the emergency update
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        // After an update or rollback re-open the `boot` partition.
        // Note: Swapping moves the image in the update partition to the boot partition.
        // TODO: As we're using singletons (i.e. BOOT, UPDT), swap the following `rustBoot header` fields -
        //       size, sha_hash, signature_ok, sha_ok, hdr_ok.
        let boot = PartDescriptor::open_partition(Boot, self).unwrap();
        match boot {
            ImageType::BootInNewState(img) => {
                let boot_part = img.part_desc.get().unwrap();
                let base_img_addr = RefinedUsize::<0, 0, BOOT_FWBASE>::single_valued_int(
                    boot_part.fw_base as usize,
                )
                .0;
                hal_preboot();
                hal_boot_from(base_img_addr)
            }
            ImageType::BootInSuccessState(img) => {
                let boot_part = img.part_desc.get().unwrap();
                let base_img_addr = RefinedUsize::<0, 0, BOOT_FWBASE>::single_valued_int(
                    boot_part.fw_base as usize,
                )
                .0;
                hal_preboot();
                hal_boot_from(base_img_addr)
            }
            // If an update is successful, this is the state of the boot partition.
            ImageType::BootInTestingState(img) => {
                let boot_part = img.part_desc.get().unwrap();
                let base_img_addr = RefinedUsize::<0, 0, BOOT_FWBASE>::single_valued_int(
                    boot_part.fw_base as usize,
                )
                .0;
                hal_preboot();
                hal_boot_from(base_img_addr)
            }
            _ => panic!("reached an unreachable state"),
        }
    }

    fn update_trigger(self) -> Result<()> {
        let updt = PartDescriptor::open_partition(Update, self).unwrap();
        Self::flash_unlock();
        match updt {
            ImageType::UpdateInNewState(img) => {
                let new_img = img.into_updating_state();
                let part_desc = new_img.part_desc.get();
                match part_desc {
                    Some(part) => part.set_state(self, new_img.get_state()),
                    None => return Err(RustbootError::__Nonexhaustive),
                };
            }
            ImageType::UpdateInUpdatingState(img) => {} // do nothing as update has been triggered
            _ => return Err(RustbootError::Unreachable),
        }
        Self::flash_lock();
        Ok(())
    }

    fn update_success(self) -> Result<()> {
        let boot = PartDescriptor::open_partition(Boot, self).unwrap();
        Self::flash_unlock();
        match boot {
            ImageType::BootInTestingState(img) => {
                let new_img = img.into_success_state();
                let part_desc = new_img.part_desc.get();
                match part_desc {
                    Some(part) => part.set_state(self, new_img.get_state()),
                    None => return Err(RustbootError::__Nonexhaustive),
                };
            }
            ImageType::BootInSuccessState(img) => {} // do nothing as we've successfully updated & booted
            _ => return Err(RustbootError::Unreachable),
        }
        Self::flash_lock();
        Ok(())
    }
}
