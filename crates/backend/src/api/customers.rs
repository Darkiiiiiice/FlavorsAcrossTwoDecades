//! 顾客 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::db::models::customer::{CustomerRecord, PreferenceRecord};
use crate::db::repositories::customer::CustomerRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;
use crate::game::customer::{CustomerType, Customer};
use crate::game::customer::preference::{Preference, FlavorPreference, DietaryRestriction};

/// 偏好响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PreferenceResponse {
    /// 偏好ID
    pub id: i64,
    /// 口味偏好 (0=Light, 1=Medium, 2=Heavy, 3=Spicy, 4=SweetSour)
    pub flavor: i32,
    /// 饮食限制 (0=None, 1=Vegetarian, 2=Halal, 3=GlutenFree, 4=LowSugar)
    pub dietary: i32,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: u32,
    /// 耐心值 (0-100)
    pub patience: u32,
    /// 喜欢的菜品类型
    pub favorite_categories: Vec<String>,
}

impl From<PreferenceRecord> for PreferenceResponse {
    fn from(pref: PreferenceRecord) -> Self {
        let favorite_categories: Vec<String> = serde_json::from_str(&pref.favorite_categories)
            .unwrap_or_default();
        Self {
            id: pref.id,
            flavor: i32::from(pref.flavor),
            dietary: i32::from(pref.dietary),
            price_sensitivity: pref.price_sensitivity,
            patience: pref.patience,
            favorite_categories,
        }
    }
}

/// 更新顾客请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCustomerRequest {
    /// 好感度 (0-1000)
    pub affinity: Option<u32>,
    /// 访问次数
    pub visit_count: Option<u32>,
    /// 故事背景
    pub story_background: Option<String>,
    /// 偏好更新
    pub preference: Option<UpdatePreferenceRequest>,
}

/// 更新偏好请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePreferenceRequest {
    /// 口味偏好 (0-4)
    pub flavor: Option<i32>,
    /// 饮食限制 (0-4)
    pub dietary: Option<i32>,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: Option<u32>,
    /// 耐心值 (0-100)
    pub patience: Option<u32>,
    /// 喜欢的菜品类型
    pub favorite_categories: Option<Vec<String>>,
}

/// 顾客响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerResponse {
    /// 顾客 ID
    pub id: i64,
    /// 顾客类型 (0=Normal, 1=Foodie, 2=Critic)
    pub customer_type: i32,
    /// 顾客名称
    pub name: String,
    /// 好感度
    pub affinity: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 故事背景
    pub story_background: String,
    /// 偏好
    pub preference: PreferenceResponse,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl From<CustomerRecord> for CustomerResponse {
    fn from(customer: CustomerRecord) -> Self {
        Self {
            id: customer.id,
            customer_type: i32::from(customer.customer_type),
            name: customer.name,
            affinity: customer.affinity,
            visit_count: customer.visit_count,
            story_background: customer.story_background,
            preference: PreferenceResponse::from(customer.preference),
            created_at: customer.created_at.to_rfc3339(),
            updated_at: customer.updated_at.to_rfc3339(),
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
    path = "/api/v1/customers",
    tag = "customers",
    responses(
        (status = 200, description = "获取顾客列表成功", body = CustomerListResponse)
    )
)]
pub async fn list_customers(
    State(state): State<Arc<AppState>>,
) -> GameResult<Json<CustomerListResponse>> {
    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let customers = repo.find_recent().await?;

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
    path = "/api/v1/customers/{customer_id}",
    tag = "customers",
    params(
        ("customer_id" = i64, Path, description = "顾客 ID")
    ),
    responses(
        (status = 200, description = "获取顾客成功", body = CustomerResponse),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<i64>,
) -> GameResult<Json<CustomerResponse>> {
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
    path = "/api/v1/customers/{customer_id}",
    tag = "customers",
    params(
        ("customer_id" = i64, Path, description = "顾客 ID")
    ),
    request_body = UpdateCustomerRequest,
    responses(
        (status = 200, description = "更新成功"),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<i64>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> GameResult<()> {
    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let record = repo
        .find_by_id(customer_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Customer".to_string(),
            entity_id: customer_id.to_string(),
        })?;

    // 构建更新后的 Customer
    let mut customer = record.to_customer();

    if let Some(affinity) = payload.affinity {
        customer.affinity = affinity;
    }
    if let Some(visit_count) = payload.visit_count {
        customer.visit_count = visit_count;
    }
    if let Some(story_background) = payload.story_background {
        customer.story_background = story_background;
    }
    if let Some(pref_update) = payload.preference {
        if let Some(flavor) = pref_update.flavor {
            customer.preference.flavor = FlavorPreference::try_from(flavor)
                .map_err(|e| GameError::Validation { details: e })?;
        }
        if let Some(dietary) = pref_update.dietary {
            customer.preference.dietary = DietaryRestriction::try_from(dietary)
                .map_err(|e| GameError::Validation { details: e })?;
        }
        if let Some(price_sensitivity) = pref_update.price_sensitivity {
            customer.preference.price_sensitivity = price_sensitivity;
        }
        if let Some(patience) = pref_update.patience {
            customer.preference.patience = patience;
        }
        if let Some(favorite_categories) = pref_update.favorite_categories {
            customer.preference.favorite_categories = favorite_categories;
        }
    }

    repo.update(&customer).await?;

    Ok(())
}

/// 删除顾客
#[utoipa::path(
    delete,
    path = "/api/v1/customers/{customer_id}",
    tag = "customers",
    params(
        ("customer_id" = i64, Path, description = "顾客 ID")
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 404, description = "顾客不存在")
    )
)]
pub async fn delete_customer(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<i64>,
) -> GameResult<()> {
    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    repo.delete(customer_id).await?;

    Ok(())
}

