//! Panda 机器人 API 模块

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::models::panda::PandaState;
use crate::db::repositories::panda::PandaRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 更新 Panda 状态请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePandaRequest {
    /// 机器人名字
    pub name: Option<String>,
    /// 信任等级
    pub trust_level: Option<u32>,
    /// 当前状态
    pub current_state: Option<String>,
    /// 当前任务
    pub current_task: Option<String>,
    /// 位置
    pub location: Option<String>,
}

/// Panda 状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PandaResponse {
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

impl PandaResponse {
    fn from_state(state: PandaState) -> Self {
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

/// 获取 Panda 状态
#[utoipa::path(
    get,
    path = "/api/v1/panda",
    tag = "panda",
    responses(
        (status = 200, description = "获取 Panda 状态成功", body = PandaResponse),
        (status = 404, description = "Panda 状态不存在")
    )
)]
pub async fn get_panda(State(state): State<Arc<AppState>>) -> GameResult<Json<PandaResponse>> {
    let repo = PandaRepository::new(state.db_pool.pool().clone());
    let panda = repo.get().await?.ok_or_else(|| GameError::NotFound {
        entity_type: "PandaState".to_string(),
        entity_id: "current".to_string(),
    })?;

    Ok(Json(PandaResponse::from_state(panda)))
}

/// 更新 Panda 状态
#[utoipa::path(
    patch,
    path = "/api/v1/panda",
    tag = "panda",
    request_body = UpdatePandaRequest,
    responses(
        (status = 200, description = "更新成功", body = PandaResponse),
        (status = 404, description = "Panda 状态不存在")
    )
)]
pub async fn update_panda(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdatePandaRequest>,
) -> GameResult<Json<PandaResponse>> {
    let repo = PandaRepository::new(state.db_pool.pool().clone());
    let mut panda = repo.get().await?.ok_or_else(|| GameError::NotFound {
        entity_type: "PandaState".to_string(),
        entity_id: "current".to_string(),
    })?;

    if let Some(trust_level) = payload.trust_level {
        panda.trust_level = trust_level;
    }
    if let Some(current_state) = payload.current_state {
        panda.current_state = current_state;
    }
    if let Some(current_task) = payload.current_task {
        panda.current_task = Some(current_task);
    }
    if let Some(location) = payload.location {
        panda.location = location;
    }
    if let Some(name) = payload.name {
        if !name.trim().is_empty() {
            panda.name = name.trim().to_string();
        }
    }

    repo.update(&panda).await?;

    Ok(Json(PandaResponse::from_state(panda)))
}

fn emotion_to_string(emotion: &crate::game::panda::Emotion) -> String {
    use crate::game::panda::Emotion;
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
