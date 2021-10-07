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

type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Res<()> {

    let response = reqwest::get(CRATES_IO).await?;
    let text = response.text().await?;
    println!("response text: {} bytes", text.len());
    if text.len() < 1024 {
        println!("response text: {}", text);
    }
    Ok(())
}

// #[tokio::main]
async fn other_main() -> Res<()> {
    let mut running = Vec::new();
    for site in SITES {
        running.push((site, first_url(site)));
    }

    for (site, handle) in running {
        println!("{}: {:?}", site, handle.await?);
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

const S: &'static str = r###"
<!doctype html>
<!--[if !(IE 8)&!(IE 9)]><!-->
<html data-19ax5a9jf="dingo" lang="en-us" class="a-no-js"><!--<![endif]-->
<head></head><body></body></html>
"###;

const S2: &'static str = r###"
<!DOCTYPE html>
<!--[if lt IE 9 ]> <html class="ie8"> <![endif]-->
<!--[if IE 9 ]> <html class="ie9"> <![endif]-->
<!--[if (gt IE 9)|!(IE)]><!--> <html> <!--<![endif]-->
"###;
