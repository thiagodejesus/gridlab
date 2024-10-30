use core::panic;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, LazyLock, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use grid_engine::{
    grid_engine::{EventName, EventValue, GridEngine},
    grid_view::GridView,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Request, Response},
    http::{Response as HttpResponse, StatusCode},
    Message,
};

use crate::logger::Logger;

// Needs to implement graceful shutdown

static GRID_STORAGE: LazyLock<Mutex<HashMap<String, GridView>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_grid(id: &str) -> Option<GridEngine> {
    let locked = GRID_STORAGE.lock().unwrap();
    let grid = locked.get(id);
    match grid {
        Some(grid) => Some(GridEngine::from(grid)),
        None => None,
    }
}

fn save_grid(id: &str, grid: GridEngine) {
    let grid_view = grid.get_grid_view();
    let mut locked = GRID_STORAGE.lock().unwrap();
    locked.insert(id.to_string(), grid_view);
}

struct Client {
    id: String,
    grid_changes_listener_id: String,
    message: mpsc::UnboundedSender<Message>,
}

struct Room {
    clients: HashMap<String, Client>,
    grid: GridEngine,
}

impl Room {
    fn broadcast_change(&mut self, from: &str, event_value: EventValue) {
        let clients_to_close: Vec<String> = self
            .clients
            .iter()
            .filter(|(_, client)| {
                if client.id != from {
                    return false;
                }

                match client.message.send(Message::binary(&event_value)) {
                    Ok(_) => false,
                    Err(_) => true,
                }
            })
            .map(|(_, client)| client.id.clone())
            .collect();

        for client_id in clients_to_close {
            self.close_connection(&client_id);
        }
    }

    fn close_connection(&mut self, client_id: &str) {
        println!("Closing connection for {}", client_id);
        let client = self.clients.remove(client_id);

        if let Some(client) = client {
            self.grid
                .events
                .remove_listener(EventName::BatchChange, &client.grid_changes_listener_id);
        }
    }
}

type ArcRoom = Arc<Mutex<Room>>;

type ChangeClosure = Box<dyn Fn()>;
type CloseClosure = Box<dyn Fn()>;

type ArcRoomsMap = Arc<Mutex<HashMap<String, ArcRoom>>>;

pub struct GridMultiplayerServer {
    rooms: ArcRoomsMap,
    pub url: String,
}

pub struct GridMultiplayerServerBuilder {
    change_closures: Vec<ChangeClosure>,
    close_closures: Vec<CloseClosure>,
    logger: Logger,
}

impl GridMultiplayerServerBuilder {
    pub fn new() -> GridMultiplayerServerBuilder {
        GridMultiplayerServerBuilder {
            change_closures: Vec::new(),
            close_closures: Vec::new(),
            logger: Logger {
                context: "Server".to_string(),
            },
        }
    }

    pub async fn start_server(self) -> GridMultiplayerServer {
        let rooms: ArcRoomsMap = Arc::new(Mutex::new(HashMap::new()));

        let addr = "127.0.0.1:8080".to_string();

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");

        self.logger.info(&format!(
            "Local address: {:?}",
            listener.local_addr().unwrap()
        ));
        listener.local_addr().unwrap();

        let rooms_clone = Arc::clone(&rooms);
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let rooms_clone = Arc::clone(&rooms_clone);
                tokio::spawn(handle_connection(
                    rooms_clone,
                    stream,
                    addr,
                    self.logger.append_context(format!(" {}", addr)),
                ));
            }
        });
        // Let's spawn the handling of each connection in a separate task.

        GridMultiplayerServer {
            rooms: rooms,
            url: format!("ws://{}", addr),
        }
    }

    pub fn on_change(&mut self, closure: ChangeClosure) -> () {
        self.change_closures.push(closure);
    }

    pub fn on_close(&mut self, closure: CloseClosure) -> () {
        self.close_closures.push(closure);
    }
}

