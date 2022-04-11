mod fitsigner;

use fitsigner::{import_signing_key, CurveType};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use crate::fitsigner::sign_fit;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    let mut itb = fs::File::open(args[1]).expect("Need path to itb_blob as argument");
    let mut kf = fs::File::open(args[2]).expect("Need path to key_file as argument");
    itb.read_to_end(&mut buf1).unwrap();
    kf.read_to_end(&mut buf2).unwrap();

    let kf = buf2.as_slice();
    let _signed_fit = match args[3] {
        "nistp256" => {
            let sk = import_signing_key(CurveType::NistP256, &kf[0x40..]).unwrap();
            let signed_fit = sign_fit(CurveType::NistP256, buf1, sk);
            match signed_fit {
                Ok(val) => {
                    // println!(
                    //     "signed_fit: {:?}",
                    //     &val.as_slice()[(val.len() - 1071)..(val.len() - 800)]
                    // );
                    let file =
                        File::create("../boards/bootloaders/rpi4/apertis/signed-rpi4-apertis.itb");
                    match file {
                        Ok(mut file) => {
                            let bytes_written = file.write(val.as_slice());
                            if let Ok(val) = bytes_written {
                                println!("bytes_written: {:?}", val);
                            }
                        }
                        Err(e) => panic!("error: {:?}", e),
                    }
                }
                Err(_e) => {}
            }
        }
        _ => {}
    };
}
