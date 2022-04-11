use super::Result;
use core::fmt;
use core::mem::size_of;
use core::ops::Deref;
// use log::info;

use super::common::*;
use super::internal::*;
use super::reader::*;

pub const MAX_BOOTARGS_LEN: usize = 200;
pub const MAX_STRINGS_BLOCK_LEN: usize = 5000;

/// A vector with a fixed capacity of `M` elements allocated on the stack.
pub struct SerializedBuffer<const M: usize> {
    buffer: [u8; M],
    len: usize,
}

impl<const M: usize> SerializedBuffer<M> {
    /// Creates a new [`SerializedBuffer`]
    pub fn new(buffer: [u8; M], len: usize) -> Self {
        SerializedBuffer { buffer, len }
    }
    /// Returns a slice containing the entire vector.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.len) }
    }

    pub fn as_str<'a>(&'a self) -> Result<&'a str> {
        let val = core::str::from_utf8(self.as_slice())
            .map_err(|val| Error::BadStrEncoding(val))?
            .strip_suffix("\u{0}");
        Ok(val.unwrap())
    }
}

impl<const M: usize> Deref for SerializedBuffer<M> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const M: usize> fmt::Debug for SerializedBuffer<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <[u8] as fmt::Debug>::fmt(self, f)
    }
}

/// Concatenates a slice of bytes with `self` and converts the result to a [`SerializedBuffer`]  
///
/// Note:
/// - The resultant buffer is contains the concatenated string-literal
/// - The length of concatenated string-literal is `< 50`
///
pub trait Concat {
    fn concat<const N: usize>(self, slice_2: &[u8]) -> SerializedBuffer<N>;
}

impl<'a> Concat for &'a str {
    /// Concatenates a slice of bytes with `self` and converts the result to a [`SerializedBuffer`]  
    ///
    /// Note:
    /// - The resultant buffer contains the concatenated string-literal
    /// - The length of concatenated string-literal has to be `< 50`
    ///
    fn concat<const N: usize>(self, slice_2: &[u8]) -> SerializedBuffer<N> {
        let mut buffer = [0u8; N];
        let slice_1 = self.as_bytes();

        let _ = slice_1
            .iter()
            .chain(slice_2.iter())
            .enumerate()
            .for_each(|(idx, byte)| buffer[idx] = *byte);
        let len = slice_1.len() + slice_2.len();
        SerializedBuffer { buffer, len }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
/// A [`RawNodeConstructor`] represents the binary form of a device-tree node. This includes
/// a `FDT_BEGIN_NODE` and the node’s name as extra data.
pub struct RawNodeConstructor<'a> {
    fdt_begin_node: u32,
    node_name: &'a [u8],
}

impl<'a> RawNodeConstructor<'a> {
    pub fn new(fdt_begin_node: u32, node_name: &'a [u8]) -> Self {
        RawNodeConstructor {
            fdt_begin_node,
            node_name,
        }
    }
    /// Constructs a device-tree `node`, given a name and buffer. The buffer must be adequately sized.
    pub fn make_raw_node(buf: &'a mut [u8], name: &'a str) -> Result<Self> {
        // calculate `raw node size and count` in bytes. size includes null + padding bytes
        let node_size_in_bytes;
        let count;
        let name_len = name.as_bytes().len();
        match (TOKEN_SIZE + name_len) % 2 == 0 {
            true => {
                // if even
                node_size_in_bytes = (TOKEN_SIZE + name_len) + ((TOKEN_SIZE + name_len) % 4);
                count = TOKEN_SIZE + name_len;
            }
            false => {
                // if odd
                node_size_in_bytes = (TOKEN_SIZE + name_len) + ((TOKEN_SIZE + name_len + 2) % 4);
                count = TOKEN_SIZE + name_len + 2;
            }
        }
        if buf.len() < node_size_in_bytes {
            return Err(Error::BufferTooSmall);
        }
        // construct raw node
        let node_name = name.as_bytes();
        if count % 4 == 0 {
            let padding = [0; 4]; // node names are always null terminated
            buf[..name_len].copy_from_slice(node_name);
            buf[name_len..name_len + 4].copy_from_slice(&padding[..]);
            Ok(RawNodeConstructor {
                fdt_begin_node: TOK_BEGIN_NODE,
                node_name: &buf[..name_len + 4],
            })
        } else {
            let padding = count % 4;
            let max_padding_bytes = [0; 4]; // max padding is 3 bytes
            buf[..name_len].copy_from_slice(node_name);
            buf[name_len..name_len + padding].copy_from_slice(&max_padding_bytes[..padding]);
            Ok(RawNodeConstructor {
                fdt_begin_node: TOK_BEGIN_NODE,
                node_name: &buf[..name_len + padding],
            })
        }
    }

