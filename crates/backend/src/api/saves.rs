//! 存档 API 模块
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 存档信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveInfo {
    /// 存档 ID
    pub id: String,
    /// 存档名称
    pub name: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 创建存档请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSaveRequest {
    /// 存档名称
    pub name: String,
}

/// 创建存档响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateSaveResponse {
    /// 存档 ID
    pub id: String,
    /// 存档名称
    pub name: String,
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
    path = "/api/saves",
    tag = "saves",
    responses(
        (status = 200, description = "获取存档列表成功", body = SaveListResponse)
    )
)]
pub async fn list_saves(State(state): State<Arc<AppState>>) -> GameResult<Json<SaveListResponse>> {
    let pool = state.db_pool.pool();

    let rows = sqlx::query("SELECT id, name, created_at, updated_at FROM saves")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            GameError::Database(crate::error::DatabaseError::QueryFailed(format!(
                "Failed to list saves: {}",
                e
            )))
        })?;

    let saves: Vec<SaveInfo> = rows
        .iter()
        .map(|row| SaveInfo {
            id: row.get(0),
            name: row.get(1),
            created_at: row.get(2),
            updated_at: row.get(3),
        })
        .collect();

    Ok(Json(SaveListResponse {
        total: saves.len(),
        saves,
    }))
}

/// 创建新存档
#[utoipa::path(
    post,
    path = "/api/saves",
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
    let pool = state.db_pool.pool();
    let id = Uuid::new_v4().to_string();
    let name = payload.name;
    let now = Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO saves (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(&id)
        .bind(&name)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| {
            GameError::Database(crate::error::DatabaseError::WriteFailed(format!(
                "Failed to create save: {}",
                e
            )))
        })?;

    Ok(Json(CreateSaveResponse {
        id,
        name,
        created_at: now,
        message: "Save created successfully".to_string(),
    }))
}

/// 获取存档详情
#[utoipa::path(
    get,
    path = "/api/saves/{save_id}",
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
    let pool = state.db_pool.pool();

    let row = sqlx::query("SELECT id, name, created_at, updated_at FROM saves WHERE id = ?1")
        .bind(&save_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            GameError::Database(crate::error::DatabaseError::QueryFailed(format!(
                "Failed to get save: {}",
                e
            )))
        })?;

    match row {
        Some(row) => Ok(Json(SaveInfo {
            id: row.get(0),
            name: row.get(1),
            created_at: row.get(2),
            updated_at: row.get(3),
        })),
        None => Err(GameError::NotFound {
            entity_type: "Save".to_string(),
            entity_id: save_id,
        }),
    }
}

/// 删除存档
#[utoipa::path(
    delete,
    path = "/api/saves/{save_id}",
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
    let pool = state.db_pool.pool();

    let result = sqlx::query("DELETE FROM saves WHERE id = ?1")
        .bind(&save_id)
        .execute(pool)
        .await
        .map_err(|e| {
            GameError::Database(crate::error::DatabaseError::WriteFailed(format!(
                "Failed to delete save: {}",
                e
            )))
        })?;

    if result.rows_affected() == 0 {
        Err(GameError::NotFound {
            entity_type: "Save".to_string(),
            entity_id: save_id,
        })
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
