use rustBoot::dt::Reader;

use std::convert::TryInto;
use std::{fs, io::Read};

fn main() {
    let mut buf = Vec::new();
    let mut file = fs::File::open(
        std::env::args()
            .nth(1)
            .expect("Need path to FIT Blob file as argument"),
    )
    .unwrap();
    file.read_to_end(&mut buf).unwrap();

    let reader = Reader::read(buf.as_slice()).unwrap();
    let root = &reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/").next().unwrap();

    let timestamp = node_iter.get_node_property("timestamp");
    println!(
        "\nfitImage timestamp: {:?}",
        u32::from_be_bytes(timestamp.unwrap().try_into().unwrap())
    );
}