async fn handle_connection(
    rooms: ArcRoomsMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    logger: Logger,
) {
    logger.info(&format!("Incoming TCP connection"));

    let (grid_id_sender, grid_id_receiver) = tokio::sync::oneshot::channel::<String>();

    let ws_stream =
        match tokio_tungstenite::accept_hdr_async(raw_stream, |req: &Request, res: Response| {
            let grid_id = match req.headers().get("x-grid-id") {
                Some(header) => header.to_str().unwrap().to_string(),
                None => {
                    logger.info("No grid id");

                    return Err(HttpResponse::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Some("Missing x-grid-id".to_string()))
                        .unwrap());
                }
            };

            {
                let mut unlocked_room = rooms.lock().unwrap();

                match unlocked_room.get(&grid_id) {
                    Some(room) => {
                        logger.info("Room already exists");
                        let clone = Arc::clone(&room);
                        clone
                    }
                    None => {
                        logger.info("Creating new room");
                        let grid = match get_grid(&grid_id) {
                            Some(grid) => grid,
                            None => GridEngine::new(16, 12),
                        };

                        let new_room = Arc::new(Mutex::new(Room {
                            clients: HashMap::new(),
                            grid,
                        }));
                        unlocked_room.insert(grid_id.clone(), Arc::clone(&new_room));
                        new_room
                    }
                };
            }

            grid_id_sender.send(grid_id).unwrap();

            Ok(res)
        })
        .await
        {
            Err(e) => {
                logger.error(&format!(
                    "Error during the websocket handshake occurred: {}",
                    e
                ));
                // Should see if needs to handle it
                return;
            }
            Ok(ws_stream) => ws_stream,
        };

    logger.info("WebSocket connection established");

    let grid_id = grid_id_receiver.await.expect("Should never happen as something that may cause the grid_id to not exist should be handled before");

    let (this_client_sender, mut this_client_receiver) = mpsc::unbounded_channel();

    let (mut grid_events_rx, listener) = {
        // Add event listener to grid
        let grid_id = grid_id.clone();
        let room = rooms.lock().unwrap();
        let mut room = room.get(&grid_id).unwrap().lock().unwrap();
        let grid = &mut room.grid;

        let (grid_events_tx, grid_events_rx) = mpsc::unbounded_channel::<EventValue>();

        let logger_clone = logger.append_context("Listener".to_string());
        let listener = grid.events.add_listener(
            EventName::BatchChange,
            Box::new(move |_, event_value| {
                logger_clone.info("Triggered listener");
                grid_events_tx.send(event_value.clone()).unwrap();
            }),
        );
        (grid_events_rx, listener)
    };

    let client = Client {
        id: addr.to_string(),
        message: this_client_sender,
        grid_changes_listener_id: listener,
    };

    let client_id = client.id.clone();

    {
        let mut unlocked = rooms.lock().unwrap();
        let room = unlocked.get_mut(&grid_id).unwrap();

        // What if by some reason this client alerady exists?
        room.lock()
            .unwrap()
            .clients
            .insert(client.id.clone(), client);
    }

    let (mut ws_out, mut ws_in) = ws_stream.split();

    let grid_binary: Vec<u8> = {
        let room = rooms.lock().unwrap();
        let grid = &room.get(&grid_id).unwrap().lock().unwrap().grid;

        grid.into()
    };

    match ws_out.send(Message::binary(grid_binary)).await {
        Err(e) => {
            // Should handle possibly connection closed, or just assume that if this error happens the connection is closed
            logger.error(&format!("Error sending grid to client: {}", e));
            let room = rooms.lock().unwrap();
            let mut room = room.get(&grid_id).unwrap().lock().unwrap();
            room.close_connection(&client_id);
            return;
        }
        Ok(_) => {}
    };
    logger.info("Grid send");

    loop {
        tokio::select! {
            Some(event_value) = grid_events_rx.recv() => {
                logger.info("1 Option: Received event from grid");
                let room = rooms.lock().unwrap();
                let mut room = room.get(&grid_id).unwrap().lock().unwrap();

                room.broadcast_change(&addr.to_string(), event_value);
                logger.info("Broadcasted event");
            },
            Some(msg) = ws_in.next() => {
                logger.info(&format!(
                    "2 Option: Received a message from {}: {:?}",
                    addr,
                    msg
                ));
                let msg = match msg {
                    Err(e) => {
                        logger.error(&format!("Error receiving message: {}", e));
                        // Should close the connection

                        // panic!("Unhandled closed connection");
                        let mut unlocked = rooms.lock().unwrap();
                        let room = unlocked.get_mut(&grid_id).unwrap();
                        room.lock()
                            .unwrap()
                            .close_connection(&client_id);

                        break;
                    }
                    Ok(msg) => {
                        msg
                    }
                };

                if msg.is_close() {
                    let mut unlocked = rooms.lock().unwrap();
                    let room = unlocked.get_mut(&grid_id).unwrap();
                    room.lock()
                        .unwrap()
                        .close_connection(&client_id);
                    break;
                }

                let room = rooms.lock().unwrap();
                let mut room = room.get(&grid_id).unwrap().lock().unwrap();
                let event = EventValue::try_from(msg.into_data()).unwrap();
                match event {
                    EventValue::BatchChange(changes) => {
                        logger.info("Applying external change");
                        if changes.hash_after == room.grid.get_grid_view().hash() {
                            logger.error("Hash mismatch");
                            continue;
                        }
                        room.grid.apply_changes(&changes.changes);
                        logger.info(&format!(
                            "\n {}",
                            room.grid.get_grid_view().get_grid_formatted(1)
                        ));
                    }
                }
            }
            Some(msg) = this_client_receiver.recv() => {
                logger.info("3 Option: Received a message from client");
                ws_out.send(msg).await.unwrap();
            }
        }
    }

    let mut unlocked = rooms.lock().unwrap();
    let room = unlocked.get_mut(&grid_id).unwrap();
    room.lock().unwrap().close_connection(&client_id);
    logger.info(&format!("{} disconnected", &addr));
    // Should remove the client from the room
    // peer_map.lock().unwrap().remove(&addr);
}
