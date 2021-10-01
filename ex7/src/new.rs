const APACHE: &'static str = "https://www.apache.org/";
const AMAZON: &'static str = "https://www.amazon.com/";
const CRATES_IO: &'static str = "https://crates.io/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const PLAY: &'static str = "https://play.rust-lang.org/";
const R_RUST: &'static str = "https://www.reddit.com/r/rust/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

const SITES: &'static [&'static str] = &[
    APACHE, AMAZON, CRATES_IO,
    DOCS_RS, MOZILLA, PLAY,
    R_RUST, RUST_LANG, WIKIPEDIA
];

type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let mut running = Vec::new();
    for site in SITES {
        running.push((site, tokio::spawn(page_len(site))));
    }

    for chunk in running.chunks(3) {
        let site0 = chunk.get(0);
        let site1 = chunk.get(1);
        let site2 = chunk.get(2);
        tokio::select! {
            len0 = page_len(site0.unwrap())? if site0.is_some() => {
                printkn!("site0 won the race, {} {} bytes", site0, len0);
            }
            len1 = page_len(site1.unwrap())? if site1.is_some() => {
                printkn!("site1 won the race, {} {} bytes", site1, len1);
            }
            len2 = page_len(site2.unwrap())? if site2.is_some() => {
                printkn!("site2 won the race, {} {} bytes", site2, len2);
            }
        }
    }

    Ok(())
}

async fn page_len(site: &str) -> Res<usize> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    return Ok(text.len());
}
