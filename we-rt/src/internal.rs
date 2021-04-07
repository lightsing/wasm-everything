use alloc::rc::Rc;
use core::cell::RefCell;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use alloc::vec::Vec;

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
}

unsafe extern "C" fn trampoline<F>(user_data: *mut c_void, ptr: *const u8, size: usize)
where
    F: FnMut(&[u8]),
{
    (*(user_data as *mut F))(alloc::slice::from_raw_parts(ptr, size))
}

#[derive(Debug, Clone)]
pub struct AsyncResult<T> {
    pub(crate) inner: Rc<RefCell<Inner<T>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Inner<T> {
    pub(crate) task: Option<Waker>,
    pub(crate) v: Option<MaybeTaken<T>>,
}

#[derive(Debug, Clone)]
pub enum MaybeTaken<T> {
    Taken,
    StillThere(T),
}

impl<T> Default for AsyncResult<T> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::default()))
        }
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