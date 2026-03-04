//! WebSocket 模块
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::game::AppState;

/// WebSocket 消息类型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// 连接确认
    #[serde(rename = "connected")]
    Connected {
        client_id: String,
        timestamp: String,
    },
    /// 游戏状态更新
    #[serde(rename = "game_state_update")]
    GameStateUpdate { data: serde_json::Value },
    /// 事件通知
    #[serde(rename = "event")]
    Event {
        event_type: String,
        data: serde_json::Value,
        timestamp: String,
    },
    /// 错误通知
    #[serde(rename = "error")]
    Error { code: String, message: String },
    /// 心跳
    #[serde(rename = "ping")]
    Ping,
    /// 心跳响应
    #[serde(rename = "pong")]
    Pong,
}

/// WebSocket 升级处理
pub async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(handle_socket)
}

/// 处理 WebSocket 连接
async fn handle_socket(mut socket: WebSocket) {
    let client_id = uuid::Uuid::new_v4().to_string();

    // 发送连接确认
    let connected_msg = WsMessage::Connected {
        client_id: client_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&connected_msg)
        && socket.send(Message::Text(json.into())).await.is_err()
    {
        return;
    }

    tracing::info!("WebSocket client {} connected", client_id);

    // 处理接收到的消息
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                // 尝试解析消息
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Ping) => {
                        // 响应心跳
                        if let Ok(pong) = serde_json::to_string(&WsMessage::Pong)
                            && socket.send(Message::Text(pong.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Ok(WsMessage::Pong) => {
                        // 忽略心跳响应
                    }
                    Ok(other) => {
                        // 处理其他消息类型
                        tracing::debug!("Received WebSocket message: {:?}", other);
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                tracing::debug!("Received binary message: {} bytes", data.len());
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket client {} disconnected", client_id);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    tracing::info!("WebSocket connection closed for client {}", client_id);
}
