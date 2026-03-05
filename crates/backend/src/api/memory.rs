//! 记忆碎片 API 模块

use axum::{
    Json,
    extract::{Path, State, Query},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::db::models::memory::MemoryFragment;
use crate::db::repositories::memory::MemoryRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 记忆碎片查询参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct MemoryQueryParams {
    /// 只获取已解锁的
    pub unlocked_only: Option<bool>,
}

/// 解锁记忆碎片请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UnlockMemoryRequest {
    /// 解锁条件
    pub trigger_condition: Option<String>,
}

/// 记忆碎片响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryResponse {
    /// 碎片 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 碎片类型
    pub fragment_type: String,
    /// 标题
    pub title: String,
    /// 内容
    pub content: String,
    /// 是否已解锁
    pub is_unlocked: bool,
    /// 解锁时间
    pub unlocked_at: Option<String>,
    /// 触发条件
    pub trigger_condition: String,
}

impl From<MemoryFragment> for MemoryResponse {
    fn from(fragment: MemoryFragment) -> Self {
        Self {
            id: fragment.id.to_string(),
            save_id: fragment.save_id.to_string(),
            fragment_type: fragment.fragment_type,
            title: fragment.title,
            content: fragment.content,
            is_unlocked: fragment.is_unlocked,
            unlocked_at: fragment.unlocked_at.map(|t| t.to_rfc3339()),
            trigger_condition: fragment.trigger_condition,
        }
    }
}

/// 记忆碎片列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryListResponse {
    /// 碎片列表
    pub memories: Vec<MemoryResponse>,
    /// 总数
    pub total: usize,
}

/// 获取记忆碎片列表
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/memories",
    tag = "memories",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("unlocked_only" = Option<bool>, Query, description = "只获取已解锁的")
    ),
    responses(
        (status = 200, description = "获取记忆碎片列表成功", body = MemoryListResponse)
    )
)]
pub async fn list_memories(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Query(params): Query<MemoryQueryParams>,
) -> GameResult<Json<MemoryListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        }
    })?;

    let repo = MemoryRepository::new(state.db_pool.pool().clone());
    let memories = if params.unlocked_only.unwrap_or(false) {
        repo.find_unlocked(save_id).await?
    } else {
        repo.find_by_save_id(save_id).await?
    };

    let memory_responses: Vec<MemoryResponse> = memories
        .into_iter()
        .map(MemoryResponse::from)
        .collect();

    Ok(Json(MemoryListResponse {
        total: memory_responses.len(),
        memories: memory_responses,
    }))
}

/// 获取记忆碎片详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/memories/{memory_id}",
    tag = "memories",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("memory_id" = String, Path, description = "记忆碎片 ID")
    ),
    responses(
        (status = 200, description = "获取记忆碎片成功", body = MemoryResponse),
        (status = 404, description = "记忆碎片不存在")
    )
)]
pub async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path((_save_id, memory_id)): Path<(String, String)>,
) -> GameResult<Json<MemoryResponse>> {
    let memory_id = Uuid::parse_str(&memory_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid memory_id UUID: {}", e),
        }
    })?;

    let repo = MemoryRepository::new(state.db_pool.pool().clone());
    let memory = repo.find_by_id(memory_id).await?.ok_or_else(|| {
        GameError::NotFound {
            entity_type: "MemoryFragment".to_string(),
            entity_id: memory_id.to_string(),
        }
    })?;

    Ok(Json(MemoryResponse::from(memory)))
}

/// 解锁记忆碎片
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/memories/{memory_id}/unlock",
    tag = "memories",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("memory_id" = String, Path, description = "记忆碎片 ID")
    ),
    request_body = UnlockMemoryRequest,
    responses(
        (status = 200, description = "解锁成功", body = MemoryResponse),
        (status = 404, description = "记忆碎片不存在")
    )
)]
pub async fn unlock_memory(
    State(state): State<Arc<AppState>>,
    Path((_save_id, memory_id)): Path<(String, String)>,
    Json(_payload): Json<UnlockMemoryRequest>,
) -> GameResult<Json<MemoryResponse>> {
    let memory_id = Uuid::parse_str(&memory_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid memory_id UUID: {}", e),
        }
    })?;

    let repo = MemoryRepository::new(state.db_pool.pool().clone());
    let mut memory = repo.find_by_id(memory_id).await?.ok_or_else(|| {
        GameError::NotFound {
            entity_type: "MemoryFragment".to_string(),
            entity_id: memory_id.to_string(),
        }
    })?;

    if !memory.is_unlocked {
        memory.is_unlocked = true;
        memory.unlocked_at = Some(chrono::Utc::now());
        repo.update(&memory).await?;
    }

    Ok(Json(MemoryResponse::from(memory)))
}
