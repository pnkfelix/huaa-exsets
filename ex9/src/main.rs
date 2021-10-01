// NOTE: these "cute" numberic-stream combiners may be more trouble then they are
// worth. It is subtle to get the logic right, and I am not sure whether I would
// be promoting ant-patterns.

use tokio::sync::broadcast::{self, Receiver, Sender};

type OpaqueError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), OpaqueError> {
    println!("Hello, world!");

    let (tx_ones, mut rx_ones) = broadcast::channel(8);
    let (mut tx, mut rx1) = broadcast::channel(8);

    let mut rx2 = tx.subscribe();
    tx.send(1_u128)?;

    // rx1.recv().await?; // offset by one

    loop {
        tokio::select! {
            biased;
            x = rx2.recv() => {
                println!("x: {:?}", x);
                let x = x?;
                if x >= 100 {
                    println!("it worked! x: {}", x);
                    break;
                }
            }
            _ = sum_chan(&mut rx1, &mut rx_ones, &mut tx) => {}
            _ = async { tx_ones.send(1)?;
                        tokio::time::sleep(tokio::time::Duration::from_secs(0)).await;
                        Ok::<(), broadcast::error::SendError<_>>(()) } => {}
        }
    }

    Ok(())
}

async fn sum_chan(rx_a: &mut Receiver<u128>, rx_b: &mut Receiver<u128>, tx: &mut Sender<u128>) -> Result<usize, OpaqueError> {
    let v1 = rx_a.recv().await?;
    let v2 = rx_b.recv().await?;
    let v3 = v1 + v2;
    tx.send(v3).map_err(|err| Box::new(err) as _)
}
