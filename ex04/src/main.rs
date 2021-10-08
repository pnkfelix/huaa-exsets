// Computes factorial of n, printing progress along the way.
async fn fact(n: u32) -> f64 {
    let mut i = 0_u32;
    let mut accum = 1_f64;
    loop {
        println!("i: {I} fact_{N}({I}): {A}", N=n, I=i, A=accum);
        if i == n { break; }
        i += 1;
        accum *= i as f64;
    }
    return accum;
}

#[tokio::main]
async fn main() {
    let answer = tokio::select! {
        fact_5 = fact(5) => fact_5,
        fact_10 = fact(10) => fact_10,
    };
    println!("answer: {}", answer);
}
