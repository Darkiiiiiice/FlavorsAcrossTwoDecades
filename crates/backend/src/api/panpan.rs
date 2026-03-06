//! 盼盼机器人 API 模块

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::models::panpan::PanpanState;
use crate::db::repositories::panpan::PanpanRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 更新盼盼状态请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePanpanRequest {
    /// 信任等级
    pub trust_level: Option<u32>,
    /// 当前状态
    pub current_state: Option<String>,
    /// 当前任务
    pub current_task: Option<String>,
    /// 位置
    pub location: Option<String>,
}

/// 盼盼状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PanpanResponse {
    /// 名称
    pub name: String,
    /// 型号
    pub model: String,
    /// 制造日期
    pub manufacture_date: String,
    /// 个性特征 (JSON)
    pub personality: String,
    /// 信任等级
    pub trust_level: u32,
    /// 当前情绪
    pub emotion: String,
    /// 当前能量
    pub energy_current: u32,
    /// 最大能量
    pub energy_max: u32,
    /// 位置
    pub location: String,
    /// 当前状态
    pub current_state: String,
    /// 当前任务
    pub current_task: Option<String>,
}

impl PanpanResponse {
    fn from_state(state: PanpanState) -> Self {
        let personality_json =
            serde_json::to_string(&state.personality).unwrap_or_else(|_| "{}".to_string());
        Self {
            name: state.name,
            model: state.model,
            manufacture_date: state.manufacture_date.to_rfc3339(),
            personality: personality_json,
            trust_level: state.trust_level,
            emotion: emotion_to_string(&state.emotion),
            energy_current: state.energy_current,
            energy_max: state.energy_max,
            location: state.location,
            current_state: state.current_state,
            current_task: state.current_task,
        }
    }
}

/// 获取盼盼状态
#[utoipa::path(
    get,
    path = "/api/v1/panpan",
    tag = "panpan",
    responses(
        (status = 200, description = "获取盼盼状态成功", body = PanpanResponse),
        (status = 404, description = "盼盼状态不存在")
    )
)]
pub async fn get_panpan(State(state): State<Arc<AppState>>) -> GameResult<Json<PanpanResponse>> {
    let repo = PanpanRepository::new(state.db_pool.pool().clone());
    let panpan = repo.get().await?.ok_or_else(|| GameError::NotFound {
        entity_type: "PanpanState".to_string(),
        entity_id: "current".to_string(),
    })?;

    Ok(Json(PanpanResponse::from_state(panpan)))
}

/// 更新盼盼状态
#[utoipa::path(
    patch,
    path = "/api/v1/panpan",
    tag = "panpan",
    request_body = UpdatePanpanRequest,
    responses(
        (status = 200, description = "更新成功", body = PanpanResponse),
        (status = 404, description = "盼盼状态不存在")
    )
)]
pub async fn update_panpan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdatePanpanRequest>,
) -> GameResult<Json<PanpanResponse>> {
    let repo = PanpanRepository::new(state.db_pool.pool().clone());
    let mut panpan = repo.get().await?.ok_or_else(|| GameError::NotFound {
        entity_type: "PanpanState".to_string(),
        entity_id: "current".to_string(),
    })?;

    if let Some(trust_level) = payload.trust_level {
        panpan.trust_level = trust_level;
    }
    if let Some(current_state) = payload.current_state {
        panpan.current_state = current_state;
    }
    if let Some(current_task) = payload.current_task {
        panpan.current_task = Some(current_task);
    }
    if let Some(location) = payload.location {
        panpan.location = location;
    }

    repo.update(&panpan).await?;

    Ok(Json(PanpanResponse::from_state(panpan)))
}

fn emotion_to_string(emotion: &crate::game::panpan::Emotion) -> String {
    use crate::game::panpan::Emotion;
    match emotion {
        Emotion::Happy => "happy".to_string(),
        Emotion::Calm => "calm".to_string(),
        Emotion::Tired => "tired".to_string(),
        Emotion::Confused => "confused".to_string(),
        Emotion::Worried => "worried".to_string(),
        Emotion::Lonely => "lonely".to_string(),
        Emotion::Excited => "excited".to_string(),
    }
}
