#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use core::ptr::null_mut;

pub type Result<T> = core::result::Result<T, error::Error>;

mod error;
mod internal;
mod mem;

pub fn invoke<N, M, A, R>(
    name: N,
    method: M,
    args: Vec<A>,
) -> Result<R>
where N: AsRef<str>, M: AsRef<str>, A: serde::Serialize, R: serde::de::DeserializeOwned
{
    let name_value = name.as_ref().as_bytes();
    let method_value = method.as_ref().as_bytes();

    let args_json = serde_json::to_string(&args)?;
    let args_value = args_json.as_bytes();


    unsafe {
        let ptr: *mut u8 = null_mut();
        let mut size: usize = 0;
        internal::invoke(
            name_value.as_ptr(), name_value.len(),
            method_value.as_ptr(), method_value.len(),
            args_value.as_ptr(), args_value.len(),
            ptr, &mut size as *mut usize
        );
    }
    Ok(serde_json::from_str(r#"{"bar": 1}"#)?)
}
