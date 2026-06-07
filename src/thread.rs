use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Wake, Waker};

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

struct State {
    queue: VecDeque<Arc<Task>>,
    active_tasks: usize,
}

struct SimpleWaker {
    task: Arc<Task>,
    executor: Executor,
}

impl Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        let mut state = self.executor.state.lock().unwrap();
        state.queue.push_back(self.task.clone());
        self.executor.ready.notify_one();
    }
}

#[derive(Clone)]
pub struct Executor {
    state: Arc<Mutex<State>>,
    ready: Arc<Condvar>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            state: Arc::new(Mutex::new(State {
                queue: VecDeque::new(),
                active_tasks: 0,
            })),
            ready: Arc::new(Condvar::new()),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let mut state = self.state.lock().unwrap();
        state.active_tasks += 1;
        state.queue.push_back(Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
        }));
        self.ready.notify_one();
    }

    pub fn run(&self) {
        loop {
            let task = {
                let mut state = self.state.lock().unwrap();
                loop {
                    if let Some(task) = state.queue.pop_front() {
                        break task;
                    }

                    if state.active_tasks == 0 {
                        return;
                    }

                    state = self.ready.wait(state).unwrap();
                }
            };

            let waker = Waker::from(Arc::new(SimpleWaker {
                task: task.clone(),
                executor: self.clone(),
            }));
            let mut cx = Context::from_waker(&waker);
            let mut future_slot = task.future.lock().unwrap();
            let Some(future) = future_slot.as_mut() else {
                continue;
            };

            if future.as_mut().poll(&mut cx).is_ready() {
                *future_slot = None;

                let mut state = self.state.lock().unwrap();
                state.active_tasks -= 1;
                if state.active_tasks == 0 {
                    self.ready.notify_all();
                }
            }
        }
    }
}

struct BlockingState<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

pub struct Blocking<T> {
    state: Arc<Mutex<BlockingState<T>>>,
}

impl<T> Future for Blocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn spawn_blocking<F, T>(f: F) -> Blocking<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let state = Arc::new(Mutex::new(BlockingState {
        result: None,
        waker: None,
    }));
    let thread_state = state.clone();

    std::thread::spawn(move || {
        let result = f();
        let waker = {
            let mut state = thread_state.lock().unwrap();
            state.result = Some(result);
            state.waker.take()
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    });

    Blocking { state }
}
