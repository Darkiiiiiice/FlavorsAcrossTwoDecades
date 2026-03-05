//! 存档 API 模块

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::models::save::Save;
use crate::db::repositories::save_repository::SaveRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 存档信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveInfo {
    /// 存档 ID
    pub id: String,
    /// 存档名称
    pub name: String,
    /// 玩家名称
    pub player_name: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl From<Save> for SaveInfo {
    fn from(save: Save) -> Self {
        Self {
            id: save.id.to_string(),
            name: save.name,
            player_name: save.player_name,
            created_at: save.created_at.to_rfc3339(),
            updated_at: save.last_played.to_rfc3339(),
        }
    }
}

/// 创建存档请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSaveRequest {
    /// 存档名称
    pub name: String,
    /// 玩家名称
    pub player_name: String,
}

/// 创建存档响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateSaveResponse {
    /// 存档 ID
    pub id: String,
    /// 存档名称
    pub name: String,
    /// 玩家名称
    pub player_name: String,
    /// 创建时间
    pub created_at: String,
    /// 消息
    pub message: String,
}

/// 存档列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct SaveListResponse {
    /// 存档列表
    pub saves: Vec<SaveInfo>,
    /// 总数
    pub total: usize,
}

/// 获取所有存档
#[utoipa::path(
    get,
    path = "/api/v1/saves",
    tag = "saves",
    responses(
        (status = 200, description = "获取存档列表成功", body = SaveListResponse)
    )
)]
pub async fn list_saves(State(state): State<Arc<AppState>>) -> GameResult<Json<SaveListResponse>> {
    let repo = SaveRepository::new(state.db_pool.pool().clone());
    let saves = repo.find_all().await?;

    let save_infos: Vec<SaveInfo> = saves.into_iter().map(SaveInfo::from).collect();

    Ok(Json(SaveListResponse {
        total: save_infos.len(),
        saves: save_infos,
    }))
}

/// 创建新存档
#[utoipa::path(
    post,
    path = "/api/v1/saves",
    tag = "saves",
    request_body = CreateSaveRequest,
    responses(
        (status = 201, description = "存档创建成功", body = CreateSaveResponse),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_save(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSaveRequest>,
) -> GameResult<Json<CreateSaveResponse>> {
    let repo = SaveRepository::new(state.db_pool.pool().clone());

    let save = Save::new(payload.name, payload.player_name);
    let save_id = save.id;
    let created_at = save.created_at;

    repo.create(&save).await?;

    Ok(Json(CreateSaveResponse {
        id: save_id.to_string(),
        name: save.name,
        player_name: save.player_name,
        created_at: created_at.to_rfc3339(),
        message: "Save created successfully".to_string(),
    }))
}

/// 获取存档详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}",
    tag = "saves",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取存档成功", body = SaveInfo),
        (status = 404, description = "存档不存在")
    )
)]
pub async fn get_save(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<SaveInfo>> {
    let repo = SaveRepository::new(state.db_pool.pool().clone());

    let save_id = uuid::Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let save = repo
        .find_by_id(save_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Save".to_string(),
            entity_id: save_id.to_string(),
        })?;

    Ok(Json(SaveInfo::from(save)))
}

/// 删除存档
#[utoipa::path(
    delete,
    path = "/api/v1/saves/{save_id}",
    tag = "saves",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 204, description = "存档删除成功"),
        (status = 404, description = "存档不存在")
    )
)]
pub async fn delete_save(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<StatusCode> {
    let repo = SaveRepository::new(state.db_pool.pool().clone());

    let save_id = uuid::Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    // 先检查存档是否存在
    let save = repo.find_by_id(save_id).await?;
    if save.is_none() {
        return Err(GameError::NotFound {
            entity_type: "Save".to_string(),
            entity_id: save_id.to_string(),
        });
    }

    repo.delete(save_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
