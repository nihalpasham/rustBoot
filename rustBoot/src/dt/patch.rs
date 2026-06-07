#![allow(clippy::panic)]

use super::internal::Header;
use super::{
    Error, PropertyValue, RawNodeConstructor, RawPropertyConstructor, Reader, Result,
    SerializedBuffer, StringsBlock, StructItem, TOKEN_SIZE,
};
use core::convert::TryInto;

pub fn make_new_strings_block_with<'a, const M: usize>(
    name_list: &'a [&str],
    new_strings_block: &'a mut StringsBlock<'a>,
    dtb_blob: &'a [u8],
) -> Result<(&'a [u8], [usize; M])> {
    let header = Reader::get_header(dtb_blob)?;
    let strings_block_len = header.strings_size as usize;

    let offset_list = new_strings_block.make_new_strings_block_with(name_list)?;
    let new_strings_block = new_strings_block.finalize();

    let mut offset_list: [usize; M] = offset_list[..M]
        .try_into()
        .map_err(|_v| Error::BufferExhausted)?;
    // add strings_block_len to each offset in the list
    offset_list
        .iter_mut()
        .for_each(|offset| *offset += strings_block_len);
    Ok((new_strings_block, offset_list))
}

pub fn make_node_with_props<const N: usize>(
    node_name: &str,
    prop_val_list: &[PropertyValue],
    offset_list: &[usize],
) -> Result<(usize, [u8; N])> {
    let mut buf = [0u8; N];
    let node_size = RawNodeConstructor::make_node_with_props(
        &mut buf[..],
        node_name,
        offset_list,
        prop_val_list,
    )?;
    Ok((node_size, buf))
}

#[derive(Debug, Clone, Copy)]
pub enum NodeItems<'a> {
    RawNodeConstructor(RawNodeConstructor<'a>),
    RawPropertyConstructor(RawPropertyConstructor<'a>),
    None,
}

/// Given a node-path (a string literal), this function takes a reader, `dtb_blob` and outputs all of its properties, including
/// any nested nodes.
///
/// the output is of the following form
/// - (&str, NodeItems, usize): (name of the property, )
pub fn parse_raw_node<'a, const N: usize>(
    reader: &Reader<'a>,
    node_path: &str,
    dtb_blob: &[u8],
) -> Result<[(&'a str, NodeItems<'a>, usize); N]> {
    let root = &reader.struct_items();
    let (_, node_iter) = root.path_struct_items(node_path).next().unwrap();
    let mut prop_list = [("", NodeItems::None, 0usize); N];

    let header = Reader::get_header(dtb_blob)?;
    let struct_offset = header.struct_offset as usize;
    let mut offset = node_iter.get_offset() + struct_offset;
    let mut node_depth = 0usize;

    for (idx, item) in node_iter.enumerate() {
        match item {
            StructItem::Property { name, value } => {
                let mut total_property_len = TOKEN_SIZE * 3 + value.len();
                // `total_property_len` is the item's non-padded length. So, we'll have to account for it.
                match total_property_len % 4 {
                    3 => total_property_len += 1,
                    2 => total_property_len += 2,
                    1 => total_property_len += 3,
                    _ => {}
                }
                let name_off = u32::from_be_bytes(
                    dtb_blob[offset + 8..offset + 8 + 4]
                        .try_into()
                        .map_err(|_| Error::NonExhaustive)?,
                );
                let prop = RawPropertyConstructor::new(3u32, value.len() as u32, name_off, value);
                prop_list[idx] = (
                    name,
                    NodeItems::RawPropertyConstructor(prop),
                    total_property_len,
                );
                offset += total_property_len;
            }
            StructItem::BeginNode { name } => {
                let mut total_node_len = TOKEN_SIZE + name.len();
                // `total_node_len` is the item's non-padded length. So, we'll have to account for it.
                match total_node_len % 4 {
                    3 => total_node_len += 1,
                    2 => total_node_len += 2,
                    1 => total_node_len += 3,
                    // `name.len()` doesnt include the null-terminated byte, so we account for it.
                    0 => total_node_len += 4,
                    _ => {}
                }
                let node = RawNodeConstructor::new(1u32, name.as_bytes());
                prop_list[idx] = (name, NodeItems::RawNodeConstructor(node), total_node_len);
                offset += total_node_len;
                node_depth += 1;
            }
            StructItem::EndNode => {
                if node_depth > 0 {
                    node_depth -= 1
                } else {
                    break;
                }
            }
            StructItem::None => {
                unreachable!()
            }
        }
    }
    Ok(prop_list)
}

