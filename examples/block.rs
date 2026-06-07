use rust_simple_executor::single::Executor;

async fn worker(name: &'static str) {
    println!("{name}: exec");
}

fn main() {
    let mut executor = Executor::new();

    executor.spawn(worker("task 1"));
    executor.spawn(worker("task 2"));

    executor.run();
}
