use core::{
    future::Future, pin::Pin, sync::atomic::{AtomicI64, AtomicU64, Ordering}, task::{Context, Poll}
};
pub mod simple_executor;
pub mod keyboard;
pub mod executor;
use alloc::{borrow::ToOwned, boxed::Box, collections::VecDeque};

pub struct Task {
    pub id:TaskId,
    pub future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            id:TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self,task:Task){
        self.task_queue.push_back(task);
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
struct TaskId(u64);

impl TaskId{
    fn new()->Self{
        static NEXT_ID:AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}