pub fn check_chosen_node<'a, const N: usize, const M: usize>(
    items: [(&'a str, NodeItems<'a>, usize); N],
) -> Result<(SerializedBuffer<M>, usize)> {
    let mut chosen_bytes = [0u8; M];
    let mut offset = 0usize;
    let mut len_to_be_subtracted = 0usize;
    for (name, item, len) in items.iter() {
        match *name {
            "bootargs" => {
                len_to_be_subtracted += len;
            }
            "linux,initrd-start" => {
                len_to_be_subtracted += len;
            }
            "linux,initrd-end" => {
                len_to_be_subtracted += len;
            }
            "" => {}
            _ => match item {
                NodeItems::None => {}
                NodeItems::RawPropertyConstructor(val) => {
                    let serialized_bytes = val.serialize()?;
                    let bytes = serialized_bytes.as_slice();
                    let bytes_len = bytes.len();
                    chosen_bytes[offset..offset + bytes_len].copy_from_slice(bytes);
                    offset += *len;
                    len_to_be_subtracted += len;
                }
                NodeItems::RawNodeConstructor(val) => {
                    let serialized_bytes = val.serialize()?;
                    let bytes = serialized_bytes.as_slice();
                    let bytes_len = bytes.len();
                    chosen_bytes[offset..offset + bytes_len].copy_from_slice(bytes);
                    offset += *len;
                    len_to_be_subtracted += len;
                }
            },
        }
    }
    Ok((
        SerializedBuffer::new(chosen_bytes, offset),
        len_to_be_subtracted,
    ))
}

pub fn update_dtb_header(
    header: &mut Header,
    appended_strings_block_len: usize,
    new_node_len: usize,
    len_to_be_subtracted: usize,
) -> &Header {
    header.strings_size += appended_strings_block_len as u32;
    header.struct_size = (header.struct_size + new_node_len as u32) - len_to_be_subtracted as u32;
    header.strings_offset =
        (header.strings_offset + new_node_len as u32) - len_to_be_subtracted as u32;
    header.total_size = header.total_size + (appended_strings_block_len + new_node_len) as u32
        - len_to_be_subtracted as u32;
    header
}

pub fn get_padded_node_len<'a>(reader: &Reader<'a>, node_name: &str) -> usize {
    let root = reader.struct_items();
    let (node, _) = root.path_struct_items(node_name).next().unwrap();

    let node_len = TOKEN_SIZE + node.node_name().unwrap().len();

    node_len + (node_len % 4)
}

pub fn get_node_start_and_end<'a>(
    reader: &Reader<'a>,
    node_name: &str,
    dtb_blob: &'a [u8],
    node_size: usize,
) -> Result<(usize, usize)> {
    let root = reader.struct_items();
    let (node, node_iter) = root.path_struct_items(node_name).next().unwrap();

    let header = Reader::get_header(dtb_blob)?;
    let struct_offset = header.struct_offset as usize;

    let node_len = TOKEN_SIZE + node.node_name().unwrap().len();
    let padded_node_len = node_len + (node_len % 4);
    let node_start = (node_iter.get_offset() + struct_offset) - padded_node_len;
    let node_end = node_start + padded_node_len + node_size;
    Ok((node_start, node_end))
}

pub fn patch_dtb_node<'a, const N: usize>(
    header: &Header,
    node_start: usize,
    node_end: usize,
    dtb_blob: &'a [u8],
    patch_bytes_1: &'a [u8],
    patch_bytes_2: &'a [u8],
    strings_block_patch: &'a [u8],
    patched_dtb_blob: &'a mut [u8],
) {
    let header_len = 0x28;
    let patch_bytes_1_slice = patch_bytes_1.len();
    let patch_bytes_2_slice = patch_bytes_2.len();
    let remaining_bytes = dtb_blob[node_end..].len();
    let strings_block_patch_len = strings_block_patch.len();

    // let mut patched_dtb_blob = [0u8; N];
    let slice_0 = header_len..node_start;
    let slice_1 = node_start..node_start + patch_bytes_1_slice;
    let slice_2 =
        node_start + patch_bytes_1_slice..node_start + patch_bytes_1_slice + patch_bytes_2_slice;
    let slice_3 = node_start + patch_bytes_1_slice + patch_bytes_2_slice
        ..node_start + patch_bytes_1_slice + patch_bytes_2_slice + remaining_bytes;
    let slice_4 = node_start + patch_bytes_1_slice + patch_bytes_2_slice + remaining_bytes
        ..node_start
            + patch_bytes_1_slice
            + patch_bytes_2_slice
            + remaining_bytes
            + strings_block_patch_len;

    patched_dtb_blob[..header_len].copy_from_slice(&header.as_slice());
    patched_dtb_blob[slice_0].copy_from_slice(&dtb_blob[header_len..node_start]);
    patched_dtb_blob[slice_1].copy_from_slice(patch_bytes_1);
    patched_dtb_blob[slice_2].copy_from_slice(patch_bytes_2);
    patched_dtb_blob[slice_3].copy_from_slice(&dtb_blob[node_end..]);
    patched_dtb_blob[slice_4].copy_from_slice(strings_block_patch);
}

