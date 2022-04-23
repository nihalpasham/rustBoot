use super::internal::Header;
use super::{
    Error, PropertyValue, RawNodeConstructor, RawPropertyConstructor, Reader, Result,
    SerializedBuffer, StringsBlock, StructItem, TOKEN_SIZE,
};
use as_slice::AsSlice;
use core::convert::TryInto;

pub fn make_new_strings_block_with<'a, const M: usize>(
    name_list: &'a [&str],
    new_strings_block: &'a mut StringsBlock<'a>,
    dtb_blob: &'a [u8],
) -> Result<(&'a [u8], [usize; M])> {
    let header = Reader::get_header(dtb_blob.as_slice())?;
    let strings_block_len = header.strings_size as usize;

    let offset_list = new_strings_block.make_new_strings_block_with(&name_list)?;
    let new_strings_block = new_strings_block.finalize();

    let mut offset_list: [usize; M] = offset_list[..M]
        .try_into()
        .map_err(|_v| Error::BufferExhausted)?;
    // add strings_block_len to each offset in the list
    offset_list
        .iter_mut()
        .for_each(|offset| *offset = *offset + strings_block_len);
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
        &prop_val_list,
    )?;
    Ok((node_size, buf))
}

#[derive(Debug, Clone, Copy)]
pub enum NodeItems<'a> {
    RawNodeConstructor(RawNodeConstructor<'a>),
    RawPropertyConstructor(RawPropertyConstructor<'a>),
    None,
}

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
            }
            StructItem::EndNode => break,
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
    header.strings_size = header.strings_size + appended_strings_block_len as u32;
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
    let padded_node_len = node_len + (node_len % 4);
    padded_node_len
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
    let node_start = (node_iter.get_offset() + struct_offset as usize) - padded_node_len;
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
            + strings_block_patch_len as usize;

    patched_dtb_blob[..header_len].copy_from_slice(header.as_slice());
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
    let res = make_node_with_props::<200>(node_name, &prop_val_list, &offset_list);
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

    let _ = patch_dtb_node::<N>(
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

    let res = byte_1 | byte_2 | byte_3 | byte_4;
    res
}
