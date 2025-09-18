use super::Task;
use alloc::{collections::VecDeque};
use core::task::{RawWaker, RawWakerVTable, Waker, Context, Poll};

/// Simple task executor
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    // Creates a SimpleExecutor with no tasks.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor { task_queue: VecDeque::new() }
    }

    // Spawn a new task.
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => { }                              // Task done
                Poll::Pending => self.task_queue.push_back(task),   // Push task back to the end of the queue
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // Define RawWaker clone, wake, and drop operations
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}