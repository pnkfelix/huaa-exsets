#[tokio::main]
async fn main() {
    println!("Hello, world!");

    assert_eq!(double(4).await, 8);
}

async fn double(x: u32) -> u32 {
    x * 2
}

#[tokio::test]
async fn double_four() {
    assert_eq!(double(4).await, 8);
}
