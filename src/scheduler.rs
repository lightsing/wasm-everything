use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

struct WasmModuleExecution;

impl Future for WasmModuleExecution {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}