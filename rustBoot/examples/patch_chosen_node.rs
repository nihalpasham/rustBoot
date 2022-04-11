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

    let _ = log_init();
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
    let mut buf = [0; 27000];
    let (res, len) = patch_chosen_node(reader, dtb_blob, &prop_val_list, &mut buf);
    println!("len: {}", len);
    let patched_dtb_blob = &res[..len];

    dump(patched_dtb_blob);
}

pub fn dump<'a>(dtb_blob: &'a [u8]) {
    println!("test");
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

use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("\x1b[93m[{}]\x1b[0m  {}", record.level(), record.args());
            match (record.module_path(), record.line()) {
                (Some(file), Some(line)) => {
                    println!("\t \u{2319} {} @ line:{}", file, line);
                }
                (_, None) => {
                    println!("... ")
                }
                (_, Some(line)) => println!("\t  \u{2a3d} {} @ line:{}", record.target(), line),
                (Some(file), None) => println!("\t  \u{2a3d} @ {}", file),
            }
        }
    }

    fn flush(&self) {}
}

pub fn log_init() -> core::result::Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}
