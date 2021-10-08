use tokio::sync::broadcast::{self, Receiver, Sender};

type OpaqueError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), OpaqueError> {
    println!("Hello, world!");

    let (mut tx, mut rx1) = broadcast::channel(8);

    let mut rx2 = tx.subscribe();
    let mut rx3 = tx.subscribe();
    tx.send(1_u128)?;
    tx.send(1)?;

    rx2.recv().await?; // offset by one

    loop {
        tokio::select! {
            biased;
            x = async {
                let r = rx3.recv().await;
                println!("r: {:?}", r);
                r
            } => {
                let x = x?;
                if x >= 1000000 {
                    println!("it worked! x: {}", x);
                    break;
                }
            }
            _ = sum_chan(&mut rx1, &mut rx2, &mut tx) => {}
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
