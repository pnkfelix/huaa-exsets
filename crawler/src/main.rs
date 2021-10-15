const AMAZON: &'static str = "https://www.amazon.com/";
const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";

type MyError = Box<dyn std::error::Error + Send + Sync>;

use tokio::sync::mpsc::{channel, Sender};

#[derive(Debug)]
struct Msg {
    site: Url,
    link: Url,
}

const MSG_BUF_SIZE: usize = 4;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let mut sites: Vec<Url> = Vec::new();
    for link in [AMAZON, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA] {
        sites.push(Url::parse(link)?);
    }

    crawl_sites(sites).await?;
    Ok(())
}

async fn crawl_sites(sites: impl IntoIterator<Item=Url>) -> Result<(), MyError> {
    let (tx, mut rx) = channel::<Msg>(MSG_BUF_SIZE);

    let mut site_handles = Vec::new();
    for site in sites {
        site_handles.push((site.clone(), tokio::task::spawn(all_urls(site.clone(), tx.clone()))));
    }

    drop(tx);

    while let Some(msg) = rx.recv().await {
        println!("site: {} => link: {}", msg.site, msg.link);
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

async fn all_urls(site: Url, tx: Sender<Msg>) -> Result<usize, MyError> {
    let response = reqwest::get(site.clone()).await?;
    let text = response.text().await?;
    // println!("response text: {} bytes", text.len());
    let urls = all_urls_in_text(&text)?;
    let count = urls.len();
    for url in urls {
        tx.send(Msg { site: site.clone(), link: url }).await?
    }
    Ok(count)
}

fn all_urls_in_text(text: &str) -> Result<Vec<Url>, MyError>
{
    let mut discovered = Vec::new();
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
        discovered.push(url);
    }
    Ok(discovered)
}
