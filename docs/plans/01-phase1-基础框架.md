# Phase 1: 基础框架（第1-2周）

## 开发目标

- [ ] 项目初始化，配置 Cargo.toml
- [ ] 实现配置管理模块
- [ ] 实现数据库连接和迁移
- [ ] 实现 HTTP API 框架
- [ ] 实现 WebSocket 连接

---

## 一、技术栈选型

| 组件 | 技术选择 | 理由 |
|------|---------|------|
| 异步运行时 | Tokio | 成熟稳定，生态丰富 |
| Web 框架 | Axum | 高性能，与 Tokio 生态无缝集成 |
| 数据库 | SQLite (sqlx) | 轻量级，适合单机部署，支持异步 |
| 序列化 | serde + serde_json | Rust 标准选择 |
| 配置管理 | config-rs | 支持多格式配置文件 |
| 日志 | tracing + tracing-subscriber | 结构化日志 |
| 时间处理 | chrono | 处理时区和时间同步 |
| 随机数 | rand | 事件和旅行随机性 |
| CLI | clap | 命令行参数解析 |
| **LLM 客户端** | async-ollama | 调用 Ollama API，异步支持 |
| **Prompt 管理** | handlebars-rust | 模板化 Prompt，动态注入上下文 |

---

## 二、统一错误处理系统

### 2.1 设计理念

采用统一的错误类型体系，确保：
- 所有错误都有清晰的分类和编码
- 错误信息对玩家友好（不暴露技术细节）
- 错误可追踪（包含请求ID）
- 支持国际化错误消息

### 2.2 错误类型体系

```rust
use thiserror::Error;

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

/// 子系统专用错误
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
```

### 2.3 错误响应格式

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

