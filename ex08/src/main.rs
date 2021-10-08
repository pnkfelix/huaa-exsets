const APACHE: &'static str = "https://www.apache.org/";
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
    use tokio::task::JoinHandle;

    let mut site_handles: Vec<(&'static str, JoinHandle<_>)> = Vec::new();

    let (tx, mut rx) = channel::<Msg>(MSG_BUF_SIZE);
    let mut push_site = |site| {
        site_handles.push((site, tokio::task::spawn(all_urls(site, tx.clone()))));
    };

    for site in [APACHE, AMAZON, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA] {
        push_site(site);
    }

    let recv_task = async move {
        while let Some(msg) = rx.recv().await {
            println!("site: {} => link: {}", msg.site, msg.link);
        }
    };

    let join_task = async move {
        for (site, handle) in site_handles {
            dbg!((site, handle.await.unwrap().unwrap()));
        }
    };

    tokio::select! {
        () = recv_task => {}
        () = join_task => {}
    }

    Ok(())
}

async fn all_urls(site: &str, tx: Sender<Msg>) -> Result<usize, MyError>
{
    let response = reqwest::get(site).await?;
    let site = Url::parse(site)?;
    let text = response.text().await?;
    // println!("response text: {} bytes", text.len());
    let urls = all_urls_in_text(&text)?;
    let count = urls.len();
    for url in urls {
        tx.send(Msg { site: site.clone(), link: url }).await?
    }
    Ok(count)
}

use url::Url;
use scraper::{Html, Selector};

fn all_urls_in_text(text: &str) -> Result<Vec<Url>, MyError>
{
    let mut urls = Vec::new();
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
        urls.push(url);
    }
    return Ok(urls);
}

