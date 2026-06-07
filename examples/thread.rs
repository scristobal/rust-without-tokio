use std::time::Duration;

use rust_simple_executor::r#thread::{Executor, spawn_blocking};

async fn worker(name: &'static str) {
    println!("{name}: start");

    let message = spawn_blocking(move || {
        println!("{name}: start of blocking work");
        std::thread::sleep(Duration::from_millis(10));
        println!("{name}: end of blocking work");
        format!("blocking work done")
    })
    .await;
    println!("{name}: {message}");

    println!("{name}: end");
}

fn main() {
    let executor = Executor::new();

    executor.spawn(worker("task 1"));
    executor.spawn(worker("task 2"));

    executor.run();
}
