const APACHE: &'static str = "https://www.apache.org/";
const AMAZON: &'static str = "https://www.amazon.com/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

const SITES: &'static [&'static str] = &[
    APACHE, AMAZON, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA
];

type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Res<()> {
    tokio::select! {
        local_url = first_url("https://127.0.0.1/") => { dbg!(local_url?); }
        apache_url = first_url(APACHE) => { dbg!(apache_url?); }
        amazon_url = first_url(AMAZON) => { dbg!(amazon_url?); }
        docsrs_url = first_url(DOCS_RS) => { dbg!(docsrs_url?); }
        mozilla_url = first_url(MOZILLA) => { dbg!(mozilla_url?); }
        rustlang_url = first_url(RUST_LANG) => { dbg!(rustlang_url?); }
        wikipedia_url = first_url(WIKIPEDIA) => { dbg!(wikipedia_url?); }
    }

    Ok(())
}

use scraper::{Html, Selector};
use url::Url;
use std::collections::{HashMap, VecDeque};

// Name your user agent after your app?
static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

async fn first_url(site: &str) -> Res<Option<Url>> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let response = client.get(site).send().await?;
    let text = response.text().await?;
    println!("site: {} response text: {} bytes", site, text.len());
    if text.len() < 1024 {
        println!("site: {} response text: \n```\n{}\n```", site, text);
    }
    first_url_in_text(&text)
}

fn first_url_in_text(text: &str) -> Res<Option<Url>>
{
    let doc = Html::parse_document(&text);
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
    println!("text: ```\n{}\n```", text);
    return Ok(None);
}
