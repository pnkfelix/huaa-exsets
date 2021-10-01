const APACHE: &'static str = "https://www.apache.org/";
const AMAZON: &'static str = "https://www.amazon.com/";
const CRATES_IO: &'static str = "https://crates.io/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

const SITES: &'static [&'static str] = &[
    APACHE, AMAZON, CRATES_IO, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA
];

type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut running = Vec::new();
    for site in SITES {
        running.push((site, tokio::spawn(page_len(site))));
    }

    for (site, handle) in running {
        println!("{}: {}", site, handle.await??);
    }

    Ok(())
}

async fn page_len(site: &str) -> Res<usize> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    return Ok(text.len());
}
