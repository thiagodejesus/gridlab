use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body, extract::Request, http::StatusCode, response::IntoResponse, routing::get, Router,
};
use fastwebsockets::{upgrade, Frame, OpCode, Payload, WebSocketError};
use grid_engine::grid_engine::{EventName, EventValue, GridEngine};
use tokio::sync::{mpsc, Mutex};

// Needs to implement graceful shutdown

static mut ITEM_ID: i32 = 0;

struct Client {
    id: String,
    sender: tokio::sync::mpsc::Sender<EventValue>,
}

// type ArcClient = Arc<Mutex<Client>>;

struct Room {
    clients: HashMap<String, Client>,
    grid: GridEngine,
}

impl Room {
    async fn broadcast_change(&self, from: &str, event_value: EventValue) {
        for (_, client) in self.clients.iter() {
            if client.id == from {
                continue;
            }
            println!("Sending something");
            client.sender.send(event_value.clone()).await.unwrap();
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

        let rooms_clone = Arc::clone(&rooms);

        let app = Router::new().route(
            "/",
            get(move |ws, request: Request| {
                println!("Received request: {:?}", request);
                return ws_handler(ws, rooms_clone, request);
            }),
        );

        println!("Starting server");
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();

        println!("Server started");
        GridMultiplayerServer { rooms: rooms }
    }

    pub fn on_change(&mut self, closure: ChangeClosure) -> () {
        self.change_closures.push(closure);
    }

    pub fn on_close(&mut self, closure: CloseClosure) -> () {
        self.close_closures.push(closure);
    }
}

async fn ws_handler(
    ws: upgrade::IncomingUpgrade,
    rooms: ArcRoomsMap,
    request: Request,
) -> impl IntoResponse {
    println!("Handling websocket request");
    let client_id = match request.headers().get("x-identification") {
        Some(header) => header,
        None => {
            println!("No identification header");
            return (StatusCode::BAD_REQUEST).into_response();
        }
    }
    .to_str()
    .unwrap()
    .to_string();

    let grid_id = match request.headers().get("x-grid-id") {
        Some(header) => header,
        None => {
            println!("No grid id");
            return (StatusCode::BAD_REQUEST).into_response();
        }
    }
    .to_str()
    .unwrap()
    .to_string();

    let mut unlocked = rooms.lock().await;

    let room = match unlocked.get(&grid_id) {
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
            unlocked.insert(grid_id.clone(), Arc::clone(&new_room));
            new_room
        }
    };

    // return (StatusCode::FORBIDDEN).into_response();

    let (response, fut) = ws.upgrade().unwrap();
    tokio::task::spawn(async move {
        if let Err(e) = handle_client(fut, &room, client_id).await {
            eprintln!("Error in websocket connection: {}", e);
        }
    });

    println!("Response {:?}", response);
    let response = response.map(|_| Body::empty());
    return response;
}

async fn handle_client(
    fut: upgrade::UpgradeFut,
    room: &ArcRoom,
    client_id: String,
) -> Result<(), WebSocketError> {
    let awaited_fut = fut.await?;
    let mut ws = fastwebsockets::FragmentCollector::new(awaited_fut);
    println!("Client {} connected", client_id);

    let (tx, mut rx) = mpsc::channel(1024);
    let client = Client {
        id: client_id.clone(),
        sender: tx,
    };

    room.lock().await.clients.insert(client.id.clone(), client);

    let another_room_clone = Arc::clone(&room);

    let temp_client_id = client_id.clone();
    room.lock().await.grid.events.add_listener(
        EventName::BatchChange,
        Box::new(move |_, event_value| {
            println!("Batch change triggered");

            let another_clone = Arc::clone(&another_room_clone);
            let event_value = event_value.clone();
            let client_id = temp_client_id.clone();
            // Should understand the tradeoffs of this spawn
            tokio::spawn(async move {
                println!("Inside tokio spawn");
                another_clone
                    .lock()
                    .await
                    .broadcast_change(&client_id, event_value)
                    .await;
                println!("Broadcasted change");
            });
            // futures::executor::block_on(async {
            //     println!("Inside block_on");

            //     another_room_clone
            //         .lock()
            //         .await
            //         .broadcast_change(event_value.clone())
            //         .await;
            // });
            println!("Batch change listener executed");
        }),
    );
    //  Generates a random id for the item

    let gen_id = || unsafe {
        ITEM_ID += 1;
        ITEM_ID.to_string()
    };

    loop {
        tokio::select! {
            // Receive and handle messages from other clients
            Some(message) = rx.recv() => {
                println!("Handling messages from other clients");
                let message_string = message.to_string();
                let frame = Frame::text(Payload::from(message_string.as_bytes()));
                ws.write_frame(frame).await.unwrap();
            }
            // Read a frame from the WebSocket
            frame = ws.read_frame() => {
                let frame = frame?;

                match frame.opcode {
                    OpCode::Close => {
                        println!("Received Close frame");
                        break;
                    }
                    // OpCode::Binary => {
                    //     let a = frame.payload.to_vec();
                    //     let text = std::str::from_utf8(&a).unwrap().to_string();

                    //     println!("Received Text: {:?}", text);
                    // }
                    _ => {
                        println!("Adding new item to grid");
                        let new_id = gen_id();
                        println!("Generated new id: {:?}", new_id);
                        room.lock().await.grid.add_item(new_id, 0, 0, 2, 2).unwrap();
                        println!("Added new item to grid");
                        println!("Received unMapped frame of {:?}", frame.opcode);
                    }
                }
            }
        }
    }

    println!("Client {} disconnected", client_id);
    room.lock().await.clients.remove(&client_id);
    Ok(())
}
