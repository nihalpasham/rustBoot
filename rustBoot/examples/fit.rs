#![allow(warnings)]
use rustBoot::dt::{parse_fit, prepare_img_hash, Reader, Result};
use sha2::{Digest, Sha256};

use std::fs;
use std::io::Read;

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
    let res = parse_fit::<32, 64, 4>(reader);
    match res {
        Ok((config, images)) => {
            println!("\nconfig: {:?}\n", config);
            println!("images: {:?}", images)
        }
        Err(e) => panic!("error: {:?}", e),
    }

    let fit = prepare_img_hash::<Sha256, 32, 64, 4>(buf.as_slice());
    match fit {
        Ok(val) => {
            println!("\nfit_sha: {:?}\n", val);
        }
        Err(e) => panic!("error: {:?}", e),
    }
}
