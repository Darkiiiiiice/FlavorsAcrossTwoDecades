//! 对话 API 模块

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::db::models::dialogue::DialogueMessage;
use crate::db::repositories::dialogue::DialogueRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 发送消息请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    /// 消息内容
    pub content: String,
    /// 消息类型
    #[serde(default = "default_message_type")]
    pub message_type: String,
}

fn default_message_type() -> String {
    "command".to_string()
}

/// 消息响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    /// 消息 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 发送者
    pub sender: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: String,
    /// 消息类型
    pub message_type: String,
    /// 状态
    pub status: String,
}

impl From<DialogueMessage> for MessageResponse {
    fn from(msg: DialogueMessage) -> Self {
        Self {
            id: msg.id.to_string(),
            save_id: msg.save_id.to_string(),
            sender: msg.sender,
            content: msg.content,
            timestamp: msg.timestamp.to_rfc3339(),
            message_type: msg.message_type,
            status: msg.status,
        }
    }
}

/// 消息列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageListResponse {
    /// 消息列表
    pub messages: Vec<MessageResponse>,
    /// 总数
    pub total: usize,
}

/// 查询参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct MessageQuery {
    /// 限制数量
    pub limit: Option<i64>,
}

/// 发送消息
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/dialogues",
    tag = "dialogues",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "消息发送成功", body = MessageResponse),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> GameResult<Json<MessageResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let message = DialogueMessage::player_message(save_id, payload.content);

    let repo = DialogueRepository::new(state.db_pool.pool().clone());
    repo.create(&message).await?;

    Ok(Json(MessageResponse::from(message)))
}

/// 获取对话历史
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/dialogues",
    tag = "dialogues",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        MessageQuery
    ),
    responses(
        (status = 200, description = "获取对话历史成功", body = MessageListResponse)
    )
)]
pub async fn get_dialogue_history(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Query(query): Query<MessageQuery>,
) -> GameResult<Json<MessageListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = DialogueRepository::new(state.db_pool.pool().clone());

    let messages = if let Some(limit) = query.limit {
        repo.find_recent(save_id, limit).await?
    } else {
        repo.find_by_save_id(save_id).await?
    };

    let message_responses: Vec<MessageResponse> =
        messages.into_iter().map(MessageResponse::from).collect();

    Ok(Json(MessageListResponse {
        total: message_responses.len(),
        messages: message_responses,
    }))
}

/// 获取消息详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/dialogues/{message_id}",
    tag = "dialogues",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("message_id" = String, Path, description = "消息 ID")
    ),
    responses(
        (status = 200, description = "获取消息成功", body = MessageResponse),
        (status = 404, description = "消息不存在")
    )
)]
pub async fn get_message(
    State(state): State<Arc<AppState>>,
    Path((save_id, message_id)): Path<(String, String)>,
) -> GameResult<Json<MessageResponse>> {
    let _save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid save_id UUID: {}", e),
    })?;

    let message_id = Uuid::parse_str(&message_id).map_err(|e| GameError::Validation {
        details: format!("Invalid message_id UUID: {}", e),
    })?;

    let repo = DialogueRepository::new(state.db_pool.pool().clone());
    let message = repo
        .find_by_id(message_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Message".to_string(),
            entity_id: message_id.to_string(),
        })?;

    Ok(Json(MessageResponse::from(message)))
}
