use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::println;

use super::SimpleExecutor;

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    RawWaker::new(0 as *const (), vtable)
}

impl SimpleExecutor {
    pub fn run(&mut self) {
        println!("Starting executor run");
        while let Some(mut task) = self.task_queue.pop_front() {
            // println!("Polling a task");
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // println!("Task completed");
                } // task done
                Poll::Pending => {
                    // println!("Task pending, pushing back to queue");
                    self.task_queue.push_back(task)
                }
            }
        }
        println!("Executor finished");
    }
}