    /// A raw node can be serialized to its constituent bytes i.e. a [`SerializedBuffer`]. The `len` field of the
    /// [`SerializedBuffer`] contains the `length` of the buffer.
    pub fn serialize(&self) -> Result<SerializedBuffer<30>> {
        let mut node_name_size;
        let len = self.fdt_begin_node.to_be_bytes().len() + self.node_name.len();
        if len < 30 {
            // dts-spec requires node name sizes to be between 1-31 chars.
            node_name_size = [0u8; 30]
        } else {
            return Err(Error::BadNodeName);
        }
        self.fdt_begin_node
            .to_be_bytes()
            .iter()
            .chain(self.node_name.iter())
            .enumerate()
            .for_each(|(idx, byte)| node_name_size[idx] = *byte);

        Ok(SerializedBuffer {
            buffer: node_name_size,
            len,
        })
    }

    /// Constructs a `device-tree node` with supplied `device-tree properties`.
    ///
    /// Note:
    /// - `buf` contains the result i.e. concatenated byte-sequence of node and properties
    /// - `node_name` is the name of the node to be constructed
    /// - `name_offset_list` - is the list of offsets in the device-tree's strings-block. These
    /// offsets are used to locate the corresponding *property-names*.
    /// - `prop_val_list` - is a ref to a list of property values.
    /// - returns the length of the `node + properties`
    ///
    pub fn make_node_with_props(
        buf: &'a mut [u8],
        node_name: &'a str,
        name_offset_list: &[usize],
        prop_val_list: &[PropertyValue],
    ) -> Result<usize> {
        if name_offset_list.len() != prop_val_list.len() {
            return Err(Error::NonExhaustive);
        }
        let raw_chosen_node = RawNodeConstructor::make_raw_node(&mut buf[..], node_name).unwrap();
        let serialized_node_buffer = raw_chosen_node.serialize()?;
        let raw_node = serialized_node_buffer.as_slice();
        let node_len = raw_chosen_node.serialize()?.len;
        buf[..node_len].copy_from_slice(raw_node);
        let mut buffer_offset = node_len;
        for (idx, property_val) in prop_val_list.iter().enumerate() {
            let raw_prop_node = RawPropertyConstructor::make_raw_property(
                &mut buf[buffer_offset..],
                name_offset_list[idx],
                property_val,
            )
            .unwrap();
            let serialized_prop_buffer = raw_prop_node.serialize()?;
            let raw_prop = serialized_prop_buffer.as_slice();
            let prop_len = raw_prop_node.serialize()?.len;
            buf[buffer_offset..buffer_offset + prop_len].copy_from_slice(raw_prop);
            buffer_offset += prop_len
        }
        Ok(buffer_offset)
    }
}

/// A property value is an array of zero or more bytes that contain information associated with the property.
/// [Table 2.3](https://github.com/devicetree-org/devicetree-specification/releases/download/v0.4-rc1/devicetree-specification-v0.4-rc1.pdf) of the DTSpec
/// describes the set of basic value types.
///
/// Note:  This impl doesnt account for all property value types.
pub enum PropertyValue<'a> {
    String(&'a str),
    U32([u8; 4]),
    Empty,
}

impl<'a> AsRef<[u8]> for PropertyValue<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Empty => &[],
            Self::String(val) => val.as_ref(),
            Self::U32(val) => val.as_ref(),
        }
    }
}

