#![no_std]
extern crate alloc;

use core::ops::Deref;

pub use semi_async::{AsyncResult, Runtime, MaybeTaken};
pub use we_logger::init as init_logger;

pub use crate::internal::HostCallback;
use crate::internal::invoke_callback;

pub type Result<T> = core::result::Result<T, error::Error>;

mod error;
mod internal;
mod mem;

pub fn invoke<N, M, A, R>(name: N, method: M, args: A) -> AsyncResult<Result<R>>
where
    N: AsRef<str>,
    M: AsRef<str>,
    A: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let result = AsyncResult::default();
    let inner = result.clone_inner();

    match bincode::serialize(&args) {
        Ok(args_value) => invoke_callback(name.as_ref().as_bytes(), method.as_ref().as_bytes(), args_value,move |data: &[u8]| {
            let mut inner = inner.deref().borrow_mut();
            inner.set_value(bincode::deserialize(data).map_err(|e| e.into()));

            let task_op = inner.waker_ref();
            if task_op.is_some() {
                task_op.unwrap().wake_by_ref();
            };
        }),
        Err(e) => inner.deref().borrow_mut().set_value(Err(e.into()))
    };

    result
}

pub fn callback(data: &[u8], cb: i64, user_data: i64) {
    unsafe {
        internal::callback(
            data.as_ptr(),
            data.len(),
            cb,
            user_data
        )
    }
}