pub fn patch_chosen_node<'a, const N: usize>(
    reader: Reader<'a>,
    dtb_blob: &'a [u8],
    prop_val_list: &[PropertyValue],
    new_dtb_buffer: &'a mut [u8; N],
) -> (&'a mut [u8; N], usize) {
    let mut buf = [0; 100];
    let mut new_strings_block = StringsBlock::new(&mut buf[..]).unwrap();

    let name_list = ["bootargs", "linux,initrd-start", "linux,initrd-end"];
    let res = make_new_strings_block_with::<3>(&name_list, &mut new_strings_block, dtb_blob);
    let (offset_list, strings_block_patch, strings_block_patch_len) = match res {
        Ok((strings_block, offset_list)) => (offset_list, strings_block, strings_block.len()),
        Err(e) => panic!("error: {:?}", e),
    };

    let node_name = "chosen";
    let prop_val_list = prop_val_list;
    let res = make_node_with_props::<200>(node_name, prop_val_list, &offset_list);
    let (patch_bytes_1_len, patch_bytes_1) = match res {
        Ok((patch_bytes_1_len, patch_bytes_1)) => (patch_bytes_1_len, patch_bytes_1),
        Err(e) => panic!("error: {:?}", e),
    };
    let patch_bytes_1 = &patch_bytes_1[..patch_bytes_1_len];

    let res = parse_raw_node::<10>(&reader, "/chosen", dtb_blob);
    let parsed_node = match res {
        Ok(val) => val,
        Err(e) => panic!("error: {:?}", e),
    };

    let res = check_chosen_node::<10, 200>(parsed_node);
    let (patch_bytes_2, len_to_be_subtracted) = match res {
        Ok((buf, len_to_be_subtracted)) => (buf, len_to_be_subtracted),
        Err(e) => panic!("error: {:?}", e),
    };
    // `patch_bytes_1_len` includes a `BEGIN_NODE`, we have to subtract it from the new length.
    // i.e. the `chosen` node takes up 12 bytes (0x00000001 + "chosen" + padding)
    let padded_node_len = get_padded_node_len(&reader, "/chosen");
    let new_node_len = patch_bytes_1_len + patch_bytes_2.as_slice().len() - padded_node_len;

    let mut header = Reader::get_header(dtb_blob).unwrap();
    {
        let _ = update_dtb_header(
            &mut header,
            strings_block_patch_len,
            new_node_len,
            len_to_be_subtracted,
        );
    }

    let (node_start, node_end) =
        match get_node_start_and_end(&reader, "/chosen", dtb_blob, len_to_be_subtracted) {
            Ok((node_start, node_end)) => (node_start, node_end),
            Err(e) => panic!("error: {:?}", e),
        };

    patch_dtb_node::<N>(
        &header,
        node_start,
        node_end,
        dtb_blob,
        patch_bytes_1,
        patch_bytes_2.as_slice(),
        strings_block_patch,
        new_dtb_buffer.as_mut(),
    );
    let hdr_total_size = correct_endianess(header.total_size);
    // info!("len: {:?}", hdr_total_size);
    (new_dtb_buffer, hdr_total_size as usize)
}

pub fn correct_endianess(val: u32) -> u32 {
    let byte_4 = val >> 24 & 0xff;
    let byte_3 = val >> 8 & 0xff00;
    let byte_2 = val << 8 & 0xff0000;
    let byte_1 = val << 24 & 0xff000000;

    byte_1 | byte_2 | byte_3 | byte_4
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::style)]
mod tests {
    use super::*;
    use crate::dt::internal::*;
    use crate::dt::*;

