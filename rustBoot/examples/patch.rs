#![allow(warnings)]

use as_slice::AsSlice;
use rustBoot::dt::{
    PropertyValue, RawNodeConstructor, Reader, StringsBlock, StructItem, TOKEN_SIZE,
};
use std::fs;
use std::io::Read;

fn main() {
    let mut buf = Vec::new();
    let mut file = fs::File::open(
        std::env::args()
            .nth(1)
            .expect("Need path to DTB file as argument"),
    )
    .unwrap();
    file.read_to_end(&mut buf).unwrap();
    let dtb_blob = buf.as_slice();
    let reader = Reader::read(buf.as_slice()).unwrap();
    let mut header = Reader::get_header(buf.as_slice()).unwrap();
    let strings_block_len = header.strings_size as usize;
    let struct_offset = header.struct_offset;
    let header_len = header.len();

    // *** Add the suppiled device-tree property names to the strings-block ***

    let mut buf = [0; 100];
    let name_list = ["bootargs", "linux,initrd-start", "linux,initrd-end"];
    let mut appended_strings_block = StringsBlock::new(&mut buf[..]).unwrap();
    let mut offset_list = appended_strings_block
        .make_new_strings_block_with(&name_list)
        .unwrap();
    let appended_strings_block = appended_strings_block.finalize();
    let offset_list = &mut offset_list[..name_list.len()];
    println!("appended_strings_block: {:?}", appended_strings_block);
    // add strings_block_len to each offset in the list
    offset_list
        .iter_mut()
        .for_each(|offset| *offset = *offset + strings_block_len);
    println!("offset_list: {:?}", offset_list);

    // *** Construct a device-tree node with supplied node_name and property-values ***

    let mut buf = [0u8; 200];
    let node_name = "chosen";
    let prop_val_list = [
        PropertyValue::String(
            "root=UUID=f2fa8d24-c392-4176-ab1c-367d60b66c6a \
    rootwait ro plymouth.ignore-serial-consoles \
    fsck.mode=auto fsck.repair=yes cma=128M",
        ),
        PropertyValue::U32([0x05, 0x89, 0x00, 0x00]),
        PropertyValue::U32([0x07, 0x7f, 0x08, 0x4a]),
    ];
    let node_size = RawNodeConstructor::make_node_with_props(
        &mut buf[..],
        node_name,
        offset_list,
        &prop_val_list,
    )
    .unwrap();
    let serialized_node = &buf[..node_size];
    println!("serialized_node: {:?} ", serialized_node);
    println!("node_len: {:?} ", node_size);

    // *** Find the chosen node start and end ***

    let root = reader.struct_items();
    let (node, node_iter) = root.path_struct_items("/chosen").next().unwrap();
    let chosen_node_check_len = node_iter.check_chosen_for_properties();
    println!("check_chosen_node: {:?}", chosen_node_check_len);

    let chosen_node_len = TOKEN_SIZE + node.node_name().unwrap().len();
    let chosen_node_padded_len = chosen_node_len + (chosen_node_len % 4);
    let chosen_node_start =
        (node_iter.get_offset() + struct_offset as usize) - chosen_node_padded_len;
    let chosen_node_end = chosen_node_start + chosen_node_padded_len + chosen_node_check_len;
    println!(
        "chosen_node_start: {}, chosen_node_end: {:?}",
        chosen_node_start, chosen_node_end
    );

    // *** Update device-tree header ***

    println!("strings_offset_before: {:?}", header.strings_offset);

    let appended_strings_block_len = appended_strings_block.len() as u32;
    header.strings_size = header.strings_size + appended_strings_block_len;
    header.struct_size = (header.struct_size + node_size as u32)
        - (chosen_node_padded_len + chosen_node_check_len) as u32;
    header.strings_offset = (header.strings_offset + node_size as u32)
        - (chosen_node_padded_len + chosen_node_check_len) as u32;
    header.total_size = (header.total_size + appended_strings_block_len + node_size as u32)
        - (chosen_node_padded_len + chosen_node_check_len) as u32;
    let strings_offset = header.strings_offset as usize;
    let hdr_total_size = header.total_size as usize;

    println!("struct_offset: {:?}", header.struct_offset);
    println!("struct_size: {:?}", header.struct_size);

    println!("strings_offset_after: {:?}", strings_offset);
    println!("total_size: {:?}", header.total_size);

    let header = header.as_slice();
    println!("updated header: {:?}", header);

    // *** Relocate device tree and patch chosen node ***

    let dtb_slice = dtb_blob[chosen_node_end..].len();
    let slice_1 = chosen_node_start..chosen_node_start + node_size;
    let slice_2 = chosen_node_start + node_size..chosen_node_start + node_size + dtb_slice;
    let slice_3 = chosen_node_start + node_size + dtb_slice
        ..chosen_node_start + node_size + dtb_slice + appended_strings_block_len as usize;
    let mut dt = [0u8; 27000];
    dt[..header_len].copy_from_slice(&header);
    dt[header_len..chosen_node_start].copy_from_slice(&dtb_blob[header_len..chosen_node_start]);
    dt[slice_1].copy_from_slice(serialized_node);
    dt[slice_2].copy_from_slice(&dtb_blob[chosen_node_end..]);
    dt[slice_3].copy_from_slice(appended_strings_block);

    // println!("string_offset: {:?}", &dt[strings_offset - 50..]);

    let reader = Reader::read(&dt[..hdr_total_size]).unwrap();
    for entry in reader.reserved_mem_entries() {
        println!("reserved: {:?} bytes at {:?}", entry.size, entry.address);
    }
    let mut indent = 0;
    for entry in reader.struct_items() {
        match entry {
            StructItem::BeginNode { name } => {
                println!("{:indent$}{} {{", "", name, indent = indent);
                indent += 2;
            }
            StructItem::EndNode => {
                indent -= 2;
                println!("{:indent$}}}", "", indent = indent);
            }
            StructItem::Property { name, value } => {
                println!("{:indent$}{}: {:?}", "", name, value, indent = indent)
            }
            _ => {
                panic!("invalid device tree blob")
            }
        }
    }
}
