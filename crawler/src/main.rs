const DOCS_RS: &'static str = "https://docs.rs/";
const MOZILLA: &'static str = "https://www.mozilla.org/";
const RUST_LANG: &'static str = "https://www.rust-lang.org/";
const WIKIPEDIA: &'static str = "http://www.wikipedia.org/";
// const XML: &'static str = "https://www.w3.org/TR/2008/REC-xml-20081126/";

type MyError = Box<dyn std::error::Error + Send + Sync>;

use tokio::sync::mpsc::{channel, Sender};

#[derive(Debug)]
struct Msg {
    #[allow(dead_code)]
    site: Url,
    link: Url,
}

const MSG_BUF_SIZE: usize = 4;

const MAX_DEPTH: usize = 4;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .event_buffer_capacity(1024 * 1024 * 10)
        .client_buffer_capacity(1024 * 1024 * 4)
        .init();
/*
    tracing_subscriber::fmt()
        // line below parses directives from `RUST_LOG` environment variable.
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // line below makes the tracing output go to stderr.
        .with_writer(std::io::stderr)
        // line below makes this *the* subscriber for all tracing in this app.
        .init();
*/
    let mut sites: Vec<Url> = Vec::new();
    for link in [DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA] {
        sites.push(Url::parse(link)?);
    }

    let mut todo = sites;
    for _depth in 0..MAX_DEPTH {
        let mut next_todo = Vec::new();
        while todo.len() > 0 {
            let mut prefix = Vec::new();
            for _ in 0..std::cmp::min(todo.len(), 30) {
                prefix.push(todo.pop().unwrap());
            }
            let next_todo_ = crawl_sites(prefix).await?;
            next_todo.extend(next_todo_);
        }
        todo = next_todo;
    }

    Ok(())
}

use std::sync::Arc;
use tokio::sync::Semaphore;

async fn crawl_sites(sites: impl IntoIterator<Item=Url>) -> Result<Vec<Url>, MyError> {
    let mut discovered: Vec<Url> = Vec::new();

    let parse_sem = Arc::new(Semaphore::new(6));
    let mut sites_len = 0;
    let sites = sites.into_iter();
    let (tx, mut rx) = channel::<Msg>(MSG_BUF_SIZE);
    let mut site_handles = Vec::new();
    for site in sites {
        let tx = tx.clone();
        let parse_sem = parse_sem.clone();
        let name = format!("crawl {}", site.domain().unwrap_or("no-domain"));
        site_handles.push((site.clone(), tokio::task::Builder::default()
                           .name(&name)
                           .spawn(async {
                               let res = all_urls(site, tx, parse_sem).await;
                               if res.is_ok() {
                                   tracing::trace!("crawler result: {:?}", res);
                               } else {
                                   tracing::error!("crawler error: {:?}", res);
                               }
                               res
                           })));
    }
    drop(tx);

    while let Some(msg) = rx.recv().await {
        print!(".");
        // dbg!(format!("site: {} => link: {}", msg.site, msg.link));
        discovered.push(msg.link);
    }
    println!("");

    for (site, handle) in site_handles {
        if let Err(site_err) = handle.await? {
            dbg!((site, site_err));
        } else {
            sites_len += 1;
        }
    }

    println!("processed {} sites, discovered {}", sites_len, discovered.len());
    Ok(discovered)
}

async fn _always_err() -> Result<(), MyError> {
    Err("demo-err".into())
}

use url::Url;
use scraper::{Html, Selector};

async fn all_urls(site: Url, tx: Sender<Msg>, sem: Arc<Semaphore>) -> Result<usize, MyError> {
    let _permit = sem.acquire().await;

    let name = format!("reqwest {}", site.domain().unwrap_or("no-domain"));
    let site2 = site.clone();
    let text = tokio::task::Builder::default()
        .name(&name)
        .spawn(async {
            let response = reqwest::get(site2).await?;
            response.text().await
        }).await??;

    let name = format!("parse {}", site.domain().unwrap_or("no-domain"));
    let urls = tokio::task::Builder::default()
        .name(&name)
        .spawn(async move { all_urls_in_text(&text) }).await??;

    let name = format!("send-urls {}", site.domain().unwrap_or("no-domain"));
    let count = urls.len();
    tokio::task::Builder::default()
        .name(&name)
        .spawn(async move {
            for url in urls {
                tx.send(Msg { site: site.clone(), link: url }).await?
            }
            Ok::<(), tokio::sync::mpsc::error::SendError<Msg>>(())
        })
        .await??;
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
