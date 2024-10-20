use grid_multiplayer::GridMultiplayerServerBuilder;

#[tokio::main]
pub async fn main() {
    let server = GridMultiplayerServerBuilder::new().start_server().await;

    println!("Server started");
}
