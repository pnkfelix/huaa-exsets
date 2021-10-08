#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("http://www.wikipedia.org/").await?;
    println!("response text: {} bytes", response.text().await?.len());
    Ok(())
}

