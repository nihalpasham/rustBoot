//! adapted from embedded-sdmmc-rs - FAT16/FAT32 file system implementation
//!
//! Implements the File Allocation Table file system. Supports FAT16 and FAT32 volumes.

use super::blockdevice::{Block, BlockCount, BlockDevice, BlockIdx};
use super::controller::{Controller, Error, VolumeType};
use super::filesystem::{
    Attributes, Cluster, DirEntry, Directory, ShortFileName, TimeSource, Timestamp,
};
use super::structure::define_field;

use byteorder::{ByteOrder, LittleEndian};
use core::convert::{TryFrom, TryInto};
use log::{info, warn};

/// Number of entries reserved at the start of a File Allocation Table
pub const RESERVED_ENTRIES: u32 = 2;

const MAX_FAT_SECTORS: u32 = 5000;
pub(crate) const MAX_FAT_ENTRIES: u32 = (MAX_FAT_SECTORS * Block::LEN as u32) / 4;
pub struct FatCache(pub [[u8; 4]; MAX_FAT_ENTRIES as usize]);
impl FatCache {
    /// Creates an initialized FAT_CACHE.
    pub const fn new() -> Self {
        Self([[0u8; 4]; MAX_FAT_ENTRIES as usize])
    }
}
pub static mut FAT_CACHE: FatCache = FatCache::new();

/// Indentifies the supported types of FAT format
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FatType {
    /// FAT16 Format
    Fat16,
    /// FAT32 Format
    Fat32,
}

/// Indentifies the supported types of FAT format
#[derive(Debug, Eq, PartialEq)]
pub enum FatSpecificInfo {
    /// Fat16 Format
    Fat16(Fat16Info),
    /// Fat32 Format
    Fat32(Fat32Info),
}

/// FAT32 specific data
#[derive(Debug, Eq, PartialEq)]
pub struct Fat32Info {
    /// The root directory does not have a reserved area in FAT32. This is the
    /// cluster it starts in (nominally 2).
    pub(crate) first_root_dir_cluster: Cluster,
    /// Block idx of the info sector
    pub(crate) info_location: BlockIdx,
}

/// FAT16 specific data
#[derive(Debug, Eq, PartialEq)]
pub struct Fat16Info {
    /// The block the root directory starts in. Relative to start of partition (so add `self.lba_offset` before passing to controller)
    pub(crate) first_root_dir_block: BlockCount,
    /// Number of entries in root directory (it's reserved and not in the FAT)
    pub(crate) root_entries_count: u16,
}

/// The name given to a particular FAT formatted volume.
#[derive(PartialEq, Eq)]
pub struct VolumeName {
    data: [u8; 11],
}

impl VolumeName {
    /// Create a new VolumeName
    pub fn new(data: [u8; 11]) -> VolumeName {
        VolumeName { data }
    }
}

/// Identifies a FAT16 Volume on the disk.
#[derive(PartialEq, Eq, Debug)]
pub struct FatVolume {
    /// The block number of the start of the partition. All other BlockIdx values are relative to this.
    pub(crate) lba_start: BlockIdx,
    /// The number of blocks in this volume
    pub(crate) num_blocks: BlockCount,
    /// The name of this volume
    pub(crate) name: VolumeName,
    /// Number of 512 byte blocks (or Blocks) in a cluster
    pub(crate) blocks_per_cluster: u8,
    /// The block the data starts in. Relative to start of partition (so add `self.lba_offset` before passing to controller)
    pub(crate) first_data_block: BlockCount,
    /// The block the FAT starts in. Relative to start of partition (so add `self.lba_offset` before passing to controller)
    pub(crate) fat_start: BlockCount,
    /// Expected number of free clusters
    pub(crate) free_clusters_count: Option<u32>,
    /// Number of the next expected free cluster
    pub(crate) next_free_cluster: Option<Cluster>,
    /// Total number of clusters
    pub(crate) cluster_count: u32,
    /// Type of FAT
    pub(crate) fat_specific_info: FatSpecificInfo,
}

impl core::fmt::Debug for VolumeName {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match core::str::from_utf8(&self.data) {
            Ok(s) => write!(fmt, "{:?}", s),
            Err(_e) => write!(fmt, "{:?}", &self.data),
        }
    }
}

/// Represents a Boot Parameter Block. This is the first sector of a FAT
/// formatted partition, and it describes various properties of the FAT
/// filesystem.
pub struct Bpb<'a> {
    data: &'a [u8; 512],
    fat_type: FatType,
    cluster_count: u32,
}

impl<'a> Bpb<'a> {
    const FOOTER_VALUE: u16 = 0xAA55;

    /// Attempt to parse a Boot Parameter Block from a 512 byte sector.
    pub fn create_from_bytes(data: &[u8; 512]) -> Result<Bpb, &'static str> {
        let mut bpb = Bpb {
            data,
            fat_type: FatType::Fat16,
            cluster_count: 0,
        };
        if bpb.footer() != Self::FOOTER_VALUE {
            return Err("Bad BPB footer");
        }

        let root_dir_blocks = ((u32::from(bpb.root_entries_count()) * OnDiskDirEntry::LEN_U32)
            + (Block::LEN_U32 - 1))
            / Block::LEN_U32;
        let data_blocks = bpb.total_blocks()
            - (u32::from(bpb.reserved_block_count())
                + (u32::from(bpb.num_fats()) * bpb.fat_size())
                + root_dir_blocks);
        bpb.cluster_count = data_blocks / u32::from(bpb.blocks_per_cluster());
        if bpb.cluster_count < 4085 {
            return Err("FAT12 is unsupported");
        } else if bpb.cluster_count < 65525 {
            bpb.fat_type = FatType::Fat16;
        } else {
            bpb.fat_type = FatType::Fat32;
        }

