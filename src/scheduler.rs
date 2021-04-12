use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

use wasmer::{Function, Val};
use std::ffi::c_void;


fn call<F>(function: &Function, mut callback: F) where F: FnMut(Vec<u8>) {
    let trampoline = trampoline::<F> as *mut c_void;
    let user_data = &mut callback as *mut _ as *mut c_void;
    function.call(&[
        Val::I64(trampoline as i64),
        Val::I64(user_data as i64)
    ]);
}

struct WasmFunctionExecution;

impl Future for WasmFunctionExecution {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}