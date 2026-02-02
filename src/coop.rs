use std::collections::VecDeque;
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

struct SimpleWaker {
    task: Arc<Task>,
    executor: Executor,
}

impl Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        self.executor.queue.lock().unwrap().push_back(self.task.clone());
    }
}

#[derive(Clone)]
pub struct Executor {
    queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.queue.lock().unwrap().push_back(Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
        }));
    }

    pub fn run(&self) {
        loop {
            let task = self.queue.lock().unwrap().pop_front();
            let Some(task) = task else { break };

            let waker = Waker::from(Arc::new(SimpleWaker {
                task: task.clone(),
                executor: self.clone(),
            }));
            let mut cx = Context::from_waker(&waker);
            let _ = task.future.lock().unwrap().as_mut().poll(&mut cx);
        }
    }
}

pub fn yield_now() -> impl Future<Output = ()> {
    let mut yielded = false;
    poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
}
