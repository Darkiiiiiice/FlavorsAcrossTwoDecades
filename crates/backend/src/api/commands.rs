//! 指令 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::models::command::Command;
use crate::db::repositories::command::CommandRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;
use crate::game::command::CommandStatus;

/// 发送指令请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendCommandRequest {
    /// 指令内容
    pub content: String,
}

/// 指令响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CommandResponse {
    /// 指令 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 指令内容
    pub content: String,
    /// 创建时间
    pub created_at: String,
    /// 到达时间
    pub arrival_time: String,
    /// 状态
    pub status: String,
    /// 结果
    pub result: Option<String>,
}

impl From<Command> for CommandResponse {
    fn from(cmd: Command) -> Self {
        Self {
            id: cmd.id.to_string(),
            save_id: cmd.save_id.to_string(),
            content: cmd.content,
            created_at: cmd.created_at.to_rfc3339(),
            arrival_time: cmd.arrival_time.to_rfc3339(),
            status: status_to_string(&cmd.status),
            result: cmd.result,
        }
    }
}

/// 指令列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CommandListResponse {
    /// 指令列表
    pub commands: Vec<CommandResponse>,
    /// 总数
    pub total: usize,
}

/// 发送指令
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/commands",
    tag = "commands",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = SendCommandRequest,
    responses(
        (status = 201, description = "指令发送成功", body = CommandResponse),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn send_command(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(payload): Json<SendCommandRequest>,
) -> GameResult<Json<CommandResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    // 获取配置的通信延迟（240-600秒，即4-10分钟）
    // TODO: 从配置文件中读取
    let delay_min = 240i64; // 4分钟
    let delay_max = 600i64; // 10分钟
    let delay_range = delay_max - delay_min;

    let delay = rand::random::<i64>() % delay_range + delay_min;

    let command = Command::new(save_id, payload.content, delay);

    let repo = CommandRepository::new(state.db_pool.pool().clone());
    repo.create(&command).await?;

    Ok(Json(CommandResponse::from(command)))
}

/// 获取指令列表
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/commands",
    tag = "commands",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取指令列表成功", body = CommandListResponse)
    )
)]
pub async fn list_commands(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<CommandListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = CommandRepository::new(state.db_pool.pool().clone());
    let commands = repo.find_by_save_id(save_id).await?;

    let command_responses: Vec<CommandResponse> =
        commands.into_iter().map(CommandResponse::from).collect();

    Ok(Json(CommandListResponse {
        total: command_responses.len(),
        commands: command_responses,
    }))
}

/// 获取指令详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/commands/{command_id}",
    tag = "commands",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("command_id" = String, Path, description = "指令 ID")
    ),
    responses(
        (status = 200, description = "获取指令成功", body = CommandResponse),
        (status = 404, description = "指令不存在")
    )
)]
pub async fn get_command(
    State(state): State<Arc<AppState>>,
    Path((save_id, command_id)): Path<(String, String)>,
) -> GameResult<Json<CommandResponse>> {
    let _save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid save_id UUID: {}", e),
    })?;

    let command_id = Uuid::parse_str(&command_id).map_err(|e| GameError::Validation {
        details: format!("Invalid command_id UUID: {}", e),
    })?;

    let repo = CommandRepository::new(state.db_pool.pool().clone());
    let command = repo
        .find_by_id(command_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Command".to_string(),
            entity_id: command_id.to_string(),
        })?;

    Ok(Json(CommandResponse::from(command)))
}

/// 将状态转换为字符串
fn status_to_string(status: &CommandStatus) -> String {
    match status {
        CommandStatus::Pending => "pending".to_string(),
        CommandStatus::InTransit => "in_transit".to_string(),
        CommandStatus::Arrived => "arrived".to_string(),
        CommandStatus::Processing => "processing".to_string(),
        CommandStatus::Completed => "completed".to_string(),
        CommandStatus::Failed(msg) => format!("failed:{}", msg),
    }
}
