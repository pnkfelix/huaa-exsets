const AMAZON: &'static str = "https://www.amazon.com/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

type MyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    use tokio::task::JoinHandle;

    let mut site_handles: Vec<(&'static str, JoinHandle<_>)> = Vec::new();

    let sites = [AMAZON, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA];
    for site in sites {
        site_handles.push((site, tokio::task::spawn(first_url(site))));
    }

    for (site, handle) in site_handles {
        dbg!((site, handle.await??));
    }

    Ok(())
}

async fn _always_err() -> Result<(), MyError> {
    Err("demo-err".into())
}

use url::Url;
use scraper::{Html, Selector};

async fn first_url(site: &str) -> Result<Option<Url>, MyError> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    first_url_in_text(&text)
}

fn first_url_in_text(text: &str) -> Result<Option<Url>, MyError>
{
    let doc = Html::parse_document(&text);
    // (This unwrap should never fail; the input is a known constant.)
    let selector = Selector::parse("a")
        .unwrap_or_else(|err| panic!("failed to parse tag `a`: {:?}.", err));
    for element in doc.select(&selector) {
        let link = match element.value().attr("href") {
            Some(link) => link,
            None => continue,
        };
        let url = match Url::parse(link) {
            Ok(u) => u,
            Err(_) => continue,
        };
        return Ok(Some(url));
    }
    return Ok(None);
}
