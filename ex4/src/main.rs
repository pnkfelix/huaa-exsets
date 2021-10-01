type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Res<()> {
    dbg!(page_len("http://www.wikipedia.org/").await?);
    Ok(())
}

async fn page_len(site: &str) -> Res<usize> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    return Ok(text.len());
}
