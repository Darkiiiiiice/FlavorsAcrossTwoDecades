//! 菜园 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::models::garden::GardenPlot;
use crate::db::repositories::garden::GardenRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 种植请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlantRequest {
    /// 作物类型
    pub crop_type: String,
}

/// 浇水请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct WaterRequest {
    /// 浇水量
    pub water_amount: u32,
}

/// 地块响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PlotResponse {
    /// 地块 ID
    pub id: String,
    /// 地块编号
    pub plot_number: u32,
    /// 是否已解锁
    pub is_unlocked: bool,
    /// 当前作物 (JSON)
    pub current_crop: Option<String>,
    /// 肥力
    pub fertility: u32,
    /// 湿度
    pub moisture: u32,
}

impl From<GardenPlot> for PlotResponse {
    fn from(plot: GardenPlot) -> Self {
        Self {
            id: plot.id,
            plot_number: plot.plot_number,
            is_unlocked: plot.is_unlocked,
            current_crop: plot.current_crop,
            fertility: plot.fertility,
            moisture: plot.moisture,
        }
    }
}

/// 地块列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PlotListResponse {
    /// 地块列表
    pub plots: Vec<PlotResponse>,
    /// 总数
    pub total: usize,
}

/// 获取菜园地块列表
#[utoipa::path(
    get,
    path = "/api/v1/garden/plots",
    tag = "garden",
    responses(
        (status = 200, description = "获取地块列表成功", body = PlotListResponse)
    )
)]
pub async fn list_plots(State(state): State<Arc<AppState>>) -> GameResult<Json<PlotListResponse>> {
    let repo = GardenRepository::new(state.db_pool.pool().clone());
    let plots = repo.find_all().await?;

    let plot_responses: Vec<PlotResponse> = plots.into_iter().map(PlotResponse::from).collect();

    Ok(Json(PlotListResponse {
        total: plot_responses.len(),
        plots: plot_responses,
    }))
}

/// 获取地块详情
#[utoipa::path(
    get,
    path = "/api/v1/garden/plots/{plot_id}",
    tag = "garden",
    params(
        ("plot_id" = String, Path, description = "地块 ID")
    ),
    responses(
        (status = 200, description = "获取地块成功", body = PlotResponse),
        (status = 404, description = "地块不存在")
    )
)]
pub async fn get_plot(
    State(state): State<Arc<AppState>>,
    Path(plot_id): Path<String>,
) -> GameResult<Json<PlotResponse>> {
    let repo = GardenRepository::new(state.db_pool.pool().clone());
    let plot = repo
        .find_by_id(&plot_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Plot".to_string(),
            entity_id: plot_id,
        })?;

    Ok(Json(PlotResponse::from(plot)))
}

/// 种植作物
#[utoipa::path(
    post,
    path = "/api/v1/garden/plots/{plot_id}/plant",
    tag = "garden",
    params(
        ("plot_id" = String, Path, description = "地块 ID")
    ),
    request_body = PlantRequest,
    responses(
        (status = 200, description = "种植成功", body = PlotResponse),
        (status = 404, description = "地块不存在")
    )
)]
pub async fn plant_crop(
    State(state): State<Arc<AppState>>,
    Path(plot_id): Path<String>,
    Json(payload): Json<PlantRequest>,
) -> GameResult<Json<PlotResponse>> {
    let repo = GardenRepository::new(state.db_pool.pool().clone());
    let mut plot = repo
        .find_by_id(&plot_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Plot".to_string(),
            entity_id: plot_id,
        })?;

    if !plot.is_unlocked {
        return Err(GameError::Validation {
            details: "Plot is not unlocked".to_string(),
        });
    }

    if plot.current_crop.is_some() {
        return Err(GameError::Validation {
            details: "Plot already has a crop".to_string(),
        });
    }

    // 将作物信息存储为 JSON
    let crop_info = serde_json::json!({
        "type": payload.crop_type,
        "planted_at": chrono::Utc::now().to_rfc3339()
    })
    .to_string();
    plot.current_crop = Some(crop_info);

    repo.update(&plot).await?;

    Ok(Json(PlotResponse::from(plot)))
}

/// 浇水
#[utoipa::path(
    post,
    path = "/api/v1/garden/plots/{plot_id}/water",
    tag = "garden",
    params(
        ("plot_id" = String, Path, description = "地块 ID")
    ),
    request_body = WaterRequest,
    responses(
        (status = 200, description = "浇水成功", body = PlotResponse),
        (status = 404, description = "地块不存在")
    )
)]
pub async fn water_plot(
    State(state): State<Arc<AppState>>,
    Path(plot_id): Path<String>,
    Json(payload): Json<WaterRequest>,
) -> GameResult<Json<PlotResponse>> {
    let repo = GardenRepository::new(state.db_pool.pool().clone());
    let mut plot = repo
        .find_by_id(&plot_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Plot".to_string(),
            entity_id: plot_id,
        })?;

    plot.moisture = (plot.moisture + payload.water_amount).min(100);
    repo.update(&plot).await?;

    Ok(Json(PlotResponse::from(plot)))
}

/// 收获作物
#[utoipa::path(
    post,
    path = "/api/v1/garden/plots/{plot_id}/harvest",
    tag = "garden",
    params(
        ("plot_id" = String, Path, description = "地块 ID")
    ),
    responses(
        (status = 200, description = "收获成功", body = PlotResponse),
        (status = 404, description = "地块不存在")
    )
)]
pub async fn harvest_crop(
    State(state): State<Arc<AppState>>,
    Path(plot_id): Path<String>,
) -> GameResult<Json<PlotResponse>> {
    let repo = GardenRepository::new(state.db_pool.pool().clone());
    let mut plot = repo
        .find_by_id(&plot_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Plot".to_string(),
            entity_id: plot_id,
        })?;

    if plot.current_crop.is_none() {
        return Err(GameError::Validation {
            details: "No crop to harvest".to_string(),
        });
    }

    // TODO: 检查是否成熟，添加收获物品到库存
    // 这里简化处理，直接清除作物
    plot.current_crop = None;

    repo.update(&plot).await?;

    Ok(Json(PlotResponse::from(plot)))
}
