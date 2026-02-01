use rust_simple_executor::single::block_on;

async fn compute() -> i32 {
    let a = async { 10 }.await;
    let b = async { 20 }.await;
    a + b
}

fn main() {
    let result = block_on(compute());
    assert_eq!(result, 30)
}
