//! # adapted from embedded-sdmmc
//!
//! > An EMMCfat Library written in Embedded Rust
//!
//! This module allows you to read/write files on a FAT formatted SD
//! card on your Rust Embedded device, as easily as using the `SdFat` Arduino
//! library. It is written in pure-Rust, is `#![no_std]` and does not use `alloc`
//! or `collections` to keep the memory footprint low. In the first instance it is
//! designed for readability and simplicity over performance.
//!

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use byteorder::{ByteOrder, LittleEndian};
use core::convert::TryFrom;
use log::info;

use super::blockdevice::{Block, BlockCount, BlockDevice, BlockIdx};
use super::fat;
use super::fat::FatVolume;
use super::fat::RESERVED_ENTRIES;
use super::filesystem::{
    Attributes, Cluster, DirEntry, Directory, File, FilenameError, Mode, ShortFileName, TimeSource,
    Timestamp, MAX_FILE_SIZE,
};

pub use super::fat::FAT_CACHE;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// Represents all the ways the functions in this crate can fail.
#[derive(Debug, Clone)]
pub enum Error<E>
where
    E: core::fmt::Debug,
{
    /// The underlying block device threw an error.
    DeviceError(E),
    /// The filesystem is badly formatted (or this code is buggy).
    FormatError(&'static str),
    /// The given `VolumeIdx` was bad,
    NoSuchVolume,
    /// The given filename was bad
    FilenameError(FilenameError),
    /// Out of memory opening directories
    TooManyOpenDirs,
    /// Out of memory opening files
    TooManyOpenFiles,
    /// That file doesn't exist
    FileNotFound,
    /// You can't open a file twice
    FileAlreadyOpen,
    /// You can't open a directory twice
    DirAlreadyOpen,
    /// You can't open a directory as a file
    OpenedDirAsFile,
    /// You can't delete a directory as a file
    DeleteDirAsFile,
    /// You can't delete an open file
    FileIsOpen,
    /// We can't do that yet
    Unsupported,
    /// Tried to read beyond end of file
    EndOfFile,
    /// Found a bad cluster
    BadCluster,
    /// Error while converting types
    ConversionError,
    /// The device does not have enough space for the operation
    NotEnoughSpace,
    /// Cluster was not properly allocated by the library
    AllocationError,
    /// Jumped to free space during fat traversing
    JumpedFree,
    /// Tried to open Read-Only file with write mode
    ReadOnly,
    /// Tried to create an existing file
    FileAlreadyExists,
    /// Bad block size - only 512 byte blocks supported
    BadBlockSize(u16),
    /// Entry not found in the block
    NotInBlock,
}

/// We have to track what directories are open to prevent users from modifying
/// open directories (like creating a file when we have an open iterator).
pub const MAX_OPEN_DIRS: usize = 4;

/// We have to track what files and directories are open to prevent users from
/// deleting open files (like Windows does).
pub const MAX_OPEN_FILES: usize = 4;

pub struct TestClock;

impl TimeSource for TestClock {
    fn get_timestamp(&self) -> Timestamp {
        // bogus timestamp for testing purposes. We'll need an external RTC for this
        // rustBoot does not have a networking stack, so no NTP.
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

/// A `Controller` wraps a block device and gives access to the volumes within it.
pub struct Controller<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    <D as BlockDevice>::Error: core::fmt::Debug,
{
    pub block_device: D,
    pub timesource: T,
    open_dirs: [(VolumeIdx, Cluster); MAX_OPEN_DIRS],
    open_files: [(VolumeIdx, Cluster); MAX_OPEN_DIRS],
}

/// Represents a partition with a filesystem within it.
#[derive(Debug, PartialEq, Eq)]
pub struct Volume {
    idx: VolumeIdx,
    pub volume_type: VolumeType,
}

/// This enum holds the data for the various different types of filesystems we
/// support.
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeType {
    /// FAT16/FAT32 formatted volumes.
    Fat(FatVolume),
}

/// A `VolumeIdx` is a number which identifies a volume (or partition) on a
/// disk. `VolumeIdx(0)` is the first primary partition on an MBR partitioned
/// disk.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct VolumeIdx(pub usize);

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Types
//
// ****************************************************************************

/// Marker for a FAT32 partition. Sometimes also use for FAT16 formatted
/// partitions.
const PARTITION_ID_FAT32_LBA: u8 = 0x0C;
/// Marker for a FAT16 partition with LBA. Seen on a Raspberry Pi SD card.
const PARTITION_ID_FAT16_LBA: u8 = 0x0E;
/// Marker for a FAT16 partition. Seen on a card formatted with the official
/// SD-Card formatter.
const PARTITION_ID_FAT16: u8 = 0x06;
/// Marker for a FAT32 partition. What Macosx disk utility (and also SD-Card formatter?)
/// use.
const PARTITION_ID_FAT32_CHS_LBA: u8 = 0x0B;

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Functions / Impl for Public Types
//
// ****************************************************************************

impl<D, T> Controller<D, T>
where
    D: BlockDevice,
    T: TimeSource,
    <D as BlockDevice>::Error: core::fmt::Debug,
{
    /// Create a new Disk Controller using a generic `BlockDevice`. From this
    /// controller we can open volumes (partitions) and with those we can open
    /// files.
    pub fn new(block_device: D, timesource: T) -> Controller<D, T> {
        info!("create new emmc-fat controller...");
        Controller {
            block_device,
            timesource,
            open_dirs: [(VolumeIdx(0), Cluster::INVALID); 4],
            open_files: [(VolumeIdx(0), Cluster::INVALID); 4],
        }
    }

    /// Temporarily get access to the underlying block device.
    pub fn device(&mut self) -> &mut D {
        &mut self.block_device
    }

    /// Get a volume (or partition) based on entries in the Master Boot
    /// Record. We do not support GUID Partition Table disks. Nor do we
    /// support any concept of drive letters - that is for a higher layer to
    /// handle.
    pub fn get_volume(&mut self, volume_idx: VolumeIdx) -> Result<Volume, Error<D::Error>> {
        const PARTITION1_START: usize = 446;
        const PARTITION2_START: usize = PARTITION1_START + PARTITION_INFO_LENGTH;
        const PARTITION3_START: usize = PARTITION2_START + PARTITION_INFO_LENGTH;
        const PARTITION4_START: usize = PARTITION3_START + PARTITION_INFO_LENGTH;
        const FOOTER_START: usize = 510;
        const FOOTER_VALUE: u16 = 0xAA55;
        const PARTITION_INFO_LENGTH: usize = 16;
        const PARTITION_INFO_STATUS_INDEX: usize = 0;
        const PARTITION_INFO_TYPE_INDEX: usize = 4;
        const PARTITION_INFO_LBA_START_INDEX: usize = 8;
        const PARTITION_INFO_NUM_BLOCKS_INDEX: usize = 12;

        let (part_type, lba_start, num_blocks) = {
            let mut blocks = [Block::new()];
            self.block_device
                .read(&mut blocks, BlockIdx(0), "read_mbr")
                .map_err(Error::DeviceError)?;
            let block = &blocks[0];
            // We only support Master Boot Record (MBR) partitioned cards, not
            // GUID Partition Table (GPT)
            if LittleEndian::read_u16(&block[FOOTER_START..FOOTER_START + 2]) != FOOTER_VALUE {
                return Err(Error::FormatError("Invalid MBR signature"));
            }
            let partition = match volume_idx {
                VolumeIdx(0) => {
                    &block[PARTITION1_START..(PARTITION1_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(1) => {
                    &block[PARTITION2_START..(PARTITION2_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(2) => {
                    &block[PARTITION3_START..(PARTITION3_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(3) => {
                    &block[PARTITION4_START..(PARTITION4_START + PARTITION_INFO_LENGTH)]
                }
                _ => {
                    return Err(Error::NoSuchVolume);
                }
            };
            // Only 0x80 and 0x00 are valid (bootable, and non-bootable)
            if (partition[PARTITION_INFO_STATUS_INDEX] & 0x7F) != 0x00 {
                return Err(Error::FormatError("Invalid partition status"));
            }
            let lba_start = LittleEndian::read_u32(
                &partition[PARTITION_INFO_LBA_START_INDEX..(PARTITION_INFO_LBA_START_INDEX + 4)],
            );
            let num_blocks = LittleEndian::read_u32(
                &partition[PARTITION_INFO_NUM_BLOCKS_INDEX..(PARTITION_INFO_NUM_BLOCKS_INDEX + 4)],
            );
            (
                partition[PARTITION_INFO_TYPE_INDEX],
                BlockIdx(lba_start),
                BlockCount(num_blocks),
            )
        };
        match part_type {
            PARTITION_ID_FAT32_CHS_LBA
            | PARTITION_ID_FAT32_LBA
            | PARTITION_ID_FAT16_LBA
            | PARTITION_ID_FAT16 => {
                let volume = fat::parse_volume(self, lba_start, num_blocks)?;
                Ok(Volume {
                    idx: volume_idx,
                    volume_type: volume,
                })
            }
            _ => Err(Error::FormatError("Partition type not supported")),
        }
    }

    /// Open a directory. You can then read the directory entries in a random
    /// order using `get_directory_entry`.
    ///
    /// TODO: Work out how to prevent damage occuring to the file system while
    /// this directory handle is open. In particular, stop this directory
    /// being unlinked.
    pub fn open_root_dir(&mut self, volume: &Volume) -> Result<Directory, Error<D::Error>> {
        // Find a free directory entry, and check the root dir isn't open. As
        // we already know the root dir's magic cluster number, we can do both
        // checks in one loop.
        let mut open_dirs_row = None;
        for (i, d) in self.open_dirs.iter().enumerate() {
            if *d == (volume.idx, Cluster::ROOT_DIR) {
                return Err(Error::DirAlreadyOpen);
            }
            if d.1 == Cluster::INVALID {
                open_dirs_row = Some(i);
                break;
            }
        }
        let open_dirs_row = open_dirs_row.ok_or(Error::TooManyOpenDirs)?;
        // Remember this open directory
        self.open_dirs[open_dirs_row] = (volume.idx, Cluster::ROOT_DIR);
        Ok(Directory {
            cluster: Cluster::ROOT_DIR,
            entry: None,
        })
    }

    /// Open a directory. You can then read the directory entries in a random
    /// order using `get_directory_entry`.
    ///
    /// TODO: Work out how to prevent damage occuring to the file system while
    /// this directory handle is open. In particular, stop this directory
    /// being unlinked.
    pub fn open_dir(
        &mut self,
        volume: &Volume,
        parent_dir: &Directory,
        name: &str,
    ) -> Result<Directory, Error<D::Error>> {
        // Find a free open directory table row
        let mut open_dirs_row = None;
        for (i, d) in self.open_dirs.iter().enumerate() {
            if d.1 == Cluster::INVALID {
                open_dirs_row = Some(i);
            }
        }
        let open_dirs_row = open_dirs_row.ok_or(Error::TooManyOpenDirs)?;

        // Open the directory
        let dir_entry = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.find_directory_entry(self, parent_dir, name)?,
        };

        if !dir_entry.attributes.is_directory() {
            return Err(Error::OpenedDirAsFile);
        }

        // Check it's not already open
        for (_i, dir_table_row) in self.open_dirs.iter().enumerate() {
            if *dir_table_row == (volume.idx, dir_entry.cluster) {
                return Err(Error::DirAlreadyOpen);
            }
        }
        // Remember this open directory
        self.open_dirs[open_dirs_row] = (volume.idx, dir_entry.cluster);
        Ok(Directory {
            cluster: dir_entry.cluster,
            entry: Some(dir_entry),
        })
    }

    /// Close a directory. You cannot perform operations on an open directory
    /// and so must close it if you want to do something with it.
    pub fn close_dir(&mut self, volume: &Volume, dir: Directory) {
        let target = (volume.idx, dir.cluster);
        for d in self.open_dirs.iter_mut() {
            if *d == target {
                d.1 = Cluster::INVALID;
                break;
            }
        }
        drop(dir);
    }

    /// Look in a directory for a named file.
    pub fn find_directory_entry(
        &mut self,
        volume: &Volume,
        dir: &Directory,
        name: &str,
    ) -> Result<DirEntry, Error<D::Error>> {
        match &volume.volume_type {
            VolumeType::Fat(fat) => fat.find_directory_entry(self, dir, name),
        }
    }

    /// Call a callback function for each directory entry in a directory.
    pub fn iterate_dir<F>(
        &mut self,
        volume: &Volume,
        dir: &Directory,
        func: F,
    ) -> Result<(), Error<D::Error>>
    where
        F: FnMut(&DirEntry),
    {
        match &volume.volume_type {
            VolumeType::Fat(fat) => fat.iterate_dir(self, dir, func),
        }
    }

    /// Open a file from DirEntry. This is obtained by calling iterate_dir. A file can only be opened once.
    pub fn open_dir_entry(
        &mut self,
        volume: &mut Volume,
        dir_entry: DirEntry,
        mode: Mode,
    ) -> Result<File, Error<D::Error>> {
        let open_files_row = self.get_open_files_row()?;
        // Check it's not already open
        for dir_table_row in self.open_files.iter() {
            if *dir_table_row == (volume.idx, dir_entry.cluster) {
                return Err(Error::DirAlreadyOpen);
            }
        }
        if dir_entry.attributes.is_directory() {
            return Err(Error::OpenedDirAsFile);
        }
        if dir_entry.attributes.is_read_only() && mode != Mode::ReadOnly {
            return Err(Error::ReadOnly);
        }

        let mode = solve_mode_variant(mode, true);
        let file = match mode {
            Mode::ReadOnly => File {
                starting_cluster: dir_entry.cluster,
                current_cluster: (0, dir_entry.cluster),
                current_offset: 0,
                length: dir_entry.size,
                mode,
                entry: dir_entry,
            },
            Mode::ReadWriteAppend => {
                let mut file = File {
                    starting_cluster: dir_entry.cluster,
                    current_cluster: (0, dir_entry.cluster),
                    current_offset: 0,
                    length: dir_entry.size,
                    mode,
                    entry: dir_entry,
                };
                // seek_from_end with 0 can't fail
                file.seek_from_end(0).ok();
                file
            }
            Mode::ReadWriteTruncate => {
                let mut file = File {
                    starting_cluster: dir_entry.cluster,
                    current_cluster: (0, dir_entry.cluster),
                    current_offset: 0,
                    length: dir_entry.size,
                    mode,
                    entry: dir_entry,
                };
                match &mut volume.volume_type {
                    VolumeType::Fat(fat) => {
                        fat.truncate_cluster_chain(self, file.starting_cluster)?
                    }
                };
                file.update_length(0);
                // TODO update entry Timestamps
                match &volume.volume_type {
                    VolumeType::Fat(fat) => {
                        let fat_type = fat.get_fat_type();
                        self.write_entry_to_disk(fat_type, &file.entry)?;
                    }
                };

                file
            }
            _ => return Err(Error::Unsupported),
        };
        // Remember this open file
        self.open_files[open_files_row] = (volume.idx, file.starting_cluster);
        Ok(file)
    }

    /// Open a file with the given full path. A file can only be opened once.
    pub fn open_file_in_dir(
        &mut self,
        volume: &mut Volume,
        dir: &Directory,
        name: &str,
        mode: Mode,
    ) -> Result<File, Error<D::Error>> {
        let dir_entry = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.find_directory_entry(self, dir, name),
        };

        let open_files_row = self.get_open_files_row()?;
        let dir_entry = match dir_entry {
            Ok(entry) => Some(entry),
            Err(_)
                if (mode == Mode::ReadWriteCreate)
                    | (mode == Mode::ReadWriteCreateOrTruncate)
                    | (mode == Mode::ReadWriteCreateOrAppend) =>
            {
                None
            }
            _ => return Err(Error::FileNotFound),
        };

        let mode = solve_mode_variant(mode, dir_entry.is_some());

        match mode {
            Mode::ReadWriteCreate => {
                if dir_entry.is_some() {
                    return Err(Error::FileAlreadyExists);
                }
                let file_name =
                    ShortFileName::create_from_str(name).map_err(Error::FilenameError)?;
                let att = Attributes::create_from_fat(0);
                let entry = match &mut volume.volume_type {
                    VolumeType::Fat(fat) => {
                        fat.write_new_directory_entry(self, dir, file_name, att)?
                    }
                };

                let file = File {
                    starting_cluster: entry.cluster,
                    current_cluster: (0, entry.cluster),
                    current_offset: 0,
                    length: entry.size,
                    mode,
                    entry,
                };
                // Remember this open file
                self.open_files[open_files_row] = (volume.idx, file.starting_cluster);
                Ok(file)
            }
            _ => {
                // Safe to unwrap, since we actually have an entry if we got here
                let dir_entry = dir_entry.unwrap();
                self.open_dir_entry(volume, dir_entry, mode)
            }
        }
    }

    /// Get the next entry in open_files list
    fn get_open_files_row(&self) -> Result<usize, Error<D::Error>> {
        // Find a free directory entry
        let mut open_files_row = None;
        for (i, d) in self.open_files.iter().enumerate() {
            if d.1 == Cluster::INVALID {
                open_files_row = Some(i);
            }
        }
        open_files_row.ok_or(Error::TooManyOpenDirs)
    }

    /// Delete a closed file with the given full path, if exists.
    pub fn delete_file_in_dir(
        &mut self,
        volume: &Volume,
        dir: &Directory,
        name: &str,
    ) -> Result<(), Error<D::Error>> {
        info!(
            "delete_file(volume={:?}, dir={:?}, filename={:?}",
            volume, dir, name
        );
        let dir_entry = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.find_directory_entry(self, dir, name),
        }?;

        if dir_entry.attributes.is_directory() {
            return Err(Error::DeleteDirAsFile);
        }

        let target = (volume.idx, dir_entry.cluster);
        for d in self.open_files.iter_mut() {
            if *d == target {
                return Err(Error::FileIsOpen);
            }
        }

        match &volume.volume_type {
            VolumeType::Fat(fat) => return fat.delete_directory_entry(self, dir, name),
        };
    }

    /// Populates a static cache with the `file allocation table` contents (of the supplied volume).
    /// We use the cache to walk the FAT table. This greatly improves performance when loading large
    /// files (such as fit-images).
    ///
    /// Note:
    /// - Only `FAT32` volumes are supported
    ///
    pub fn populate_fat_cache(
        &self,
        volume: &Volume,
    ) -> Result<(), Error<<D as BlockDevice>::Error>> {
        match &volume.volume_type {
            VolumeType::Fat(vol) => vol.populate_static_fat_cache(self)?,
        }
        Ok(())
    }

    /// Returns the number of contiguous clusters. If the next cluster in the sequence isn't contiguous
    /// (i.e. is fragmented), it returns a 0
    fn check_contiguous_cluster_count(
        &self,
        volume: &Volume,
        mut cluster: Cluster,
        blocks_per_cluster: u8,
    ) -> Result<u32, Error<D::Error>> {
        let mut contiguous_cluster_count = 0u32;
        let mut next_cluster = match &volume.volume_type {
            VolumeType::Fat(fat) => match fat.next_cluster_in_fat_cache(cluster) {
                Ok(cluster) => cluster,
                Err(e) => match e {
                    // If this is the last cluster for the file, simply return the same cluster.
                    Error::EndOfFile => cluster,
                    _ => panic!("Error: traversing the FAT table, {:?}", e),
                },
            },
        };
        while next_cluster.0.wrapping_sub(cluster.0) == 1 {
            cluster = next_cluster;
            next_cluster = match &volume.volume_type {
                VolumeType::Fat(fat) => match fat.next_cluster_in_fat_cache(cluster) {
                    Ok(cluster) => cluster,
                    Err(e) => match e {
                        Error::EndOfFile => break,
                        _ => panic!("Error: traversing the FAT table, {:?}", e),
                    },
                },
            };
            // avoid `block_device` timeouts for contiguous block transfers > 60000 blocks
            if (contiguous_cluster_count * blocks_per_cluster as u32) < 60000 {
                contiguous_cluster_count += 1;
            } else {
                break;
            }
        }
        Ok(contiguous_cluster_count)
    }

    /// Read from an open file. It has the same effect as the [`Self::read`] method but reduces `read time`
    /// by more than 50%, especially in the case of large files (i.e. > 1Mb)
    ///
    /// `read_multi` reads multiple contiguous blocks of a file in a single read operation,
    /// without the extra overhead of additional `data-copying`.
    ///
    /// NOTE:
    /// - This impl assumes the underlying block-device driver (and consequently the block-device) features support
    /// for multi-block reads.
    /// - The following 2 invariants must hold
    ///     - Length of buffer argument must be `>=` to the file length and
    ///     - the buffer must be a multiple of `block-size` bytes.
    /// - Providing a buffer that isn't a multiple of `block-size` bytes and is less-than file-length will result
    /// in an `out of bounds` error. In other words, for files that aren't exact multiples of `block-size` bytes,
    /// a buffer of length (block-size * (file length/ block size)) + 1 must be provided.
    ///
    pub fn read_multi(
        &mut self,
        volume: &Volume,
        file: &mut File,
        buffer: &mut [u8],
    ) -> Result<usize, Error<D::Error>> {
        let blocks_per_cluster = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.blocks_per_cluster,
        };

        let mut bytes_read = 0;
        let mut block_read_counter = 0;
        let mut starting_cluster = file.starting_cluster;
        let mut file_blocks;
        if (file.length % Block::LEN as u32) == 0 {
            file_blocks = file.length / Block::LEN as u32;
        } else {
            file_blocks = (file.length / Block::LEN as u32) + 1;
        }

        while file_blocks > 0 {
            // Walk the FAT to see if we have contiguos clusters
            let contiguous_cluster_count =
                self.check_contiguous_cluster_count(volume, starting_cluster, blocks_per_cluster)?;

            let blocks_to_read = (contiguous_cluster_count + 1) * blocks_per_cluster as u32;
            let bytes_to_read = Block::LEN * blocks_to_read as usize;
            let (blocks, _) = buffer[block_read_counter..block_read_counter + bytes_to_read]
                .as_chunks_mut::<{ Block::LEN }>();
            // `cluster_to_block` gives us the absolute block_idx i.e. gives us the block offset from the 0th Block
            let block_idx = match &volume.volume_type {
                VolumeType::Fat(fat) => fat.cluster_to_block(starting_cluster),
            };

            self.block_device
                .read(Block::from_array_slice(blocks), block_idx, "read_multi")
                .map_err(Error::DeviceError)?;

            file_blocks = match file_blocks.checked_sub(blocks_to_read) {
                // checked integer subtraction
                Some(val) => val,
                None => 0,
            };
            let next_cluster = match &volume.volume_type {
                VolumeType::Fat(fat) => {
                    match fat.next_cluster_in_fat_cache(starting_cluster + contiguous_cluster_count)
                    {
                        Ok(cluster) => cluster,
                        Err(e) => match e {
                            Error::EndOfFile => {
                                let bytes = bytes_to_read.min(file.left() as usize);
                                bytes_read += bytes;
                                file.seek_from_current(bytes as i32).unwrap();
                                break;
                            }
                            _ => panic!("Error: traversing the FAT table, {:?}", e),
                        },
                    }
                }
            };
            starting_cluster = next_cluster;

            let bytes = bytes_to_read.min(file.left() as usize);
            bytes_read += bytes;
            file.seek_from_current(bytes as i32).unwrap();
            block_read_counter += Block::LEN * blocks_to_read as usize;
        }
        Ok(bytes_read)
    }

    /// Read from an open file.
    pub fn read(
        &mut self,
        volume: &Volume,
        file: &mut File,
        buffer: &mut [u8],
    ) -> Result<usize, Error<D::Error>> {
        // Calculate which file block the current offset lies within
        // While there is more to read, read the block and copy in to the buffer.
        // If we need to find the next cluster, walk the FAT.
        let mut space = buffer.len();
        let mut read = 0;
        while space > 0 && !file.eof() {
            let (block_idx, block_offset, block_avail) =
                self.find_data_on_disk(volume, &mut file.current_cluster, file.current_offset)?;
            let mut blocks = [Block::new()];
            self.block_device
                .read(&mut blocks, block_idx, "read")
                .map_err(Error::DeviceError)?;
            let block = &blocks[0];
            let to_copy = block_avail.min(space).min(file.left() as usize);
            assert!(to_copy != 0);
            buffer[read..read + to_copy]
                .copy_from_slice(&block[block_offset..block_offset + to_copy]);
            read += to_copy;
            space -= to_copy;
            file.seek_from_current(to_copy as i32).unwrap();
        }
        Ok(read)
    }

    /// Write to a open file.
    pub fn write(
        &mut self,
        volume: &mut Volume,
        file: &mut File,
        buffer: &[u8],
    ) -> Result<usize, Error<D::Error>> {
        info!(
            "write(volume={:?}, file={:?}, buffer={:x?}",
            volume, file, buffer
        );
        if file.mode == Mode::ReadOnly {
            return Err(Error::ReadOnly);
        }
        if file.starting_cluster.0 < RESERVED_ENTRIES {
            // file doesn't have a valid allocated cluster (possible zero-length file), allocate one
            file.starting_cluster = match &mut volume.volume_type {
                VolumeType::Fat(fat) => fat.alloc_cluster(self, None, false)?,
            };
            file.entry.cluster = file.starting_cluster;
            info!("Alloc first cluster {:?}", file.starting_cluster);
        }
        if (file.current_cluster.1).0 < file.starting_cluster.0 {
            info!("Rewinding to start");
            file.current_cluster = (0, file.starting_cluster);
        }
        let bytes_until_max = usize::try_from(MAX_FILE_SIZE - file.current_offset)
            .map_err(|_| Error::ConversionError)?;
        let bytes_to_write = core::cmp::min(buffer.len(), bytes_until_max);
        let mut written = 0;

        while written < bytes_to_write {
            let mut current_cluster = file.current_cluster;
            info!(
                "Have written bytes {}/{}, finding cluster {:?}",
                written, bytes_to_write, current_cluster
            );
            let (block_idx, block_offset, block_avail) =
                match self.find_data_on_disk(volume, &mut current_cluster, file.current_offset) {
                    Ok(vars) => {
                        info!(
                            "Found block_idx={:?}, block_offset={:?}, block_avail={}",
                            vars.0, vars.1, vars.2
                        );
                        vars
                    }
                    Err(Error::EndOfFile) => {
                        info!("Extending file");
                        match &mut volume.volume_type {
                            VolumeType::Fat(ref mut fat) => {
                                if fat
                                    .alloc_cluster(self, Some(current_cluster.1), false)
                                    .is_err()
                                {
                                    return Ok(written);
                                }
                                info!("Allocated new FAT cluster, finding offsets...");
                                let new_offset = self
                                    .find_data_on_disk(
                                        volume,
                                        &mut current_cluster,
                                        file.current_offset,
                                    )
                                    .map_err(|_| Error::AllocationError)?;
                                info!("New offset {:?}", new_offset);
                                new_offset
                            }
                        }
                    }
                    Err(e) => return Err(e),
                };
            let mut blocks = [Block::new()];
            let to_copy = core::cmp::min(block_avail, bytes_to_write - written);
            if block_offset != 0 {
                info!("Partial block write");
                self.block_device
                    .read(&mut blocks, block_idx, "read")
                    .map_err(Error::DeviceError)?;
            }
            let block = &mut blocks[0];
            block[block_offset..block_offset + to_copy]
                .copy_from_slice(&buffer[written..written + to_copy]);
            info!("Writing block {:?}", block_idx);
            self.block_device
                .write(&blocks, block_idx)
                .map_err(Error::DeviceError)?;
            written += to_copy;
            file.current_cluster = current_cluster;
            let to_copy = i32::try_from(to_copy).map_err(|_| Error::ConversionError)?;
            // TODO: Should we do this once when the whole file is written?
            file.update_length(file.length + (to_copy as u32));
            file.seek_from_current(to_copy).unwrap();
            file.entry.attributes.set_archive(true);
            file.entry.mtime = self.timesource.get_timestamp();
            info!("Updating FAT info sector");
            match &mut volume.volume_type {
                VolumeType::Fat(fat) => {
                    fat.update_info_sector(self)?;
                    info!("Updating dir entry");
                    self.write_entry_to_disk(fat.get_fat_type(), &file.entry)?;
                }
            }
        }
        Ok(written)
    }

    /// Close a file with the given full path.
    pub fn close_file(&mut self, volume: &Volume, file: File) -> Result<(), Error<D::Error>> {
        let target = (volume.idx, file.starting_cluster);
        for d in self.open_files.iter_mut() {
            if *d == target {
                d.1 = Cluster::INVALID;
                break;
            }
        }
        drop(file);
        Ok(())
    }

    /// Check if any files or folders are open.
    pub fn has_open_handles(&self) -> bool {
        !self
            .open_dirs
            .iter()
            .chain(self.open_files.iter())
            .all(|(_, c)| c == &Cluster::INVALID)
    }

    /// Consume self and return BlockDevice and TimeSource
    pub fn free(self) -> (D, T) {
        (self.block_device, self.timesource)
    }

    /// This function turns `desired_offset` into an appropriate block to be
    /// read. It either calculates this based on the start of the file, or
    /// from the last cluster we read - whichever is better.
    fn find_data_on_disk(
        &mut self,
        volume: &Volume,
        start: &mut (u32, Cluster),
        desired_offset: u32,
    ) -> Result<(BlockIdx, usize, usize), Error<D::Error>> {
        let bytes_per_cluster = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.bytes_per_cluster(),
        };
        // How many clusters forward do we need to go?
        let offset_from_cluster = desired_offset - start.0;
        let num_clusters = offset_from_cluster / bytes_per_cluster;
        for _ in 0..num_clusters {
            start.1 = match &volume.volume_type {
                VolumeType::Fat(fat) => fat.next_cluster(self, start.1)?,
            };
            start.0 += bytes_per_cluster;
        }
        // How many blocks in are we?
        let offset_from_cluster = desired_offset - start.0;
        assert!(offset_from_cluster < bytes_per_cluster);
        let num_blocks = BlockCount(offset_from_cluster / Block::LEN_U32);
        let block_idx = match &volume.volume_type {
            VolumeType::Fat(fat) => fat.cluster_to_block(start.1),
        } + num_blocks;
        let block_offset = (desired_offset % Block::LEN_U32) as usize;
        let available = Block::LEN - block_offset;
        Ok((block_idx, block_offset, available))
    }

    /// Writes a Directory Entry to the disk
    fn write_entry_to_disk(
        &mut self,
        fat_type: fat::FatType,
        entry: &DirEntry,
    ) -> Result<(), Error<D::Error>> {
        let mut blocks = [Block::new()];
        self.block_device
            .read(&mut blocks, entry.entry_block, "read")
            .map_err(Error::DeviceError)?;
        let block = &mut blocks[0];

        let start = usize::try_from(entry.entry_offset).map_err(|_| Error::ConversionError)?;
        block[start..start + 32].copy_from_slice(&entry.serialize(fat_type)[..]);

        self.block_device
            .write(&blocks, entry.entry_block)
            .map_err(Error::DeviceError)?;
        Ok(())
    }
}

// ****************************************************************************
//
// Private Functions / Impl for Private Types
//
// ****************************************************************************

/// Transform mode variants (ReadWriteCreate_Or_Append) to simple modes ReadWriteAppend or
/// ReadWriteCreate
fn solve_mode_variant(mode: Mode, dir_entry_is_some: bool) -> Mode {
    let mut mode = mode;
    if mode == Mode::ReadWriteCreateOrAppend {
        if dir_entry_is_some {
            mode = Mode::ReadWriteAppend;
        } else {
            mode = Mode::ReadWriteCreate;
        }
    } else if mode == Mode::ReadWriteCreateOrTruncate {
        if dir_entry_is_some {
            mode = Mode::ReadWriteTruncate;
        } else {
            mode = Mode::ReadWriteCreate;
        }
    }
    mode
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
