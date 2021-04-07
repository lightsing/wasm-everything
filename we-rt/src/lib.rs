#![no_std]
extern crate alloc;

pub use we_logger::init as init_logger;

pub type Result<T> = core::result::Result<T, error::Error>;

mod error;
mod internal;
mod mem;
mod rt;

pub use crate::rt::runtime::Runtime;
pub use crate::internal::AsyncResult;
use crate::internal::{invoke_callback, MaybeTaken};
use core::ops::Deref;

pub fn invoke<N, M, A, R>(name: N, method: M, args: A) -> AsyncResult<Result<R>>
where
    N: AsRef<str>,
    M: AsRef<str>,
    A: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let result = AsyncResult::default();
    let inner = result.inner.clone();

    match bincode::serialize(&args) {
        Ok(args_value) => invoke_callback(name.as_ref().as_bytes(), method.as_ref().as_bytes(), args_value,move |data: &[u8]| {
            let mut inner = inner.deref().borrow_mut();
            inner.v = Some(MaybeTaken::StillThere(bincode::deserialize(data).map_err(|e| e.into())));

            let task_op = inner.task.as_ref();
            if task_op.is_some() {
                task_op.unwrap().wake_by_ref();
            };
        }),
        Err(e) => inner.deref().borrow_mut().v = Some(MaybeTaken::StillThere(Err(e.into())))
    };

    result
}
