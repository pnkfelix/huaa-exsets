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
    let nums = vec![1, 10, 100];
    let mut sums = Vec::with_capacity(nums.len());
    for s in nums.into_iter().map(|n| add(n, 2)) {
       sums.push(s.await);
    }
    println!("answer: {:?}", sums);
}
