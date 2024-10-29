use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use grid_multiplayer::client::GridMultiplayerClient;

#[tokio::main]
async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:8080/".to_string());

    // Generate random client id using system time
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let client_id = since_the_epoch.as_secs() ^ since_the_epoch.subsec_nanos() as u64;

    let grid_multiplayer = GridMultiplayerClient::initialize(url, client_id.to_string()).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
}