/// A [`RawPropertyConstructor`] represents the binary form of a device-tree property. It starts with the FDT_PROP
/// token which marks the beginning of one property in the devicetree. This is followed by the property’s length,
/// a name-offset and the property value.
///
/// - The length field gives the length of the property’s value in bytes (which may be zero, indicating an empty property)
/// - name-offset gives an offset into the strings block at which the property’s name is stored as a null-terminated string
/// - Lastly, the property’s value is given as a byte string of length `prop_len`.
///
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RawPropertyConstructor<'a> {
    fdt_prop: u32,
    prop_len: u32,
    name_off: u32,
    prop_val: &'a [u8],
}

impl Default for RawPropertyConstructor<'_> {
    fn default() -> Self {
        Self {
            fdt_prop: Default::default(),
            prop_len: Default::default(),
            name_off: Default::default(),
            prop_val: Default::default(),
        }
    }
}

impl<'a> RawPropertyConstructor<'a> {
    pub fn new(fdt_prop: u32, prop_len: u32, name_off: u32, prop_val: &'a [u8]) -> Self {
        RawPropertyConstructor {
            fdt_prop,
            prop_len,
            name_off,
            prop_val,
        }
    }
    /// Constructs a device-tree `property`, given a buffer, a property value and an offset into the
    /// device-tree `strings-block`. The buffer must be adequately sized.
    pub fn make_raw_property(
        buf: &'a mut [u8],
        prop_name_offset: usize,
        prop_val: &PropertyValue,
    ) -> Result<Self> {
        // calculate `raw property size and count` in bytes. size includes null + padding bytes
        let prop_size_in_bytes;
        let count;
        let prop_val_len = prop_val.as_ref().len();
        match (TOKEN_SIZE * 3 + prop_val_len) % 2 == 0 {
            true => {
                // if even
                prop_size_in_bytes =
                    (TOKEN_SIZE * 3 + prop_val_len) + ((TOKEN_SIZE * 3 + prop_val_len) % 4);
                count = (TOKEN_SIZE * 3) + prop_val.as_ref().len();
            }
            false => {
                // if odd
                prop_size_in_bytes =
                    (TOKEN_SIZE * 3 + prop_val_len) + ((TOKEN_SIZE * 3 + prop_val_len + 2) % 4);
                count = (TOKEN_SIZE * 3) + prop_val.as_ref().len() + 2;
            }
        }
        if buf.len() < prop_size_in_bytes {
            return Err(Error::BufferTooSmall);
        }
        // construct raw property
        if count % 4 == 0 {
            match prop_val {
                PropertyValue::String(val) => {
                    let padding = [0; 4]; // property values are always null terminated
                    buf[..prop_val_len].copy_from_slice(prop_val.as_ref());
                    buf[prop_val_len..prop_val_len + 4].copy_from_slice(&padding[..]);
                    Ok(RawPropertyConstructor {
                        fdt_prop: TOK_PROPERTY,
                        prop_len: val.len() as u32,
                        name_off: prop_name_offset as u32,
                        prop_val: &buf[..prop_val_len + 4],
                    })
                }
                PropertyValue::U32(val) => {
                    buf[..prop_val_len].copy_from_slice(prop_val.as_ref());
                    Ok(RawPropertyConstructor {
                        fdt_prop: TOK_PROPERTY,
                        prop_len: val.len() as u32,
                        name_off: prop_name_offset as u32,
                        prop_val: &buf[..prop_val_len],
                    })
                }
                _ => unimplemented!(),
            }
        } else {
            let padding = count % 4;
            let max_padding_bytes = [0; 4]; // max padding is 3 bytes
            buf[..prop_val_len].copy_from_slice(prop_val.as_ref());
            buf[prop_val_len..prop_val_len + padding]
                .copy_from_slice(&max_padding_bytes[..padding]);
            Ok(RawPropertyConstructor {
                fdt_prop: TOK_PROPERTY,
                prop_len: prop_val.as_ref().len() as u32,
                name_off: prop_name_offset as u32,
                prop_val: &buf[..prop_val_len + padding],
            })
        }
    }

