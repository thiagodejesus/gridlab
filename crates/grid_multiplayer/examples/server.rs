use grid_multiplayer::server::GridMultiplayerServerBuilder;

#[tokio::main]
pub async fn main() {
    let server = GridMultiplayerServerBuilder::new().start_server().await;

    // Improvised keep awake
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

    println!("Server started");
}
