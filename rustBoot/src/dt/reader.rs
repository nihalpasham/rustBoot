use core::convert::TryFrom;
use core::iter::FusedIterator;
use core::mem::size_of;
use core::slice::from_raw_parts;
use core::str::from_utf8;

use super::common::*;
use super::internal::*;
use super::struct_item::*;

/// Iterator for reserved memory entries.
#[derive(Clone, Debug)]
pub struct ReservedMemEntries<'a> {
    reserved_mem: &'a [ReservedMemEntry],
    index: usize,
}

impl<'a> Iterator for ReservedMemEntries<'a> {
    type Item = ReservedMemEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.reserved_mem.len() {
            return None;
        }

        let entry_be = &self.reserved_mem[self.index];
        self.index += 1;

        Some(ReservedMemEntry {
            address: u64::from_be(entry_be.address),
            size: u64::from_be(entry_be.size),
        })
    }
}

impl<'a> FusedIterator for ReservedMemEntries<'a> {}

/// Iterator for structure items.
#[derive(Clone, Copy, Debug)]
pub struct StructItems<'a> {
    struct_block: &'a [u8],
    strings_block: &'a [u8],
    offset: usize,
}

pub const TOKEN_SIZE: usize = 4;

impl<'a> StructItems<'a> {
    pub fn get_offset(&self) -> usize {
        self.offset
    }

    fn set_offset(&mut self, offset: usize) {
        self.offset = ((offset + TOKEN_SIZE - 1) / TOKEN_SIZE) * TOKEN_SIZE;
    }

    fn read_begin_node(&mut self) -> Result<StructItem<'a>> {
        let offset = self.offset + TOKEN_SIZE;
        for (i, chr) in (&self.struct_block[offset..]).iter().enumerate() {
            if *chr != 0 {
                continue;
            }
            return match from_utf8(&self.struct_block[offset..offset + i]) {
                Ok(name) => {
                    self.set_offset(offset + i + 1);
                    Ok(StructItem::BeginNode { name })
                }
                Err(err) => Err(Error::BadStrEncoding(err)),
            };
        }
        Err(Error::BadNodeName)
    }

    fn assert_enough_struct(&self, offset: usize, size: usize) -> Result<()> {
        if self.struct_block.len().checked_sub(size) < Some(offset) {
            Err(Error::UnexpectedEndOfStruct)
        } else {
            Ok(())
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn read_property(&mut self) -> Result<StructItem<'a>> {
        let mut offset = self.offset + TOKEN_SIZE;
        let desc_size = size_of::<PropertyDesc>();
        self.assert_enough_struct(offset, desc_size)?;

        let desc_be = unsafe {
            &*((&self.struct_block[offset..]).as_ptr() as *const PropertyDesc) as &PropertyDesc
        };
        offset += desc_size;

        let value_size = u32::from_be(desc_be.value_size) as usize;
        self.assert_enough_struct(offset, value_size)?;
        let value = &self.struct_block[offset..offset + value_size];
        offset += value_size;

        let name_offset = u32::from_be(desc_be.name_offset) as usize;
        let string_start = self
            .strings_block
            .get(name_offset..)
            .ok_or(Error::UnexpectedEndOfBlob)?;

        let pos = string_start
            .get(1..)
            .ok_or(Error::BadPropertyName)?
            .iter()
            .position(|&ch| ch == 0)
            .ok_or(Error::BadPropertyName)?;

        let name = from_utf8(&string_start[..=pos]).map_err(Error::BadStrEncoding)?;
        self.set_offset(offset);

        Ok(StructItem::Property { name, value })
    }

    /// Advances the iterator and returns the next structure item or error.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn next_item(&mut self) -> Result<StructItem<'a>> {
        loop {
            self.assert_enough_struct(self.offset, TOKEN_SIZE)?;

            let token = u32::from_be(unsafe {
                *((&self.struct_block[self.offset..]).as_ptr() as *const u32)
            });

            if token == TOK_NOP {
                self.offset += TOKEN_SIZE;
                continue;
            }

            return match token {
                TOK_BEGIN_NODE => self.read_begin_node(),
                TOK_PROPERTY => self.read_property(),
                TOK_END_NODE => {
                    self.offset += TOKEN_SIZE;
                    Ok(StructItem::EndNode)
                }
                TOK_END => Err(Error::NoMoreStructItems),
                _ => Err(Error::BadStructToken),
            };
        }
    }

    /// Returns the value of a property (if it exists) within a node.
    ///
    /// Note:
    /// - `self` here must point to the [`StructItem`] after the requested [`StructItem::BeginNode`]
    /// i.e. if I want the value for the `default` property within the `configurations` node of a given
    /// FIT Image, I will have to first parse the configurations node which will return a `self`. We can use
    /// this `self` to retrieve the property's value.
    /// - This methods return a `None` if you try to retrieve a `nested node as a property`.
    ///   
    pub fn get_node_property(self, name: &'a str) -> Option<&'a [u8]> {
        let mut property_val = None;
        let mut sub_node_end = false;
        for item in self {
            if item.is_property() {
                match item.name() {
                    Ok(val) if val == name => {
                        property_val = Some(item.value().unwrap());
                        break;
                    }
                    _ => {}
                }
            } else if item.is_begin_node() {
                sub_node_end = true;
            } else if item.is_end_node() {
                if sub_node_end == true {
                    sub_node_end = false;
                    continue;
                } else {
                    break;
                }
            }
        }
        property_val
    }

    /// Returns a structure path iterator for a given path.
    pub fn path_struct_items<'b>(&self, path: &'b str) -> PathStructItems<'a, 'b> {
        PathStructItems {
            error: None,
            iter: self.clone(),
            path: PathSplit::new(path),
            level: 0,
        }
    }
}

