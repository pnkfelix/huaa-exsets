const APACHE: &'static str = "https://www.apache.org/";
const AMAZON: &'static str = "https://www.amazon.com/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

type MyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let apache_handle = tokio::task::spawn(first_url(APACHE));
    let amazon_handle = tokio::task::spawn(first_url(AMAZON));
    let docsrs_handle = tokio::task::spawn(first_url(DOCS_RS));
    let mozilla_handle = tokio::task::spawn(first_url(MOZILLA));
    let rustlang_handle = tokio::task::spawn(first_url(RUST_LANG));
    let wikipedia_handle = tokio::task::spawn(first_url(WIKIPEDIA));

    dbg!(apache_handle.await??);
    dbg!(amazon_handle.await??);
    dbg!(docsrs_handle.await??);
    dbg!(mozilla_handle.await??);
    dbg!(rustlang_handle.await??);
    dbg!(wikipedia_handle.await??);

    Ok(())
}

async fn first_url(site: &str) -> Result<Option<Url>, MyError>
{
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    // println!("response text: {} bytes", text.len());
    first_url_in_text(&text)
}

use url::Url;
use scraper::{Html, Selector};

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
