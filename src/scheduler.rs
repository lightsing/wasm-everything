use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::ffi::c_void;

use wasmer::{Function, Val};
use semi_async::{trampoline, AsyncResult};
use std::marker::PhantomData;
use std::ops::Deref;
use serde::Deserialize;
use serde::de::DeserializeOwned;

type Result<T> = std::result::Result<T, crate::error::Error>;

fn call<F>(function: &Function, mut callback: F) where F: FnMut(Vec<u8>) {
    let trampoline = trampoline::<F> as *mut c_void;
    let user_data = &mut callback as *mut _ as *mut c_void;
    function.call(&[
        Val::I64(trampoline as i64),
        Val::I64(user_data as i64)
    ]);
}

pub struct WasmFunctionExecution<'a, T> {
    function: &'a Function,
    _return_type: PhantomData<T>
}

impl<'a, T> WasmFunctionExecution<'a, T> {
    pub fn new(function: &'a Function) -> Self {
        Self { function, _return_type: Default::default() }
    }

    pub fn call(&self) -> AsyncResult<Result<T>> where T: DeserializeOwned {
        let result = AsyncResult::default();
        let inner = result.clone_inner();
        call(self.function, move |data: Vec<u8>| {
            let mut inner = inner.deref().borrow_mut();
            inner.set_value(bincode::deserialize(&data).map_err(|e| e.into()));

            let task_op = inner.waker_ref();
            if task_op.is_some() {
                task_op.unwrap().wake_by_ref();
            };
        });
        result
    }
}