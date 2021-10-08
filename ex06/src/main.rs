// Computes factorial of n, printing progress along the way.
async fn fact(n: u32) -> f64 {
    let mut i = 0_u32;
    let mut accum = 1_f64;
    loop {
        println!("i: {I} fact_{N}({I}): {A}", N=n, I=i, A=accum);
        if i == n { break; }
        tokio::task::yield_now().await;
        i += 1;
        accum *= i as f64;
    }
    return accum;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fact_5 = tokio::task::spawn(fact(5));
    let fact_10 =  tokio::task::spawn(fact(10));
    dbg!((fact_5.await?, fact_10));
    Ok(())
}