        match bpb.fat_type {
            FatType::Fat16 => Ok(bpb),
            FatType::Fat32 if bpb.fs_ver() == 0 => {
                // Only support FAT32 version 0.0
                Ok(bpb)
            }
            _ => Err("Invalid FAT format"),
        }
    }

    // FAT16/FAT32
    define_field!(bytes_per_block, u16, 11);
    define_field!(blocks_per_cluster, u8, 13);
    define_field!(reserved_block_count, u16, 14);
    define_field!(num_fats, u8, 16);
    define_field!(root_entries_count, u16, 17);
    define_field!(total_blocks16, u16, 19);
    define_field!(media, u8, 21);
    define_field!(fat_size16, u16, 22);
    define_field!(blocks_per_track, u16, 24);
    define_field!(num_heads, u16, 26);
    define_field!(hidden_blocks, u32, 28);
    define_field!(total_blocks32, u32, 32);
    define_field!(footer, u16, 510);

    // FAT32 only
    define_field!(fat_size32, u32, 36);
    define_field!(fs_ver, u16, 42);
    define_field!(first_root_dir_cluster, u32, 44);
    define_field!(fs_info, u16, 48);
    define_field!(backup_boot_block, u16, 50);

    /// Get the OEM name string for this volume
    pub fn oem_name(&self) -> &[u8] {
        &self.data[3..11]
    }

    // FAT16/FAT32 functions

    /// Get the Volume Label string for this volume
    pub fn volume_label(&self) -> &[u8] {
        if self.fat_type != FatType::Fat32 {
            &self.data[43..=53]
        } else {
            &self.data[71..=81]
        }
    }

    // FAT32 only functions

    /// On a FAT32 volume, return the free block count from the Info Block. On
    /// a FAT16 volume, returns None.
    pub fn fs_info_block(&self) -> Option<BlockCount> {
        if self.fat_type != FatType::Fat32 {
            None
        } else {
            Some(BlockCount(u32::from(self.fs_info())))
        }
    }

    // Magic functions that get the right FAT16/FAT32 result

    /// Get the size of the File Allocation Table in blocks.
    pub fn fat_size(&self) -> u32 {
        let result = u32::from(self.fat_size16());
        if result != 0 {
            result
        } else {
            self.fat_size32()
        }
    }

    /// Get the total number of blocks in this filesystem.
    pub fn total_blocks(&self) -> u32 {
        let result = u32::from(self.total_blocks16());
        if result != 0 {
            result
        } else {
            self.total_blocks32()
        }
    }

    /// Get the total number of clusters in this filesystem.
    pub fn total_clusters(&self) -> u32 {
        self.cluster_count
    }
}

/// File System Information structure is only present on FAT32 partitions. It
/// may contain a valid number of free clusters and the number of the next
/// free cluster. The information contained in the structure must be
/// considered as advisory only. File system driver implementations are not
/// required to ensure that information within the structure is kept
/// consistent.
pub struct InfoSector<'a> {
    data: &'a [u8; 512],
}

impl<'a> InfoSector<'a> {
    const LEAD_SIG: u32 = 0x4161_5252;
    const STRUC_SIG: u32 = 0x6141_7272;
    const TRAIL_SIG: u32 = 0xAA55_0000;

    /// Try and create a new Info Sector from a block.
    pub fn create_from_bytes(data: &[u8; 512]) -> Result<InfoSector, &'static str> {
        let info = InfoSector { data };
        if info.lead_sig() != Self::LEAD_SIG {
            return Err("Bad lead signature on InfoSector");
        }
        if info.struc_sig() != Self::STRUC_SIG {
            return Err("Bad struc signature on InfoSector");
        }
        if info.trail_sig() != Self::TRAIL_SIG {
            return Err("Bad trail signature on InfoSector");
        }
        Ok(info)
    }

    define_field!(lead_sig, u32, 0);
    define_field!(struc_sig, u32, 484);
    define_field!(free_count, u32, 488);
    define_field!(next_free, u32, 492);
    define_field!(trail_sig, u32, 508);

    /// Return how many free clusters are left in this volume, if known.
    pub fn free_clusters_count(&self) -> Option<u32> {
        match self.free_count() {
            0xFFFF_FFFF => None,
            n => Some(n),
        }
    }

    /// Return the number of the next free cluster, if known.
    pub fn next_free_cluster(&self) -> Option<Cluster> {
        match self.next_free() {
            // 0 and 1 are reserved clusters
            0xFFFF_FFFF | 0 | 1 => None,
            n => Some(Cluster(n)),
        }
    }
}

/// Represents a 32-byte directory entry as stored on-disk in a directory file.
pub struct OnDiskDirEntry<'a> {
    data: &'a [u8],
}

impl<'a> core::fmt::Debug for OnDiskDirEntry<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OnDiskDirEntry<")?;
        write!(f, "raw_attr = {}", self.raw_attr())?;
        write!(f, ", create_time = {}", self.create_time())?;
        write!(f, ", create_date = {}", self.create_date())?;
        write!(f, ", last_access_data = {}", self.last_access_data())?;
        write!(f, ", first_cluster_hi = {}", self.first_cluster_hi())?;
        write!(f, ", write_time = {}", self.write_time())?;
        write!(f, ", write_date = {}", self.write_date())?;
        write!(f, ", first_cluster_lo = {}", self.first_cluster_lo())?;
        write!(f, ", file_size = {}", self.file_size())?;
        write!(f, ", is_end = {}", self.is_end())?;
        write!(f, ", is_valid = {}", self.is_valid())?;
        write!(f, ", is_lfn = {}", self.is_lfn())?;
        write!(
            f,
            ", first_cluster_fat32 = {:?}",
            self.first_cluster_fat32()
        )?;
        write!(
            f,
            ", first_cluster_fat16 = {:?}",
            self.first_cluster_fat16()
        )?;
        write!(f, ">")?;
        Ok(())
    }
}

/// Represents the 32 byte directory entry. This is the same for FAT16 and
/// FAT32 (except FAT16 doesn't use first_cluster_hi).
impl<'a> OnDiskDirEntry<'a> {
    pub(crate) const LEN: usize = 32;
    pub(crate) const LEN_U32: u32 = 32;

    define_field!(raw_attr, u8, 11);
    define_field!(create_time, u16, 14);
    define_field!(create_date, u16, 16);
    define_field!(last_access_data, u16, 18);
    define_field!(first_cluster_hi, u16, 20);
    define_field!(write_time, u16, 22);
    define_field!(write_date, u16, 24);
    define_field!(first_cluster_lo, u16, 26);
    define_field!(file_size, u32, 28);

    /// Create a new on-disk directory entry from a block of 32 bytes read
    /// from a directory file.
    pub fn new(data: &[u8]) -> OnDiskDirEntry {
        OnDiskDirEntry { data }
    }

    /// Is this the last entry in the directory?
    pub fn is_end(&self) -> bool {
        self.data[0] == 0x00
    }

    /// Is this a valid entry?
    pub fn is_valid(&self) -> bool {
        !self.is_end() && (self.data[0] != 0xE5)
    }

    /// Is this a Long Filename entry?
    pub fn is_lfn(&self) -> bool {
        let attributes = Attributes::create_from_fat(self.raw_attr());
        attributes.is_lfn()
    }

