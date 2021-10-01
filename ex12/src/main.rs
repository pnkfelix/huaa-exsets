use tokio::sync::mpsc::{channel, Receiver, Sender};

type Site = &'static str;

const APACHE: Site = "https://www.apache.org/";
const AMAZON: Site = "https://www.amazon.com/";
const CRATES_IO: Site = "https://crates.io/";
const DOCS_RS: Site = "https://docs.rs/";
const MOZILLA: Site = "https://www.mozilla.org/";
const RUST_LANG: Site = "https://www.rust-lang.org/";
const WIKIPEDIA: Site = "http://www.wikipedia.org/";

const SITES: &'static [Site] = &[
    APACHE, AMAZON, CRATES_IO, DOCS_RS, MOZILLA, RUST_LANG, WIKIPEDIA
];

type OpaqueError = Box<dyn std::error::Error + Send + Sync>;

type Res<T> = Result<T, OpaqueError>;

use crossterm::terminal;
use crossterm::ExecutableCommand;
use crossterm::cursor;

#[derive(Copy, Clone, Debug)]
struct SiteOrder {
    site: Site,
    pos: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel(12);
    let _ui_handle = tokio::spawn(ui_handler(rx));

    let mut running = Vec::new();
    for (j, site) in SITES.into_iter().enumerate() {
        let so = SiteOrder { site, pos: j };
        running.push((j, site, tokio::spawn(page_len(tx.clone(), so))));
        tx.send(Msg::SpawnedFetchSite(so)).await?;
    }

    tokio::try_join!(
        async {
            for (_, _, handle) in running {
                handle.await??;
            }
            Ok::<(), OpaqueError>(())
        },
    )?;

    println!("");

    // _ui_handle.await??;

    Ok(())
}

async fn page_len(tx: Sender<Msg>, so: SiteOrder) -> Res<usize> {
    let response = reqwest::get(so.site).await?;
    tx.send(Msg::GotFetchResponse(so)).await?;
    let text = response.text().await?;
    tx.send(Msg::GotSiteText { site: so, len_bytes: text.len() }).await?;
    return Ok(text.len());
}

#[derive(Debug)]
enum Msg {
    SpawnedFetchSite(SiteOrder),
    GotFetchResponse(SiteOrder),
    GotSiteText { site: SiteOrder, len_bytes: usize },
}

async fn ui_handler(mut msgs: Receiver<Msg>) -> Result<(), OpaqueError> {
    let mut stdout = std::io::stdout();

    // Clear all lines in terminal;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    let (_width, height) = terminal::size()?;

    loop {
        match msgs.recv().await {
            Some(Msg::SpawnedFetchSite(so)) => {
                stdout.execute(cursor::MoveTo(0, height - SITES.len() as u16 - 1 + so.pos as u16))?;
                print!("{}", so.site);
            }
            Some(Msg::GotFetchResponse(so)) => {
                stdout.execute(cursor::MoveTo(0, height - SITES.len() as u16 - 1 + so.pos as u16))?;
                print!("{}: ...", so.site);
            }
            Some(Msg::GotSiteText { site: so, len_bytes }) => {
                stdout.execute(cursor::MoveTo(0, height - SITES.len() as u16 - 1 + so.pos as u16))?;
                print!("{}: {:?}", so.site, len_bytes);
            }
            None => {
                // if no more messages then task is done.
                return Ok(());
            }
        }
    }
}