impl ErrorResponse {
    pub fn from_error(error: &GameError, request_id: Uuid) -> Self {
        Self {
            code: error.error_code().to_string(),
            message: error.to_player_message(),
            details: error.to_debug_details(),
            request_id: request_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
```

### 2.4 错误处理中间件

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4();
        let status = self.http_status();
        let error_response = ErrorResponse::from_error(&self, request_id);

        (status, Json(error_response)).into_response()
    }
}

impl GameError {
    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> StatusCode {
        match self {
            GameError::NotFound { .. } => StatusCode::NOT_FOUND,
            GameError::Validation { .. } => StatusCode::BAD_REQUEST,
            GameError::Conflict { .. } => StatusCode::CONFLICT,
            GameError::InvalidState { .. } => StatusCode::CONFLICT,
            GameError::InsufficientResource { .. } => StatusCode::BAD_REQUEST,
            GameError::ConditionNotMet { .. } => StatusCode::BAD_REQUEST,
            GameError::Cooldown { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::LlmUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            GameError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            GameError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GameError::IncompatibleVersion { .. } => StatusCode::BAD_REQUEST,
            GameError::CorruptedSave { .. } => StatusCode::BAD_REQUEST,
            GameError::ImportFailed { .. } => StatusCode::BAD_REQUEST,
        }
    }

    /// 错误码（用于前端国际化）
    pub fn error_code(&self) -> &'static str {
        match self {
            GameError::NotFound { .. } => "ENTITY_NOT_FOUND",
            GameError::Validation { .. } => "VALIDATION_ERROR",
            GameError::Conflict { .. } => "CONFLICT",
            GameError::InvalidState { .. } => "INVALID_STATE",
            GameError::InsufficientResource { .. } => "INSUFFICIENT_RESOURCE",
            GameError::ConditionNotMet { .. } => "CONDITION_NOT_MET",
            GameError::Cooldown { .. } => "COOLDOWN",
            GameError::LlmUnavailable => "LLM_UNAVAILABLE",
            GameError::Internal { .. } => "INTERNAL_ERROR",
            GameError::RateLimited { .. } => "RATE_LIMITED",
            GameError::Database(_) => "DATABASE_ERROR",
            GameError::IncompatibleVersion { .. } => "INCOMPATIBLE_VERSION",
            GameError::CorruptedSave { .. } => "CORRUPTED_SAVE",
            GameError::ImportFailed { .. } => "IMPORT_FAILED",
        }
    }

    /// 玩家可见的消息
    pub fn to_player_message(&self) -> String {
        match self {
            GameError::NotFound { entity_type, .. } => {
                format!("找不到{}", entity_type)
            }
            GameError::Validation { details } => details.clone(),
            GameError::InsufficientResource { resource, required, available } => {
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
            _ => "操作失败，请稍后重试".to_string(),
        }
    }

    /// 调试详细信息
    pub fn to_debug_details(&self) -> Option<String> {
        match self {
            GameError::Database(e) => Some(format!("Database error: {:?}", e)),
            GameError::Internal { request_id } => Some(format!("Request ID: {}", request_id)),
            _ => None,
        }
    }
}
```

### 2.5 错误码表

| 错误码 | HTTP状态 | 说明 | 前端处理建议 |
|--------|---------|------|-------------|
| ENTITY_NOT_FOUND | 404 | 实体不存在 | 刷新页面或返回列表 |
| VALIDATION_ERROR | 400 | 数据验证失败 | 显示具体错误信息 |
| CONFLICT | 409 | 数据冲突 | 提示用户刷新重试 |
| INVALID_STATE | 409 | 状态不允许操作 | 显示当前状态 |
| INSUFFICIENT_RESOURCE | 400 | 资源不足 | 显示缺少的资源 |
| CONDITION_NOT_MET | 400 | 条件不满足 | 显示需要满足的条件 |
| COOLDOWN | 429 | 操作冷却中 | 显示剩余时间 |
| LLM_UNAVAILABLE | 503 | LLM服务不可用 | 提示稍后重试 |
| INTERNAL_ERROR | 500 | 内部错误 | 显示请求ID用于反馈 |
| RATE_LIMITED | 429 | 请求过于频繁 | 显示重试时间 |
| DATABASE_ERROR | 500 | 数据库错误 | 提示稍后重试 |
| INCOMPATIBLE_VERSION | 400 | 存档版本不兼容 | 提示版本信息 |
| CORRUPTED_SAVE | 400 | 存档损坏 | 提供恢复选项 |
| IMPORT_FAILED | 400 | 导入失败 | 显示具体原因 |

---

## 三、后端架构设计

### 3.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     Backend Server                          │
│                     (systemd daemon)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  HTTP API   │  │  WebSocket  │  │   Internal Timer    │  │
│  │  (Axum)     │  │  (实时推送)  │  │   (时间驱动事件)    │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         └────────────────┼─────────────────────┘             │
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐   │
│  │                    Core Game Engine                    │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │   │
│  │  │ Command     │  │ Event       │  │ Time System   │  │   │
│  │  │ Processor   │──│ Dispatcher  │──│ (延迟模拟)    │  │   │
│  │  └─────────────┘  └─────────────┘  └───────────────┘  │   │
│  │                                                        │   │
│  │  ┌─────────────────────────────────────────────────┐  │   │
│  │  │              Subsystems (子系统)                 │  │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │  │   │
│  │  │  │ Panpan  │ │  Shop   │ │  Travel │ │Recipe   │ │  │   │
│  │  │  │ System  │ │ System  │ │ System  │ │Lab Sys  │ │  │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └────────┘ │  │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │  │   │
│  │  │  │ Memory  │ │ Garden  │ │Customer │ │ Event   │ │  │   │
│  │  │  │ System  │ │ System  │ │ System  │ │ System  │ │  │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └────────┘ │  │   │
│  │  └─────────────────────────────────────────────────┘  │   │
│  └───────────────────────────────────────────────────────┘   │
│                          │                                   │
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐   │
│  │                  Data Layer (数据层)                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │   │
│  │  │   SQLite    │  │   Cache     │  │  File Storage │  │   │
│  │  │  (持久化)   │  │  (内存缓存) │  │   (图片等)    │  │   │
│  │  └─────────────┘  └─────────────┘  └───────────────┘  │   │
│  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│  Frontend TUI   │
│  (ratatui)      │
└─────────────────┘
```

### 3.2 核心模块职责

#### 3.2.1 Command Processor（指令处理器）
- 接收玩家指令
- 计算通信延迟到达时间
- 将指令加入延迟队列
- 到达后分发到对应子系统

#### 3.2.2 Event Dispatcher（事件分发器）
- 定时检查触发的游戏事件
- 分发事件到对应子系统
- 生成盼盼的简报和通知

#### 3.2.3 Time System（时间系统）
- 维护地球时间（东八区）
- 计算当前火星-地球通信延迟
- 管理游戏内时间流逝

---

## 四、健康检查 API

### 4.1 设计目标

为 systemd 监控提供健康检查端点，确保服务可用性。

### 4.2 健康检查器

```rust
use std::time::{Duration, Instant};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// 健康检查器
pub struct HealthChecker {
    start_time: Instant,
    db_pool: SqlitePool,
}

/// 健康状态响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// 服务状态
    pub status: HealthStatus,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 各组件状态
    pub components: ComponentsHealth,
    /// 时间戳
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsHealth {
    pub database: ComponentHealth,
    pub llm: ComponentHealth,
    pub config: ComponentHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl HealthChecker {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            start_time: Instant::now(),
            db_pool,
        }
    }

    /// 执行健康检查
    pub async fn check(&self) -> HealthResponse {
        let db_health = self.check_database().await;
        let llm_health = self.check_llm().await;
        let config_health = self.check_config().await;

        // 综合状态判断
        let overall_status = if db_health.status == HealthStatus::Unhealthy {
            HealthStatus::Unhealthy
        } else if llm_health.status == HealthStatus::Unhealthy {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthResponse {
            status: overall_status,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components: ComponentsHealth {
                database: db_health,
                llm: llm_health,
                config: config_health,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    async fn check_database(&self) -> ComponentHealth {
        let start = Instant::now();
        match sqlx::query("SELECT 1").fetch_one(&self.db_pool).await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                latency_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Database error: {}", e)),
                latency_ms: None,
            },
        }
    }

    async fn check_llm(&self) -> ComponentHealth {
        // 检查 Ollama 服务是否可用
        // 简单实现：尝试连接 Ollama
        ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }

    fn check_config(&self) -> ComponentHealth {
        ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }
}
```

### 4.3 API 端点

```
GET    /health                          # 完整健康检查（用于 systemd）
GET    /health/ready                    # 就绪检查（服务是否可接受请求）
GET    /health/live                     # 存活检查（服务是否运行）
```

### 4.4 systemd 配置

```ini
# /etc/systemd/system/flavors-game.service
[Unit]
Description=Flavors Across Two Decades Game Server
After=network.target

[Service]
Type=notify
User=game
WorkingDirectory=/opt/flavors-game
ExecStart=/usr/local/bin/flavors-game
Restart=always
RestartSec=10

# 健康检查配置
WatchdogSec=30
ExecStartPost=/bin/sleep 2

[Install]
WantedBy=multi-user.target
```

---

## 五、项目结构

```
flavors-game/
├── Cargo.toml
├── config/
│   ├── default.toml
│   └── production.toml
├── migrations/
│   └── 001_initial.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── pool.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   └── saves.rs
│   └── game/
│       └── mod.rs
└── tests/
    └── integration_test.rs
```

---

## 六、Cargo.toml 依赖

```toml
[package]
name = "flavors-game"
version = "0.1.0"
edition = "2024"

[dependencies]
# 异步运行时
tokio = { version = "1.50.0", features = ["full"] }

# Web 框架
axum = "0.8.8"
tower = "0.5.3"
tower-http = { version = "0.6.8", features = ["cors", "trace"] }

# 数据库
sqlx = { version = "0.8.6", features = ["runtime-tokio", "sqlite"] }

# 序列化
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

# 配置
config = "0.15.19"

# 日志
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.22", features = ["env-filter"] }

# 时间
chrono = { version = "0.4.44", features = ["serde"] }

# UUID
uuid = { version = "1.21.0", features = ["v4", "serde"] }

# 错误处理
thiserror = "2.0.18"
anyhow = "1.0.102"

# CLI
clap = { version = "4.5.60", features = ["derive"] }

# LLM
async-ollama = "0.5"
handlebars = "6.4.0"

# 随机数
rand = "0.10.0"

[dev-dependencies]
reqwest = { version = "0.13.2", features = ["json"] }
```
