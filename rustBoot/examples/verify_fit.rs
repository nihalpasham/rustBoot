use log::info;
use rustBoot::dt::{verify_fit, Reader};
use std::env;
use std::fs;
use std::io::Read;

pub fn verify_authenticity(itb_blob: &[u8], itb_version: u32) -> bool {
    info!("\x1b[5m\x1b[31mauthenticating fit-image...\x1b[0m");
    let header = Reader::get_header(itb_blob).unwrap();
    let total_size = header.total_size;
    let val = match verify_fit::<32, 64, 4>(&itb_blob[..total_size as usize], itb_version) {
        Ok(val) => {
            print!(
                "######## \x1b[33mecdsa signature\x1b[0m checks out, \
                \x1b[92mimage is authentic\x1b[0m ########\n"
            );
            val
        }
        Err(e) => panic!("error: image verification failed, {}", e),
    };
    val
}

fn main() {
    let _ = log_init();

    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    let mut buf = Vec::new();
    let mut file = fs::File::open(args[1]).expect("Need path to itb_blob as argument");
    file.read_to_end(&mut buf).unwrap();
    let itb_version = args[2].parse().expect("bad version: unable to parse");

    let itb_blob = buf.as_slice();

    let auth = verify_authenticity(itb_blob, itb_version);
    info!("auth: {:?}", auth);
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
