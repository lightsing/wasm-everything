#![no_std]
extern crate alloc;

use serde::{Serialize, Deserialize};
use alloc::string::{String, ToString};
use alloc::borrow::ToOwned;

#[link(wasm_import_module = "__wasm_everything_runtime__")]
extern "C" {
    fn log_proxy(record_ptr: *const u8, record_len: usize);
}

struct Logger;

static LOGGER: &dyn log::Log = &Logger;

pub fn init() {
    log::set_logger(LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
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
pub struct Record {
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
        let record: Record = record.into();
        let json = serde_json::to_string(&record).unwrap(); // should never fail
        let bytes = json.as_bytes();
        unsafe {
            log_proxy(bytes.as_ptr(), bytes.len())
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

impl Into<log::Level> for Level {
    #[inline]
    fn into(self) -> log::Level {
        use Level::*;

        match self {
            Error => log::Level::Error,
            Warn => log::Level::Warn,
            Info => log::Level::Info,
            Debug => log::Level::Debug,
            Trace => log::Level::Trace
        }
    }
}

impl Metadata {
    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> log::Level {
        self.level.into()
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &str {
        self.target.as_str()
    }

    #[inline]
    fn into(&self) -> log::Metadata {
        log::Metadata::builder()
            .level(self.level.into())
            .target(self.target.as_str())
            .build()
    }
}

impl Record {

    /// The message body.
    #[inline]
    pub fn args(&self) -> &str {
        self.args.as_str()
    }

    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> log::Metadata {
        (&self.metadata).into()
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> log::Level {
        self.metadata.level()
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &str {
        self.metadata.target.as_str()
    }

    /// The module path of the message.
    #[inline]
    pub fn module_path(&self) -> Option<&str> {
        self.module_path.as_ref().map(|s| s.as_str())
    }

    /// The source file containing the message.
    #[inline]
    pub fn file(&self) -> Option<&str> {
        self.file.as_ref().map(|s| s.as_str())
    }

    /// The line containing the message.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.line
    }
}