/// 创建顾客请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerRequest {
    /// 顾客名称
    pub name: String,
    /// 年龄
    pub age: Option<u32>,
    /// 职业
    pub occupation: Option<String>,
    /// 顾客类型 (0=Normal, 1=Foodie, 2=Critic)
    pub customer_type: i32,
    /// 故事背景
    pub story_background: Option<String>,
    /// 偏好
    pub preference: Option<CreatePreferenceRequest>,
}

/// 创建偏好请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePreferenceRequest {
    /// 口味偏好 (0-4)
    pub flavor: Option<i32>,
    /// 饮食限制 (0-4)
    pub dietary: Option<i32>,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: Option<u32>,
    /// 耐心值 (0-100)
    pub patience: Option<u32>,
    /// 喜欢的菜品类型
    pub favorite_categories: Option<Vec<String>>,
}

/// 创建顾客响应
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateCustomerResponse {
    /// 顾客 ID
    pub id: i64,
}

/// 创建顾客
#[utoipa::path(
    post,
    path = "/api/v1/customers",
    tag = "customers",
    request_body = CreateCustomerRequest,
    responses(
        (status = 201, description = "创建成功", body = CreateCustomerResponse)
    )
)]
pub async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCustomerRequest>,
) -> GameResult<Json<CreateCustomerResponse>> {
    let customer_type = CustomerType::try_from(payload.customer_type)
        .map_err(|e| GameError::Validation { details: e })?;

    let mut preference = Preference::new();
    if let Some(pref) = payload.preference {
        if let Some(flavor) = pref.flavor {
            preference.flavor = FlavorPreference::try_from(flavor)
                .map_err(|e| GameError::Validation { details: e })?;
        }
        if let Some(dietary) = pref.dietary {
            preference.dietary = DietaryRestriction::try_from(dietary)
                .map_err(|e| GameError::Validation { details: e })?;
        }
        if let Some(price_sensitivity) = pref.price_sensitivity {
            preference.price_sensitivity = price_sensitivity;
        }
        if let Some(patience) = pref.patience {
            preference.patience = patience;
        }
        if let Some(favorite_categories) = pref.favorite_categories {
            preference.favorite_categories = favorite_categories;
        }
    }

    let customer = Customer {
        id: 0,
        name: payload.name,
        age: payload.age.unwrap_or(30),
        occupation: payload.occupation.unwrap_or_default(),
        customer_type,
        preference,
        affinity: 0,
        visit_count: 0,
        story_background: payload.story_background.unwrap_or_default(),
    };

    let repo = CustomerRepository::new(state.db_pool.pool().clone());
    let id = repo.create(&customer).await?;

    Ok(Json(CreateCustomerResponse { id }))
}
