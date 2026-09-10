use crate::WsManager;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use randscan_core::{WsClientMessage, WsServerMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct WsState {
    pub manager: Arc<WsManager>,
}

impl WsState {
    pub fn new(manager: Arc<WsManager>) -> Self {
        Self { manager }
    }
}

pub async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsServerMessage>();
    let connection_id = state.manager.register(tx.clone()).await;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if sink.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("ws serialize failed: {}", e),
            }
        }
    });

    while let Some(result) = stream.next().await {
        match result {
            Ok(Message::Text(text)) => match serde_json::from_str::<WsClientMessage>(&text) {
                Ok(WsClientMessage::Subscribe { channel }) => {
                    state.manager.subscribe(&connection_id, channel).await
                }
                Ok(WsClientMessage::Unsubscribe { channel }) => {
                    state.manager.unsubscribe(&connection_id, channel).await
                }
                Ok(WsClientMessage::Ping) => {
                    let _ = tx.send(WsServerMessage::Pong);
                }
                Err(e) => {
                    let _ = tx.send(WsServerMessage::Error {
                        message: format!("bad message: {}", e),
                    });
                }
            },
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => {}
        }
    }

    state.manager.unregister(&connection_id).await;
    send_task.abort();
    debug!("ws connection {} closed", connection_id);
}
