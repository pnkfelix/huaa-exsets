use tokio::time;
use tokio::sync::mpsc::{channel, Receiver, Sender};

type OpaqueError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), OpaqueError> {
    println!("Hello, world!");

    let (tx1, mut rx1) = channel(4);
    let (tx2, mut rx2) = channel(4);
    let (tx3, mut rx3) = channel(4);
    let (tx4, mut rx4) = channel(4);

    tx1.send(1_u128).await?;
    tx1.send(1).await?;

    tokio::select! {
        // biased;
        eek = async {
            loop {
                let r = rx1.recv().await;
                let v = match r {
                    Some(v) => v,
                    None => break Box::new(Whoops) as OpaqueError,
                };
                match tx2.send(v).await {
                    Ok(()) => {}
                    Err(err) => break Box::new(err) as OpaqueError,
                }
                match tx3.send(v).await {
                    Ok(()) => {}
                    Err(err) => break Box::new(err) as OpaqueError,
                }
                match tx4.send(v).await {
                    Ok(()) => {}
                    Err(err) => break Box::new(err) as OpaqueError,
                }
            }
        } => { return Err(eek); }

        x = async {
            loop {
                let r = rx4.recv().await;
                let v = match r {
                    Some(v) => v,
                    None => break Err(Box::new(Whoops) as OpaqueError),
                };
                if v >= 1000 {
                    break Ok(v);
                }
            }
        } => {
            println!("it worked! x: {}", x?);
        }

        _ = async {
            let _ = rx3.recv().await; // discard first element.
            sum_chan(rx2, rx3, tx1).await;
        } => {}
    }

    Ok(())
}

async fn sum_chan(mut rx_a: Receiver<u128>, mut rx_b: Receiver<u128>, tx: Sender<u128>) -> Result<(), OpaqueError> {
    loop {
        let v1 = rx_a.recv().await;
        let v2 = rx_b.recv().await;
        let v3 = match (v1, v2) {
            (Some(v1), Some(v2)) => v1 + v2,
            (None, _) |
            (Some(_), None) => return Err(Box::new(Whoops)),
        };
        tx.send(v3).await?;
    }
}

#[derive(Debug)]
struct Whoops;
impl std::fmt::Display for Whoops {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "whoops")
    }
}

impl std::error::Error for Whoops {

}
