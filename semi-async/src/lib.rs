extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
#[cfg(not(target_arch="wasm32"))]
use std::sync::Arc;

#[cfg(target_arch="wasm32")]
pub mod rt;
#[cfg(target_arch="wasm32")]
pub mod wasm_callback;
#[cfg(not(target_arch="wasm32"))]
pub mod host_callback;

#[cfg(target_arch="wasm32")]
pub use rt::runtime::Runtime;
#[cfg(target_arch="wasm32")]
pub use wasm_callback::trampoline;
#[cfg(not(target_arch="wasm32"))]
pub use host_callback::trampoline;


#[cfg(target_arch="wasm32")]
pub type AsyncResultInner<T> = Rc<RefCell<Inner<T>>>;
#[cfg(not(target_arch="wasm32"))]
pub type AsyncResultInner<T> = Arc<RefCell<Inner<T>>>;

#[derive(Debug, Clone)]
pub struct AsyncResult<T> {
    inner: AsyncResultInner<T>,
}

#[derive(Debug, Clone)]
pub struct Inner<T> {
    task: Option<Waker>,
    v: Option<MaybeTaken<T>>,
}

#[derive(Debug, Clone)]
pub enum MaybeTaken<T> {
    Taken,
    StillThere(T),
}

impl<T> Default for AsyncResult<T> {
    fn default() -> Self {
        #[cfg(target_arch="wasm32")]
        return Self {
            inner: Rc::new(RefCell::new(Inner::default()))
        };
        #[cfg(not(target_arch="wasm32"))]
        return Self {
            inner: Arc::new(RefCell::new(Inner::default()))
        };
    }
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            task: None,
            v: None
        }
    }
}

impl<T> Default for MaybeTaken<T> {
    fn default() -> Self {
        Self::Taken
    }
}

impl <T> Future for AsyncResult<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();

        if inner.v.is_some() {
            match inner.v.replace(MaybeTaken::Taken).unwrap() {
                MaybeTaken::Taken => panic!("AsyncResult got poll after Ready"),
                MaybeTaken::StillThere(v) => return Poll::Ready(v),
            }
        }

        inner.task = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl<T> AsyncResult<T> {
    pub fn clone_inner(&self) -> AsyncResultInner<T> {
        self.inner.clone()
    }
}

impl<T> Inner<T> {
    pub fn set_value(&mut self, value: T) {
        self.v = Some(MaybeTaken::StillThere(value));
    }

    pub fn waker_ref(&self) -> Option<&Waker> {
        self.task.as_ref()
    }
}