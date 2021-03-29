use wasmer::{WasmerEnv, Store, Module, Instance, Value, imports, Function, Type, LazyInit, Memory, MemoryView, Exports, NativeFunc};
use std::mem::size_of;

#[derive(WasmerEnv, Clone)]
struct Env {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export(name = "_wasm_malloc"))]
    malloc: LazyInit<NativeFunc<(i32), i32>>,
    #[wasmer(export(name = "_wasm_free"))]
    free: LazyInit<NativeFunc<(i32, i32)>>,
}

impl Env {
    fn new() -> Self {
        Self {
            memory: LazyInit::new(),
            malloc: LazyInit::new(),
            free: LazyInit::new()
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

trait GetString {
    fn get_string<P: Into<usize>, L: Into<usize>>(&self, ptr: P, len: L) -> String;
}

impl GetString for Env {
    fn get_string<P: Into<usize>, L: Into<usize>>(&self, ptr: P, len: L) -> String {
        let ptr = ptr.into();
        let len = len.into();
        let view: MemoryView<u8> = self.memory.get_ref().unwrap().view();
        let data: Vec<u8> = (&view[ptr..ptr+len]).iter().map(|u| u.get()).collect();
        return String::from_utf8(data).unwrap()
    }
}

fn invoke(
    env: &Env,
    name_ptr: i32, name_len: i32,
    method_ptr: i32, method_len: i32,
    args_ptr: i32, args_len: i32,
    result_ptr: i32, result_len: i32,
) {
    let name = env.get_string(name_ptr as usize, name_len as usize);
    let method = env.get_string(method_ptr as usize, method_len as usize);
    let args = env.get_string(args_ptr as usize, args_len as usize);
    let malloc = env.malloc.get_ref().unwrap();
    let result = r#"{ "bar": 1}"#;
    let addr = malloc.call(result.len() as i32).unwrap();
    let result = (result_ptr as *mut u8);

    println!("request {} {} {}", name, method, args);
}

fn main() -> anyhow::Result<()> {
    let store = Store::default();

    let module = Module::from_file(&store, "target/wasm32-unknown-unknown/debug/hello.wasm")?;

    let env = Env::new();
    let invoke_function = Function::new_native_with_env(
        &store,
        env,
        invoke
    );
    let mut exports = Exports::new();
    exports.insert("invoke", invoke_function);

    let import_object = imports! {
        "__wasm_everything_runtime__" => exports
    };
    let instance = Instance::new(&module, &import_object)?;


    let add_one = instance.exports.get_function("hello")?;
    let result = add_one.call(&[])?;

    Ok(())
}