use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    UsageUpdate {
        user_id: Uuid,
        cost: f64,
        tokens: i32,
        timestamp: String,
    },
    AlertNotification {
        user_id: Uuid,
        alert_type: String,
        message: String,
    },
    PredictionUpdate {
        user_id: Uuid,
        daily_cost: f64,
        weekly_cost: f64,
        monthly_cost: f64,
    },
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    let mut rx = state.ws_tx.subscribe();
    
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                tracing::debug!("Received WebSocket message: {}", text);
            }
        }
    });
    
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    tracing::info!("WebSocket connection closed");
}