#![allow(warnings)]

use core::convert::TryInto;
use rustBoot::dt::{
    correct_endianess, patch::*, Error, PropertyValue, RawNodeConstructor, RawPropertyConstructor,
    Reader, Result, StringsBlock, StructItem, TOKEN_SIZE,
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
    let reader = Reader::read(buf.as_slice()).unwrap();

    let dtb_blob = buf.as_slice();
    let mut buf = [0; 100];
    let mut new_strings_block = StringsBlock::new(&mut buf[..]).unwrap();

    let name_list = ["bootargs", "linux,initrd-start", "linux,initrd-end"];
    let res = make_new_strings_block_with::<3>(&name_list, &mut new_strings_block, dtb_blob);
    let (offset_list, strings_block_patch, strings_block_patch_len) = match res {
        Ok((strings_block, offset_list)) => {
            println!("strings_block_patch: {:?}\n", strings_block);
            println!("offset_list: {:?}\n", offset_list);
            (offset_list, strings_block, strings_block.len())
        }
        Err(e) => panic!("error: {:?}", e),
    };

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
    let res = make_node_with_props::<200>(node_name, &prop_val_list, &offset_list);
    let (patch_bytes_1_len, patch_bytes_1) = match res {
        Ok((patch_bytes_1_len, patch_bytes_1)) => {
            println!("patch_bytes_1_len: {:?}\n", patch_bytes_1_len);
            (patch_bytes_1_len, patch_bytes_1)
        }
        Err(e) => panic!("error: {:?}", e),
    };
    let patch_bytes_1 = &patch_bytes_1[..patch_bytes_1_len];
    println!("patch_bytes_1: {:?}\n", patch_bytes_1);

    let res = parse_raw_node::<10>(&reader, "/chosen", dtb_blob);
    let parsed_node = match res {
        Ok(val) => {
            val.iter().for_each(|(prop_name, item, prop_len)| {
                println!(
                    "prop_name: {:?}, item: {:?}, prop_len: {:?}\n",
                    prop_name, item, prop_len
                )
            });
            val
        }
        Err(e) => panic!("error: {:?}", e),
    };

    let res = check_chosen_node::<10, 200>(parsed_node);
    let (patch_bytes_2, len_to_be_subtracted) = match res {
        Ok((buf, len_to_be_subtracted)) => {
            println!(
                "patch_bytes_2: {:?}, subtracted_len: {:?}\n",
                buf.as_slice(),
                len_to_be_subtracted
            );
            (buf, len_to_be_subtracted)
        }
        Err(e) => panic!("error: {:?}", e),
    };
    // `patch_bytes_1_len` includes a `BEGIN_NODE`, we have to subtract it from the new length.
    // i.e. the `chosen` node takes up 12 bytes (0x00000001 + "chosen" + padding)
    let padded_node_len = get_padded_node_len(&reader, "/chosen");
    let new_node_len = patch_bytes_1_len + patch_bytes_2.as_slice().len() - padded_node_len;
    println!(
        "new_node_len: {:?}, padded_node_len: {:?}\n",
        new_node_len, padded_node_len
    );

    let mut header = Reader::get_header(dtb_blob).unwrap();
    {
        let updated_header = update_dtb_header(
            &mut header,
            strings_block_patch_len,
            new_node_len,
            len_to_be_subtracted,
        );
        println!("header: {:?}\n", updated_header);
    }

    let (node_start, node_end) =
        match get_node_start_and_end(&reader, "/chosen", dtb_blob, len_to_be_subtracted) {
            Ok((node_start, node_end)) => (node_start, node_end),
            Err(e) => panic!("error: {:?}", e),
        };

    let mut buf = [0u8; 27000];
    let _ = patch_dtb_node::<27000>(
        &header,
        node_start,
        node_end,
        dtb_blob,
        patch_bytes_1,
        patch_bytes_2.as_slice(),
        strings_block_patch,
        buf.as_mut(),
    );
    let hdr_total_size = correct_endianess(header.total_size);
    dump(&buf[..hdr_total_size as usize]);
}

pub fn dump<'a>(dtb_blob: &'a [u8]) {
    let header = Reader::get_header(dtb_blob).unwrap();
    let hdr_total_size = header.total_size;
    let reader = Reader::read(&dtb_blob[..hdr_total_size as usize]).unwrap();

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
