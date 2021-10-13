const AMAZON: &'static str = "https://www.amazon.com/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::select! {
        Ok(err) = always_err() => { panic!("should never happen"); }
        Ok(amazon_url) = first_url(AMAZON) => { dbg!(amazon_url); }
        Ok(docsrs_url) = first_url(DOCS_RS) => { dbg!(docsrs_url); }
        Ok(mozilla_url) = first_url(MOZILLA) => { dbg!(mozilla_url); }
        Ok(rustlang_url) = first_url(RUST_LANG) => { dbg!(rustlang_url); }
        Ok(wikipedia_url) = first_url(WIKIPEDIA) => { dbg!(wikipedia_url); }
    }
    Ok(())
}

async fn always_err() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Error, ErrorKind};
    Err(Box::new(Error::new(ErrorKind::Other, "demo-err")) as Box<_>)
}

use url::Url;
use scraper::{Html, Selector};

async fn first_url(site: &str) -> Result<Option<Url>, Box<dyn std::error::Error>> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    first_url_in_text(&text)
}

fn first_url_in_text(text: &str) -> Result<Option<Url>, Box<dyn std::error::Error>>
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