impl<'a> Iterator for StructItems<'a> {
    type Item = StructItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_item() {
            Ok(item) => Some(item),
            Err(_) => None,
        }
    }
}

impl<'a> FusedIterator for StructItems<'a> {}

#[derive(Clone, Debug)]
struct PathSplit<'a> {
    path: &'a str,
    comp: &'a str,
    index: usize,
    num: usize,
}

impl<'a> PathSplit<'a> {
    pub fn new(path: &'a str) -> PathSplit<'a> {
        let path = if path.ends_with('/') {
            &path[..path.len() - 1]
        } else {
            path
        };
        let mut split = PathSplit {
            path,
            comp: "",
            index: 0,
            num: path.split('/').count(),
        };
        split.update();
        split
    }

    fn update(&mut self) {
        for (i, comp) in self.path.split('/').enumerate() {
            if i == self.index {
                self.comp = comp;
                return;
            }
        }
    }

    pub fn component(&self) -> &'a str {
        self.comp
    }

    pub fn level(&self) -> usize {
        self.index
    }

    pub fn move_prev(&mut self) -> bool {
        if self.index > 0 {
            self.index -= 1;
            self.update();
            return true;
        }
        false
    }

    pub fn move_next(&mut self) -> bool {
        if self.index < self.num - 1 {
            self.index += 1;
            self.update();
            return true;
        }
        false
    }
}

/// Iterator for structure items with a given path.
#[derive(Clone, Debug)]
pub struct PathStructItems<'a, 'b> {
    error: Option<Error>,
    iter: StructItems<'a>,
    path: PathSplit<'b>,
    level: usize,
}

impl<'a, 'b> PathStructItems<'a, 'b> {
    /// Advances the iterator and returns a next structure item for a given
    /// path (with a corresponding StructItems-iterator) or error.
    pub fn next_item(&mut self) -> Result<(StructItem<'a>, StructItems<'a>)> {
        if self.error != None {
            return Err(self.error.unwrap());
        }

        loop {
            let item = self.iter.next_item()?;
            match item {
                StructItem::BeginNode { .. } => {
                    if self.level == self.path.level()
                        && self.path.component() == item.node_name().unwrap()
                        && !self.path.move_next()
                    {
                        self.level += 1;
                        return Ok((item, self.iter.clone()));
                    }
                    self.level += 1;
                }
                StructItem::Property { name, .. } => {
                    if self.level == self.path.level() && self.path.component() == name {
                        return Ok((item, self.iter.clone()));
                    }
                }
                StructItem::EndNode {} => {
                    if self.level == self.path.level() && !self.path.move_prev() {
                        self.error = Some(Error::OutOfParentNode);
                        return Err(self.error.unwrap());
                    }
                    self.level -= 1;
                }
                _ => return Err(Error::BadStructToken),
            }
        }
    }
}

