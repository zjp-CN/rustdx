use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

/// Multiple different commands are multiplexed over a single channel.
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Vec<u8>,
        resp: Responder<()>,
    },
}

/// Provided by the requester and used by the manager task to send the command
/// response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    // Clone a `tx` handle for the second f
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        // Open a connection to the mini-redis address.
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    // Ignore errors
                    let _ = resp.send(res);
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val.into()).await;
                    // Ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    // Spawn two tasks, one setting a value and other querying for key that was
    // set.
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // Send the GET request
        if tx.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        // Await the response
        let res = resp_rx.await;
        println!("GOT (Get) = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: b"bar".to_vec(),
            resp: resp_tx,
        };

        // Send the SET request
        if tx2.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        // Await the response
        let res = resp_rx.await;
        println!("GOT (Set) = {:?}", res)
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
