use std::collections::HashMap;
use axum::extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade, Query};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::AppState;

pub struct WsState {
    pub board_channels: RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>,
}

impl WsState {
    pub fn new() -> Self {
        Self {
            board_channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_create_channel(&self, board_id: Uuid) -> broadcast::Sender<WsMessage> {
        let mut channels = self.board_channels.write().await;
        channels.entry(board_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            tx
        }).clone()
    }

    pub async fn broadcast_to_board(&self, board_id: Uuid, message: WsMessage) {
        let channels = self.board_channels.read().await;
        if let Some(tx) = channels.get(&board_id) {
            let _ = tx.send(message);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub event: String,
    pub board_id: Uuid,
    pub data: serde_json::Value,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
    pub board_id: Option<Uuid>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(socket: WebSocket, state: AppState, query: WsQuery) {
    let (mut sender, mut receiver) = socket.split();

    // Validate token
    let user_id = if let Some(token) = &query.token {
        let decoded = jsonwebtoken::decode::<crate::middleware::auth::Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
        );
        match decoded {
            Ok(data) => Some(data.claims.sub),
            Err(_) => None,
        }
    } else {
        None
    };

    if user_id.is_none() {
        let _ = sender.send(Message::Text(
            serde_json::json!({"error": "Unauthorized"}).to_string().into()
        )).await;
        return;
    }

    let user_id = user_id.unwrap();

    // Subscribe to board channel if specified — verify membership/visibility first,
    // otherwise any authenticated user could listen in on (and inject into) any board.
    if let Some(board_id) = query.board_id {
        if crate::services::boards::check_board_access(&state, user_id, board_id, "").await.is_err() {
            let _ = sender.send(Message::Text(
                serde_json::json!({"error": "Forbidden"}).to_string().into()
            )).await;
            return;
        }

        let tx = state.ws_state.get_or_create_channel(board_id).await;
        let mut rx = tx.subscribe();

        // Spawn task to forward broadcast messages to this client
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(text) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Handle incoming messages from client
        let ws_state = state.ws_state.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            ws_state.broadcast_to_board(ws_msg.board_id, WsMessage {
                                user_id: Some(user_id),
                                ..ws_msg
                            }).await;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        // Wait for either task to finish
        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        }
    }
}
