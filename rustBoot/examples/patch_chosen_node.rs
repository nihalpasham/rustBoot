use std::fs;
use std::io::Read;

use rustBoot::dt::patch::patch_chosen_node;
use rustBoot::dt::{PropertyValue, Reader, StructItem};

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
    let prop_val_list = [
        PropertyValue::String(
            "root=UUID=f2fa8d24-c392-4176-ab1c-367d60b66c6a \
    rootwait ro plymouth.ignore-serial-consoles \
    fsck.mode=auto fsck.repair=yes cma=128M",
        ),
        PropertyValue::U32([0x05, 0x89, 0x00, 0x00]),
        PropertyValue::U32([0x07, 0x7f, 0x08, 0x4a]),
    ];

    let res = patch_chosen_node::<27000>(reader, dtb_blob, &prop_val_list);
    let patched_dtb_blob = res.as_slice();

    dump(patched_dtb_blob);
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
