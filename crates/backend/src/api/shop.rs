//! 商店 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::models::shop::ShopState;
use crate::db::repositories::shop::ShopRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 购买请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct PurchaseRequest {
    /// 物品 ID
    pub item_id: String,
    /// 数量
    pub quantity: u32,
}

/// 商店状态响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ShopResponse {
    /// 存档 ID
    pub save_id: String,
    /// 小馆名称
    pub name: String,
    /// 资金
    pub funds: u64,
    /// 声望
    pub reputation: f32,
    /// 餐厅等级
    pub restaurant_level: u32,
    /// 厨房等级
    pub kitchen_level: u32,
    /// 后院等级
    pub backyard_level: u32,
    /// 工作间等级
    pub workshop_level: u32,
}

impl From<ShopState> for ShopResponse {
    fn from(shop: ShopState) -> Self {
        Self {
            save_id: shop.save_id.to_string(),
            name: shop.name,
            funds: shop.funds,
            reputation: shop.reputation,
            restaurant_level: shop.restaurant_level,
            kitchen_level: shop.kitchen_level,
            backyard_level: shop.backyard_level,
            workshop_level: shop.workshop_level,
        }
    }
}

/// 更新资金请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFundsRequest {
    /// 新资金数额
    pub funds: u64,
}

/// 获取商店状态
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/shop",
    tag = "shop",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取商店状态成功", body = ShopResponse),
        (status = 404, description = "商店状态不存在")
    )
)]
pub async fn get_shop(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<ShopResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = ShopRepository::new(state.db_pool.pool().clone());
    let shop = repo
        .find_by_save_id(save_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "ShopState".to_string(),
            entity_id: save_id.to_string(),
        })?;

    Ok(Json(ShopResponse::from(shop)))
}

/// 购买物品
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/shop/purchase",
    tag = "shop",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = PurchaseRequest,
    responses(
        (status = 200, description = "购买成功", body = ShopResponse),
        (status = 400, description = "余额不足或物品不存在")
    )
)]
pub async fn purchase_item(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(_payload): Json<PurchaseRequest>,
) -> GameResult<Json<ShopResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = ShopRepository::new(state.db_pool.pool().clone());
    let shop = repo
        .find_by_save_id(save_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "ShopState".to_string(),
            entity_id: save_id.to_string(),
        })?;

    // TODO: 实际的购买逻辑，包括价格检查、库存更新等

    Ok(Json(ShopResponse::from(shop)))
}

/// 更新资金
#[utoipa::path(
    patch,
    path = "/api/v1/saves/{save_id}/shop/funds",
    tag = "shop",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = UpdateFundsRequest,
    responses(
        (status = 200, description = "更新成功", body = ShopResponse),
        (status = 404, description = "商店状态不存在")
    )
)]
pub async fn update_funds(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(payload): Json<UpdateFundsRequest>,
) -> GameResult<Json<ShopResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = ShopRepository::new(state.db_pool.pool().clone());
    let mut shop = repo
        .find_by_save_id(save_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "ShopState".to_string(),
            entity_id: save_id.to_string(),
        })?;

    shop.funds = payload.funds;
    repo.update(&shop).await?;

    Ok(Json(ShopResponse::from(shop)))
}
