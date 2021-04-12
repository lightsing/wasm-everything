use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::{RefCell, Cell};
use core::future::Future;
use core::pin::Pin;
use core::task::{RawWaker, Waker, Context, Poll};

use super::runtime::Runtime;

#[derive(Clone)]
pub struct Task {
    inner: Rc<TaskInner>
}

struct TaskInner {
    inner: RefCell<Option<Inner>>,
    is_queued: Cell<bool>,
    runtime: Runtime,
}

struct Inner {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
    waker: Waker,
}

impl Task {
    fn new(runtime: Runtime) -> Self {
        Task::from(TaskInner {
            inner: RefCell::new(None),
            is_queued: Cell::new(false),
            runtime,
        })
    }
    pub fn spawn(future: Pin<Box<dyn Future<Output = ()> + 'static>>, runtime: Runtime) {
        let task = Self::new(runtime);
        let waker: Waker = task.clone().into();
        task.inner.inner.replace(Some(Inner { future, waker }));
        task.inner.wake_by_ref()
    }

    pub fn run(&self) {
        self.inner.run()
    }
}

impl TaskInner {
    fn wake_by_ref(self: &Rc<Self>) {
        if self.is_queued.replace(true) {
            return;
        }
        self.runtime.push_task(Task::from(self.clone()))
    }

    fn run(&self) {
        let mut borrow = self.inner.borrow_mut();

        let inner = match borrow.as_mut() {
            Some(inner) => inner,
            None => return,
        };

        self.is_queued.set(false);

        let poll = {
            let mut cx = Context::from_waker(&inner.waker);
            inner.future.as_mut().poll(&mut cx)
        };

        if let Poll::Ready(()) = poll {
            *borrow = None;
        }
    }
}

impl From<TaskInner> for Task {
    fn from(inner: TaskInner) -> Self {
        Self { inner: Rc::new(inner) }
    }
}

impl From<Rc<TaskInner>> for Task {
    fn from(inner: Rc<TaskInner>) -> Self {
        Self { inner }
    }
}

impl Into<Waker> for Task {
    fn into(self) -> Waker {
        unsafe { Waker::from_raw(self.into()) }
    }
}

impl Into<RawWaker> for Task {
    fn into(self) -> RawWaker {
        use core::mem::ManuallyDrop;
        use core::task::RawWakerVTable;

        unsafe fn raw_clone(ptr: *const ()) -> RawWaker {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const TaskInner));
            Task::from((*ptr).clone()).into()
        }

        unsafe fn raw_wake(ptr: *const ()) {
            let ptr = Rc::from_raw(ptr as *const TaskInner);
            ptr.wake_by_ref()
        }

        unsafe fn raw_wake_by_ref(ptr: *const ()) {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const TaskInner));
            ptr.wake_by_ref()
        }

        unsafe fn raw_drop(ptr: *const ()) {
            drop(Rc::from_raw(ptr as *const TaskInner));
        }

        const VTABLE: RawWakerVTable =
            RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

        RawWaker::new(Rc::into_raw(self.inner) as *const (), &VTABLE)
    }
}