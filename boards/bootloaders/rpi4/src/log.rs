use cortex_a::asm::barrier;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

use rustBoot_hal::info;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            info!("\x1b[93m[{}]\x1b[0m  {}", record.level(), record.args());
            match (record.module_path(), record.line()) {
                (Some(file), Some(line)) => {
                    info!("         - {} @ line:{}", file, line);
                }
                (None, None) => {
                    info!("... ")
                }
                (None, Some(line)) => info!("\t  \u{2a3d} {} @ line:{}", record.target(), line),
                (Some(file), None) => info!("\t  \u{2a3d} @ {}", file),
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

pub fn init() -> Result<(), SetLoggerError> {
    unsafe { log::set_logger_racy(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info)) }
}
