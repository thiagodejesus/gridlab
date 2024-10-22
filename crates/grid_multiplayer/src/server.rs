use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use grid_engine::grid_engine::{EventName, EventValue, GridEngine};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::tungstenite::{
    handshake::server::{Request, Response},
    http::{Response as HttpResponse, StatusCode},
    Message,
};

// Needs to implement graceful shutdown

static mut ITEM_ID: i32 = 0;

fn gen_id() -> String {
    unsafe {
        ITEM_ID += 1;
        ITEM_ID.to_string()
    }
}

struct Client {
    id: String,
    sender: mpsc::UnboundedSender<Message>, // sender: futures_channel::mpsc::UnboundedSender<Message>,
}

// type ArcClient = Arc<Mutex<Client>>;

struct Room {
    clients: HashMap<String, Client>,
    grid: GridEngine,
}

impl Room {
    fn broadcast_change(&self, from: &str, event_value: EventValue) {
        for (_, client) in self.clients.iter() {
            if client.id == from {
                continue;
            }
            client
                .sender
                .send(Message::text(event_value.to_string()))
                .unwrap();
        }
    }
}

type ArcRoom = Arc<Mutex<Room>>;

type ChangeClosure = Box<dyn Fn()>;
type CloseClosure = Box<dyn Fn()>;

type ArcRoomsMap = Arc<Mutex<HashMap<String, ArcRoom>>>;

pub struct GridMultiplayerServer {
    rooms: ArcRoomsMap,
}

pub struct GridMultiplayerServerBuilder {
    change_closures: Vec<ChangeClosure>,
    close_closures: Vec<CloseClosure>,
}

impl GridMultiplayerServerBuilder {
    pub fn new() -> GridMultiplayerServerBuilder {
        GridMultiplayerServerBuilder {
            change_closures: Vec::new(),
            close_closures: Vec::new(),
        }
    }

    pub async fn start_server(self) -> GridMultiplayerServer {
        let rooms: ArcRoomsMap = Arc::new(Mutex::new(HashMap::new()));

        let addr = "127.0.0.1:8080".to_string();

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");

        println!("Local address: {:?}", listener.local_addr().unwrap());
        listener.local_addr().unwrap();

        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            let rooms_clone = Arc::clone(&rooms);
            tokio::spawn(handle_connection(rooms_clone, stream, addr));
        }

        GridMultiplayerServer { rooms: rooms }
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
) -> tokio_tungstenite::tungstenite::Result<()> {
    println!("Incoming TCP connection from: {}", addr);

    // This should be a one shot channel
    let grid_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let ws_stream =
        tokio_tungstenite::accept_hdr_async(raw_stream, |req: &Request, res: Response| {
            let received_grid_id = match req.headers().get("x-grid-id") {
                Some(header) => header.to_str().unwrap().to_string(),
                None => {
                    println!("No grid id");

                    return Err(HttpResponse::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Some("Missing x-grid-id".to_string()))
                        .unwrap());
                }
            };

            {
                let mut unlocked = rooms.lock().unwrap();

                match unlocked.get(&received_grid_id) {
                    Some(room) => {
                        println!("Room already exists");
                        let clone = Arc::clone(&room);
                        clone
                    }
                    None => {
                        println!("Creating new room");
                        let new_room = Arc::new(Mutex::new(Room {
                            clients: HashMap::new(),
                            grid: GridEngine::new(12, 16),
                        }));
                        unlocked.insert(received_grid_id.clone(), Arc::clone(&new_room));
                        new_room
                    }
                };
            }

            {
                grid_id.lock().unwrap().replace(received_grid_id);
            }

            Ok(res)
        })
        .await
        .expect("Error during the websocket handshake occurred");

    println!("WebSocket connection established: {}", addr);

    let grid_id = { grid_id.lock().unwrap().as_ref().unwrap().to_string() };

    let (this_client_sender, mut this_client_receiver) = mpsc::unbounded_channel();

    let client = Client {
        id: addr.to_string(),
        sender: this_client_sender,
    };

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

    let mut grid_events_rx = {
        // Add event listener to grid
        let grid_id = grid_id.clone();
        let room = rooms.lock().unwrap();
        let mut room = room.get(&grid_id).unwrap().lock().unwrap();
        let grid = &mut room.grid;

        let (grid_events_tx, grid_events_rx) = mpsc::unbounded_channel::<EventValue>();

        grid.events.add_listener(
            EventName::BatchChange,
            Box::new(move |_, event_value| {
                println!("Triggered listener");
                grid_events_tx.send(event_value.clone()).unwrap();
            }),
        );
        grid_events_rx
    };

    loop {
        tokio::select! {
            Some(event_value) = grid_events_rx.recv() => {
                println!("1 Option: Received event from grid");
                let room = rooms.lock().unwrap();
                let room = room.get(&grid_id).unwrap().lock().unwrap();
                let room = &room;

                room.broadcast_change(&addr.to_string(), event_value);
                println!("Broadcasted event");
            },
            Some(msg) = ws_in.next() => {
                let msg = msg?;
                if msg.is_close() {
                    break;
                }

                println!(
                    "2 Option: Received a message from {}: {}",
                    addr,
                    msg.to_text().unwrap()
                );

                println!("Adding new item to grid");
                let new_id = gen_id();

                println!("Generated new id: {:?}", new_id);
                {
                    rooms
                        .lock()
                        .unwrap()
                        .get(&grid_id)
                        .unwrap()
                        .lock()
                        .unwrap()
                        .grid
                        .add_item(new_id, 0, 0, 2, 2)
                        .unwrap();
                }
                println!("Added new item to grid");
            },
            Some(message) = this_client_receiver.recv() => {
                ws_out.send(message).await?;
            }
        }
    }

    println!("{} disconnected", &addr);
    Ok(())
    // Should remove the client from the room
    // peer_map.lock().unwrap().remove(&addr);
}