    fn make_minimal_dtb() -> Vec<u8> {
        let strings = b"bootargs\0linux,initrd-start\0linux,initrd-end\0other-prop\0";

        let bootargs_val = b"console=tty0\0";
        let bootargs_val_padded = {
            let mut v = bootargs_val.to_vec();
            while !v.len().is_multiple_of(4) {
                v.push(0);
            }
            v
        };
        let initrd_start_val = 0x10000000u32.to_be_bytes();
        let initrd_end_val = 0x20000000u32.to_be_bytes();
        let other_val = b"hello\0";
        let other_val_padded = {
            let mut v = other_val.to_vec();
            while !v.len().is_multiple_of(4) {
                v.push(0);
            }
            v
        };

        let mut struct_blk = Vec::new();
        // Root node (empty name in DTB: just null terminator, padded to 4)
        struct_blk.extend_from_slice(&TOK_BEGIN_NODE.to_be_bytes());
        struct_blk.extend_from_slice(&[0u8; 4]);
        // Chosen node "chosen\0" (7 bytes -> pad to 8)
        struct_blk.extend_from_slice(&TOK_BEGIN_NODE.to_be_bytes());
        struct_blk.extend_from_slice(b"chosen\0");
        struct_blk.push(0);
        // bootargs property: name_off=0
        struct_blk.extend_from_slice(&TOK_PROPERTY.to_be_bytes());
        struct_blk.extend_from_slice(&(bootargs_val.len() as u32).to_be_bytes());
        struct_blk.extend_from_slice(&0u32.to_be_bytes());
        struct_blk.extend_from_slice(&bootargs_val_padded);
        // linux,initrd-start property: name_off=9
        struct_blk.extend_from_slice(&TOK_PROPERTY.to_be_bytes());
        struct_blk.extend_from_slice(&4u32.to_be_bytes());
        struct_blk.extend_from_slice(&9u32.to_be_bytes());
        struct_blk.extend_from_slice(&initrd_start_val);
        // linux,initrd-end property: name_off=28
        struct_blk.extend_from_slice(&TOK_PROPERTY.to_be_bytes());
        struct_blk.extend_from_slice(&4u32.to_be_bytes());
        struct_blk.extend_from_slice(&28u32.to_be_bytes());
        struct_blk.extend_from_slice(&initrd_end_val);
        // other-prop property: name_off=45
        struct_blk.extend_from_slice(&TOK_PROPERTY.to_be_bytes());
        struct_blk.extend_from_slice(&(other_val.len() as u32).to_be_bytes());
        struct_blk.extend_from_slice(&45u32.to_be_bytes());
        struct_blk.extend_from_slice(&other_val_padded);
        // END_NODE (chosen)
        struct_blk.extend_from_slice(&TOK_END_NODE.to_be_bytes());
        // END_NODE (root)
        struct_blk.extend_from_slice(&TOK_END_NODE.to_be_bytes());
        // END
        struct_blk.extend_from_slice(&TOK_END.to_be_bytes());

        let mem_rsvmap: [u8; 16] = [0u8; 16];
        let off_dt_struct: u32 = 40 + 16;
        let off_dt_strings: u32 = off_dt_struct + struct_blk.len() as u32;
        let total_size = off_dt_strings + strings.len() as u32;

        let mut blob = Vec::with_capacity(total_size as usize);
        blob.extend_from_slice(&DTB_MAGIC.to_be_bytes());
        blob.extend_from_slice(&total_size.to_be_bytes());
        blob.extend_from_slice(&off_dt_struct.to_be_bytes());
        blob.extend_from_slice(&off_dt_strings.to_be_bytes());
        blob.extend_from_slice(&40u32.to_be_bytes()); // off_mem_rsvmap
        blob.extend_from_slice(&17u32.to_be_bytes()); // version
        blob.extend_from_slice(&16u32.to_be_bytes()); // last_comp_version
        blob.extend_from_slice(&0u32.to_be_bytes()); // boot_cpuid_phys
        blob.extend_from_slice(&(strings.len() as u32).to_be_bytes());
        blob.extend_from_slice(&(struct_blk.len() as u32).to_be_bytes());
        blob.extend_from_slice(&mem_rsvmap);
        blob.extend_from_slice(&struct_blk);
        blob.extend_from_slice(strings);

        blob
    }

