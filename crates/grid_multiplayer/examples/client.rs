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

    let instance = GridMultiplayerClient::initialize(url, client_id.to_string()).await;

    {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let component_id = since_the_epoch.as_secs() ^ since_the_epoch.subsec_nanos() as u64;
        // get 3 last digits of component_id
        let component_id = component_id % 1000;

        let locked = instance.unwrap();
        let mut locked = locked.grid_arc.lock().unwrap();
        locked
            .add_item(component_id.to_string(), 0, 0, 2, 2)
            .unwrap();
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
}