    /// If this is an LFN, get the contents so we can re-assemble the filename.
    pub fn lfn_contents(&self) -> Option<(bool, u8, [char; 13])> {
        if self.is_lfn() {
            let mut buffer = [' '; 13];
            let is_start = (self.data[0] & 0x40) != 0;
            let sequence = self.data[0] & 0x1F;
            // LFNs store UCS-2, so we can map from 16-bit char to 32-bit char without problem.
            buffer[0] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[1..=2]))).unwrap();
            buffer[1] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[3..=4]))).unwrap();
            buffer[2] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[5..=6]))).unwrap();
            buffer[3] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[7..=8]))).unwrap();
            buffer[4] = core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[9..=10])))
                .unwrap();
            buffer[5] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[14..=15])))
                    .unwrap();
            buffer[6] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[16..=17])))
                    .unwrap();
            buffer[7] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[18..=19])))
                    .unwrap();
            buffer[8] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[20..=21])))
                    .unwrap();
            buffer[9] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[22..=23])))
                    .unwrap();
            buffer[10] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[24..=25])))
                    .unwrap();
            buffer[11] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[28..=29])))
                    .unwrap();
            buffer[12] =
                core::char::from_u32(u32::from(LittleEndian::read_u16(&self.data[30..=31])))
                    .unwrap();
            Some((is_start, sequence, buffer))
        } else {
            None
        }
    }

    /// Does this on-disk entry match the given filename?
    pub fn matches(&self, sfn: &ShortFileName) -> bool {
        self.data[0..11] == sfn.contents
    }

    /// Which cluster, if any, does this file start at? Assumes this is from a FAT32 volume.
    pub fn first_cluster_fat32(&self) -> Cluster {
        let cluster_no =
            (u32::from(self.first_cluster_hi()) << 16) | u32::from(self.first_cluster_lo());
        Cluster(cluster_no)
    }

    /// Which cluster, if any, does this file start at? Assumes this is from a FAT16 volume.
    fn first_cluster_fat16(&self) -> Cluster {
        let cluster_no = u32::from(self.first_cluster_lo());
        Cluster(cluster_no)
    }

    /// Convert the on-disk format into a DirEntry
    pub fn get_entry(
        &self,
        fat_type: FatType,
        entry_block: BlockIdx,
        entry_offset: u32,
    ) -> DirEntry {
        let mut result = DirEntry {
            name: ShortFileName {
                contents: [0u8; 11],
            },
            mtime: Timestamp::from_fat(self.write_date(), self.write_time()),
            ctime: Timestamp::from_fat(self.create_date(), self.create_time()),
            attributes: Attributes::create_from_fat(self.raw_attr()),
            cluster: if fat_type == FatType::Fat32 {
                self.first_cluster_fat32()
            } else {
                self.first_cluster_fat16()
            },
            size: self.file_size(),
            entry_block,
            entry_offset,
        };
        result.name.contents.copy_from_slice(&self.data[0..11]);
        result
    }
}

