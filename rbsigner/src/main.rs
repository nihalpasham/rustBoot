mod curve;
mod fitsigner;
mod mcusigner;

use curve::SigningKeyType;
use curve::{import_signing_key, CurveType};
use fitsigner::sign_fit;
use mcusigner::sign_mcu_image;
use rustBoot::dt::Reader;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    // let _ = log_init();

    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    let mut key_file = Vec::new();
    let mut kf = fs::File::open(args[4]).expect("Need path to key_file as argument");
    kf.read_to_end(&mut key_file).unwrap();
    let sk: SigningKeyType;

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

            // get the timestamp
            let reader = Reader::read(&image_blob.as_slice()).unwrap();
            let root = &reader.struct_items();
            let (_, node_iter) = root.path_struct_items("/").next().unwrap();

            let timestamp = match node_iter.get_node_property("timestamp") {
                Some(ts) => u32::from_be_bytes(ts.try_into().unwrap()),
                None => panic!("bad itb file, doesnt contain a timestamp"),
            };

            let version_string = timestamp.to_string();
            let version = timestamp;
            let output_itb_name = String::from(format!("signed-v{version_string}.itb").as_str());

            println!("\nImage type:       fit-image");
            println!("Curve type:       {}", args[3]);
            #[rustfmt::skip]
            println!("Input image:      {}.itb", String::from(args[2].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
            println!("fit version:      {:?}", version);
            #[rustfmt::skip]
            println!("Public key:       {}.der", String::from(args[4].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
            println!("Output image:     {}", output_itb_name);

            let signed_fit = sign_fit(image_blob, version, sk);
            match signed_fit {
                Ok(val) => {
                    // println!(
                    //     "signed_fit: {:?}",
                    //     &val.as_slice()[(val.len() - 1071)..(val.len() - 800)]
                    // );
                    let out_file = args[2].rsplit_once('/');
                    match out_file {
                        Some((f, _)) => {
                            let file = File::create(format!("{f}/{output_itb_name}").as_str());
                            match file {
                                Ok(mut file) => {
                                    let bytes_written = file.write(val.as_slice());
                                    if let Ok(val) = bytes_written {
                                        println!("\nbytes_written: {:?}", val);
                                    }
                                }
                                Err(e) => panic!("error: {:?}", e),
                            }
                        }
                        None => {
                            panic!("something's wrong with your file_path to itb_blob ")
                        }
                    }
                }
                Err(_e) => {}
            }
        }
        "mcu-image" => {
            //String concatenation
            let image_version_args = String::from(args[5]);
            #[rustfmt::skip]
            let input_image_args = String::from(args[2].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]);
            let output_image = input_image_args + "_v" + &image_version_args + "_signed";

            println!("\nImage type:       mcu-image");
            println!("Curve type:       {}", args[3]);
            #[rustfmt::skip]
            println!("Input image:      {}.bin", String::from(args[2].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
            #[rustfmt::skip]
            println!("Public key:       {}.der", String::from(args[4].rsplit_terminator(&['/', '.'][..]).collect::<Vec<_>>()[1]));
            println!("Image version:    {}", args[5]);
            println!("Output image:     {}.bin", output_image);

            //firmware version
            let image_version_value: u32 = args[5].parse().unwrap();
            let version: [u8; 4] = image_version_value.to_le_bytes();

            let mut mcu_image =
                fs::File::open(args[2]).expect("Need path to mcu_image binary as argument");
            mcu_image.read_to_end(&mut image_blob).unwrap();

            let mcu_image = sign_mcu_image(image_blob, args[2], sk, version);
            match mcu_image {
                Ok(val) => {
                    let file = File::create(
                        "../boards/sign_images/signed_images/{output_image}.bin"
                            .replace("{output_image}", &output_image),
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
                    println!("\t \u{2a3d} {} @ line:{}", file, line);
                }
                (Some(file), None) => println!("\t  \u{2a3d} @ {}", file),
                (_, None) => {
                    println!("... ")
                }
                (_, Some(line)) => println!("\t  \u{2a3d} {} @ line:{}", record.target(), line),
            }
        }
    }

    fn flush(&self) {}
}

pub fn log_init() -> core::result::Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}