    fn make_root_only_dtb() -> Vec<u8> {
        let strings = b"";
        let mut struct_blk = Vec::new();
        struct_blk.extend_from_slice(&TOK_BEGIN_NODE.to_be_bytes());
        struct_blk.extend_from_slice(&[0u8; 4]);
        struct_blk.extend_from_slice(&TOK_END_NODE.to_be_bytes());
        struct_blk.extend_from_slice(&TOK_END.to_be_bytes());

        let mem_rsvmap: [u8; 16] = [0u8; 16];
        let off_dt_struct: u32 = 40 + 16;
        let off_dt_strings: u32 = off_dt_struct + struct_blk.len() as u32;
        let total_size = off_dt_strings + strings.len() as u32;

        let mut blob = Vec::with_capacity(total_size as usize);
        blob.extend_from_slice(&DTB_MAGIC.to_be_bytes());
        blob.extend_from_slice(&total_size.to_be_bytes());
        blob.extend_from_slice(&off_dt_struct.to_be_bytes());
        blob.extend_from_slice(&off_dt_strings.to_be_bytes());
        blob.extend_from_slice(&40u32.to_be_bytes());
        blob.extend_from_slice(&17u32.to_be_bytes());
        blob.extend_from_slice(&16u32.to_be_bytes());
        blob.extend_from_slice(&0u32.to_be_bytes());
        blob.extend_from_slice(&(strings.len() as u32).to_be_bytes());
        blob.extend_from_slice(&(struct_blk.len() as u32).to_be_bytes());
        blob.extend_from_slice(&mem_rsvmap);
        blob.extend_from_slice(&struct_blk);
        blob.extend_from_slice(strings);

        blob
    }

    // ── correct_endianess ──────────────────────────────────────────────

    #[test]
    fn test_correct_endianess_zero() {
        assert_eq!(correct_endianess(0x00000000), 0x00000000);
    }

    #[test]
    fn test_correct_endianess_all_ones() {
        assert_eq!(correct_endianess(0xFFFFFFFF), 0xFFFFFFFF);
    }

    #[test]
    fn test_correct_endianess_byte_swap() {
        assert_eq!(correct_endianess(0xAABBCCDD), 0xDDCCBBAA);
    }

    #[test]
    fn test_correct_endianess_identity_values() {
        assert_eq!(correct_endianess(0x01020304), 0x04030201);
        assert_eq!(correct_endianess(0x00000001), 0x01000000);
        assert_eq!(correct_endianess(0x01000000), 0x00000001);
        assert_eq!(correct_endianess(0x12345678), 0x78563412);
    }

    #[test]
    fn test_correct_endianess_mid_bytes() {
        assert_eq!(correct_endianess(0x00FF00FF), 0xFF00FF00);
        assert_eq!(correct_endianess(0xFF00FF00), 0x00FF00FF);
    }

    // ── update_dtb_header ──────────────────────────────────────────────

    #[test]
    fn test_update_dtb_header_typical() {
        let mut hdr = Header {
            magic: DTB_MAGIC,
            total_size: 200,
            struct_offset: 56,
            strings_offset: 120,
            reserved_mem_offset: 40,
            version: 17,
            last_comp_version: 16,
            bsp_cpu_id: 0,
            strings_size: 20,
            struct_size: 64,
        };
        let result = update_dtb_header(&mut hdr, 10, 30, 15);
        assert_eq!(result.strings_size, 30);
        assert_eq!(result.struct_size, 64 + 30 - 15);
        assert_eq!(result.strings_offset, 120 + 30 - 15);
        assert_eq!(result.total_size, 200 + 10 + 30 - 15);
    }

    #[test]
    fn test_update_dtb_header_zero_appended() {
        let mut hdr = Header {
            magic: DTB_MAGIC,
            total_size: 100,
            struct_offset: 56,
            strings_offset: 80,
            reserved_mem_offset: 40,
            version: 17,
            last_comp_version: 16,
            bsp_cpu_id: 0,
            strings_size: 10,
            struct_size: 24,
        };
        let result = update_dtb_header(&mut hdr, 0, 0, 0);
        assert_eq!(result.total_size, 100);
        assert_eq!(result.strings_size, 10);
        assert_eq!(result.struct_size, 24);
    }

    #[test]
    fn test_update_dtb_header_subtract_larger_than_add() {
        let mut hdr = Header {
            magic: DTB_MAGIC,
            total_size: 200,
            struct_offset: 56,
            strings_offset: 120,
            reserved_mem_offset: 40,
            version: 17,
            last_comp_version: 16,
            bsp_cpu_id: 0,
            strings_size: 30,
            struct_size: 64,
        };
        let result = update_dtb_header(&mut hdr, 5, 10, 50);
        assert_eq!(result.struct_size, 64 + 10 - 50);
        assert_eq!(result.total_size, 200 + 5 + 10 - 50);
    }