impl FatVolume {
    /// Write a new entry in the FAT
    pub fn update_info_sector<D, T>(
        &mut self,
        controller: &mut Controller<D, T>,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_) => {}
            FatSpecificInfo::Fat32(fat32_info) => {
                if self.free_clusters_count.is_none() && self.next_free_cluster.is_none() {
                    return Ok(());
                }
                let mut blocks = [Block::new()];
                controller
                    .block_device
                    .read(&mut blocks, fat32_info.info_location, "read_info_sector")
                    .map_err(Error::DeviceError)?;
                let block = &mut blocks[0];
                if let Some(count) = self.free_clusters_count {
                    block[488..492].copy_from_slice(&count.to_le_bytes());
                }
                if let Some(next_free_cluster) = self.next_free_cluster {
                    block[492..496].copy_from_slice(&next_free_cluster.0.to_le_bytes());
                }
                controller
                    .block_device
                    .write(&blocks, fat32_info.info_location)
                    .map_err(Error::DeviceError)?;
            }
        }
        Ok(())
    }

    /// Get the type of FAT this volume is
    pub(crate) fn get_fat_type(&self) -> FatType {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_) => FatType::Fat16,
            FatSpecificInfo::Fat32(_) => FatType::Fat32,
        }
    }

    /// Write a new entry in the FAT
    fn update_fat<D, T>(
        &mut self,
        controller: &mut Controller<D, T>,
        cluster: Cluster,
        new_value: Cluster,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let mut blocks = [Block::new()];
        let this_fat_block_num;
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_fat16_info) => {
                let fat_offset = cluster.0 * 2;
                this_fat_block_num = self.lba_start + self.fat_start.offset_bytes(fat_offset);
                let this_fat_ent_offset = (fat_offset % Block::LEN_U32) as usize;
                controller
                    .block_device
                    .read(&mut blocks, this_fat_block_num, "read_fat")
                    .map_err(Error::DeviceError)?;
                let entry = match new_value {
                    Cluster::INVALID => 0xFFF6,
                    Cluster::BAD => 0xFFF7,
                    Cluster::EMPTY => 0x0000,
                    Cluster::END_OF_FILE => 0xFFFF,
                    _ => new_value.0 as u16,
                };
                LittleEndian::write_u16(
                    &mut blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 1],
                    entry,
                );
            }
            FatSpecificInfo::Fat32(_fat32_info) => {
                // FAT32 => 4 bytes per entry
                let fat_offset = cluster.0 as u32 * 4;
                this_fat_block_num = self.lba_start + self.fat_start.offset_bytes(fat_offset);
                let this_fat_ent_offset = (fat_offset % Block::LEN_U32) as usize;
                controller
                    .block_device
                    .read(&mut blocks, this_fat_block_num, "read_fat")
                    .map_err(Error::DeviceError)?;
                let entry = match new_value {
                    Cluster::INVALID => 0x0FFF_FFF6,
                    Cluster::BAD => 0x0FFF_FFF7,
                    Cluster::EMPTY => 0x0000_0000,
                    _ => new_value.0,
                };
                let existing = LittleEndian::read_u32(
                    &blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 3],
                );
                let new = (existing & 0xF000_0000) | (entry & 0x0FFF_FFFF);
                LittleEndian::write_u32(
                    &mut blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 3],
                    new,
                );
            }
        }
        controller
            .block_device
            .write(&blocks, this_fat_block_num)
            .map_err(Error::DeviceError)?;
        Ok(())
    }

    /// Walking the FAT table for large files can be really slow.
    ///
    /// This method allows us to cache the `file allocation table` contents.
    ///
    /// TODO:
    /// - need to ensure that the cache is only ever populated once i.e. the cache is static
    /// - `rustBoot` has no need to update the `fat` as it does NOT support file-system writes to a `block-device`.
    /// This is a security design-goal.
    ///
    /// Note:
    /// - the maximum `cache-size` is fixed at 5000 sectors/blocks
    pub(crate) fn populate_static_fat_cache<D, T>(
        &self,
        controller: &Controller<D, T>,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        // retrieve the boot parameter block
        let mut blocks = [Block::new()];
        controller
            .block_device
            .read(&mut blocks, self.lba_start, "read_bpb")
            .map_err(Error::DeviceError)?;
        let block = &blocks[0];
        let bpb = Bpb::create_from_bytes(&block).map_err(Error::FormatError)?;

        // retrieve the block idx where the `fat` starts
        let fat_start_blockidx = self.lba_start + self.fat_start;
        info!("fat_start_blockidx: {:?}", fat_start_blockidx);
        let fat_size = bpb.fat_size();
        info!("fat_size: {:?}", fat_size);
        assert!(fat_size <= MAX_FAT_SECTORS);

        // populate fat cache
        let fat_buffer = Block::from_fat_entries(unsafe { &mut FAT_CACHE.0 });
        controller
            .block_device
            .read(fat_buffer, fat_start_blockidx, "fat_read")
            .map_err(Error::DeviceError)?;
        let fat_entries: [[u8; 4]; MAX_FAT_ENTRIES as usize] = Block::to_fat_entries(fat_buffer)
            .try_into()
            .map_err(|_| Error::ConversionError)?;
        info!("4 entries of fat_cache: {:?}", &fat_entries[..4]);
        info!("number of fat_cache entries: {:?}", (fat_size * 512) / 4);

        Ok(())
    }

    /// Look in the FAT_CACHE to see which cluster comes next.
    pub(crate) fn next_cluster_in_fat_cache(
        &self,
        cluster: Cluster,
    ) -> Result<Cluster, Error<&'static str>> {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_fat16_info) => {
                unimplemented!()
            }
            FatSpecificInfo::Fat32(_fat32_info) => {
                let fat_entry_idx = cluster.0 as usize;
                let fat_entry =
                    LittleEndian::read_u32(unsafe { &FAT_CACHE.0[fat_entry_idx] }) & 0x0FFF_FFFF;
                match fat_entry {
                    0x0000_0000 => {
                        // Jumped to free space
                        Err(Error::JumpedFree)
                    }
                    0x0FFF_FFF7 => {
                        // Bad cluster
                        Err(Error::BadCluster)
                    }
                    0x0000_0001 | 0x0FFF_FFF8..=0x0FFF_FFFF => {
                        // There is no next cluster
                        Err(Error::EndOfFile)
                    }
                    f => {
                        // Seems legit
                        Ok(Cluster(f))
                    }
                }
            }
        }
    }

    /// Look in the FAT to see which cluster comes next.
    pub(crate) fn next_cluster<D, T>(
        &self,
        controller: &Controller<D, T>,
        cluster: Cluster,
    ) -> Result<Cluster, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let mut blocks = [Block::new()];
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_fat16_info) => {
                let fat_offset = cluster.0 * 2;
                let this_fat_block_num = self.lba_start + self.fat_start.offset_bytes(fat_offset);
                let this_fat_ent_offset = (fat_offset % Block::LEN_U32) as usize;
                controller
                    .block_device
                    .read(&mut blocks, this_fat_block_num, "next_cluster")
                    .map_err(Error::DeviceError)?;
                let fat_entry = LittleEndian::read_u16(
                    &blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 1],
                );
                match fat_entry {
                    0xFFF7 => {
                        // Bad cluster
                        Err(Error::BadCluster)
                    }
                    0xFFF8..=0xFFFF => {
                        // There is no next cluster
                        Err(Error::EndOfFile)
                    }
                    f => {
                        // Seems legit
                        Ok(Cluster(u32::from(f)))
                    }
                }
            }
            FatSpecificInfo::Fat32(_fat32_info) => {
                let fat_offset = cluster.0 * 4;
                let this_fat_block_num = self.lba_start + self.fat_start.offset_bytes(fat_offset);
                let this_fat_ent_offset = (fat_offset % Block::LEN_U32) as usize;
                controller
                    .block_device
                    .read(&mut blocks, this_fat_block_num, "next_cluster")
                    .map_err(Error::DeviceError)?;
                let fat_entry = LittleEndian::read_u32(
                    &blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 3],
                ) & 0x0FFF_FFFF;
                match fat_entry {
                    0x0000_0000 => {
                        // Jumped to free space
                        Err(Error::JumpedFree)
                    }
                    0x0FFF_FFF7 => {
                        // Bad cluster
                        Err(Error::BadCluster)
                    }
                    0x0000_0001 | 0x0FFF_FFF8..=0x0FFF_FFFF => {
                        // There is no next cluster
                        Err(Error::EndOfFile)
                    }
                    f => {
                        // Seems legit
                        Ok(Cluster(f))
                    }
                }
            }
        }
    }

    /// Number of bytes in a cluster.
    pub(crate) fn bytes_per_cluster(&self) -> u32 {
        u32::from(self.blocks_per_cluster) * Block::LEN_U32
    }

    /// Converts a cluster number (or `Cluster`) to a block number (or
    /// `BlockIdx`). Gives an absolute `BlockIdx` you can pass to the
    /// controller.
    pub(crate) fn cluster_to_block(&self, cluster: Cluster) -> BlockIdx {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(fat16_info) => {
                let block_num = match cluster {
                    Cluster::ROOT_DIR => fat16_info.first_root_dir_block,
                    Cluster(c) => {
                        // FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                        let first_block_of_cluster =
                            BlockCount((c - 2) * u32::from(self.blocks_per_cluster));
                        self.first_data_block + first_block_of_cluster
                    }
                };
                self.lba_start + block_num
            }
            FatSpecificInfo::Fat32(fat32_info) => {
                let cluster_num = match cluster {
                    Cluster::ROOT_DIR => fat32_info.first_root_dir_cluster.0,
                    c => c.0,
                };
                // FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                let first_block_of_cluster =
                    BlockCount((cluster_num - 2) * u32::from(self.blocks_per_cluster));
                self.lba_start + self.first_data_block + first_block_of_cluster
            }
        }
    }

    /// Finds a empty entry space and writes the new entry to it, allocates a new cluster if it's
    /// needed
    pub(crate) fn write_new_directory_entry<D, T>(
        &mut self,
        controller: &mut Controller<D, T>,
        dir: &Directory,
        name: ShortFileName,
        attributes: Attributes,
    ) -> Result<DirEntry, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(fat16_info) => {
                let mut first_dir_block_num = match dir.cluster {
                    Cluster::ROOT_DIR => self.lba_start + fat16_info.first_root_dir_block,
                    _ => self.cluster_to_block(dir.cluster),
                };
                let mut current_cluster = Some(dir.cluster);
                let mut blocks = [Block::new()];

                let dir_size = match dir.cluster {
                    Cluster::ROOT_DIR => BlockCount(
                        ((u32::from(fat16_info.root_entries_count) * 32) + (Block::LEN as u32 - 1))
                            / Block::LEN as u32,
                    ),
                    _ => BlockCount(u32::from(self.blocks_per_cluster)),
                };
                while let Some(cluster) = current_cluster {
                    for block in first_dir_block_num.range(dir_size) {
                        controller
                            .block_device
                            .read(&mut blocks, block, "read_dir")
                            .map_err(Error::DeviceError)?;
                        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
                            let start = entry * OnDiskDirEntry::LEN;
                            let end = (entry + 1) * OnDiskDirEntry::LEN;
                            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
                            // 0x00 or 0xE5 represents a free entry
                            if !dir_entry.is_valid() {
                                let ctime = controller.timesource.get_timestamp();
                                let entry = DirEntry::new(
                                    name,
                                    attributes,
                                    Cluster(0),
                                    ctime,
                                    block,
                                    start as u32,
                                );
                                blocks[0][start..start + 32]
                                    .copy_from_slice(&entry.serialize(FatType::Fat16)[..]);
                                controller
                                    .block_device
                                    .write(&blocks, block)
                                    .map_err(Error::DeviceError)?;
                                return Ok(entry);
                            }
                        }
                    }
                    if cluster != Cluster::ROOT_DIR {
                        current_cluster = match self.next_cluster(controller, cluster) {
                            Ok(n) => {
                                first_dir_block_num = self.cluster_to_block(n);
                                Some(n)
                            }
                            Err(Error::EndOfFile) => {
                                let c = self.alloc_cluster(controller, Some(cluster), true)?;
                                first_dir_block_num = self.cluster_to_block(c);
                                Some(c)
                            }
                            _ => None,
                        };
                    } else {
                        current_cluster = None;
                    }
                }
                Err(Error::NotEnoughSpace)
            }
            FatSpecificInfo::Fat32(fat32_info) => {
                let mut first_dir_block_num = match dir.cluster {
                    Cluster::ROOT_DIR => self.cluster_to_block(fat32_info.first_root_dir_cluster),
                    _ => self.cluster_to_block(dir.cluster),
                };
                let mut current_cluster = Some(dir.cluster);
                let mut blocks = [Block::new()];

                let dir_size = BlockCount(u32::from(self.blocks_per_cluster));
                while let Some(cluster) = current_cluster {
                    for block in first_dir_block_num.range(dir_size) {
                        controller
                            .block_device
                            .read(&mut blocks, block, "read_dir")
                            .map_err(Error::DeviceError)?;
                        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
                            let start = entry * OnDiskDirEntry::LEN;
                            let end = (entry + 1) * OnDiskDirEntry::LEN;
                            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
                            // 0x00 or 0xE5 represents a free entry
                            if !dir_entry.is_valid() {
                                let ctime = controller.timesource.get_timestamp();
                                let entry = DirEntry::new(
                                    name,
                                    attributes,
                                    Cluster(0),
                                    ctime,
                                    block,
                                    start as u32,
                                );
                                blocks[0][start..start + 32]
                                    .copy_from_slice(&entry.serialize(FatType::Fat32)[..]);
                                controller
                                    .block_device
                                    .write(&blocks, block)
                                    .map_err(Error::DeviceError)?;
                                return Ok(entry);
                            }
                        }
                    }
                    current_cluster = match self.next_cluster(controller, cluster) {
                        Ok(n) => {
                            first_dir_block_num = self.cluster_to_block(n);
                            Some(n)
                        }
                        Err(Error::EndOfFile) => {
                            let c = self.alloc_cluster(controller, Some(cluster), true)?;
                            first_dir_block_num = self.cluster_to_block(c);
                            Some(c)
                        }
                        _ => None,
                    };
                }
                Err(Error::NotEnoughSpace)
            }
        }
    }

    /// Calls callback `func` with every valid entry in the given directory.
    /// Useful for performing directory listings.
    pub(crate) fn iterate_dir<D, T, F>(
        &self,
        controller: &Controller<D, T>,
        dir: &Directory,
        mut func: F,
    ) -> Result<(), Error<D::Error>>
    where
        F: FnMut(&DirEntry),
        D: BlockDevice,
        T: TimeSource,
    {
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(fat16_info) => {
                let mut first_dir_block_num = match dir.cluster {
                    Cluster::ROOT_DIR => self.lba_start + fat16_info.first_root_dir_block,
                    _ => self.cluster_to_block(dir.cluster),
                };
                let mut current_cluster = Some(dir.cluster);
                let dir_size = match dir.cluster {
                    Cluster::ROOT_DIR => BlockCount(
                        ((u32::from(fat16_info.root_entries_count) * 32) + (Block::LEN as u32 - 1))
                            / Block::LEN as u32,
                    ),
                    _ => BlockCount(u32::from(self.blocks_per_cluster)),
                };
                let mut blocks = [Block::new()];
                while let Some(cluster) = current_cluster {
                    for block in first_dir_block_num.range(dir_size) {
                        controller
                            .block_device
                            .read(&mut blocks, block, "read_dir")
                            .map_err(Error::DeviceError)?;
                        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
                            let start = entry * OnDiskDirEntry::LEN;
                            let end = (entry + 1) * OnDiskDirEntry::LEN;
                            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
                            if dir_entry.is_end() {
                                // Can quit early
                                return Ok(());
                            } else if dir_entry.is_valid() && !dir_entry.is_lfn() {
                                // Safe, since Block::LEN always fits on a u32
                                let start = u32::try_from(start).unwrap();
                                let entry = dir_entry.get_entry(FatType::Fat16, block, start);
                                func(&entry);
                            }
                        }
                    }
                    if cluster != Cluster::ROOT_DIR {
                        current_cluster = match self.next_cluster(controller, cluster) {
                            Ok(n) => {
                                first_dir_block_num = self.cluster_to_block(n);
                                Some(n)
                            }
                            _ => None,
                        };
                    } else {
                        current_cluster = None;
                    }
                }
                Ok(())
            }
            FatSpecificInfo::Fat32(fat32_info) => {
                let mut current_cluster = match dir.cluster {
                    Cluster::ROOT_DIR => Some(fat32_info.first_root_dir_cluster),
                    _ => Some(dir.cluster),
                };
                let mut blocks = [Block::new()];
                while let Some(cluster) = current_cluster {
                    let block_idx = self.cluster_to_block(cluster);
                    for block in block_idx.range(BlockCount(u32::from(self.blocks_per_cluster))) {
                        controller
                            .block_device
                            .read(&mut blocks, block, "read_dir")
                            .map_err(Error::DeviceError)?;
                        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
                            let start = entry * OnDiskDirEntry::LEN;
                            let end = (entry + 1) * OnDiskDirEntry::LEN;
                            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
                            if dir_entry.is_end() {
                                // Can quit early
                                return Ok(());
                            } else if dir_entry.is_valid() && !dir_entry.is_lfn() {
                                // Safe, since Block::LEN always fits on a u32
                                let start = u32::try_from(start).unwrap();
                                let entry = dir_entry.get_entry(FatType::Fat32, block, start);
                                func(&entry);
                            }
                        }
                    }
                    current_cluster = match self.next_cluster(controller, cluster) {
                        Ok(n) => Some(n),
                        _ => None,
                    };
                }
                Ok(())
            }
        }
    }

    /// Get an entry from the given directory
    pub(crate) fn find_directory_entry<D, T>(
        &self,
        controller: &mut Controller<D, T>,
        dir: &Directory,
        name: &str,
    ) -> Result<DirEntry, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let match_name = ShortFileName::create_from_str(name).map_err(Error::FilenameError)?;
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(fat16_info) => {
                let mut current_cluster = Some(dir.cluster);
                let mut first_dir_block_num = match dir.cluster {
                    Cluster::ROOT_DIR => self.lba_start + fat16_info.first_root_dir_block,
                    _ => self.cluster_to_block(dir.cluster),
                };
                let dir_size = match dir.cluster {
                    Cluster::ROOT_DIR => BlockCount(
                        ((u32::from(fat16_info.root_entries_count) * 32) + (Block::LEN as u32 - 1))
                            / Block::LEN as u32,
                    ),
                    _ => BlockCount(u32::from(self.blocks_per_cluster)),
                };

                while let Some(cluster) = current_cluster {
                    for block in first_dir_block_num.range(dir_size) {
                        match self.find_entry_in_block(
                            controller,
                            FatType::Fat16,
                            &match_name,
                            block,
                        ) {
                            Err(Error::NotInBlock) => continue,
                            x => return x,
                        }
                    }
                    if cluster != Cluster::ROOT_DIR {
                        current_cluster = match self.next_cluster(controller, cluster) {
                            Ok(n) => {
                                first_dir_block_num = self.cluster_to_block(n);
                                Some(n)
                            }
                            _ => None,
                        };
                    } else {
                        current_cluster = None;
                    }
                }
                Err(Error::FileNotFound)
            }
            FatSpecificInfo::Fat32(fat32_info) => {
                let mut current_cluster = match dir.cluster {
                    Cluster::ROOT_DIR => Some(fat32_info.first_root_dir_cluster),
                    _ => Some(dir.cluster),
                };
                while let Some(cluster) = current_cluster {
                    let block_idx = self.cluster_to_block(cluster);
                    for block in block_idx.range(BlockCount(u32::from(self.blocks_per_cluster))) {
                        match self.find_entry_in_block(
                            controller,
                            FatType::Fat32,
                            &match_name,
                            block,
                        ) {
                            Err(Error::NotInBlock) => continue,
                            x => return x,
                        }
                    }
                    current_cluster = match self.next_cluster(controller, cluster) {
                        Ok(n) => Some(n),
                        _ => None,
                    }
                }
                Err(Error::FileNotFound)
            }
        }
    }

    /// Finds an entry in a given block
    fn find_entry_in_block<D, T>(
        &self,
        controller: &mut Controller<D, T>,
        fat_type: FatType,
        match_name: &ShortFileName,
        block: BlockIdx,
    ) -> Result<DirEntry, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let mut blocks = [Block::new()];
        controller
            .block_device
            .read(&mut blocks, block, "read_dir")
            .map_err(Error::DeviceError)?;
        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
            let start = entry * OnDiskDirEntry::LEN;
            let end = (entry + 1) * OnDiskDirEntry::LEN;
            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
            if dir_entry.is_end() {
                // Can quit early
                return Err(Error::FileNotFound);
            } else if dir_entry.matches(&match_name) {
                // Found it
                // Safe, since Block::LEN always fits on a u32
                let start = u32::try_from(start).unwrap();
                return Ok(dir_entry.get_entry(fat_type, block, start));
            }
        }
        Err(Error::NotInBlock)
    }

    /// Delete an entry from the given directory
    pub(crate) fn delete_directory_entry<D, T>(
        &self,
        controller: &mut Controller<D, T>,
        dir: &Directory,
        name: &str,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let match_name = ShortFileName::create_from_str(name).map_err(Error::FilenameError)?;
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(fat16_info) => {
                let mut current_cluster = Some(dir.cluster);
                let mut first_dir_block_num = match dir.cluster {
                    Cluster::ROOT_DIR => self.lba_start + fat16_info.first_root_dir_block,
                    _ => self.cluster_to_block(dir.cluster),
                };
                let dir_size = match dir.cluster {
                    Cluster::ROOT_DIR => BlockCount(
                        ((u32::from(fat16_info.root_entries_count) * 32) + (Block::LEN as u32 - 1))
                            / Block::LEN as u32,
                    ),
                    _ => BlockCount(u32::from(self.blocks_per_cluster)),
                };

                while let Some(cluster) = current_cluster {
                    for block in first_dir_block_num.range(dir_size) {
                        match self.delete_entry_in_block(controller, &match_name, block) {
                            Err(Error::NotInBlock) => continue,
                            x => return x,
                        }
                    }
                    if cluster != Cluster::ROOT_DIR {
                        current_cluster = match self.next_cluster(controller, cluster) {
                            Ok(n) => {
                                first_dir_block_num = self.cluster_to_block(n);
                                Some(n)
                            }
                            _ => None,
                        };
                    } else {
                        current_cluster = None;
                    }
                }
                Err(Error::FileNotFound)
            }
            FatSpecificInfo::Fat32(fat32_info) => {
                let mut current_cluster = match dir.cluster {
                    Cluster::ROOT_DIR => Some(fat32_info.first_root_dir_cluster),
                    _ => Some(dir.cluster),
                };
                while let Some(cluster) = current_cluster {
                    let block_idx = self.cluster_to_block(cluster);
                    for block in block_idx.range(BlockCount(u32::from(self.blocks_per_cluster))) {
                        match self.delete_entry_in_block(controller, &match_name, block) {
                            Err(Error::NotInBlock) => continue,
                            x => return x,
                        }
                    }
                    current_cluster = match self.next_cluster(controller, cluster) {
                        Ok(n) => Some(n),
                        _ => None,
                    }
                }
                Err(Error::FileNotFound)
            }
        }
    }

    /// Deletes an entry in a given block
    fn delete_entry_in_block<D, T>(
        &self,
        controller: &mut Controller<D, T>,
        match_name: &ShortFileName,
        block: BlockIdx,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let mut blocks = [Block::new()];
        controller
            .block_device
            .read(&mut blocks, block, "read_dir")
            .map_err(Error::DeviceError)?;
        for entry in 0..Block::LEN / OnDiskDirEntry::LEN {
            let start = entry * OnDiskDirEntry::LEN;
            let end = (entry + 1) * OnDiskDirEntry::LEN;
            let dir_entry = OnDiskDirEntry::new(&blocks[0][start..end]);
            if dir_entry.is_end() {
                // Can quit early
                return Err(Error::FileNotFound);
            } else if dir_entry.matches(&match_name) {
                let mut blocks = blocks;
                blocks[0].contents[start] = 0xE5;
                controller
                    .block_device
                    .write(&blocks, block)
                    .map_err(Error::DeviceError)?;
                return Ok(());
            }
        }
        Err(Error::NotInBlock)
    }

    /// Finds the next free cluster after the start_cluster and before end_cluster
    pub(crate) fn find_next_free_cluster<D, T>(
        &self,
        controller: &mut Controller<D, T>,
        start_cluster: Cluster,
        end_cluster: Cluster,
    ) -> Result<Cluster, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        let mut blocks = [Block::new()];
        let mut current_cluster = start_cluster;
        match &self.fat_specific_info {
            FatSpecificInfo::Fat16(_fat16_info) => {
                while current_cluster.0 < end_cluster.0 {
                    // info!(
                    //     "current_cluster={:?}, end_cluster={:?}",
                    //     current_cluster, end_cluster
                    // );
                    let fat_offset = current_cluster.0 * 2;
                    // info!("fat_offset = {:?}", fat_offset);
                    let this_fat_block_num =
                        self.lba_start + self.fat_start.offset_bytes(fat_offset);
                    // info!("this_fat_block_num = {:?}", this_fat_block_num);
                    let mut this_fat_ent_offset = usize::try_from(fat_offset % Block::LEN_U32)
                        .map_err(|_| Error::ConversionError)?;
                    // info!("Reading block {:?}", this_fat_block_num);
                    controller
                        .block_device
                        .read(&mut blocks, this_fat_block_num, "next_cluster")
                        .map_err(Error::DeviceError)?;

                    while this_fat_ent_offset <= Block::LEN - 2 {
                        let fat_entry = LittleEndian::read_u16(
                            &blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 1],
                        );
                        if fat_entry == 0 {
                            return Ok(current_cluster);
                        }
                        this_fat_ent_offset += 2;
                        current_cluster += 1;
                    }
                }
            }
            FatSpecificInfo::Fat32(_fat32_info) => {
                while current_cluster.0 < end_cluster.0 {
                    info!(
                        "current_cluster={:?}, end_cluster={:?}",
                        current_cluster, end_cluster
                    );
                    let fat_offset = current_cluster.0 * 4;
                    info!("fat_offset = {:?}", fat_offset);
                    let this_fat_block_num =
                        self.lba_start + self.fat_start.offset_bytes(fat_offset);
                    info!("this_fat_block_num = {:?}", this_fat_block_num);
                    let mut this_fat_ent_offset = usize::try_from(fat_offset % Block::LEN_U32)
                        .map_err(|_| Error::ConversionError)?;
                    info!("Reading block {:?}", this_fat_block_num);
                    controller
                        .block_device
                        .read(&mut blocks, this_fat_block_num, "next_cluster")
                        .map_err(Error::DeviceError)?;

                    while this_fat_ent_offset <= Block::LEN - 4 {
                        let fat_entry = LittleEndian::read_u32(
                            &blocks[0][this_fat_ent_offset..=this_fat_ent_offset + 3],
                        ) & 0x0FFF_FFFF;
                        if fat_entry == 0 {
                            return Ok(current_cluster);
                        }
                        this_fat_ent_offset += 4;
                        current_cluster += 1;
                    }
                }
            }
        }
        warn!("Out of space...");
        Err(Error::NotEnoughSpace)
    }

    /// Tries to allocate a cluster
    pub(crate) fn alloc_cluster<D, T>(
        &mut self,
        controller: &mut Controller<D, T>,
        prev_cluster: Option<Cluster>,
        zero: bool,
    ) -> Result<Cluster, Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        info!("Allocating new cluster, prev_cluster={:?}", prev_cluster);
        let end_cluster = Cluster(self.cluster_count + RESERVED_ENTRIES);
        let start_cluster = match self.next_free_cluster {
            Some(cluster) if cluster.0 < end_cluster.0 => cluster,
            _ => Cluster(RESERVED_ENTRIES),
        };
        info!(
            "Finding next free between {:?}..={:?}",
            start_cluster, end_cluster
        );
        let new_cluster = match self.find_next_free_cluster(controller, start_cluster, end_cluster)
        {
            Ok(cluster) => cluster,
            Err(_) if start_cluster.0 > RESERVED_ENTRIES => {
                info!(
                    "Retrying, finding next free between {:?}..={:?}",
                    Cluster(RESERVED_ENTRIES),
                    end_cluster
                );
                self.find_next_free_cluster(controller, Cluster(RESERVED_ENTRIES), end_cluster)?
            }
            Err(e) => return Err(e),
        };
        self.update_fat(controller, new_cluster, Cluster::END_OF_FILE)?;
        if let Some(cluster) = prev_cluster {
            info!(
                "Updating old cluster {:?} to {:?} in FAT",
                cluster, new_cluster
            );
            self.update_fat(controller, cluster, new_cluster)?;
        }
        info!(
            "Finding next free between {:?}..={:?}",
            new_cluster, end_cluster
        );
        self.next_free_cluster =
            match self.find_next_free_cluster(controller, new_cluster, end_cluster) {
                Ok(cluster) => Some(cluster),
                Err(_) if new_cluster.0 > RESERVED_ENTRIES => {
                    match self.find_next_free_cluster(
                        controller,
                        Cluster(RESERVED_ENTRIES),
                        end_cluster,
                    ) {
                        Ok(cluster) => Some(cluster),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            };
        info!("Next free cluster is {:?}", self.next_free_cluster);
        if let Some(ref mut number_free_cluster) = self.free_clusters_count {
            *number_free_cluster -= 1;
        };
        if zero {
            let blocks = [Block::new()];
            let first_block = self.cluster_to_block(new_cluster);
            let num_blocks = BlockCount(u32::from(self.blocks_per_cluster));
            for block in first_block.range(num_blocks) {
                controller
                    .block_device
                    .write(&blocks, block)
                    .map_err(Error::DeviceError)?;
            }
        }
        info!("All done, returning {:?}", new_cluster);
        Ok(new_cluster)
    }

    /// Marks the input cluster as an EOF and all the subsequent clusters in the chain as free
    pub(crate) fn truncate_cluster_chain<D, T>(
        &mut self,
        controller: &mut Controller<D, T>,
        cluster: Cluster,
    ) -> Result<(), Error<D::Error>>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        if cluster.0 < RESERVED_ENTRIES {
            // file doesn't have any valid cluster allocated, there is nothing to do
            return Ok(());
        }
        let mut next = match self.next_cluster(controller, cluster) {
            Ok(n) => n,
            Err(Error::EndOfFile) => return Ok(()),
            Err(e) => return Err(e),
        };
        if let Some(ref mut next_free_cluster) = self.next_free_cluster {
            if next_free_cluster.0 > next.0 {
                *next_free_cluster = next;
            }
        } else {
            self.next_free_cluster = Some(next);
        }
        self.update_fat(controller, cluster, Cluster::END_OF_FILE)?;
        loop {
            match self.next_cluster(controller, next) {
                Ok(n) => {
                    self.update_fat(controller, next, Cluster::EMPTY)?;
                    next = n;
                }
                Err(Error::EndOfFile) => {
                    self.update_fat(controller, next, Cluster::EMPTY)?;
                    break;
                }
                Err(e) => return Err(e),
            }
            if let Some(ref mut number_free_cluster) = self.free_clusters_count {
                *number_free_cluster += 1;
            };
        }
        Ok(())
    }
}

