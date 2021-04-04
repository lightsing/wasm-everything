use crate::Record;

#[link(wasm_import_module = "__wasm_everything_runtime__")]
extern "C" {
    fn log_proxy(record_ptr: *const u8, record_len: usize);
}

struct Logger;

pub static LOGGER: &dyn log::Log = &Logger;

pub fn init() {
    log::set_logger(LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let record: Record = record.into();
        let serialized = bincode::serialize(&record).unwrap(); // should never fail
        unsafe {
            log_proxy(serialized.as_ptr(), serialized.len())
        }
    }

    fn flush(&self) { }
}