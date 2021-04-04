#[macro_use] extern crate log;

use std::ffi::CStr;
use std::os::raw::c_char;

use wasmer::{WasmerEnv, Store, Module, Instance, imports, Function, LazyInit, Memory, MemoryView, Exports, NativeFunc, Global};
use we_logger::Record;

use once_cell::sync::OnceCell;

#[derive(WasmerEnv, Clone, Default)]
struct Env {
    #[wasmer(export(name = "NAME"))]
    name_ptr: LazyInit<Global>,
    // #[wasmer(export(name = "VERSION"))]
    // version_ptr: LazyInit<Global>,
    name: OnceCell<Option<String>>,
    // version: Option<RefCell<String>>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export(name = "_wasm_malloc"))]
    malloc: LazyInit<NativeFunc<(i32), i32>>,
    #[wasmer(export(name = "_wasm_free"))]
    free: LazyInit<NativeFunc<(i32, i32)>>,
}

impl Env {
    fn new() -> Self {
        Self::default()
    }

    unsafe fn deref(&self, offset: usize) -> usize {
        let memory = self.memory.get_unchecked();
        let slice = memory.data_unchecked();
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&slice[offset..offset + 4]);
        std::mem::transmute::<[u8; 4], u32>(buf) as usize
    }

    unsafe fn get_str_unchecked(&self, offset: usize, len: usize) -> &str {
        let memory = self.memory.get_unchecked();
        let slice = memory.data_unchecked();
        std::str::from_utf8_unchecked(&slice[offset..offset + len])
    }

    fn name(&self) -> Option<&str> {
        self.name.get_or_init(|| {
            let offset = self.name_ptr.get_ref()?.get().i32()? as usize;
            let memory = self.memory.get_ref()?;
            unsafe {
                let name_ptr = memory.data_ptr().add(self.deref(offset)) as *const c_char;
                CStr::from_ptr(name_ptr)
                    .to_owned()
                    .into_string()
                    .ok()
            }
        }).as_ref().map(|s| s.as_str())
    }

    fn get_string(&self, offset: usize, len: usize) -> String {
        return String::from_utf8(self.get_bytes(offset, len)).unwrap()
    }

    fn get_bytes(&self, offset: usize, len: usize) -> Vec<u8> {
        let view: MemoryView<u8> = self.memory.get_ref().unwrap().view();
        (&view[offset..offset +len]).iter().map(|u| u.get()).collect()
    }
}

fn invoke(
    env: &Env,
    name_ptr: i32, name_len: i32,
    method_ptr: i32, method_len: i32,
    args_ptr: i32, args_len: i32,
    result_ptr: i32, result_len: i32,
) {
    let name = unsafe { env.get_str_unchecked(name_ptr as usize, name_len as usize) };
    let method = env.get_string(method_ptr as usize, method_len as usize);
    let args = env.get_string(args_ptr as usize, args_len as usize);
    let malloc = env.malloc.get_ref().unwrap();
    let result = r#"{ "bar": 1}"#;
    let addr = malloc.call(result.len() as i32).unwrap();
    let result = (result_ptr as *mut u8);

    info!("request from <{:?}>, {} {} {}", env.name().unwrap_or("???"), name, method, args);
}

fn log_proxy(
    env: &Env,
    record_ptr: i32,
    record_len: i32
) {
    let record_serialized = env.get_bytes(record_ptr as usize, record_len as usize);
    let record : bincode::Result<Record> = bincode::deserialize(&record_serialized);
    match record {
        Ok(record) => log!(
            target: env.name().unwrap_or("???"),
            record.level(),
            "[{:<5}] L{:3}: {}",
            record.module_path().unwrap_or("???"),
            record.line().map_or_else( || "??".to_string(), |l| l.to_string()),
            record.args()
        ),
        Err(e) => error!("cannot log module <{}>: {}", env.name().unwrap_or("???"), e)
    }

}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let store = Store::default();

    let module = Module::from_file(&store, "target/wasm32-unknown-unknown/debug/hello.wasm")?;

    let invoke_function = Function::new_native_with_env(
        &store,
        Env::new(),
        invoke
    );

    let log_function = Function::new_native_with_env(
        &store,
        Env::new(),
        log_proxy
    );

    let mut exports = Exports::new();
    exports.insert("invoke", invoke_function);
    exports.insert("log_proxy", log_function);

    let import_object = imports! {
        "__wasm_everything_runtime__" => exports
    };
    let instance = Instance::new(&module, &import_object)?;


    let hello = instance.exports.get_function("hello")?;
    hello.call(&[])?;

    Ok(())
}