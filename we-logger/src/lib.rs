#![no_std]
extern crate alloc;

use serde::{Serialize, Deserialize};
use alloc::string::{String, ToString};
use alloc::borrow::ToOwned;

#[link(wasm_import_module = "__wasm_everything_runtime__")]
extern "C" {
    fn log(record_ptr: *const u8, record_len: usize);
}

struct Logger;

const LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Metadata {
    level: Level,
    target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Record {
    metadata: Metadata,
    args: String,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // should never fail
        let record: Record = record.into();
        let json = serde_json::to_string(&record).unwrap();
        let bytes = json.as_bytes();
        unsafe {
            log(bytes.as_ptr(), bytes.len())
        }
    }

    fn flush(&self) { }
}

impl From<log::Level> for Level {
    fn from(l: log::Level) -> Self {
        use Level::*;

        match l {
            log::Level::Error => Error,
            log::Level::Warn => Warn,
            log::Level::Info => Info,
            log::Level::Debug => Debug,
            log::Level::Trace => Trace
        }
    }
}

impl <'a> From<&log::Metadata<'a>> for Metadata {
    fn from(m: &log::Metadata<'a>) -> Self {
        Self {
            level: m.level().into(),
            target: m.target().to_owned()
        }
    }
}

impl <'a> From<&log::Record<'a>> for Record {
    fn from(r: &log::Record) -> Self {
        Self {
            metadata: r.metadata().into(),
            args: r.args().to_string(),
            module_path: r.module_path().map(|s| s.to_owned()),
            file: r.file().map(|s| s.to_owned()),
            line: r.line()
        }
    }
}