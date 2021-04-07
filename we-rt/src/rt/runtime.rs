use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use core::cell::{RefCell, Cell};
use core::future::Future;

use super::task::Task;

#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner>>
}

struct RuntimeInner {
    tasks: RefCell<VecDeque<Task>>,
    is_spinning: Cell<bool>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RuntimeInner {
                tasks: RefCell::new(VecDeque::new()),
                is_spinning: Cell::new(false)
            }))
        }
    }

    pub fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + 'static,
    {
        super::task::Task::spawn(Box::pin(future), self.clone());
    }

    pub(crate) fn push_task(&self, task: Task) {
        self.inner.borrow().push_task(task)
    }
}

impl RuntimeInner {
    pub(crate) fn push_task(&self, task: Task) {
        self.tasks.borrow_mut().push_back(task);

        if !self.is_spinning.replace(true) {
            self.run_all()
        }
    }

    fn run_all(&self) {
        loop {
            let task = match self.tasks.borrow_mut().pop_front() {
                Some(task) => task,
                None => break,
            };
            task.run();
        }
        self.is_spinning.set(false);
    }
}