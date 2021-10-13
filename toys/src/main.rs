#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

async fn add(x: i32, y: i32) -> i32 {
    println!("adding x: {} and y: {}", x, y);
    x + y
}

#[cfg(does_not_compile)]
fn try_it() {
    let answer = add(1, 2);
    println!("answer: {:?}", answer);
}
