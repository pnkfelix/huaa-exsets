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

type Res<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn delay<T>(x: T) -> T { x }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    for (race_num, chunk) in SITES.chunks(3).enumerate() {
        let site0 = chunk.get(0);
        let site1 = chunk.get(1);
        let site2 = chunk.get(2);
        dbg!((site0, site1, site2));
        match (site0, site1, site2) {
            (Some(site0), Some(site1), Some(site2)) => {
                tokio::select! {
                    Ok(Some(url0)) = first_url(site0) => {
                        println!("site0 won race {}, {} {:?}", race_num, site0, url0);
                    }
                    Ok(Some(url1)) = first_url(site1) => {
                        println!("site1 won race {}, {} {:?}", race_num, site1, url1);
                    }
                    Ok(Some(url2)) = first_url(site2) => {
                        println!("site2 won race {}, {} {:?}", race_num, site2, url2);
                    }
                }
            }
            (Some(site0), Some(site1), None) => {
                tokio::select! {
                    Ok(Some(url0)) = first_url(site0) => {
                        println!("site0 won race {}, {} {:?}", race_num, site0, url0);
                    }
                    Ok(Some(url1)) = first_url(site1) => {
                        println!("site1 won race {}, {} {:?}", race_num, site1, url1);
                    }
                }
            }
            (Some(site0), None, None) => {
                tokio::select! {
                    Ok(Some(url0)) = first_url(site0) => {
                        println!("site0 ran alone race {}, {} {:?}", race_num, site0, url0);
                    }
                }
            }
            _ => panic!("should be impossible."),
        }
    }
    let mut running = Vec::new();
    for site in SITES {
        running.push((site, tokio::spawn(first_url(site))));
    }

    for (site, handle) in running {
        println!("{}: {:?}", site, handle.await??);
    }

    Ok(())
}

use html_parser::{Dom, Node};
use url::Url;
use std::collections::{HashMap, VecDeque};

async fn first_url(site: &str) -> Res<Option<Url>> {
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    let dom = Dom::parse(&text)?;
    let mut first_url = None;
    let mut node_todo: VecDeque<Node> = dom.children.into();
    while let Some(node) = node_todo.pop_front() {
        let attrs: HashMap<String, Option<String>> = match node {
            Node::Text(_) | Node::Comment(_) =>
                continue,

            Node::Element(elem) => {
                // node_todo.extend(elem.children.iter().cloned());
                for child in elem.children.iter().cloned().rev() {
                    node_todo.push_front(child);
                }

                if elem.name.as_str() == "a" {
                    elem.attributes
                } else {
                    continue;
                }
            }
        };

        let href = attrs.get("href");
        let href = match href {
            Some(Some(h)) => h,
            Some(None) | None => continue,
        };

        let url = match Url::parse(href) {
            Ok(u) => u,
            Err(_) => continue,
        };

        first_url = Some(url);
        break;
    }
    return Ok(first_url);
}
