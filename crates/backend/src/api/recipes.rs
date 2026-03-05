//! 菜谱 API 模块

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::models::recipe::Recipe;
use crate::db::repositories::recipe::RecipeRepository;
use crate::error::{GameError, GameResult};
use crate::game::AppState;
use crate::game::recipe::{RecipeCategory, RecipeSource, RecipeStatus};

/// 创建菜谱请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRecipeRequest {
    /// 菜谱名称
    pub name: String,
    /// 菜系
    pub category: String,
    /// 食材列表 (JSON)
    pub ingredients: String,
    /// 来源
    pub source: String,
    /// 解锁条件
    pub unlock_condition: Option<String>,
}

/// 更新菜谱状态请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRecipeStatusRequest {
    /// 新状态
    pub status: String,
}

/// 菜谱响应
#[derive(Debug, Serialize, ToSchema)]
pub struct RecipeResponse {
    /// 菜谱 ID
    pub id: String,
    /// 存档 ID
    pub save_id: String,
    /// 菜谱名称
    pub name: String,
    /// 菜系
    pub category: String,
    /// 状态
    pub status: String,
    /// 食材列表
    pub ingredients: String,
    /// 来源
    pub source: String,
    /// 解锁条件
    pub unlock_condition: Option<String>,
}

impl From<Recipe> for RecipeResponse {
    fn from(recipe: Recipe) -> Self {
        Self {
            id: recipe.id.to_string(),
            save_id: recipe.save_id.to_string(),
            name: recipe.name,
            category: category_to_string(&recipe.category),
            status: status_to_string(&recipe.status),
            ingredients: recipe.ingredients,
            source: source_to_string(&recipe.source),
            unlock_condition: recipe.unlock_condition,
        }
    }
}

/// 菜谱列表响应
#[derive(Debug, Serialize, ToSchema)]
pub struct RecipeListResponse {
    /// 菜谱列表
    pub recipes: Vec<RecipeResponse>,
    /// 总数
    pub total: usize,
}

/// 创建菜谱
#[utoipa::path(
    post,
    path = "/api/v1/saves/{save_id}/recipes",
    tag = "recipes",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    request_body = CreateRecipeRequest,
    responses(
        (status = 201, description = "菜谱创建成功", body = RecipeResponse),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_recipe(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
    Json(payload): Json<CreateRecipeRequest>,
) -> GameResult<Json<RecipeResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let category = string_to_category(&payload.category)?;
    let source = string_to_source(&payload.source)?;

    let recipe = Recipe {
        id: Uuid::new_v4(),
        save_id,
        name: payload.name,
        category,
        status: RecipeStatus::Fuzzy,
        ingredients: payload.ingredients,
        source,
        unlock_condition: payload.unlock_condition,
    };

    let repo = RecipeRepository::new(state.db_pool.pool().clone());
    repo.create(&recipe).await?;

    Ok(Json(RecipeResponse::from(recipe)))
}

/// 获取菜谱列表
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/recipes",
    tag = "recipes",
    params(
        ("save_id" = String, Path, description = "存档 ID")
    ),
    responses(
        (status = 200, description = "获取菜谱列表成功", body = RecipeListResponse)
    )
)]
pub async fn list_recipes(
    State(state): State<Arc<AppState>>,
    Path(save_id): Path<String>,
) -> GameResult<Json<RecipeListResponse>> {
    let save_id = Uuid::parse_str(&save_id).map_err(|e| GameError::Validation {
        details: format!("Invalid UUID: {}", e),
    })?;

    let repo = RecipeRepository::new(state.db_pool.pool().clone());
    let recipes = repo.find_by_save_id(save_id).await?;

    let recipe_responses: Vec<RecipeResponse> =
        recipes.into_iter().map(RecipeResponse::from).collect();

    Ok(Json(RecipeListResponse {
        total: recipe_responses.len(),
        recipes: recipe_responses,
    }))
}

/// 获取菜谱详情
#[utoipa::path(
    get,
    path = "/api/v1/saves/{save_id}/recipes/{recipe_id}",
    tag = "recipes",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("recipe_id" = String, Path, description = "菜谱 ID")
    ),
    responses(
        (status = 200, description = "获取菜谱成功", body = RecipeResponse),
        (status = 404, description = "菜谱不存在")
    )
)]
pub async fn get_recipe(
    State(state): State<Arc<AppState>>,
    Path((_save_id, recipe_id)): Path<(String, String)>,
) -> GameResult<Json<RecipeResponse>> {
    let recipe_id = Uuid::parse_str(&recipe_id).map_err(|e| GameError::Validation {
        details: format!("Invalid recipe_id UUID: {}", e),
    })?;

    let repo = RecipeRepository::new(state.db_pool.pool().clone());
    let recipe = repo
        .find_by_id(recipe_id)
        .await?
        .ok_or_else(|| GameError::NotFound {
            entity_type: "Recipe".to_string(),
            entity_id: recipe_id.to_string(),
        })?;

    Ok(Json(RecipeResponse::from(recipe)))
}