    /// A raw property can be serialized to its constituent bytes i.e. a [`SerializedBuffer`]. The `len` field of the
    /// [`SerializedBuffer`] contains the `length` of the buffer.
    ///
    /// - MAX_BOOTARGS_LEN: This impl limits support to properties with a maximum size of 200 bytes.
    ///
    pub fn serialize(&self) -> Result<SerializedBuffer<MAX_BOOTARGS_LEN>> {
        let mut prop_size;
        let len = (TOKEN_SIZE * 3) + self.prop_val.len();
        if len < MAX_BOOTARGS_LEN {
            // max property size allowed by this impl.
            prop_size = [0u8; MAX_BOOTARGS_LEN]
        } else {
            return Err(Error::BadNodeName);
        }
        for (idx, byte) in self
            .fdt_prop
            .to_be_bytes()
            .iter()
            .chain(self.prop_len.to_be_bytes().iter())
            .chain(self.name_off.to_be_bytes().iter())
            .chain(self.prop_val)
            .enumerate()
        {
            prop_size[idx] = *byte
        }
        Ok(SerializedBuffer {
            buffer: prop_size,
            len,
        })
    }
}

#[derive(Debug)]
/// The strings block contains strings representing all the property names used in the tree. These null terminated strings are
/// simply concatenated together in this section, and referred to from the structure block by an offset into the strings block.
///
/// This type if used to construct a new [`StringsBlock`]. The `offset` field contains the size of a populated [`StringsBlock`]
pub struct StringsBlock<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> StringsBlock<'a> {
    /// Creates a new [`StringsBlock`], from a given buffer and starting offset of 0.
    pub fn new(buf: &'a mut [u8]) -> Result<Self> {
        if buf.len() > MAX_STRINGS_BLOCK_LEN {
            return Err(Error::Unsupported);
        }
        Ok(StringsBlock { buf, offset: 0 })
    }

    /// Constructs a [`StringsBlock`] by appending [`StringsBlockEntry`] `entries`. It returns `Self` and
    /// offsets to the appended string-entry.
    ///
    pub fn add_entry(&mut self, block_entry: StringsBlockEntry) -> Result<(&mut Self, usize)> {
        if self.buf.len() < self.offset {
            return Err(Error::BufferExhausted);
        }
        let old_len = self.offset;
        let range = old_len..old_len + block_entry.entry.len();
        self.buf[range].copy_from_slice(block_entry.entry);
        // add an extra byte (+1), strings in the `strings block` are null terminated
        self.offset = old_len + block_entry.entry.len() + 1;
        Ok((self, old_len))
    }

    /// Populates a [`StringsBlock`] with a (null-terminated) sequence of strings. You'll have to supply
    /// the list of strings to be appended. It returns a list of offsets (into the strings block)
    /// for the appended strings.
    ///
    /// - An `offset` represents the starting index of a `property-name` into the strings block.
    ///
    pub fn make_new_strings_block_with(&mut self, name_list: &[&str]) -> Result<[usize; 10]> {
        // we limit the number of strings (you can append to the [`StringsBlock`]) to `10 strings`
        if name_list.len() > 10 {
            return Err(Error::Unsupported);
        }
        let mut string_block_entries = [StringsBlockEntry { entry: &[] }; 10];
        for (idx, name) in name_list.iter().enumerate() {
            if name.is_empty() {
                return Err(Error::BadPropertyName);
            }
            string_block_entries[idx] = StringsBlockEntry::new(name.as_bytes());
        }
        let mut offset_list = [0usize; 10];
        let name_list_count = name_list.len();
        // and then append the remaining entries
        for count in 0..name_list_count {
            let (_, offset) = self.add_entry(string_block_entries[count])?;
            offset_list[count] = offset;
        }
        Ok(offset_list)
    }

    pub fn finalize(&self) -> &[u8] {
        &self.buf[..self.offset]
    }
}

#[derive(Debug, Clone, Copy)]
/// A [`StringsBlockEntry`] represents a single null-terminated string. The device-tree's strings-block
/// contains a sequence of such strings.
pub struct StringsBlockEntry<'a> {
    entry: &'a [u8],
}

