#[link(wasm_import_module = "__wasm_everything_runtime__")]
extern "C" {
    pub fn invoke(
        name_ptr: *const u8, name_len: usize,
        method_ptr: *const u8, method_len: usize,
        args_ptr: *const u8, args_len: usize,
        result_ptr: *mut *mut u8, result_len: *const usize,
    );
}