#[tokio::main]
async fn main() {
    println!("Hello, world!");
    try_it().await;
}

async fn add(x: i32, y: i32) -> i32 {
    println!("adding x: {} and y: {}", x, y);
    x + y
}

async fn try_it() {
    let answer = add(1, 2).await;
    println!("answer: {:?}", answer);
}
