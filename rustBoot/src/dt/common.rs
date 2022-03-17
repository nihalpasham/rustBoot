use core::str::Utf8Error;

/// DTB-related error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// Incorrect DTB magic number.
    BadMagic,
    /// Node name is not a zero-terminated string.
    BadNodeName,
    /// Property name is not a zero-terminated string.
    BadPropertyName,
    /// Failed to decode UTF-8 string.
    BadStrEncoding(Utf8Error),
    /// Operation doesn't support a given type of StructItem.
    BadStructItemType,
    /// Unrecognized DTB structure token.
    BadStructToken,
    /// Total size in DTB header is incorrect.
    BadTotalSize,
    /// Property value cannot be decoded as list of integers.
    BadU32List,
    /// Property value cannot be decoded as string.
    BadValueStr,
    /// DTB format version is less than last compatible version.
    BadVersion,
    /// The supplied buffer was exhausted.
    BufferExhausted,
    /// Given buffer is too small to decode property value.
    BufferTooSmall,
    /// No more StructItem left in DTB structure.
    NoMoreStructItems,
    /// No zero entry found in reserved memory block.
    NoZeroReservedMemEntry,
    /// Stopped matching a given path, since the parent node has ended.
    OutOfParentNode,
    /// Reserved memory block overlaps a structure block.
    OverlappingReservedMem,
    /// Structure block overlaps an end of blob.
    OverlappingStrings,
    /// Structure block overlaps a strings block.
    OverlappingStruct,
    /// A non-exhaustive error. Simply means, all other errors.
    NonExhaustive,
    /// Given blob is not 8-byte aligned.
    UnalignedBlob,
    /// Reserved memory block is not 8-byte aligned.
    UnalignedReservedMem,
    /// Structure block is not 4-byte aligned.
    UnalignedStruct,
    /// Given blob is smaller than DTB-header.
    UnexpectedEndOfBlob,
    /// Structure block doesn't end with END-token.
    UnexpectedEndOfStruct,
    /// Unsupported last compatible version.
    UnsupportedCompVersion,
    /// Unsupported
    Unsupported,
}

/// DTB-related result.
pub type Result<T> = core::result::Result<T, Error>;

/// Reserved memory entry.
#[derive(Debug)]
#[repr(C)]
pub struct ReservedMemEntry {
    /// Memory region address.
    pub address: u64,
    /// Memory region size.
    pub size: u64,
}
