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

    terminal::enable_raw_mode()?;
    _ui_handle.await??;
    terminal::disable_raw_mode()?;

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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crossterm::event::EventStream;
    use futures_util::stream::StreamExt;

    let mut reader = EventStream::new();
    let mut stdout = std::io::stdout();


    // Clear all lines in terminal;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    let (width, height) = terminal::size()?;
    let _width = AtomicUsize::new(width as usize);
    let height = AtomicUsize::new(height as usize);

    let move_to = |offset: i16| -> Result<(), OpaqueError> {
        let mut stdout = std::io::stdout();
        let row = (height.load(Ordering::SeqCst) - SITES.len() - 1) as isize + offset as isize;
        stdout.execute(cursor::MoveTo(0, row as u16))?;
        Ok(())
    };

    loop {
        tokio::select! {
            Some(input) = reader.next() => {
                move_to(-1)?;
                println!("input: {:?}", input);

                match input? {
                    crossterm::event::Event::Key(crossterm::event::KeyEvent { code: _, modifiers }) => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            // for now, *any* control modified keystroke causes this to exit
                            return Ok(());
                        }
                    }
                    crossterm::event::Event::Mouse(_) => {}
                    crossterm::event::Event::Resize(w, h) => {
                        _width.store(w as usize, Ordering::SeqCst);
                        height.store(h as usize, Ordering::SeqCst);
                    }
                }
            }

            progress = msgs.recv() => {
                match progress {
                    Some(Msg::SpawnedFetchSite(so)) => {
                        move_to(so.pos as i16)?;
                        print!("{}", so.site);
                    }
                    Some(Msg::GotFetchResponse(so)) => {
                        move_to(so.pos as i16)?;
                        print!("{}: ...", so.site);
                    }
                    Some(Msg::GotSiteText { site: so, len_bytes }) => {
                        move_to(so.pos as i16)?;
                        print!("{}: {:?}", so.site, len_bytes);
                    }
                    None => {
                        // if no more messages then task is done.
                        return Ok(());
                    }
                }
            }
        }
    }
}
