mod curve;
mod fitsigner;
mod mcusigner;

use curve::SigningKeyType;
use curve::{import_signing_key, CurveType};
use fitsigner::sign_fit;
use mcusigner::sign_mcu_image;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    //String concatenation
    let s1 = String::from(args[5]);
    #[rustfmt::skip]
    let s2 = String::from(args[2].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]);
    let s3 = s2 + "_v" + &s1 + "_signed";
    //firmware version
    let s5: u32 = args[5].parse().unwrap();
    let version: [u8; 4] = s5.to_le_bytes();

    let mut key_file = Vec::new();
    let mut kf = fs::File::open(args[4]).expect("Need path to key_file as argument");
    kf.read_to_end(&mut key_file).unwrap();
    let sk: SigningKeyType;

    println!("\nUpdate type       :Firmware");
    println!("Curve type        :{}", args[3]);
    #[rustfmt::skip]
    println!("Input image       :{}.bin", String::from(args[2].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
    #[rustfmt::skip]
    println!("Public key        :{}.der", String::from(args[4].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
    println!("Image version     :{}", args[5]);
    println!("Output image      :{}.bin", s3);

    match args[3] {
        "nistp256" => {
            let signing_key = &key_file.as_slice()[0x40..];
            if signing_key.len() != 32 {
                panic!("invalid nistp256 key: length is not 32 bytes")
            }
            sk = import_signing_key(CurveType::NistP256, signing_key).unwrap();
        }
        _ => {
            unimplemented!()
        }
    }

    let mut image_blob = Vec::new();
    match args[1] {
        "fit-image" => {
            let mut itb = fs::File::open(args[2]).expect("Need path to itb_blob as argument");
            itb.read_to_end(&mut image_blob).unwrap();

            let signed_fit = sign_fit(image_blob, sk);
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
        "mcu-image" => {
            let mut mcu_image =
                fs::File::open(args[2]).expect("Need path to mcu_image binary as argument");
            mcu_image.read_to_end(&mut image_blob).unwrap();

            let mcu_image = sign_mcu_image(image_blob, args[2], sk, version);
            match mcu_image {
                Ok(val) => {
                    let file = File::create(
                        "../boards/rbSigner/signed_images/{s3}.bin".replace("{s3}", &s3),
                    );
                    match file {
                        Ok(mut file) => {
                            let bytes_written = file.write(val.as_slice());
                            if let Ok(val) = bytes_written {
                                println!("Output image successfully created with {} bytes.\n", val);
                            }
                        }
                        Err(e) => panic!("error: {:?}", e),
                    }
                }
                Err(_e) => {}
            }
        }
        _ => {}
    }
}
