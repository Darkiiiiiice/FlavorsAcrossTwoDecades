//! 顾客 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::models::customer::CustomerRecord;
use crate::db::repositories::customer::CustomerRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;

/// 更新顾客请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCustomerRequest {
    /// 好感度
    pub favorability: Option<u32>,
    /// 访问次数
    pub visit_count: Option<u32>,
    /// 偏好 (JSON)
    pub preferences: Option<String>,
}

/// 顾客响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerResponse {
    /// 顾客 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 顾客类型
    pub customer_type: String,
    /// 顾客名称
    pub name: String,
    /// 好感度
    pub favorability: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 上次访问时间
    pub last_visit: String,
    /// 偏好
    pub preferences: String,
}

impl From<CustomerRecord> for CustomerResponse {
    fn from(customer: CustomerRecord) -> Self {
        Self {
            id: customer.id.to_string(),
            save_id: customer.save_id.to_string(),
            customer_type: customer.customer_type,
            name: customer.name,
            favorability: customer.favorability,
            visit_count: customer.visit_count,
            last_visit: customer.last_visit.to_rfc3339(),
            preferences: customer.preferences,
        }
    }
}

/// 顾客列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerListResponse {
    /// 顾客列表
    pub customers: Vec<CustomerResponse>,
    /// 总数
    pub total: usize,
}

/// 获取顾客列表
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/customers",
    tag = "customers",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取顾客列表成功", body = CustomerListResponse)
    )
)]
pub async fn list_customers(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<CustomerListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let customers = repo.find_by_save_id(save_id).await?;

    let customer_responses: Vec<CustomerResponse> =
        customers.into_iter().map(CustomerResponse::from).collect();

    Ok(Json(CustomerListResponse {
        total: customer_responses.len(),
        customers: customer_responses,
    }))
}

/// 获取顾客详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/customers/{customer_id}",
    tag = "customers",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("customer_id" = String, Path, description = "顾客 ID")
    ),
    responses(
        (status = 200, description = "获取顾客成功", body = CustomerResponse),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path((_save_id, customer_id)): Path<(String, String)>,
) -> GameResult<Json<CustomerResponse>> {
    let customer_id = Uuid::parse_str(&customer_id).map_err(|e| GameError::Validation {
        details: format!("Invalid customer_id UUID: {}", e),
    })?;

    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let customer = repo
        .find_by_id(customer_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Customer".to_string(),
            entity_id: customer_id.to_string(),
        })?;

    Ok(Json(CustomerResponse::from(customer)))
}

/// 更新顾客信息
#[utoipa::path(
    patch,
    path = "/api/v1/saves/{save_id}/customers/{customer_id}",
    tag = "customers",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("customer_id" = String, Path, description = "顾客 ID")
    ),
    request_body = UpdateCustomerRequest,
    responses(
        (status = 200, description = "更新成功"),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path((_save_id, customer_id)): Path<(String, String)>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> GameResult<()> {
    let customer_id = Uuid::parse_str(&customer_id).map_err(|e| GameError::Validation {
        details: format!("Invalid customer_id UUID: {}", e),
    })?;

    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let mut customer = repo
        .find_by_id(customer_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Customer".to_string(),
            entity_id: customer_id.to_string(),
        })?;

    if let Some(favorability) = payload.favorability {
        customer.favorability = favorability;
    }
    if let Some(visit_count) = payload.visit_count {
        customer.visit_count = visit_count;
    }
    if let Some(preferences) = payload.preferences {
        customer.preferences = preferences;
    }

    repo.update(&customer).await?;

    Ok(())
}

/// 删除顾客
#[utoipa::path(
    delete,
    path = "/api/v1/saves/{save_id}/customers/{customer_id}",
    tag = "customers",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("customer_id" = String, Path, description = "顾客 ID")
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn delete_customer(
    State(state): State<Arc<AppState>>,
    Path((_save_id, customer_id)): Path<(String, String)>,
) -> GameResult<()> {
    let customer_id = Uuid::parse_str(&customer_id).map_err(|e| GameError::Validation {
        details: format!("Invalid customer_id UUID: {}", e),
    })?;

    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    repo.delete(customer_id).await?;

    Ok(())
}
