//! 统一错误处理模块

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

/// 游戏统一错误类型
#[derive(Debug, Error)]
pub enum GameError {
    // ========== 数据库错误 ==========
    #[error("数据库操作失败: {0}")]
    Database(#[from] DatabaseError),

    #[error("实体未找到: {entity_type}({entity_id})")]
    NotFound {
        entity_type: String,
        entity_id: String,
    },

    #[error("数据验证失败: {details}")]
    Validation { details: String },

    #[error("数据冲突: {conflict_type}")]
    Conflict { conflict_type: String },

    // ========== 游戏逻辑错误 ==========
    #[error("状态不允许此操作: 当前状态={current_state}, 需要状态={required_states:?}")]
    InvalidState {
        current_state: String,
        required_states: Vec<String>,
    },

    #[error("资源不足: {resource} 需要 {required}, 当前 {available}")]
    InsufficientResource {
        resource: String,
        required: u32,
        available: u32,
    },

    #[error("条件不满足: {condition}")]
    ConditionNotMet { condition: String },

    #[error("操作冷却中: 剩余 {remaining_seconds} 秒")]
    Cooldown { remaining_seconds: u32 },

    // ========== 系统错误 ==========
    #[error("LLM 服务暂时不可用")]
    LlmUnavailable,

    #[error("服务内部错误: {request_id}")]
    Internal { request_id: String },

    #[error("服务繁忙，请稍后重试")]
    RateLimited { retry_after_seconds: u32 },

    // ========== 存档错误 ==========
    #[error("存档版本不兼容: 存档版本={save_version}, 当前版本={current_version}")]
    IncompatibleVersion {
        save_version: u32,
        current_version: u32,
    },

    #[error("存档损坏: {details}")]
    CorruptedSave { details: String },

    #[error("导入失败: {reason}")]
    ImportFailed { reason: String },

    // ========== 配置错误 ==========
    #[error("配置加载失败: {0}")]
    Config(String),
}

/// 数据库错误
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("查询失败: {0}")]
    QueryFailed(String),

    #[error("写入失败: {0}")]
    WriteFailed(String),

    #[error("事务失败: {0}")]
    TransactionFailed(String),

    #[error("数据库损坏")]
    CorruptionDetected,
}

/// 菜园子系统错误
#[derive(Debug, Error)]
pub enum GardenError {
    #[error("菜地未解锁")]
    PlotLocked,

    #[error("菜地已被占用")]
    PlotOccupied,

    #[error("种子不足")]
    InsufficientSeeds,

    #[error("作物未成熟")]
    CropNotMature,

    #[error("季节不适宜种植此作物")]
    SeasonNotSuitable,

    #[error("肥力不足")]
    InsufficientFertility,

    #[error("病害严重，需要先治疗")]
    SevereDisease,
}

/// 厨房子系统错误
#[derive(Debug, Error)]
pub enum KitchenError {
    #[error("设备不可用: {device}")]
    DeviceUnavailable { device: String },

    #[error("食材不足: {ingredient}")]
    InsufficientIngredient { ingredient: String },

    #[error("菜谱未解锁")]
    RecipeLocked,

    #[error("正在被占用")]
    InUse,
}

/// 旅行子系统错误
#[derive(Debug, Error)]
pub enum TravelError {
    #[error("正在冷却中")]
    OnCooldown,

    #[error("盼盼电量不足")]
    InsufficientEnergy,

    #[error("小馆需要人手")]
    ShopNeedsAttention,

    #[error("旅行中无法执行此操作")]
    AlreadyTravelling,
}

/// API 错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误码（用于前端国际化）
    pub code: String,
    /// 错误消息（玩家可见）
    pub message: String,
    /// 详细信息（调试用，仅开发环境显示）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// 请求ID（用于追踪）
    pub request_id: String,
    /// 时间戳
    pub timestamp: String,
    /// 额外数据（如冷却剩余时间）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn from_error(error: &GameError, request_id: Uuid) -> Self {
        Self {
            code: error.error_code().to_string(),
            message: error.to_player_message(),
            details: error.to_debug_details(),
            request_id: request_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: error.additional_data(),
        }
    }
}

