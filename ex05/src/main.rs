#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dbg!(first_url("http://www.wikipedia.org/").await?);
    Ok(())
}

async fn first_url(site: &str) -> Result<Option<Url>, Box<dyn std::error::Error>>
{
    let response = reqwest::get(site).await?;
    let text = response.text().await?;
    // println!("response text: {} bytes", text.len());
    first_url_in_text(&text)
}

use url::Url;
use scraper::{Html, Selector};

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