impl<'a> StringsBlockEntry<'a> {
    /// Creates a new [`StringsBlockEntry`], from a given buffer.
    pub fn new(buf: &'a [u8]) -> Self {
        StringsBlockEntry { entry: buf }
    }
}
/// Reserved memory block.
#[derive(Debug)]
pub struct ReservedMem<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ReservedMem<'a> {
    /// Creates a new reserved memory block from a given buffer.
    pub fn from_buf(buf: &'a mut [u8]) -> Result<ReservedMem<'a>> {
        let buf = align_buf::<ReservedMemEntry>(buf)?;

        if buf.len() < size_of::<Header>() {
            return Err(Error::BufferTooSmall);
        }

        Ok(ReservedMem {
            buf,
            offset: size_of::<Header>(),
        })
    }

    /// Adds a new reserved memory entry.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn add_entry(&mut self, address: u64, size: u64) -> Result<()> {
        if self.buf.len() < self.offset + size_of::<ReservedMemEntry>() {
            return Err(Error::BufferTooSmall);
        }

        let entry_be =
            unsafe { &mut *(self.buf.as_mut_ptr().add(self.offset) as *mut ReservedMemEntry) };

        entry_be.address = u64::to_be(address);
        entry_be.size = u64::to_be(size);

        self.offset += size_of::<ReservedMemEntry>();

        Ok(())
    }
}

/// Device tree blob writer.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Writer<'a> {
    buf: &'a mut [u8],
    reserved_mem_offset: usize,
    struct_offset: usize,
    strings_offset: usize,
}

impl<'a> Writer<'a> {
    /// Creates a DTB writer from a given buffer.
    pub fn from_buf(buf: &'a mut [u8]) -> Result<Writer<'a>> {
        Writer::from_reserved_mem(ReservedMem::from_buf(buf)?)
    }

    /// Creates a DTB writer from a given reserved memory block.
    pub fn from_reserved_mem(mut reserved_mem: ReservedMem<'a>) -> Result<Writer<'a>> {
        reserved_mem.add_entry(0, 0)?;
        let len = reserved_mem.buf.len();
        Ok(Writer {
            buf: reserved_mem.buf,
            reserved_mem_offset: reserved_mem.offset,
            struct_offset: reserved_mem.offset,
            strings_offset: len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER_U32_NUM: usize = size_of::<Header>() / size_of::<u32>();
    const ENTRY_U32_NUM: usize = size_of::<ReservedMemEntry>() / size_of::<u32>();

    fn assert_reserved_mem<'a, T>(func: fn(buf: &'a mut [u8]) -> Result<T>)
    where
        T: std::fmt::Debug,
    {
        aligned_buf!(tmp, [0u32; HEADER_U32_NUM + 1]);
        let len = tmp.len();
        let unaligned_buf = &mut tmp[1..len - 1];
        assert_eq!(func(unaligned_buf).unwrap_err(), Error::BufferTooSmall);

        aligned_buf!(buf, [0u32; HEADER_U32_NUM - 1]);
        assert_eq!(func(buf).unwrap_err(), Error::BufferTooSmall);
    }

    #[test]
    fn test_reserved_mem() {
        assert_reserved_mem(|buf| ReservedMem::from_buf(buf));

        aligned_buf!(buf, [0u32; HEADER_U32_NUM]);
        let mut reserved_mem = ReservedMem::from_buf(buf).unwrap();
        assert_eq!(
            reserved_mem.add_entry(1, 1).unwrap_err(),
            Error::BufferTooSmall
        );

        aligned_buf!(buf, [0u32; HEADER_U32_NUM + ENTRY_U32_NUM]);
        let mut reserved_mem = ReservedMem::from_buf(buf).unwrap();
        reserved_mem.add_entry(1, 1).unwrap();
        assert_eq!(
            reserved_mem.add_entry(1, 1).unwrap_err(),
            Error::BufferTooSmall
        );
    }

    #[test]
    fn test_new_writer() {
        assert_reserved_mem(|buf| Writer::from_buf(buf));

        aligned_buf!(buf, [0u32; HEADER_U32_NUM]);
        assert_eq!(Writer::from_buf(buf).unwrap_err(), Error::BufferTooSmall);

        let reserved_mem = ReservedMem::from_buf(buf).unwrap();
        assert_eq!(
            Writer::from_reserved_mem(reserved_mem).unwrap_err(),
            Error::BufferTooSmall
        );
    }
}
