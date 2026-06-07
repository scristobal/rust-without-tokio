use std::collections::VecDeque;
use std::future::Future;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

pub fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(val) = fut.as_mut().poll(&mut cx) {
            return val;
        }
    }
}

pub struct Executor {
    tasks: VecDeque<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.tasks.push_back(Box::pin(future));
    }

    pub fn run(&mut self) {
        while let Some(task) = self.tasks.pop_front() {
            block_on(task);
        }
    }
}