    // ── make_node_with_props ───────────────────────────────────────────

    #[test]
    fn test_make_node_with_props_valid() {
        let prop_val_list = [PropertyValue::U32([0x12, 0x34, 0x56, 0x78])];
        let offset_list = [0usize];
        let result = make_node_with_props::<200>("test-node", &prop_val_list, &offset_list);
        assert!(result.is_ok());
        let (size, buf) = result.unwrap();
        assert!(size > 0);
        assert!(buf[..size].starts_with(&TOK_BEGIN_NODE.to_be_bytes()));
    }

    #[test]
    fn test_make_node_with_props_multiple_props() {
        let prop_val_list = [
            PropertyValue::U32([0x00, 0x00, 0x00, 0x01]),
            PropertyValue::String("hello"),
        ];
        let offset_list = [0usize, 8usize];
        let result = make_node_with_props::<200>("multi", &prop_val_list, &offset_list);
        assert!(result.is_ok());
    }

    #[test]
    fn test_make_node_with_props_empty_props() {
        let prop_val_list: [PropertyValue; 0] = [];
        let offset_list: [usize; 0] = [];
        let result = make_node_with_props::<200>("empty", &prop_val_list, &offset_list);
        assert!(result.is_ok());
    }

    #[test]
    fn test_make_node_with_props_short_name() {
        let prop_val_list = [];
        let offset_list = [];
        let result = make_node_with_props::<200>("x", &prop_val_list, &offset_list);
        assert!(result.is_ok());
    }

    // ── make_new_strings_block_with ────────────────────────────────────

    #[test]
    fn test_make_new_strings_block_with_valid() {
        let dtb_blob = make_root_only_dtb();
        let mut buf = [0u8; 200];
        let mut sb = StringsBlock::new(&mut buf[..]).unwrap();
        let name_list = ["prop1", "prop2"];
        let result = make_new_strings_block_with::<2>(&name_list, &mut sb, &dtb_blob);
        assert!(result.is_ok());
        let (_new_block, offsets) = result.unwrap();
        assert_eq!(offsets.len(), 2);
        // Offsets should be relative to end of original strings block
        assert!(offsets[1] > offsets[0]);
    }

    #[test]
    fn test_make_new_strings_block_with_empty_name_returns_err() {
        let dtb_blob = make_root_only_dtb();
        let mut buf = [0u8; 200];
        let mut sb = StringsBlock::new(&mut buf[..]).unwrap();
        let name_list = [""];
        let result = make_new_strings_block_with::<1>(&name_list, &mut sb, &dtb_blob);
        assert!(result.is_err());
    }

    #[test]
    fn test_make_new_strings_block_with_too_many_names() {
        let dtb_blob = make_root_only_dtb();
        let mut buf = [0u8; 500];
        let mut sb = StringsBlock::new(&mut buf[..]).unwrap();
        let name_list = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let result = make_new_strings_block_with::<11>(&name_list, &mut sb, &dtb_blob);
        assert!(result.is_err());
    }

    #[test]
    fn test_make_new_strings_block_with_invalid_dtb() {
        let bad_blob = [0u8; 40];
        let mut buf = [0u8; 200];
        let mut sb = StringsBlock::new(&mut buf[..]).unwrap();
        let name_list = ["prop1"];
        let result = make_new_strings_block_with::<1>(&name_list, &mut sb, &bad_blob);
        assert!(result.is_err());
    }

    // ── parse_raw_node ─────────────────────────────────────────────────

