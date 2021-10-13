#[tokio::main]
async fn main() {
    let t = try_it();
    println!("Hello, world!");
    t.await;
}

async fn add(x: i32, y: i32) -> i32 {
    println!("adding x: {} and y: {}", x, y);
    x + y
}

async fn try_it() {
    let answer = add(1, 2).await;
    println!("answer: {:?}", answer);
}

/*
fn try_sums_sync() {
    let nums = vec![1, 10, 100];
    let sums: Vec<i32> = nums.into_iter().map(|n| n + 2).collect();
    println!("answer: {:?}", sums);
}

async fn try_sums() {
    let nums = vec![1, 10, 100];
    let sums: Vec<i32> = nums.into_iter().map(|n| add(n, 2).await).collect();
    println!("answer: {:?}", sums);
}
*/
