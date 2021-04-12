use core::ffi::c_void;
use alloc::vec::Vec;

use once_cell::sync::OnceCell;
use semi_async::trampoline;

static INSTANCE_ID: OnceCell<u64> = OnceCell::new();

pub type HostCallback = fn(*mut c_void, &[u8]);

#[link(wasm_import_module = "__wasm_everything_runtime__")]
extern "C" {
    pub fn invoke(
        name_ptr: *const u8,
        name_len: usize,
        method_ptr: *const u8,
        method_len: usize,
        args_ptr: *const u8,
        args_len: usize,
        cb: unsafe extern "C" fn(*mut c_void, *const u8, usize),
        user_data: *mut c_void,
    );

    pub fn callback(
        ptr: *const u8,
        len: usize,
        cb: i64, // compiler error
        user_data: i64,
    );
}

pub(crate) fn invoke_callback<F>(name: &[u8], method: &[u8], args: Vec<u8>, mut f: F)
where
    F: FnMut(&[u8]),
{
    let user_data = &mut f as *mut _ as *mut c_void;

    unsafe {
        invoke(
            name.as_ptr(), name.len(),
            method.as_ptr(), method.len(),
            args.as_ptr(), args.len(),
            trampoline::<F>, user_data
        );
    };
}

#[no_mangle]
pub extern "C" fn call_invoke_callback_fn(
    ptr: *const u8,
    size: usize,
    cb: unsafe extern "C" fn(*mut c_void, *const u8, usize),
    user_data: *mut c_void,
) {
    unsafe { cb(user_data, ptr, size) }
}

#[no_mangle]
pub extern "C" fn set_instance_id(id: u64) -> bool {
    INSTANCE_ID.set(id).is_ok()
}

#[no_mangle]
pub extern "C" fn get_instance_id() -> u64 {
    *INSTANCE_ID.get().unwrap_or(&0u64)
}