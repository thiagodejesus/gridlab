use std::env;

use futures_util::{SinkExt, StreamExt};
use grid_engine::grid_engine::GridEngine;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
    sync::mpsc,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub struct GridMultiplayerClient {}

impl GridMultiplayerClient {
    pub async fn initialize(url: String) {
        // let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Message>();

        // Waits for a oneshot channel to receive the grid

        // Returns the instance
    }

    async fn handle_connection(
        url: String,
        grid_sender_oneshot: tokio::sync::oneshot::Sender<GridEngine>,
    ) {
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        let (mut write, mut read) = ws_stream.split();

        // Awaits for the first message to be received that is supposed to be the Grid
        let grid = read.next().await.unwrap().unwrap().into_data();
        
        // Will parse the Message into the expected struct
        // Will get the grid out of the struct
        // Will send the grid to the oneshot channel

        tokio::spawn(async move {
            // Should spawn this loop on another thread and return a way to manage the thread
            loop {
                select! {
                    Some(message) = read.next() => {
                        let message = message.unwrap();
                        let data = message.into_data();
                        tokio::io::stdout().write_all(&data).await.unwrap();
                    }
                }
            }
        });
    }
}