impl<'a, 'b> Iterator for PathStructItems<'a, 'b> {
    type Item = (StructItem<'a>, StructItems<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_item() {
            Ok(item) => Some(item),
            Err(_) => None,
        }
    }
}

impl<'a, 'b> FusedIterator for PathStructItems<'a, 'b> {}

/// DTB blob reader.
#[derive(Debug)]
pub struct Reader<'a> {
    reserved_mem: &'a [ReservedMemEntry],
    struct_block: &'a [u8],
    strings_block: &'a [u8],
}

impl<'a> Reader<'a> {
    #[allow(clippy::cast_ptr_alignment)]
    pub fn get_header(blob: &'a [u8]) -> Result<Header> {
        if blob.as_ptr() as usize % size_of::<u64>() != 0 {
            return Err(Error::UnalignedBlob);
        }

        if blob.len() < 4 {
            return Err(Error::BadMagic);
        }

        let be_header = blob.as_ptr() as *const Header;
        let be_magic = unsafe { (*be_header).magic };

        if u32::from_be(be_magic) != DTB_MAGIC {
            return Err(Error::BadMagic);
        }

        if blob.len() < size_of::<Header>() {
            return Err(Error::UnexpectedEndOfBlob);
        }

        let be_header = unsafe { &*be_header };

        Ok(Header {
            magic: DTB_MAGIC,
            total_size: u32::from_be(be_header.total_size),
            struct_offset: u32::from_be(be_header.struct_offset),
            strings_offset: u32::from_be(be_header.strings_offset),
            reserved_mem_offset: u32::from_be(be_header.reserved_mem_offset),
            version: u32::from_be(be_header.version),
            last_comp_version: u32::from_be(be_header.last_comp_version),
            bsp_cpu_id: u32::from_be(be_header.bsp_cpu_id),
            strings_size: u32::from_be(be_header.strings_size),
            struct_size: u32::from_be(be_header.struct_size),
        })
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn get_reserved_mem(blob: &'a [u8], header: &Header) -> Result<&'a [ReservedMemEntry]> {
        let entry_size = size_of::<ReservedMemEntry>();
        if header.struct_offset.checked_sub(entry_size as u32) < Some(header.reserved_mem_offset) {
            return Err(Error::OverlappingReservedMem);
        }

        if header.reserved_mem_offset % 8 != 0 {
            return Err(Error::UnalignedReservedMem);
        }

        let reserved_max_size = (header.struct_offset - header.reserved_mem_offset) as usize;
        let reserved = unsafe {
            // SAFETY: we checked this index during header parsing. It is also
            // properly aligned.
            let ptr =
                blob.as_ptr().add(header.reserved_mem_offset as usize) as *const ReservedMemEntry;
            from_raw_parts(ptr, reserved_max_size / entry_size)
        };

        let index = reserved
            .iter()
            .position(|ref e| e.address == 0 && e.size == 0);
        if index.is_none() {
            return Err(Error::NoZeroReservedMemEntry);
        }

        Ok(&reserved[..index.unwrap()])
    }

    fn get_struct_block(blob: &'a [u8], header: &Header) -> Result<&'a [u8]> {
        if header.struct_offset % 4 != 0 || header.struct_size % 4 != 0 {
            return Err(Error::UnalignedStruct);
        }

        if header.strings_offset.checked_sub(header.struct_size) < Some(header.struct_offset) {
            return Err(Error::OverlappingStruct);
        }

        let offset = header.struct_offset as usize;
        Ok(&blob[offset..offset + header.struct_size as usize])
    }

    pub fn get_strings_block(blob: &'a [u8], header: &Header) -> Result<&'a [u8]> {
        if header.total_size.checked_sub(header.strings_size) < Some(header.strings_offset) {
            return Err(Error::OverlappingStrings);
        }

        let offset = header.strings_offset as usize;
        Ok(&blob[offset..offset + header.strings_size as usize])
    }

    /// Reads a given DTB blob and returns a corresponding reader.
    pub fn read(blob: &'a [u8]) -> Result<Self> {
        let header = Reader::get_header(blob)?;

        if header.version < header.last_comp_version {
            return Err(Error::BadVersion);
        }

        if header.last_comp_version != COMP_VERSION {
            return Err(Error::UnsupportedCompVersion);
        }

        if header.total_size < header.struct_offset
            || header.total_size < header.strings_offset
            || header.total_size < header.reserved_mem_offset
        {
            return Err(Error::BadTotalSize);
        }

        if u32::try_from(blob.len()) != Ok(header.total_size) {
            return Err(Error::BadTotalSize);
        }

        Ok(Reader::<'a> {
            reserved_mem: Reader::get_reserved_mem(blob, &header)?,
            struct_block: Reader::get_struct_block(blob, &header)?,
            strings_block: Reader::get_strings_block(blob, &header)?,
        })
    }

    /// Reads DTB from a given address and returns a corresponding reader.
    pub unsafe fn read_from_address(addr: usize) -> Result<Self> {
        let blob = from_raw_parts(addr as *const u8, size_of::<Header>());
        let header = Reader::get_header(blob)?;

        let blob = core::slice::from_raw_parts(addr as *const u8, header.total_size as usize);
        Reader::read(blob)
    }

    /// Returns a reserved memory entry iterator.
    pub fn reserved_mem_entries(&self) -> ReservedMemEntries<'a> {
        ReservedMemEntries::<'a> {
            reserved_mem: self.reserved_mem,
            index: 0,
        }
    }

    /// Returns a structure item iterator.
    pub fn struct_items(&self) -> StructItems<'a> {
        StructItems::<'a> {
            struct_block: self.struct_block,
            strings_block: self.strings_block,
            offset: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    #[test]
    fn test_unaligned_blob() {
        let mut buf = Vec::new();
        let filename = Path::new(file!())
            .parent()
            .unwrap()
            .strip_prefix("rustBoot/")
            .unwrap()
            .join("test_dtb")
            .join("sample.dtb");
        let mut file = File::open(filename).unwrap();
        buf.push(0);
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(
            Reader::read(&buf.as_slice()[1..]).unwrap_err(),
            Error::UnalignedBlob
        );
    }

    fn read_dtb_vec(buf: &mut Vec<u8>, name: &str) {
        let path = Path::new(file!())
            .parent()
            .unwrap()
            .strip_prefix("rustBoot/")
            .unwrap()
            .join("test_dtb");
        let filename = path.join(String::from(name) + ".dtb");
        let mut file = File::open(filename).unwrap();
        buf.resize(0, 0);
        file.read_to_end(buf).unwrap();
    }

    fn read_dtb<'a>(buf: &'a mut Vec<u8>, name: &str) -> Result<Reader<'a>> {
        read_dtb_vec(buf, name);
        Reader::read(buf.as_slice())
    }

    macro_rules! test_read_dtb {
        ($fn_name:ident, $err:ident) => {
            #[test]
            fn $fn_name() {
                let mut buf = Vec::new();
                let reader = read_dtb(&mut buf, &stringify!($fn_name)[5..]);
                assert_eq!(reader.unwrap_err(), Error::$err);
            }
        };
    }

    test_read_dtb!(test_bad_magic, BadMagic);
    test_read_dtb!(test_unexpected_end_of_blob, UnexpectedEndOfBlob);
    test_read_dtb!(test_bad_version, BadVersion);
    test_read_dtb!(test_unsupported_comp_version, UnsupportedCompVersion);
    test_read_dtb!(test_bad_total_size, BadTotalSize);
    test_read_dtb!(test_unaligned_reserved_mem, UnalignedReservedMem);
    test_read_dtb!(test_overlapping_reserved_mem, OverlappingReservedMem);
    test_read_dtb!(test_no_zero_reserved_mem_entry, NoZeroReservedMemEntry);
    test_read_dtb!(test_unaligned_struct, UnalignedStruct);
    test_read_dtb!(test_unaligned_struct2, UnalignedStruct);
    test_read_dtb!(test_overlapping_struct, OverlappingStruct);
    test_read_dtb!(test_overlapping_strings, OverlappingStrings);

    #[test]
    fn test_reserved_mem() {
        let mut buf = Vec::new();
        let mut iter = read_dtb(&mut buf, "sample").unwrap().reserved_mem_entries();

        let entry = iter.next().unwrap();
        assert_eq!(entry.address, 0x12345);
        assert_eq!(entry.size, 0x23456);

        let entry = iter.next().unwrap();
        assert_eq!(entry.address, 0x34567);
        assert_eq!(entry.size, 0x45678);

        assert!(!iter.next().is_some());
    }

    fn assert_node<'a>(iter: &mut StructItems<'a>, name: &str) {
        let item = iter.next_item().unwrap();
        assert!(item.is_begin_node());
        assert_eq!(item.name().unwrap(), name);
    }

    fn assert_str_property<'a>(iter: &mut StructItems<'a>, name: &str, value: &str) {
        let item = iter.next_item().unwrap();
        assert!(item.is_property());
        assert_eq!(item.name().unwrap(), name);
        assert_eq!(item.value_str().unwrap(), value);
    }

    fn assert_str_list_property<'a>(iter: &mut StructItems<'a>, name: &str, value: &[&str]) {
        let item = iter.next_item().unwrap();
        assert!(item.is_property());
        assert_eq!(item.name().unwrap(), name);
        let mut buf = [0; size_of::<&str>() * 8];
        assert_eq!(item.value_str_list(&mut buf).unwrap(), value);
    }

    fn assert_u32_list_property<'a>(iter: &mut StructItems<'a>, name: &str, value: &[u32]) {
        let item = iter.next_item().unwrap();
        assert!(item.is_property());
        assert_eq!(item.name().unwrap(), name);
        let mut buf = [0; 4 * 8];
        assert_eq!(item.value_u32_list(&mut buf).unwrap(), value);
    }

    macro_rules! test_struct_items {
        ($fn_name:ident, $err:ident) => {
            #[test]
            fn $fn_name() {
                let mut buf = Vec::new();
                let reader = read_dtb(&mut buf, &stringify!($fn_name)[5..]);
                let mut iter = reader.unwrap().struct_items();
                let err = loop {
                    match iter.next_item() {
                        Ok(_) => continue,
                        Err(err) => break err,
                    }
                };
                assert_eq!(err, Error::$err);
            }
        };
    }

    test_struct_items!(test_unexpected_end_of_struct, UnexpectedEndOfStruct);
    test_struct_items!(test_bad_struct_token, BadStructToken);
    test_struct_items!(test_bad_node_name, BadNodeName);
    test_struct_items!(test_unexpected_end_of_struct2, UnexpectedEndOfStruct);
    test_struct_items!(test_unexpected_end_of_struct3, UnexpectedEndOfStruct);
    test_struct_items!(test_bad_property_name, BadPropertyName);

    macro_rules! test_bad_str_encoding {
        ($fn_name:ident) => {
            #[test]
            fn $fn_name() {
                let mut buf = Vec::new();
                let reader = read_dtb(&mut buf, &stringify!($fn_name)[5..]);
                let mut iter = reader.unwrap().struct_items();
                loop {
                    match iter.next_item() {
                        Ok(_) => continue,
                        Err(Error::BadStrEncoding(_)) => break,
                        Err(err) => {
                            assert!(false, "unexpected error: {:?}", err)
                        }
                    }
                }
            }
        };
    }

    test_bad_str_encoding!(test_bad_str_encoding);
    test_bad_str_encoding!(test_bad_str_encoding2);

    #[test]
    fn test_struct_items() {
        let mut buf = Vec::new();
        let mut iter = read_dtb(&mut buf, "sample").unwrap().struct_items();
        assert_node(&mut iter, "");
        assert_node(&mut iter, "node1");
        assert_str_property(&mut iter, "a-string-property", "A string");
        assert_str_list_property(
            &mut iter,
            "a-string-list-property",
            &["first string", "second string"],
        );
        assert_eq!(
            iter.next_item().unwrap(),
            StructItem::Property {
                name: "a-byte-data-property",
                value: &[0x01, 0x23, 0x34, 0x56],
            }
        );
        assert_node(&mut iter, "child-node1");
        assert_eq!(
            iter.next_item().unwrap(),
            StructItem::Property {
                name: "first-child-property",
                value: &[],
            }
        );
        assert_u32_list_property(&mut iter, "second-child-property", &[1]);
        assert_str_property(&mut iter, "a-string-property", "Hello, world");
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_node(&mut iter, "child-node2");
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_node(&mut iter, "node2");
        assert_eq!(
            iter.next_item().unwrap(),
            StructItem::Property {
                name: "an-empty-property",
                value: &[],
            }
        );
        assert_u32_list_property(&mut iter, "a-cell-property", &[1, 2, 3, 4]);
        assert_node(&mut iter, "child-node1");
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_eq!(iter.next_item().unwrap(), StructItem::EndNode);
        assert_eq!(iter.next_item().unwrap_err(), Error::NoMoreStructItems);
        assert_eq!(iter.next_item().unwrap_err(), Error::NoMoreStructItems);
    }

    #[test]
    fn test_out_of_parent_node() {
        let mut buf = Vec::new();
        let root = read_dtb(&mut buf, "out_of_parent_node")
            .unwrap()
            .struct_items();

        let mut iter = root.path_struct_items("/foo");
        assert_eq!(iter.next_item().unwrap_err(), Error::OutOfParentNode);
        assert_eq!(iter.next_item().unwrap_err(), Error::OutOfParentNode);
    }

    fn assert_not_found<'a>(iter: &StructItems<'a>, path: &str) {
        let mut iter = iter.path_struct_items(path);
        let err = iter.next_item().unwrap_err();
        assert!(err == Error::NoMoreStructItems || err == Error::OutOfParentNode);
    }

    fn assert_nodes_found<'a>(iter: &StructItems<'a>, path: &str, expected_names: &[&str]) {
        let mut index = 0;
        for (item, _) in iter.path_struct_items(path) {
            assert!(item.is_begin_node());
            assert_eq!(item.name().unwrap(), expected_names[index]);
            index += 1;
        }
        assert_eq!(index, expected_names.len());
    }

    fn assert_properties_found<'a>(iter: &StructItems<'a>, path: &str, expected_values: &[&str]) {
        let mut index = 0;
        for (item, _) in iter.path_struct_items(path) {
            assert_eq!(item.value_str().unwrap(), expected_values[index]);
            index += 1;
        }
        assert_eq!(index, expected_values.len());
    }

    #[test]
    fn test_path_struct_items() {
        let mut buf = Vec::new();
        let root = read_dtb(&mut buf, "sample2").unwrap().struct_items();

        assert_nodes_found(&root, "/", &[""]);
        assert_not_found(&root, "//");

        assert_not_found(&root, "foo");
        assert_nodes_found(&root, "/foo", &["foo@1", "foo@4"]);
        assert_nodes_found(&root, "/foo/foo", &["foo@2", "foo@3", "foo@5", "foo@6"]);

        assert_not_found(&root, "bar");
        assert_not_found(&root, "/bar");
        assert_properties_found(&root, "/foo/bar", &["1", "4"]);
        assert_properties_found(&root, "/foo/foo/bar", &["2", "3", "5", "6"]);

        let (_, iter) = root.path_struct_items("/").next().unwrap();
        assert_not_found(&iter, "/foo");
        assert_nodes_found(&iter, "foo", &["foo@1", "foo@4"]);

        let mut iter = root.path_struct_items("/foo");
        let (_, iter2) = iter.next_item().unwrap();
        assert_not_found(&iter2, "/foo");
        assert_nodes_found(&iter2, "foo", &["foo@2", "foo@3"]);
        assert_properties_found(&iter2, "foo/bar", &["2", "3"]);

        let (_, iter2) = iter.next_item().unwrap();
        assert_not_found(&iter2, "/foo");
        assert_nodes_found(&iter2, "foo", &["foo@5", "foo@6"]);
        assert_properties_found(&iter2, "foo/bar", &["5", "6"]);

        assert_eq!(iter.next_item().unwrap_err(), Error::NoMoreStructItems);

        assert_properties_found(&root, "/foo/foo/bar", &["2", "3", "5", "6"]);
    }

    // Regression test for a prior unsafety issue: #5
    test_read_dtb!(test_bad_reserved_mem_offset, BadTotalSize);

    #[test]
    fn test_read_from_address() {
        let mut buf = Vec::new();
        read_dtb_vec(&mut buf, "sample2");

        unsafe {
            let root = Reader::read_from_address(buf.as_ptr() as usize)
                .unwrap()
                .struct_items();

            // Let's skip thorough testing for now, since the function shares
            // the same implementation.
            assert_nodes_found(&root, "/foo", &["foo@1", "foo@4"]);
        }
    }
}