/// 更新菜谱状态
#[utoipa::path(
    patch,
    path = "/api/v1/saves/{save_id}/recipes/{recipe_id}/status",
    tag = "recipes",
    params(
        ("save_id" = String, Path, description = "存档 ID"),
        ("recipe_id" = String, Path, description = "菜谱 ID")
    ),
    request_body = UpdateRecipeStatusRequest,
    responses(
        (status = 200, description = "状态更新成功"),
        (status = 404, description = "菜谱不存在")
    )
)]
pub async fn update_recipe_status(
    State(state): State<Arc<AppState>>,
    Path((_save_id, recipe_id)): Path<(String, String)>,
    Json(payload): Json<UpdateRecipeStatusRequest>,
) -> GameResult<()> {
    let recipe_id = Uuid::parse_str(&recipe_id).map_err(|e| GameError::Validation {
        details: format!("Invalid recipe_id UUID: {}", e),
    })?;

    let status = string_to_status(&payload.status)?;

    let repo = RecipeRepository::new(state.db_pool.pool().clone());
    repo.update_status(recipe_id, status).await?;

    Ok(())
}

// Helper functions
fn category_to_string(category: &RecipeCategory) -> String {
    match category {
        RecipeCategory::Sichuan => "sichuan".to_string(),
        RecipeCategory::Cantonese => "cantonese".to_string(),
        RecipeCategory::Hunan => "hunan".to_string(),
        RecipeCategory::Shandong => "shandong".to_string(),
        RecipeCategory::Jiangsu => "jiangsu".to_string(),
        RecipeCategory::Zhejiang => "zhejiang".to_string(),
        RecipeCategory::Fujian => "fujian".to_string(),
        RecipeCategory::Anhui => "anhui".to_string(),
        RecipeCategory::HomeStyle => "homestyle".to_string(),
        RecipeCategory::Creative => "creative".to_string(),
        RecipeCategory::Foreign => "foreign".to_string(),
    }
}

fn string_to_category(s: &str) -> GameResult<RecipeCategory> {
    match s {
        "sichuan" => Ok(RecipeCategory::Sichuan),
        "cantonese" => Ok(RecipeCategory::Cantonese),
        "hunan" => Ok(RecipeCategory::Hunan),
        "shandong" => Ok(RecipeCategory::Shandong),
        "jiangsu" => Ok(RecipeCategory::Jiangsu),
        "zhejiang" => Ok(RecipeCategory::Zhejiang),
        "fujian" => Ok(RecipeCategory::Fujian),
        "anhui" => Ok(RecipeCategory::Anhui),
        "homestyle" => Ok(RecipeCategory::HomeStyle),
        "creative" => Ok(RecipeCategory::Creative),
        "foreign" => Ok(RecipeCategory::Foreign),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe category: {}", s),
        }),
    }
}

fn status_to_string(status: &RecipeStatus) -> String {
    match status {
        RecipeStatus::Damaged => "damaged".to_string(),
        RecipeStatus::Fuzzy => "fuzzy".to_string(),
        RecipeStatus::Precise => "precise".to_string(),
        RecipeStatus::Mastered => "mastered".to_string(),
    }
}

fn string_to_status(s: &str) -> GameResult<RecipeStatus> {
    match s {
        "damaged" => Ok(RecipeStatus::Damaged),
        "fuzzy" => Ok(RecipeStatus::Fuzzy),
        "precise" => Ok(RecipeStatus::Precise),
        "mastered" => Ok(RecipeStatus::Mastered),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe status: {}", s),
        }),
    }
}

fn source_to_string(source: &RecipeSource) -> String {
    match source {
        RecipeSource::Inherited => "inherited".to_string(),
        RecipeSource::Travel => "travel".to_string(),
        RecipeSource::Innovation => "innovation".to_string(),
    }
}

fn string_to_source(s: &str) -> GameResult<RecipeSource> {
    match s {
        "inherited" => Ok(RecipeSource::Inherited),
        "travel" => Ok(RecipeSource::Travel),
        "innovation" => Ok(RecipeSource::Innovation),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe source: {}", s),
        }),
    }
}