impl GameError {
    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            GameError::NotFound { .. } => StatusCode::NOT_FOUND,
            GameError::Validation { .. } => StatusCode::BAD_REQUEST,
            GameError::Conflict { .. } => StatusCode::CONFLICT,
            GameError::InvalidState { .. } => StatusCode::CONFLICT,
            GameError::InsufficientResource { .. } => StatusCode::BAD_REQUEST,
            GameError::ConditionNotMet { .. } => StatusCode::FAILED_DEPENDENCY,
            GameError::Cooldown { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::LlmUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            GameError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            GameError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GameError::IncompatibleVersion { .. } => StatusCode::BAD_REQUEST,
            GameError::CorruptedSave { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            GameError::ImportFailed { .. } => StatusCode::BAD_REQUEST,
            GameError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 错误码（用于前端国际化）
    pub fn error_code(&self) -> &'static str {
        match self {
            GameError::Database(_) => "E001",
            GameError::NotFound { .. } => "E002",
            GameError::Validation { .. } => "E003",
            GameError::Conflict { .. } => "E004",
            GameError::InvalidState { .. } => "E101",
            GameError::InsufficientResource { .. } => "E102",
            GameError::ConditionNotMet { .. } => "E103",
            GameError::Cooldown { .. } => "E104",
            GameError::LlmUnavailable => "E201",
            GameError::Internal { .. } => "E500",
            GameError::RateLimited { .. } => "E429",
            GameError::IncompatibleVersion { .. } => "E301",
            GameError::CorruptedSave { .. } => "E302",
            GameError::ImportFailed { .. } => "E303",
            GameError::Config(_) => "E005",
        }
    }

    /// 玩家可见的消息
    pub fn to_player_message(&self) -> String {
        match self {
            GameError::NotFound { entity_type, .. } => {
                format!("找不到{}", entity_type)
            }
            GameError::Validation { details } => details.clone(),
            GameError::InsufficientResource {
                resource,
                required,
                available,
            } => {
                format!("{}不足，需要{}，当前只有{}", resource, required, available)
            }
            GameError::Cooldown { remaining_seconds } => {
                let minutes = remaining_seconds / 60;
                if minutes > 0 {
                    format!("还需要等待{}分钟", minutes)
                } else {
                    format!("还需要等待{}秒", remaining_seconds)
                }
            }
            GameError::LlmUnavailable => "盼盼暂时无法思考，请稍后再试".to_string(),
            GameError::RateLimited { .. } => "操作太频繁，请稍后再试".to_string(),
            GameError::IncompatibleVersion {
                save_version,
                current_version,
            } => {
                format!(
                    "存档版本不兼容（存档: v{}, 当前: v{}）",
                    save_version, current_version
                )
            }
            GameError::CorruptedSave { .. } => "存档数据已损坏".to_string(),
            GameError::ImportFailed { reason } => format!("导入失败: {}", reason),
            GameError::Database(_) => "系统繁忙，请稍后重试".to_string(),
            GameError::Internal { .. } => "发生未知错误".to_string(),
            GameError::Config(_) => "配置加载失败".to_string(),
            _ => "操作失败，请稍后重试".to_string(),
        }
    }

    /// 调试详细信息
    pub fn to_debug_details(&self) -> Option<String> {
        match self {
            GameError::Database(e) => Some(format!("Database error: {:?}", e)),
            GameError::Internal { request_id } => Some(format!("Request ID: {}", request_id)),
            GameError::Config(e) => Some(format!("Config error: {}", e)),
            _ => None,
        }
    }

    /// 额外数据
    pub fn additional_data(&self) -> Option<serde_json::Value> {
        match self {
            GameError::Cooldown { remaining_seconds } => {
                Some(serde_json::json!({ "remaining_seconds": remaining_seconds }))
            }
            GameError::InsufficientResource {
                resource,
                required,
                available,
            } => Some(serde_json::json!({
                "resource": resource,
                "required": required,
                "available": available
            })),
            GameError::RateLimited {
                retry_after_seconds,
            } => Some(serde_json::json!({ "retry_after_seconds": retry_after_seconds })),
            GameError::IncompatibleVersion {
                save_version,
                current_version,
            } => Some(serde_json::json!({
                "save_version": save_version,
                "current_version": current_version
            })),
            _ => None,
        }
    }
}

/// 统一错误响应转换
impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4();
        let status_code = self.status_code();

        // 记录错误日志
        error!(
            request_id = %request_id,
            error = %self,
            error_code = %self.error_code(),
            "Request failed"
        );

        let error_response = ErrorResponse::from_error(&self, request_id);

        (status_code, Json(error_response)).into_response()
    }
}

/// 结果类型别名
pub type GameResult<T> = Result<T, GameError>;
