//! 旅行 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::models::travel::Travel;
use crate::db::repositories::travel::TravelRepository;
use crate::error::{GameError, GameResult};
use crate::game::travel::TravelStatus;
use crate::game::AppState;

/// 开始旅行请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartTravelRequest {
    /// 目的地 ID
    pub destination_id: String,
    /// 旅行天数
    pub duration_days: Option<u32>,
}

/// 旅行状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct TravelResponse {
    /// 旅行 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 目的地
    pub destination: String,
    /// 开始时间
    pub started_at: String,
    /// 预计返回时间
    pub expected_return: String,
    /// 状态
    pub status: String,
    /// 奖励 (JSON)
    pub rewards: Option<String>,
}

impl TravelResponse {
    fn from_travel(travel: Travel) -> Self {
        Self {
            id: travel.id.to_string(),
            save_id: travel.save_id.to_string(),
            destination: travel.destination,
            started_at: travel.started_at.to_rfc3339(),
            expected_return: travel.expected_return.to_rfc3339(),
            status: status_to_string(&travel.status),
            rewards: travel.rewards,
        }
    }
}

/// 旅行列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct TravelListResponse {
    /// 旅行列表
    pub travels: Vec<TravelResponse>,
    /// 总数
    pub total: usize,
}

/// 获取旅行记录列表
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/travels",
    tag = "travels",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取旅行列表成功", body = TravelListResponse)
    )
)]
pub async fn list_travels(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<TravelListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        }
    })?;

    let repo = TravelRepository::new(state.db_pool.pool().clone());
    let travels = repo.find_by_save_id(save_id).await?;

    let travel_responses: Vec<TravelResponse> = travels
        .into_iter()
        .map(TravelResponse::from_travel)
        .collect();

    Ok(Json(TravelListResponse {
        total: travel_responses.len(),
        travels: travel_responses,
    }))
}

/// 获取旅行详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/travels/{travel_id}",
    tag = "travels",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("travel_id" = String, Path, description = "旅行 ID")
    ),
    responses(
        (status = 200, description = "获取旅行成功", body = TravelResponse),
        (status = 404, description = "旅行不存在")
    )
)]
pub async fn get_travel(
    State(state): State<Arc<AppState>>,
    Path((_save_id, travel_id)): Path<(String, String)>,
) -> GameResult<Json<TravelResponse>> {
    let travel_id = Uuid::parse_str(&travel_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid travel_id UUID: {}", e),
        }
    })?;

    let repo = TravelRepository::new(state.db_pool.pool().clone());
    let travel = repo.find_by_id(travel_id).await?.ok_or_else(|| {
        GameError::NotFound {
            entity_type: "Travel".to_string(),
            entity_id: travel_id.to_string(),
        }
    })?;

    Ok(Json(TravelResponse::from_travel(travel)))
}

/// 开始旅行
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/travels",
    tag = "travels",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = StartTravelRequest,
    responses(
        (status = 201, description = "旅行开始成功", body = TravelResponse),
        (status = 400, description = "已有进行中的旅行")
    )
)]
pub async fn start_travel(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(payload): Json<StartTravelRequest>,
) -> GameResult<Json<TravelResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        }
    })?;

    let repo = TravelRepository::new(state.db_pool.pool().clone());

    // 检查是否有进行中的旅行
    let existing_travels = repo.find_by_save_id(save_id).await?;
    let has_ongoing = existing_travels.iter().any(|t| t.status == TravelStatus::InProgress || t.status == TravelStatus::Preparing);
    
    if has_ongoing {
        return Err(GameError::Validation {
            details: "Already have an ongoing travel".to_string(),
        });
    }

    let duration_days = payload.duration_days.unwrap_or(7);
    let travel = Travel::new(save_id, payload.destination_id, duration_days);

    repo.create(&travel).await?;

    Ok(Json(TravelResponse::from_travel(travel)))
}

/// 完成旅行
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/travels/{travel_id}/complete",
    tag = "travels",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("travel_id" = String, Path, description = "旅行 ID")
    ),
    responses(
        (status = 200, description = "旅行完成", body = TravelResponse),
        (status = 404, description = "旅行不存在")
    )
)]
pub async fn complete_travel(
    State(state): State<Arc<AppState>>,
    Path((_save_id, travel_id)): Path<(String, String)>,
) -> GameResult<Json<TravelResponse>> {
    let travel_id = Uuid::parse_str(&travel_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid travel_id UUID: {}", e),
        }
    })?;

    let repo = TravelRepository::new(state.db_pool.pool().clone());
    let mut travel = repo.find_by_id(travel_id).await?.ok_or_else(|| {
        GameError::NotFound {
            entity_type: "Travel".to_string(),
            entity_id: travel_id.to_string(),
        }
    })?;

    travel.status = TravelStatus::Completed;
    // TODO: 生成旅行奖励

    repo.update(&travel).await?;

    Ok(Json(TravelResponse::from_travel(travel)))
}

/// 获取当前旅行状态
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/travels/current",
    tag = "travels",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取当前旅行成功", body = Option<TravelResponse>)
    )
)]
pub async fn get_current_travel(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<Option<TravelResponse>>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| {
        GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        }
    })?;

    let repo = TravelRepository::new(state.db_pool.pool().clone());
    let travels = repo.find_by_save_id(save_id).await?;
    
    let current = travels.into_iter()
        .find(|t| t.status == TravelStatus::InProgress || t.status == TravelStatus::Preparing);

    Ok(Json(current.map(TravelResponse::from_travel)))
}

fn status_to_string(status: &TravelStatus) -> String {
    match status {
        TravelStatus::Preparing => "preparing".to_string(),
        TravelStatus::InProgress => "in_progress".to_string(),
        TravelStatus::Completed => "completed".to_string(),
        TravelStatus::Cancelled => "cancelled".to_string(),
    }
}
