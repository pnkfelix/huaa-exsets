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
const MAX_DEPTH: usize = 3;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    console_subscriber::init();

    let mut sites: Vec<Url> = Vec::new();
    for link in [APACHE, AMAZON, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA] {
        sites.push(Url::parse(link)?);
    }

    let mut todo = sites;
    let semaphore = Arc::new(Semaphore::new(15));
    let client = Client::new();
    for _depth in 0..MAX_DEPTH {
        todo = crawl_sites(todo.into_iter(), client.clone(), semaphore.clone()).await?;
    }

    Ok(())
}

use tokio::sync::Semaphore;
use std::sync::Arc;
use reqwest::Client;

async fn crawl_sites(sites: impl Iterator<Item=Url>, client: Client, semaphore: Arc<Semaphore>) -> Result<Vec<Url>, MyError> {
    use tokio::task::JoinHandle;

    let mut todo: Vec<Url> = Vec::new();

    let mut site_handles: Vec<(Url, JoinHandle<_>)> = Vec::new();

    let (tx, mut rx) = channel::<Msg>(MSG_BUF_SIZE);
    let mut push_site = |site: Url| {
        site_handles.push((site.clone(), tokio::task::spawn(all_urls(site, tx.clone(), client.clone(), semaphore.clone()))));
    };

    for site in sites {
        push_site(site);
    }

    let recv_task = async {
        while let Some(msg) = rx.recv().await {
            println!("site: {} => link: {}", msg.site, msg.link);
            todo.push(msg.link);
        }
    };

    let join_task = async move {
        let mut sum = 0;
        for (_site, handle) in site_handles {
            sum += handle.await??;
        }
        let res: Result<usize, MyError> = Ok(sum);
        res
    };

    tokio::select! {
        () = recv_task => {}
        res = join_task => {
            dbg!(res?);
        }
    }

    Ok(todo)
}

async fn all_urls(site: Url, tx: Sender<Msg>, client: Client, semaphore: Arc<Semaphore>) -> Result<usize, MyError>
{
    let text = {
        let _permit = semaphore.acquire().await?;
        let response = client.get(site.clone()).send().await?;
        let text = response.text().await?;
        text
    };
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

