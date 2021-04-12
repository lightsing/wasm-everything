#[macro_use]
extern crate log;

use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::borrow::{Borrow, BorrowMut};
use std::sync::atomic::{AtomicU64, Ordering};

use chashmap::CHashMap;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use wasmer::{
    imports, Exports, Function, Global, Instance, LazyInit, Memory, MemoryView, Module, NativeFunc,
    RuntimeError, Store, Val, WasmerEnv,
};
use we_logger::Record;
use std::ops::Div;
use crate::scheduler::WasmFunctionExecution;

mod error;
mod scheduler;

static GLOBAL_INSTANCE_MAP: Lazy<CHashMap<u64, Instance>> = Lazy::new(|| CHashMap::new());

#[derive(WasmerEnv, Clone)]
struct Env {
    name: OnceCell<Option<String>>,
    instance_id: OnceCell<u64>,
    #[wasmer(export(name = "NAME"))]
    name_ptr: LazyInit<Global>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export)]
    get_instance_id: LazyInit<NativeFunc<(), i64>>,
    #[wasmer(export(name = "_wasm_malloc"))]
    malloc: LazyInit<NativeFunc<i32, i32>>,
    #[wasmer(export(name = "_wasm_free"))]
    free: LazyInit<NativeFunc<(i32, i32)>>,
    channel: tokio::sync::mpsc::UnboundedSender<(String, Vec<u8>)>,
}

impl Env {
    fn new(channel: tokio::sync::mpsc::UnboundedSender<(String, Vec<u8>)>) -> Self {
        Self {
            name: Default::default(),
            instance_id: Default::default(),
            name_ptr: Default::default(),
            memory: Default::default(),
            get_instance_id: Default::default(),
            malloc: Default::default(),
            free: Default::default(),
            channel,
        }
    }

    fn instance_id(&self) -> u64 {
        *self.instance_id.get_or_init(|| {
            self.get_instance_id
                .get_ref()
                .and_then(|f| f.call().map(|id| id as u64).ok())
                .unwrap_or(0)
        })
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

    unsafe fn malloc(&self, size: usize) -> Result<usize, RuntimeError> {
        let malloc = self.malloc.get_unchecked();
        malloc.call(size as i32).map(|ret| ret as usize)
    }

    unsafe fn copy_from_slice(&self, addr: usize, slice: &[u8]) {
        let memory = self.memory.get_unchecked();
        let target = &mut memory.data_unchecked_mut()[addr..addr + slice.len()];
        target.copy_from_slice(slice)
    }

    unsafe fn write_u32(&self, addr: usize, value: u32) {
        let buf = std::mem::transmute::<u32, [u8; 4]>(value);
        self.copy_from_slice(addr, &buf)
    }

    #[inline]
    fn name(&self) -> Option<&str> {
        self.name
            .get_or_init(|| {
                let offset = self.name_ptr.get_ref()?.get().i32()? as usize;
                let memory = self.memory.get_ref()?;
                unsafe {
                    let name_ptr = memory.data_ptr().add(self.deref(offset)) as *const c_char;
                    CStr::from_ptr(name_ptr).to_owned().into_string().ok()
                }
            })
            .as_ref()
            .map(|s| s.as_str())
    }

    #[inline]
    fn get_string(&self, offset: usize, len: usize) -> String {
        return String::from_utf8(self.get_bytes(offset, len)).unwrap();
    }

    #[inline]
    fn get_bytes(&self, offset: usize, len: usize) -> Vec<u8> {
        let view: MemoryView<u8> = self.memory.get_ref().unwrap().view();
        (&view[offset..offset + len])
            .iter()
            .map(|u| u.get())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Response {
    bar: i32,
}

fn callback(env: &Env, ptr: i32, len: i32, cb: i64, user_data: i64) {
    use std::mem::{forget, transmute};

    let mut rt = env.get_bytes(ptr as usize, len as usize);
    let ptr = rt.as_mut_ptr();
    let size = rt.len();
    let cap = rt.capacity();
    forget(rt);
    unsafe {
        let cb = cb as *mut c_void;
        transmute::<_, unsafe extern "C" fn(*mut c_void, *const u8, usize, usize)>(cb)(user_data as *mut c_void, ptr, size, cap)
    }
}

fn invoke(
    env: &Env,
    name_ptr: i32,
    name_len: i32,
    method_ptr: i32,
    method_len: i32,
    args_ptr: i32,
    args_len: i32,
    cb: i32,
    user_data: i32,
) {
    let name = unsafe { env.get_str_unchecked(name_ptr as usize, name_len as usize) };
    let method = env.get_string(method_ptr as usize, method_len as usize);
    let args = env.get_string(args_ptr as usize, args_len as usize);

    info!(
        "request from <{}>#{}, {} {} {}",
        env.name().unwrap_or("???"),
        env.instance_id(),
        name,
        method,
        args
    );
}

fn log_proxy(env: &Env, record_ptr: i32, record_len: i32) {
    let record_serialized = env.get_bytes(record_ptr as usize, record_len as usize);
    let name = env.name().unwrap_or("???").to_string();
    env.channel.send((name, record_serialized)).ok();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let (log_channel_tx, mut log_channel_rx) =
        tokio::sync::mpsc::unbounded_channel::<(String, Vec<u8>)>();
    tokio::spawn(async move {
        while let Some((name, record_serialized)) = log_channel_rx.recv().await {
            let record: bincode::Result<Record> = bincode::deserialize(&record_serialized);
            match record {
                Ok(record) => log!(
                    target: &name,
                    record.level(),
                    "[{:<5}:{}]: {}",
                    record.module_path().unwrap_or("???"),
                    record
                        .line()
                        .map_or_else(|| "??".to_string(), |l| l.to_string()),
                    record.args()
                ),
                Err(e) => error!("cannot log module <{}>: {}", name, e),
            }
        }
    });

    let store = Store::default();
    let instance_id = AtomicU64::new(1);

    let module = Module::from_file(&store, "target/wasm32-unknown-unknown/debug/hello.wasm")?;

    let import_object = imports! {
        "__wasm_everything_runtime__" => {
            "invoke" => Function::new_native_with_env(
                &store,
                Env::new(log_channel_tx.clone()),
                invoke
            ),
            "log_proxy" => Function::new_native_with_env(
                &store,
                Env::new(log_channel_tx.clone()),
                log_proxy
            ),
            "callback" => Function::new_native_with_env(
                &store,
                Env::new(log_channel_tx.clone()),
                callback
            )
        }
    };

    let mut instance = Instance::new(&module, &import_object)?;

    // set instance id
    let this_instance_id = instance_id.fetch_add(1, Ordering::SeqCst);
    let set_instance_id = instance.exports.get_function("set_instance_id")?;
    let set_instance_id_result = set_instance_id.call(&[Val::I64(this_instance_id as i64)])?;
    if cfg!(debug_assertions) {
        assert_eq!(set_instance_id_result[0].unwrap_i32(), 1);
        let get_instance_id = instance.exports.get_function("get_instance_id")?;
        assert_eq!(get_instance_id.call(&[])?[0].unwrap_i64(), this_instance_id as i64);
    }
    let rt = GLOBAL_INSTANCE_MAP.insert(this_instance_id, instance);
    debug_assert!(rt.is_none());

    let instance = GLOBAL_INSTANCE_MAP.get(&this_instance_id).unwrap();
    let hello = instance.exports.get_function("hello")?;
    let response: Response = WasmFunctionExecution::new(hello).call().await?;
    info!("{:?}", response);

    Ok(())
}
