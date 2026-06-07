use std::collections::VecDeque;
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

#[derive(Clone)]
pub struct Executor {
    queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

struct SimpleWaker {
    task: Arc<Task>,
    executor: Executor,
}

impl Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        self.executor.schedule(self.task.clone());
    }
}

impl Task {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> Arc<Self> {
        Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
        })
    }

    fn poll(&self, cx: &mut Context) -> Poll<()> {
        self.future.lock().unwrap().as_mut().poll(cx)
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn schedule(&self, task: Arc<Task>) {
        self.queue.lock().unwrap().push_back(task);
    }

    fn unschedule(&self) -> Option<Arc<Task>> {
        self.queue.lock().unwrap().pop_front()
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.schedule(Task::new(future));
    }

    pub fn run(&self) {
        loop {
            let Some(task) = self.unschedule() else { break };

            let waker = Waker::from(Arc::new(SimpleWaker {
                task: task.clone(),
                executor: self.clone(),
            }));
            let mut cx = Context::from_waker(&waker);
            let _ = task.poll(&mut cx);
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
