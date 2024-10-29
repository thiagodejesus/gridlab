use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use grid_engine::grid_engine::{EventName, EventValue, GridEngine};
use http::Uri;
use tokio::{select, sync::mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::ClientRequestBuilder, client::IntoClientRequest, protocol::Message},
};

use crate::{error::GridMultiplayerError, logger::Logger, message::GridDeliver};

pub struct GridMultiplayerClient {
    logger: Logger,
    pub grid_arc: Arc<Mutex<GridEngine>>,
}

impl GridMultiplayerClient {
    pub async fn initialize(
        url: String,
        client_id: String,
    ) -> Result<GridMultiplayerClient, GridMultiplayerError> {
        let logger = Logger::new(format!("Client: {}", client_id));

        let (external_events_sender, mut external_events_receiver) =
            mpsc::unbounded_channel::<EventValue>();
        let (internal_events_sender, internal_events_receiver) =
            mpsc::unbounded_channel::<EventValue>();

        let grid_id = "1";

        let uri = match Uri::from_str(&url) {
            Ok(uri) => uri,
            Err(e) => {
                // If is an invalid Uri, should stop here as its caused by misconfiguration
                return Err(GridMultiplayerError::new(
                    "Failed to parse URI",
                    &format!("Failed to parse URI: {}", e.to_string()),
                    None,
                ));
            }
        };

        let request = match ClientRequestBuilder::new(uri)
            .with_header("x-grid-id", grid_id)
            .into_client_request()
        {
            Ok(request) => request,
            Err(e) => {
                // Could try to handle errors like connection closed
                return Err(GridMultiplayerError::new(
                    "Failed to build request",
                    &format!("Failed to build request: {}", e.to_string()),
                    None,
                ));
            }
        };

        logger.info(&format!("Connecting to server with request {:?}", request));

        let grid: GridEngine = GridMultiplayerClient::connect(
            request,
            external_events_sender,
            internal_events_receiver,
            &logger,
        )
        .await;

        let grid_arc = Arc::new(Mutex::new(grid));
        let clonned_grid_arc = Arc::clone(&grid_arc);

        let logger_clone = logger.append_context("Listener".to_string());
        {
            let mut unlocked_grid = clonned_grid_arc.lock().unwrap();
            unlocked_grid.events.add_listener(
                EventName::BatchChange,
                Box::new(move |_, event_value| {
                    logger_clone.info("Triggered listener");
                    internal_events_sender.send(event_value.clone()).unwrap();
                }),
            );
        }

        let logger_clone = logger.append_context("External".to_string());
        tokio::spawn(async move {
            loop {
                let external_event = match external_events_receiver.recv().await {
                    Some(event) => event,
                    None => {
                        logger_clone.info(&format!("External event channel closed"));
                        break;
                    }
                };

                match external_event {
                    EventValue::BatchChange(changes) => {
                        logger_clone.info(&format!("Batch change received"));
                        let mut grid = clonned_grid_arc.lock().unwrap();
                        grid.apply_changes(&changes.changes);
                        logger_clone
                            .info(&format!("{}", grid.get_grid_view().get_grid_formatted(1)));
                    }
                }
            }
        });

        Ok(GridMultiplayerClient { logger, grid_arc })
        // Returns the instance
    }

    async fn connect(
        request: http::Request<()>,
        external_events_sender: tokio::sync::mpsc::UnboundedSender<EventValue>,
        mut internal_events_receiver: tokio::sync::mpsc::UnboundedReceiver<EventValue>,
        logger: &Logger,
    ) -> GridEngine {
        let (ws_stream, _) = match connect_async(request).await {
            Ok(r) => r,
            Err(e) => {
                // Could try to handle errors like connection closed, maybe a retry
                logger.error(&format!("Failed to connect to server: {}", e.to_string()));
                panic!("Failed to connect to server");
            }
        };
        logger.info(&format!(
            "WebSocket handshake has been successfully completed"
        ));

        let (mut write, mut read) = ws_stream.split();

        logger.info(&format!("Waiting for grid"));
        // Awaits for the first message to be received that is supposed to be the Grid

        // Should handle none received or server closed connection
        let grid_delivery = read.next().await.unwrap().unwrap().into_data();

        logger.info(&format!("Grid received"));

        let grid = GridDeliver::try_from(grid_delivery)
            .expect("Should never happen as the server should always send grid as first message")
            .grid;

        let logger = logger.clone();
        tokio::spawn(async move {
            // Should spawn this loop on another thread and return a way to manage the thread
            loop {
                select! {
                    Some(message) = read.next() => {
                        logger.info(&format!("Message received {:?}", message));
                        let message = message.unwrap();
                        let data = message.into_data();
                        let event = EventValue::try_from(data).unwrap();
                        external_events_sender.send(event).unwrap();
                    },
                    Some(event) = internal_events_receiver.recv() => {
                        logger.info(&format!("Internal event received"));
                        write
                        .send(Message::Binary(event.into()))
                        .await
                        .expect("Failed to send message");
                    }
                }
            }
        });

        grid
    }
}
