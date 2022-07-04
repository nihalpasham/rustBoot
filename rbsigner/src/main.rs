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

    let mut key_file = Vec::new();
    let mut kf = fs::File::open(args[3]).expect("Need path to key_file as argument");
    kf.read_to_end(&mut key_file).unwrap();
    let sk: SigningKeyType;

    match args[4] {
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
         
            let mcu_image = sign_mcu_image(image_blob, args[2], sk);
           // yash
           let x = args[2].to_string();
           let d: Vec<_> = x.split(&['/', '.',][..]).collect();
           let val = d.len();
           let mut str  = d[val-2].to_string();
           if str.contains("boot")
           {
              let footer = String::from("_v1234_signed.bin");
              str = str + &footer;
              println!("{}",str);
           }
           else
           {
             let footer = String::from("_v1235_signed.bin");
              str = str + &footer;
              println!("{}",str);
           }
            // yash
            match mcu_image {
                Ok(val) => {
                    let file =
                        File::create("../boards/rbSigner/signed_images/".to_owned()+&str);
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
    }
}
