use rust_simple_executor::coop::{Executor, yield_now};

async fn compute() -> i32 {
    let a = async { 10 }.await;
    yield_now().await;
    let b = async { 20 }.await;
    a + b
}

fn main() {
    let executor = Executor::new();

    executor.spawn(async {
        let result = compute().await;
        assert_eq!(result, 30);
    });

    executor.run();
}
