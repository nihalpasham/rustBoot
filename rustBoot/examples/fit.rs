use rustBoot::dt::{parse_fit, prepare_img_hash, verify_fit, Reader};
use sha2::{Digest, Sha256};

use std::env;
use std::fs;
use std::io::Read;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    let mut buf = Vec::new();
    let mut file = fs::File::open(args[1]).expect("Need path to itb_blob as argument");
    file.read_to_end(&mut buf).unwrap();
    let version = args[2].parse().expect("bad version: unable to parse");

    // log_init();

    let reader = Reader::read(buf.as_slice()).unwrap();
    let res = parse_fit::<Sha256, 32, 64, 4>(reader);
    match res {
        Ok((config, images)) => {
            println!("\nconfig: {:?}\n", config);
            println!("images: {:?}", images)
        }
        Err(e) => panic!("error: {:?}", e),
    }

    let fit = prepare_img_hash::<Sha256, 32, 64, 4>(buf.as_slice(), version);
    match fit {
        Ok((fit_hash, _signature)) => {
            println!("\nfit_sha: {:x}\n", fit_hash.finalize());
        }
        Err(e) => panic!("error: {:?}", e),
    }

    let header = Reader::get_header(buf.as_slice()).unwrap();
    println!("header: {:?}\n", header);

    let _verified_fit = match verify_fit::<32, 64, 4>(buf.as_slice(), version) {
        Ok(val) => {
            print!("\n*********** \x1b[5m\x1b[33mecdsa signature\x1b[0m checks out, \x1b[92mimage is authentic\x1b[0m ***********\n");
            val
        }
        Err(e) => panic!("error: image verification failed, {}", e),
    };
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
                (_, None) => {
                    println!("... ")
                }
                (_, Some(line)) => println!("\t  \u{2a3d} {} @ line:{}", record.target(), line),
                // (Some(file), None) => println!("\t  \u{2a3d} @ {}", file),
            }
        }
    }

    fn flush(&self) {}
}

pub fn log_init() -> core::result::Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}
