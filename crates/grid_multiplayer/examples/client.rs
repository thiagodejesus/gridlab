use std::env;

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
    sync::mpsc,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:8080/".to_string());

    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Message>();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();
    //
    loop {
        select! {
            Some(message) = stdin_rx.recv() => {
                write.send(message).await.unwrap();
            }
            Some(message) = read.next() => {
                let message = message.unwrap();
                let data = message.into_data();
                tokio::io::stdout().write_all(&data).await.unwrap();
            }
        }
    }
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.send(Message::binary(buf)).unwrap();
    }
}