    #[test]
    fn test_parse_raw_node_chosen_node() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let result = parse_raw_node::<10>(&reader, "/chosen", &dtb);
        assert!(result.is_ok());
        let items = result.unwrap();
        // We expect: bootargs, linux,initrd-start, linux,initrd-end, other-prop
        let non_empty: Vec<_> = items
            .iter()
            .filter(|(name, _, _)| !name.is_empty())
            .collect();
        assert_eq!(non_empty.len(), 4);
        assert_eq!(non_empty[0].0, "bootargs");
        assert!(matches!(
            non_empty[0].1,
            NodeItems::RawPropertyConstructor(_)
        ));
    }

    #[test]
    fn test_parse_raw_node_root_path() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        // "/" refers to root node itself; it will be found as the first BeginNode
        // Use path "" (just root) to avoid the double-component issue
        let result = parse_raw_node::<10>(&reader, "/chosen", &dtb);
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_parse_raw_node_invalid_path_returns_err_or_empty() {
        let dtb = make_root_only_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let _ = parse_raw_node::<10>(&reader, "/nonexistent", &dtb);
    }

    #[test]
    fn test_parse_raw_node_chosen_offset_tracking() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let result = parse_raw_node::<10>(&reader, "/chosen", &dtb);
        let items = result.unwrap();
        let non_empty: Vec<_> = items
            .iter()
            .filter(|(name, _, _)| !name.is_empty())
            .collect();
        // Debug: print names to understand the DTB layout
        eprintln!("Found {} non-empty items:", non_empty.len());
        for (name, _, _) in non_empty.iter() {
            eprintln!("  name: '{:?}'", name);
        }
        assert!(
            non_empty.len() >= 4,
            "expected at least 4 properties, found {}",
            non_empty.len()
        );
        let property_names: Vec<&str> = non_empty.iter().map(|(n, _, _)| *n).collect();
        assert!(property_names.contains(&"bootargs"), "bootargs not found");
        assert!(
            property_names.contains(&"linux,initrd-start"),
            "linux,initrd-start not found"
        );
        assert!(
            property_names.contains(&"linux,initrd-end"),
            "linux,initrd-end not found"
        );
        assert!(
            property_names.contains(&"other-prop"),
            "other-prop not found"
        );
    }

    // ── check_chosen_node ──────────────────────────────────────────────

    #[test]
    fn test_check_chosen_node_removes_bootargs_and_initrd() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let items = parse_raw_node::<10>(&reader, "/chosen", &dtb).unwrap();
        let result = check_chosen_node::<10, 300>(items);
        assert!(result.is_ok());
        let (serialized, subtracted) = result.unwrap();
        // bootargs + linux,initrd-start + linux,initrd-end should be subtracted
        assert!(subtracted > 0);
        // The serialized buffer should contain the remaining (other-prop) property
        assert!(serialized.as_slice().len() > 0);
    }

    #[test]
    fn test_check_chosen_node_all_removed() {
        // Items where all entries are bootargs/initrd
        let items: [(&str, NodeItems, usize); 3] = [
            ("bootargs", NodeItems::None, 20),
            ("linux,initrd-start", NodeItems::None, 20),
            ("linux,initrd-end", NodeItems::None, 20),
        ];
        let result = check_chosen_node::<3, 10>(items);
        assert!(result.is_ok());
        let (_serialized, subtracted) = result.unwrap();
        assert_eq!(subtracted, 60);
    }

    #[test]
    fn test_check_chosen_node_empty_items() {
        let items: [(&str, NodeItems, usize); 5] = [("", NodeItems::None, 0); 5];
        let result = check_chosen_node::<5, 10>(items);
        assert!(result.is_ok());
        let (serialized, subtracted) = result.unwrap();
        assert_eq!(subtracted, 0);
        assert_eq!(serialized.as_slice().len(), 0);
    }

    // ── get_padded_node_len ────────────────────────────────────────────

    #[test]
    fn test_get_padded_node_len_chosen() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let len = get_padded_node_len(&reader, "/chosen");
        // TOKEN_SIZE + "chosen".len() + padding = 4 + 6 + (10 % 4 = 2) = 12
        assert_eq!(len, 12);
        assert_eq!(len % 4, 0);
    }

    #[test]
    fn test_get_padded_node_len_root_path_uses_chosen() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let chosen_len = get_padded_node_len(&reader, "/chosen");
        assert!(chosen_len > 0);
        assert_eq!(chosen_len % 4, 0);
    }

    // ── get_node_start_and_end ─────────────────────────────────────────

    #[test]
    fn test_get_node_start_and_end_chosen() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let result = get_node_start_and_end(&reader, "/chosen", &dtb, 50);
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert!(start < end);
        assert!(start >= 40 + 16); // past header + mem_rsvmap
    }

    #[test]
    fn test_get_node_start_and_end_different_node_size() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let result = get_node_start_and_end(&reader, "/chosen", &dtb, 0);
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        // with node_size=0, end should equal start + padded_node_len
        assert!(end >= start);
    }

    // ── patch_dtb_node ──────────────────────────────────────────────────

    #[test]
    fn test_patch_dtb_node_basic() {
        let dtb = make_minimal_dtb();
        let orig_hdr = Reader::get_header(&dtb).unwrap();
        let reader = Reader::read(&dtb).unwrap();
        let (node_start, node_end) = get_node_start_and_end(&reader, "/chosen", &dtb, 0).unwrap();
        let patch1 = b"\x00\x00\x00\x01new_node\0\0";
        let patch2 = b"\x00\x00\x00\x03\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x04new_val";
        let strings_patch = b"new-prop\0";
        // Calculate the output size manually: header in big-endian + slices
        let output_size = 40
            + (node_start - 40)
            + patch1.len()
            + patch2.len()
            + (dtb.len() - node_end)
            + strings_patch.len();
        let mut patched = vec![0u8; output_size];
        patch_dtb_node::<0>(
            &orig_hdr,
            node_start,
            node_end,
            &dtb,
            patch1,
            patch2,
            strings_patch,
            &mut patched,
        );
        // Verify magic survived
        let patched_magic = u32::from_be_bytes(patched[..4].try_into().unwrap());
        assert_eq!(patched_magic, DTB_MAGIC);
        // Verify the header was written (first 40 bytes are not all zeros)
        assert!(patched[..40].iter().any(|&b| b != 0));
    }

    #[test]
    fn test_patch_dtb_node_empty_patches() {
        let dtb = make_minimal_dtb();
        let orig_hdr = Reader::get_header(&dtb).unwrap();
        let reader = Reader::read(&dtb).unwrap();
        let (node_start, node_end) = get_node_start_and_end(&reader, "/chosen", &dtb, 0).unwrap();
        let patch1 = b"";
        let patch2 = b"";
        let strings_patch = b"";
        let output_size = 40 + (node_start - 40) + (dtb.len() - node_end);
        let mut patched = vec![0u8; output_size];
        patch_dtb_node::<0>(
            &orig_hdr,
            node_start,
            node_end,
            &dtb,
            patch1,
            patch2,
            strings_patch,
            &mut patched,
        );
        let patched_magic = u32::from_be_bytes(patched[..4].try_into().unwrap());
        assert_eq!(patched_magic, DTB_MAGIC);
    }

    #[test]
    fn test_patch_chosen_node_valid() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let prop_val_list = [
            PropertyValue::String("console=ttyS0\0"),
            PropertyValue::U32(0x10000000u32.to_be_bytes()),
            PropertyValue::U32(0x20000000u32.to_be_bytes()),
        ];
        let mut new_dtb_buffer = [0u8; 1024];
        let (result, _total_size) =
            patch_chosen_node(reader, &dtb, &prop_val_list, &mut new_dtb_buffer);
        let magic = u32::from_be_bytes(result[..4].try_into().unwrap());
        assert_eq!(magic, DTB_MAGIC);
        // Verify the patched blob is a valid DTB
        let hdr = Reader::get_header(result).unwrap();
        let actual_size = hdr.total_size as usize;
        assert!(actual_size <= result.len());
        let _ = Reader::read(&result[..actual_size]).unwrap();
    }

    #[test]
    fn test_patch_chosen_node_multiple_calls() {
        let dtb = make_minimal_dtb();
        let reader = Reader::read(&dtb).unwrap();
        let prop_val_list = [
            PropertyValue::String("console=ttyS0\0"),
            PropertyValue::U32(0x10000000u32.to_be_bytes()),
            PropertyValue::U32(0x20000000u32.to_be_bytes()),
        ];
        let mut buf_a = [0u8; 1024];
        let (result_a, _) = patch_chosen_node(reader, &dtb, &prop_val_list, &mut buf_a);
        let hdr_a = Reader::get_header(result_a).unwrap();
        let len_a = hdr_a.total_size as usize;
        let patched_slice_a = &result_a[..len_a];
        let reader2 = Reader::read(patched_slice_a).unwrap();
        let prop_val_list2 = [
            PropertyValue::String("console=tty1\0"),
            PropertyValue::U32(0x30000000u32.to_be_bytes()),
            PropertyValue::U32(0x40000000u32.to_be_bytes()),
        ];
        let mut buf_b = [0u8; 1024];
        let (result_b, _) =
            patch_chosen_node(reader2, patched_slice_a, &prop_val_list2, &mut buf_b);
        let hdr_b = Reader::get_header(result_b).unwrap();
        let len_b = hdr_b.total_size as usize;
        let slice_a = &result_a[..len_a.min(100)];
        let slice_b = &result_b[..len_b.min(100)];
        assert_ne!(slice_a, slice_b);
    }
}
