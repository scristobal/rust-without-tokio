use rust_simple_executor::r#yield::{Executor, yield_now};

async fn worker(name: &'static str) {
    println!("{name}: start");
    yield_now().await;

    println!("{name}: middle");
    yield_now().await;

    println!("{name}: end");
}

fn main() {
    let executor = Executor::new();

    executor.spawn(worker("task 1"));
    executor.spawn(worker("task 2"));

    executor.run();
}
