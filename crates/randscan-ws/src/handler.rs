//! WebSocket handler for Axum

use crate::WsManager;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use randscan_core::{WsClientMessage, WsServerMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// WebSocket handler state
#[derive(Clone)]
pub struct WsState {
    pub manager: Arc<WsManager>,
}

impl WsState {
    pub fn new(manager: Arc<WsManager>) -> Self {
        Self { manager }
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
pub async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for sending messages to this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<WsServerMessage>();

    // Register connection
    let connection_id = state.manager.register(tx).await;
    info!("WebSocket connected: {}", connection_id);

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if sender.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                }
            }
        }
    });

    // Handle incoming messages
    let manager = state.manager.clone();
    let conn_id = connection_id.clone();

    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                handle_message(&manager, &conn_id, &text).await;
            }
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by axum
                debug!("Received ping from {}", conn_id);
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong from {}", conn_id);
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket closed by client: {}", conn_id);
                break;
            }
            Ok(Message::Binary(_)) => {
                warn!("Received binary message, ignoring");
            }
            Err(e) => {
                error!("WebSocket error for {}: {}", conn_id, e);
                break;
            }
        }
    }

    // Cleanup
    state.manager.unregister(&connection_id).await;
    send_task.abort();
    info!("WebSocket disconnected: {}", connection_id);
}

/// Handle an incoming text message
async fn handle_message(manager: &WsManager, connection_id: &str, text: &str) {
    let message: Result<WsClientMessage, _> = serde_json::from_str(text);

    match message {
        Ok(WsClientMessage::Subscribe { channel, params }) => {
            manager.subscribe(connection_id, channel, params).await;
        }
        Ok(WsClientMessage::Unsubscribe { channel, .. }) => {
            manager.unsubscribe(connection_id, channel).await;
        }
        Ok(WsClientMessage::Ping) => {
            // Already handled by channel sender
            debug!("Application-level ping from {}", connection_id);
        }
        Err(e) => {
            warn!(
                "Failed to parse WebSocket message from {}: {}",
                connection_id, e
            );
        }
    }
}