/// Load the boot parameter block from the start of the given partition and
/// determine if the partition contains a valid FAT16 or FAT32 file system.
pub fn parse_volume<D, T>(
    controller: &mut Controller<D, T>,
    lba_start: BlockIdx,
    num_blocks: BlockCount,
) -> Result<VolumeType, Error<D::Error>>
where
    D: BlockDevice,
    T: TimeSource,
    D::Error: core::fmt::Debug,
{
    let mut blocks = [Block::new()];
    controller
        .block_device
        .read(&mut blocks, lba_start, "read_bpb")
        .map_err(Error::DeviceError)?;
    let block = &blocks[0];
    let bpb = Bpb::create_from_bytes(&block).map_err(Error::FormatError)?;
    match bpb.fat_type {
        FatType::Fat16 => {
            if bpb.bytes_per_block() as usize != Block::LEN {
                return Err(Error::BadBlockSize(bpb.bytes_per_block()));
            }
            // FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
            let root_dir_blocks = ((u32::from(bpb.root_entries_count()) * OnDiskDirEntry::LEN_U32)
                + (Block::LEN_U32 - 1))
                / Block::LEN_U32;
            let fat_start = BlockCount(u32::from(bpb.reserved_block_count()));
            let first_root_dir_block =
                fat_start + BlockCount(u32::from(bpb.num_fats()) * bpb.fat_size());
            let first_data_block = first_root_dir_block + BlockCount(root_dir_blocks);
            let mut volume = FatVolume {
                lba_start,
                num_blocks,
                name: VolumeName { data: [0u8; 11] },
                blocks_per_cluster: bpb.blocks_per_cluster(),
                first_data_block: (first_data_block),
                fat_start: BlockCount(u32::from(bpb.reserved_block_count())),
                free_clusters_count: None,
                next_free_cluster: None,
                cluster_count: bpb.total_clusters(),
                fat_specific_info: FatSpecificInfo::Fat16(Fat16Info {
                    root_entries_count: bpb.root_entries_count(),
                    first_root_dir_block,
                }),
            };
            volume.name.data[..].copy_from_slice(bpb.volume_label());
            Ok(VolumeType::Fat(volume))
        }
        FatType::Fat32 => {
            // FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz);
            let first_data_block = u32::from(bpb.reserved_block_count())
                + (u32::from(bpb.num_fats()) * bpb.fat_size());

            // Safe to unwrap since this is a Fat32 Type
            let info_location = bpb.fs_info_block().unwrap();
            let mut info_blocks = [Block::new()];
            controller
                .block_device
                .read(
                    &mut info_blocks,
                    lba_start + info_location,
                    "read_info_sector",
                )
                .map_err(Error::DeviceError)?;
            let info_block = &info_blocks[0];
            let info_sector =
                InfoSector::create_from_bytes(&info_block).map_err(Error::FormatError)?;

            let mut volume = FatVolume {
                lba_start,
                num_blocks,
                name: VolumeName { data: [0u8; 11] },
                blocks_per_cluster: bpb.blocks_per_cluster(),
                first_data_block: BlockCount(first_data_block),
                fat_start: BlockCount(u32::from(bpb.reserved_block_count())),
                free_clusters_count: info_sector.free_clusters_count(),
                next_free_cluster: info_sector.next_free_cluster(),
                cluster_count: bpb.total_clusters(),
                fat_specific_info: FatSpecificInfo::Fat32(Fat32Info {
                    info_location: lba_start + info_location,
                    first_root_dir_cluster: Cluster(bpb.first_root_dir_cluster()),
                }),
            };
            volume.name.data[..].copy_from_slice(bpb.volume_label());
            Ok(VolumeType::Fat(volume))
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
