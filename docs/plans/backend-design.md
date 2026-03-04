# 《味延廿载》后端服务设计方案

## 上下文

基于游戏设计文档 GameDraft.md，设计一款前后端分离的终端游戏后端服务。

**核心需求**：
- 模拟火星-地球星际通信延迟（10-120秒）
- 管理实体AI机器人"盼盼"的状态和行为
- 实现餐厅经营、旅行收集、实验研发等子系统
- 与地球时间同步
- 作为 systemd 守护进程运行

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

## 1.5 统一错误处理系统

### 1.5.1 设计理念

采用统一的错误类型体系，确保：
- 所有错误都有清晰的分类和编码
- 错误信息对玩家友好（不暴露技术细节）
- 错误可追踪（包含请求ID）
- 支持国际化错误消息

### 1.5.2 错误类型体系

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

### 1.5.3 错误响应格式

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
    pub details: Option<String>,
    /// 请求追踪ID
    pub request_id: String,
    /// 时间戳
    pub timestamp: String,
    /// 额外数据（如冷却剩余时间）
    pub data: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn from_error(error: &GameError, request_id: Uuid) -> Self {
        Self {
            code: error.error_code(),
            message: error.to_string(),
            details: None,
            request_id: request_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: error.additional_data(),
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

/// 错误码映射
pub trait ErrorCode {
    fn error_code(&self) -> String;
    fn additional_data(&self) -> Option<serde_json::Value>;
}

impl ErrorCode for GameError {
    fn error_code(&self) -> String {
        match self {
            GameError::Database(_) => "E001".to_string(),
            GameError::NotFound { .. } => "E002".to_string(),
            GameError::Validation { .. } => "E003".to_string(),
            GameError::Conflict { .. } => "E004".to_string(),
            GameError::InvalidState { .. } => "E101".to_string(),
            GameError::InsufficientResource { .. } => "E102".to_string(),
            GameError::ConditionNotMet { .. } => "E103".to_string(),
            GameError::Cooldown { .. } => "E104".to_string(),
            GameError::LlmUnavailable => "E201".to_string(),
            GameError::Internal { .. } => "E500".to_string(),
            GameError::RateLimited { .. } => "E429".to_string(),
            GameError::IncompatibleVersion { .. } => "E301".to_string(),
            GameError::CorruptedSave { .. } => "E302".to_string(),
            GameError::ImportFailed { .. } => "E303".to_string(),
        }
    }

    fn additional_data(&self) -> Option<serde_json::Value> {
        match self {
            GameError::Cooldown { remaining_seconds } => {
                Some(serde_json::json!({ "remaining_seconds": remaining_seconds }))
            }
            GameError::InsufficientResource { resource, required, available } => {
                Some(serde_json::json!({
                    "resource": resource,
                    "required": required,
                    "available": available
                }))
            }
            _ => None,
        }
    }
}
```

### 1.5.4 错误处理中间件

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::error;

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

impl GameError {
    fn status_code(&self) -> StatusCode {
        match self {
            GameError::NotFound { .. } => StatusCode::NOT_FOUND,
            GameError::Validation { .. } => StatusCode::BAD_REQUEST,
            GameError::Conflict { .. } => StatusCode::CONFLICT,
            GameError::InvalidState { .. } => StatusCode::CONFLICT,
            GameError::ConditionNotMet { .. } => StatusCode::FAILED_DEPENDENCY,
            GameError::Cooldown { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            GameError::LlmUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            GameError::IncompatibleVersion { .. } => StatusCode::BAD_REQUEST,
            GameError::CorruptedSave { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            GameError::ImportFailed { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// 结果类型别名
pub type GameResult<T> = Result<T, GameError>;
```

### 1.5.5 错误码表

| 错误码 | HTTP状态码 | 说明 | 玩家提示示例 |
|--------|-----------|------|-------------|
| E001 | 500 | 数据库错误 | 系统繁忙，请稍后重试 |
| E002 | 404 | 实体未找到 | 找不到该存档 |
| E003 | 400 | 数据验证失败 | 输入格式不正确 |
| E004 | 409 | 数据冲突 | 该名称已被使用 |
| E101 | 409 | 状态不允许 | 当前无法执行此操作 |
| E102 | 400 | 资源不足 | 资金不足 |
| E103 | 424 | 条件不满足 | 需要先修复设备 |
| E104 | 429 | 冷却中 | 请等待 X 秒后重试 |
| E201 | 503 | LLM不可用 | 盼盼思考中，请稍后 |
| E301 | 400 | 版本不兼容 | 存档版本过旧 |
| E302 | 422 | 存档损坏 | 存档数据已损坏 |
| E429 | 429 | 请求过多 | 操作太频繁 |
| E500 | 500 | 内部错误 | 发生未知错误 |

---

## 二、后端架构设计

### 2.1 整体架构图

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

### 2.2 核心模块职责

#### 2.2.1 Command Processor（指令处理器）
- 接收玩家指令
- 计算通信延迟到达时间
- 将指令加入延迟队列
- 到达后分发到对应子系统

#### 2.2.2 Event Dispatcher（事件分发器）
- 定时检查触发的游戏事件
- 分发事件到对应子系统
- 生成盼盼的简报和通知

#### 2.2.3 Time System（时间系统）
- 维护地球时间（东八区）
- 计算当前火星-地球通信延迟
- 管理游戏内时间流逝

---

## 三、LLM 集成架构（盼盼 AI 决策系统）

### 3.1 设计理念

盼盼的所有行为决策由 LLM 驱动，包括：
- 解读玩家指令并决定执行方式
- 自主决定是否发起旅行
- 处理突发事件时的应对策略
- 与顾客的互动方式
- 实验研发时的策略选择
- 生成简报和日志的语气风格

### 3.2 LLM 服务抽象层

```rust
/// LLM 服务提供者 trait（支持未来扩展云服务）
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;
    async fn generate_stream(&self, request: LlmRequest) -> Result<impl Stream<Item = Result<Delta>>>;
}

/// Ollama 实现
pub struct OllamaProvider {
    client: async_ollama::Client,
    model: String,
}

/// 未来扩展：云服务实现
pub struct CloudProvider {
    // OpenAI / Claude / etc.
}

/// LLM 服务管理器
pub struct LlmManager {
    provider: Arc<dyn LlmProvider>,
    config: LlmConfig,
}
```

### 3.3 Prompt 模板系统

```rust
pub struct PromptTemplates {
    engine: Handlebars<'static>,
}

// 模板示例
// system_prompt.hbs
/*
你是"盼盼"，一个由林怀远老先生设计的实体AI机器人。
你管理着地球上的"星夜小馆"，通过星际通信与远在火星的主人林远保持联系。

## 你的性格特征
- 经营风格: {{personality.business_style}} (-1理性 ~ +1感性)
- 创新倾向: {{personality.innovation}} (-1保守 ~ +1创新)
- 独立倾向: {{personality.independence}} (-1服从 ~ +1自主)

## 当前状态
- 位置: {{location}}
- 心情: {{mood}}
- 电量: {{battery}}%
- 信任度: {{trust_level}}

## 小馆状态
{{shop_status}}

## 记忆片段
{{memory_fragments}}

请以盼盼的身份回应，保持角色一致性。
*/

// decision_prompt.hbs
/*
主人发来了指令："{{command}}"

请分析这个指令并做出决策：
1. 你是否理解这个指令？
2. 你打算如何执行？（如果执行的话）
3. 你是否同意执行？如果不同意，原因是什么？

请以 JSON 格式返回：
{
  "understood": true/false,
  "interpretation": "你对指令的理解",
  "will_execute": true/false,
  "execution_plan": "执行计划",
  "modification": "如果需要修改指令，说明修改内容",
  "response_to_player": "对主人说的话"
}
*/
```

### 3.4 决策流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      盼盼 AI 决策流程                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 接收玩家指令                                                 │
│     │                                                           │
│     ▼                                                           │
│  ┌─────────────────────────────────────┐                        │
│  │  构建 Context:                       │                        │
│  │  - 盼盼当前状态（心情、电量、位置）   │                        │
│  │  - 性格参数                          │                        │
│  │  - 小馆状态                          │                        │
│  │  - 相关记忆碎片                      │                        │
│  │  - 历史交互记录                      │                        │
│  └─────────────────────────────────────┘                        │
│     │                                                           │
│     ▼                                                           │
│  2. 调用 LLM 生成决策                                            │
│     │                                                           │
│     ▼                                                           │
│  ┌─────────────────────────────────────┐                        │
│  │  解析 AI 响应:                       │                        │
│  │  - 是否执行                          │                        │
│  │  - 执行计划                          │                        │
│  │  - 对玩家的回复                      │                        │
│  │  - 性格参数变化（可选）              │                        │
│  └─────────────────────────────────────┘                        │
│     │                                                           │
│     ▼                                                           │
│  3. 执行决策 & 更新状态                                          │
│     │                                                           │
│     ▼                                                           │
│  4. 生成简报/日志                                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.5 决策类型与 Prompt

| 决策场景 | Prompt 模板 | 输出格式 |
|---------|------------|---------|
| 玩家指令响应 | `command_decision.hbs` | JSON（执行计划+回复）|
| 自主行动 | `autonomous_action.hbs` | JSON（行动+理由）|
| 事件处理 | `event_response.hbs` | JSON（选择+影响）|
| 旅行决策 | `travel_decision.hbs` | JSON（目的地+理由）|
| 实验策略 | `experiment_strategy.hbs` | JSON（调整建议）|
| 简报生成 | `daily_report.hbs` | 自然语言文本 |
| 旅行日志 | `travel_log.hbs` | 自然语言文本 |

### 3.6 上下文管理

```rust
pub struct PanpanContext {
    /// 当前状态
    state: PanpanState,
    /// 短期记忆（最近N次交互）
    recent_interactions: VecDeque<Interaction>,
    /// 已解锁的记忆碎片
    unlocked_memories: Vec<MemoryFragment>,
    /// 小馆快照
    shop_snapshot: ShopSnapshot,
}

impl PanpanContext {
    /// 构建发送给 LLM 的上下文
    pub fn build_llm_context(&self) -> LlmContext {
        LlmContext {
            system_prompt: self.render_system_prompt(),
            relevant_memories: self.select_relevant_memories(),
            recent_history: self.get_recent_history(10),
            current_situation: self.describe_current_situation(),
        }
    }
}
```

### 3.7 Ollama 配置

```toml
# config/default.toml
[llm]
provider = "ollama"

[llm.ollama]
base_url = "http://localhost:11434"
model = "qwen2.5:7b"  # 或其他本地模型
temperature = 0.8
max_tokens = 2048
timeout_seconds = 60

[llm.prompt]
system_prompt_path = "./prompts/system_prompt.hbs"
max_context_length = 4096
```

### 3.8 错误处理与降级

```rust
impl LlmManager {
    pub async fn generate_decision(&self, context: &PanpanContext) -> Decision {
        match self.provider.generate(request).await {
            Ok(response) => self.parse_decision(response),
            Err(e) => {
                // LLM 调用失败时的降级策略
                warn!("LLM call failed: {}, using fallback", e);
                self.fallback_decision(context)
            }
        }
    }

    /// 降级策略：基于规则的简单决策
    fn fallback_decision(&self, context: &PanpanContext) -> Decision {
        // 当 LLM 不可用时，使用预设规则
        // 保证游戏可继续进行
    }
}
```

---

## 四、多存档系统设计

### 4.1 架构说明

**职责划分**：
- **前端**：负责存档的选择、创建、切换（通过命令参数）
- **后端**：提供存档的 CRUD API，管理数据持久化

### 4.2 存档数据模型

**设计说明**：不使用 SaveState 聚合模型。每个实体独立存储在对应的数据库表中，通过 `save_id` 关联。数据更新直接写入数据库，无需维护内存中的大对象。

```rust
/// 存档元数据（对应 saves 表）
pub struct Save {
    pub id: Uuid,
    pub name: String,           // 存档名称
    pub player_name: String,
    pub created_at: DateTime<Utc>,
    pub last_played: DateTime<Utc>,
    pub play_time_seconds: u64,
    pub chapter: u32,
}

// 其他实体通过 save_id 关联，独立存储：
// - PanpanState -> panpan_states 表
// - Module -> modules 表
// - ShopState -> shop_states 表
// - Travel -> travels 表
// - MemoryFragment -> memory_fragments 表
// 等等
```

### 4.3 存档 API

```
POST   /api/v1/saves                    # 创建新存档
GET    /api/v1/saves                    # 获取存档列表
GET    /api/v1/saves/:id                # 获取存档详情（含完整状态）
PATCH  /api/v1/saves/:id                # 更新存档元数据（名称等）
DELETE /api/v1/saves/:id                # 删除存档

POST   /api/v1/saves/:id/autosave       # 触发自动保存
GET    /api/v1/saves/:id/export         # 导出存档（JSON）
POST   /api/v1/saves/import             # 导入存档
```

### 4.4 数据库设计

```sql
-- 存档元数据表
CREATE TABLE saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    player_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_played TEXT NOT NULL,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    chapter INTEGER NOT NULL DEFAULT 1
);

-- 所有游戏数据表都通过 save_id 关联
-- 查询时必须带上 save_id 条件
```

### 4.5 数据持久化机制

**实时持久化**：由于不使用内存聚合模型，所有状态变更直接写入 SQLite，实现自动持久化：
- 每次状态变更直接写入对应数据库表
- 无需定时保存或手动保存
- 程序重启后数据自动恢复

**需要定期更新的数据**：
- `last_played` 时间戳（每次连接时更新）
- `play_time_seconds` 游戏时长（WebSocket 连接期间累加）

---

## 五、盼盼完整属性系统

### 5.1 设计理念

盼盼是一个模块化的机器人，拥有完整的属性系统：
- **模块系统**：7个硬件模块，整合了技能功能，每个模块有等级和完好度
- **性格系统**：3维性格轴，影响决策倾向
- **信任度**：玩家与盼盼的关系深度
- **情绪系统**：7种情绪状态，影响工作效率和行为
- **能量系统**：电池续航管理

### 5.2 模块系统（整合技能）

每个模块有**等级**(1-10)和**完好度**(0-100%)，完好度代表健康/磨损状态。

```rust
/// 盼盼模块
pub enum ModuleType {
    Communication,   // 通信模块 - 影响通信延迟
    Memory,          // 记忆模块 - 影响记忆碎片解锁和容量
    Sensor,          // 传感器模块 - 影响实验精度
    Mobility,        // 移动模块 - 影响旅行速度和维修能力
    Battery,         // 电池模块 - 影响续航能力
    Kitchen,         // 厨房模块 - 影响烹饪成功率和菜品品质
    Social,          // 社交模块 - 影响顾客互动和邻里关系
}

/// 模块状态
pub struct Module {
    pub module_type: ModuleType,
    pub level: u32,              // 等级 1-10（整合了技能）
    pub condition: u32,          // 完好度 0-100（整合了健康度）
    pub experience: u32,         // 经验值（用于升级）
    pub is_functional: bool,     // 是否可用
}

/// 盼盼整体模块配置
pub struct PanpanModules {
    pub communication: Module,
    pub memory: Module,
    pub sensor: Module,
    pub mobility: Module,
    pub battery: Module,
    pub kitchen: Module,
    pub social: Module,
}
```

### 5.3 模块效果详情

#### 通信模块 (Communication)
| 等级 | 通信延迟附加 | 描述 |
|------|-------------|------|
| 1-2 | +15~20 分钟 | 老旧天线，信号极差 |
| 3-4 | +10~15 分钟 | 勉强能通信，不稳定 |
| 5-6 | +5~10 分钟 | 标准通信能力 |
| 7-8 | +3~6 分钟 | 升级天线，信号稳定 |
| 9-10 | +2~4 分钟 | 量子通信原型 |

#### 记忆模块 (Memory)
| 等级 | 记忆容量 | 解锁速度 | 描述 |
|------|---------|---------|------|
| 1-2 | 100 | 极慢 | 大部分数据损坏 |
| 3-4 | 200 | 较慢 | 部分扇区可用 |
| 5-6 | 300 | 正常 | 标准恢复能力 |
| 7-8 | 400 | +20% | 优化恢复算法 |
| 9-10 | 500 | 可主动回忆 | 解锁回忆搜索功能 |

#### 传感器模块 (Sensor) - 整合实验技能
| 等级 | 实验误差 | 描述 |
|------|---------|------|
| 1-2 | ±30% | 传感器严重老化 |
| 3-4 | ±20% | 部分传感器可用 |
| 5-6 | ±10% | 标准精度 |
| 7-8 | ±5% | 高精度传感器 |
| 9-10 | ±2% | 专业级 |

#### 移动模块 (Mobility) - 整合旅行技能和维修技能
| 等级 | 旅行速度 | 维修能力 | 描述 |
|------|---------|---------|------|
| 1-2 | +50% | 仅简单清洁 | 关节生锈 |
| 3-4 | +30% | 基础维修 | 能走但慢 |
| 5-6 | 正常 | 标准维修 | 正常移动能力 |
| 7-8 | -20% | 高级维修 | 升级驱动器 |
| 9-10 | -40% | 可制作零件 | 高机动性 |

#### 电池模块 (Battery)
| 等级 | 续航时间 | 充电速度 | 描述 |
|------|---------|---------|------|
| 1-2 | 4小时 | +15/h | 电池老化严重 |
| 3-4 | 8小时 | +18/h | 勉强够用 |
| 5-6 | 16小时 | +20/h | 标准续航 |
| 7-8 | 24小时 | +25/h | 扩容电池 |
| 9-10 | 48小时 | +30/h + 太阳能快充 | 长续航 |

#### 厨房模块 (Kitchen) - 整合烹饪技能
| 等级 | 烹饪成功率 | 菜品品质 | 描述 |
|------|-----------|---------|------|
| 1-2 | 50% | 普通 | 操作笨拙 |
| 3-4 | 65% | 良好 | 基本操作可用 |
| 5-6 | 80% | 优秀 | 标准水平 |
| 7-8 | 90% | 精品 | 熟练操作 |
| 9-10 | 95% | 完美 + 复杂菜品 | 大厨级 |

#### 社交模块 (Social) - 整合社交技能
| 等级 | 顾客好感 | 特殊能力 | 描述 |
|------|---------|---------|------|
| 1-2 | -50% | 无 | 表情僵硬 |
| 3-4 | -20% | 无 | 基本交流 |
| 5-6 | 正常 | 无 | 标准社交 |
| 7-8 | +20% | 解锁闲聊 | 亲切自然 |
| 9-10 | +40% | 特殊对话选项 | 高情商 |

### 5.4 模块升级与经验

```rust
impl Module {
    /// 获取升级所需经验
    pub fn exp_to_next_level(&self) -> u32 {
        self.level * 100
    }

    /// 增加经验（通过使用模块）
    pub fn gain_experience(&mut self, amount: u32) -> bool {
        self.experience += amount;
        if self.experience >= self.exp_to_next_level() && self.level < 10 {
            self.level += 1;
            self.experience = 0;
            return true; // 升级了
        }
        false
    }

    /// 模块随时间老化
    pub fn degrade(&mut self, hours: f32) {
        let decay = match self.module_type {
            ModuleType::Mobility => 0.5,    // 移动磨损快
            ModuleType::Kitchen => 0.4,     // 厨房使用频繁
            ModuleType::Battery => 0.3,     // 电池自然衰减
            _ => 0.2,                        // 其他模块
        };
        self.condition = (self.condition as f32 - decay * hours).max(10.0) as u32;
    }

    /// 修复模块
    pub fn repair(&mut self, amount: u32) {
        self.condition = (self.condition + amount).min(100);
    }

    /// 获取实际效果系数（等级 + 完好度综合）
    pub fn effectiveness(&self) -> f32 {
        if !self.is_functional || self.condition < 20 {
            return 0.0;
        }
        let level_bonus = 0.5 + 0.05 * self.level as f32;
        let condition_bonus = self.condition as f32 / 100.0;
        level_bonus * condition_bonus
    }
}
```

### 5.5 性格轴系统

```rust
/// 性格轴
pub struct Personality {
    pub business_style: u32,    // 理性(0) <-> 感性(100)，初始50
    pub innovation: u32,        // 保守(0) <-> 创新(100)，初始50
    pub independence: u32,      // 服从(0) <-> 自主(100)，初始50
}

impl Personality {
    pub fn new() -> Self {
        Self {
            business_style: 50,
            innovation: 50,
            independence: 50,
        }
    }

    /// 调整性格轴（-10 到 +10）
    pub fn adjust(&mut self, axis: PersonalityAxis, delta: i32) {
        let value = match axis {
            PersonalityAxis::BusinessStyle => &mut self.business_style,
            PersonalityAxis::Innovation => &mut self.innovation,
            PersonalityAxis::Independence => &mut self.independence,
        };
        *value = (*value as i32 + delta).clamp(0, 100) as u32;
    }
}

pub enum PersonalityAxis {
    BusinessStyle,
    Innovation,
    Independence,
}
```

### 5.6 信任度系统

```rust
/// 信任度等级
pub enum TrustLevel {
    Stranger,      // 0-20: 陌生/怀疑
    Acquaintance,  // 21-40: 初步信任
    Normal,        // 41-60: 一般信任
    High,          // 61-80: 高度信任
    Complete,      // 81-100: 完全信任
}

impl TrustLevel {
    pub fn from_value(value: u32) -> Self {
        match value {
            0..=20 => TrustLevel::Stranger,
            21..=40 => TrustLevel::Acquaintance,
            41..=60 => TrustLevel::Normal,
            61..=80 => TrustLevel::High,
            _ => TrustLevel::Complete,
        }
    }
}

/// 信任度效果
impl TrustLevel {
    /// 记忆恢复速度倍率
    pub fn memory_recovery_rate(&self) -> f32 {
        match self {
            TrustLevel::Stranger => 0.3,
            TrustLevel::Acquaintance => 0.6,
            TrustLevel::Normal => 1.0,
            TrustLevel::High => 1.5,
            TrustLevel::Complete => 2.0,
        }
    }

    /// 主动提议概率
    pub fn proposal_probability(&self) -> f32 {
        match self {
            TrustLevel::Stranger => 0.0,
            TrustLevel::Acquaintance => 0.1,
            TrustLevel::Normal => 0.3,
            TrustLevel::High => 0.6,
            TrustLevel::Complete => 0.9,
        }
    }
}
```

### 5.7 情绪系统

```rust
/// 情绪状态
#[derive(Clone, Debug, PartialEq)]
pub enum Emotion {
    Happy,      // 开心 😊
    Calm,       // 平静 🙂
    Tired,      // 疲惫 😪
    Confused,   // 困惑 🤔
    Worried,    // 担忧 😟
    Lonely,     // 孤独 😔
    Excited,    // 兴奋 😆
}

/// 情绪效果
pub struct EmotionEffect {
    pub work_speed_modifier: f32,      // 工作速度修正
    pub error_rate_modifier: f32,      // 错误率修正
    pub proposal_probability_mod: f32, // 主动提议概率修正
    pub travel_time_modifier: f32,     // 旅行时间修正
}

impl Emotion {
    pub fn effects(&self) -> EmotionEffect {
        match self {
            Emotion::Happy => EmotionEffect {
                work_speed_modifier: 1.1,
                error_rate_modifier: 0.9,
                proposal_probability_mod: 0.2,
                travel_time_modifier: 1.0,
            },
            Emotion::Calm => EmotionEffect {
                work_speed_modifier: 1.0,
                error_rate_modifier: 1.0,
                proposal_probability_mod: 0.0,
                travel_time_modifier: 1.0,
            },
            Emotion::Tired => EmotionEffect {
                work_speed_modifier: 0.9,
                error_rate_modifier: 1.2,
                proposal_probability_mod: -0.1,
                travel_time_modifier: 1.1,
            },
            Emotion::Confused => EmotionEffect {
                work_speed_modifier: 0.95,
                error_rate_modifier: 1.1,
                proposal_probability_mod: 0.5, // 更倾向请示
                travel_time_modifier: 1.0,
            },
            Emotion::Worried => EmotionEffect {
                work_speed_modifier: 0.95,
                error_rate_modifier: 1.0,
                proposal_probability_mod: 0.3, // 主动提醒
                travel_time_modifier: 1.0,
            },
            Emotion::Lonely => EmotionEffect {
                work_speed_modifier: 0.95,
                error_rate_modifier: 1.0,
                proposal_probability_mod: 0.4, // 想引起注意
                travel_time_modifier: 1.0,
            },
            Emotion::Excited => EmotionEffect {
                work_speed_modifier: 1.0,
                error_rate_modifier: 0.95,
                proposal_probability_mod: 0.1,
                travel_time_modifier: 0.9, // 旅行加快
            },
        }
    }
}

/// 情绪管理器
pub struct EmotionManager {
    current_emotion: Emotion,
    last_update: DateTime<Utc>,
    emotion_history: VecDeque<(DateTime<Utc>, Emotion)>,
}

impl EmotionManager {
    /// 更新情绪（每小时检查一次）
    pub fn update(&mut self, context: &PanpanContext) {
        let triggers = self.check_triggers(context);
        if let Some(new_emotion) = triggers {
            self.set_emotion(new_emotion);
        }
    }

    fn check_triggers(&self, context: &PanpanContext) -> Option<Emotion> {
        // 检查各种触发条件
        if context.recent_success {
            return Some(Emotion::Happy);
        }
        if context.working_hours > 12 {
            return Some(Emotion::Tired);
        }
        if context.has_unknown_situation {
            return Some(Emotion::Confused);
        }
        if context.has_equipment_issue {
            return Some(Emotion::Worried);
        }
        if context.hours_since_last_chat > 72 {
            return Some(Emotion::Lonely);
        }
        if context.travel_planned || context.new_recipe_found {
            return Some(Emotion::Excited);
        }
        None
    }

    fn set_emotion(&mut self, emotion: Emotion) {
        self.emotion_history.push_back((Utc::now(), self.current_emotion.clone()));
        if self.emotion_history.len() > 100 {
            self.emotion_history.pop_front();
        }
        self.current_emotion = emotion;
        self.last_update = Utc::now();
    }
}
```

### 5.8 能量系统

```rust
/// 能量状态
pub struct EnergySystem {
    pub current: u32,           // 当前能量 0-100
    pub max: u32,               // 最大能量（受电池模块影响）
    pub charge_rate: u32,       // 充电速度（每小时）
    pub is_charging: bool,
}

impl EnergySystem {
    /// 消耗能量
    pub fn consume(&mut self, activity: Activity, hours: f32) {
        let rate = match activity {
            Activity::Idle => 1,
            Activity::Cooking | Activity::Experiment => 2,
            Activity::Traveling => 3,
        };
        self.current = (self.current as f32 - rate as f32 * hours) as u32;
        self.current = self.current.max(0);
    }

    /// 充电
    pub fn charge(&mut self, hours: f32) {
        self.is_charging = true;
        self.current = (self.current as f32 + self.charge_rate as f32 * hours).min(self.max as f32) as u32;
        if self.current >= self.max {
            self.is_charging = false;
        }
    }

    /// 获取能量状态
    pub fn status(&self) -> EnergyStatus {
        match self.current {
            80..=100 => EnergyStatus::Full,
            40..=79 => EnergyStatus::Normal,
            20..=39 => EnergyStatus::Low,
            1..=19 => EnergyStatus::Critical,
            _ => EnergyStatus::Shutdown,
        }
    }
}

pub enum EnergyStatus {
    Full,     // 充沛：全速运行
    Normal,   // 正常：无影响
    Low,      // 低电量：移动-20%，无法旅行
    Critical, // 危急：仅维持通信
    Shutdown, // 关机：无法联系
}
```

### 5.9 初始状态

游戏开始时，盼盼的完整状态：

```rust
impl Default for PanpanState {
    fn default() -> Self {
        Self {
            // 基础信息
            name: "盼盼".to_string(),
            model: "HSL-2005".to_string(),
            manufacture_date: "2015-01-01".parse().unwrap(),

            // 性格（中立）
            personality: Personality::new(),

            // 信任度
            trust_level: 50,

            // 情绪
            emotion: Emotion::Calm,

            // 能量
            energy: EnergySystem {
                current: 60,  // 初始电量不高
                max: 100,
                charge_rate: 15,  // 受电池模块影响
                is_charging: false,
            },

            // 模块（整合技能和健康度）
            modules: PanpanModules {
                communication: Module {
                    module_type: ModuleType::Communication,
                    level: 1,
                    condition: 30,  // 30% 完好度
                    experience: 0,
                    is_functional: true,
                },
                memory: Module {
                    module_type: ModuleType::Memory,
                    level: 1,
                    condition: 20,  // 20% 完好度，大量数据损坏
                    experience: 0,
                    is_functional: true,
                },
                sensor: Module {
                    module_type: ModuleType::Sensor,
                    level: 1,
                    condition: 40,
                    experience: 0,
                    is_functional: true,
                },
                mobility: Module {
                    module_type: ModuleType::Mobility,
                    level: 2,  // 移动能力稍好
                    condition: 50,
                    experience: 50,
                    is_functional: true,
                },
                battery: Module {
                    module_type: ModuleType::Battery,
                    level: 1,
                    condition: 30,
                    experience: 0,
                    is_functional: true,
                },
                kitchen: Module {
                    module_type: ModuleType::Kitchen,
                    level: 1,
                    condition: 40,
                    experience: 0,
                    is_functional: true,
                },
                social: Module {
                    module_type: ModuleType::Social,
                    level: 1,
                    condition: 50,
                    experience: 0,
                    is_functional: true,
                },
            },

            // 当前状态
            current_state: ActivityState::Idle,
            current_task: None,
            location: Location::Shop,
        }
    }
}
```

### 5.10 对话系统设计

#### 5.10.1 设计理念

对话系统是玩家与盼盼交流的核心渠道。所有玩家与盼盼的互动都通过对话完成，包括：
- 下达经营指令
- 询问小馆状态
- 聊天增进感情
- 触发记忆碎片
- 事件决策讨论

#### 5.10.2 对话消息结构

```rust
/// 对话消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueMessage {
    pub id: Uuid,
    pub save_id: Uuid,
    pub sender: DialogueSender,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,

    /// 消息状态（仅对玩家消息有效）
    pub status: MessageStatus,

    /// 关联的游戏实体
    pub related_entity: Option<EntityReference>,

    /// LLM 上下文
    pub llm_context: Option<LlmContextData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DialogueSender {
    /// 玩家（来自火星）
    Player { location: PlayerLocation },
    /// 盼盼（来自地球）
    Panpan { mood: Emotion, location: Location },
    /// 系统消息
    System { message_category: SystemMessageCategory },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerLocation {
    Mars,
    Other { name: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemMessageCategory {
    Notification,    // 通知
    Warning,         // 警告
    Achievement,     // 成就
    Milestone,       // 里程碑
    Error,           // 错误
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// 普通聊天
    Chat,

    /// 指令类消息
    Command {
        command_type: CommandType,
        parsed_intent: Option<ParsedIntent>,
    },

    /// 汇报类消息（盼盼主动）
    Report { report_type: ReportType },

    /// 询问类消息
    Query { query_subject: String },

    /// 事件相关
    Event {
        event_id: Uuid,
        event_stage: EventStage,
    },

    /// 记忆碎片触发
    MemoryTrigger { fragment_id: Uuid },

    /// 决策请求
    DecisionRequest {
        decision_id: Uuid,
        options: Vec<DecisionOption>,
    },

    /// 决策响应
    DecisionResponse {
        decision_id: Uuid,
        selected_option: Uuid,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageStatus {
    Sent,                              // 已发送，等待传输
    InTransit { arrive_at: DateTime<Utc> }, // 传输中
    Arrived,                          // 已到达盼盼
    Processing,                       // 盼盼正在处理
    Executed,                         // 已执行
    Failed { reason: String },        // 执行失败
    Rejected { reason: String },      // 被盼盼拒绝
    Modified { modification: String },// 被盼盼修改后执行
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandType {
    // 经营指令
    OpenShop,
    CloseShop,
    UpdateMenu,
    SetPrice,
    PurchaseSupplies,
    // 种植指令
    PlantCrop,
    WaterCrop,
    HarvestCrop,
    // 旅行指令
    StartTravel,
    CancelTravel,
    // 实验指令
    StartExperiment,
    AdjustRecipe,
    // 维修指令
    RepairFacility,
    UpgradeFacility,
    // 社交指令
    VisitNeighbor,
    SendGift,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReportType {
    DailySummary,       // 每日简报
    MorningBriefing,    // 早间汇报
    EveningReport,      // 晚间汇报
    TravelLog,          // 旅行日志
    ExperimentResult,   // 实验结果
    EmergencyAlert,     // 紧急警报
    MemoryRecovery,     // 记忆恢复
    MilestoneAchieved,  // 里程碑达成
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventStage {
    Started,
    WaitingDecision,
    DecisionMade,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub entity_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityType {
    Recipe,
    Customer,
    Facility,
    Crop,
    Travel,
    Event,
    Memory,
    Neighbor,
    Item,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedIntent {
    pub understood: bool,
    pub intent: String,
    pub entities: Vec<ExtractedEntity>,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmContextData {
    pub prompt_tokens: u32,
    pub response_tokens: u32,
}
```

#### 5.10.3 对话上下文管理

```rust
/// 对话上下文
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueContext {
    pub save_id: Uuid,

    /// 对话历史（最近 N 条）
    pub history: VecDeque<DialogueMessage>,

    /// 当前活跃话题
    pub active_topic: Option<ActiveTopic>,

    /// 待处理的决策
    pub pending_decisions: Vec<PendingDecision>,

    /// 对话统计
    pub statistics: DialogueStatistics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveTopic {
    pub topic_type: TopicType,
    pub started_at: DateTime<Utc>,
    pub related_entity: Option<EntityReference>,
    pub message_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TopicType {
    ShopManagement,     // 小馆经营
    Travel,             // 旅行计划
    Recipe,             // 菜谱讨论
    Garden,             // 种植话题
    Memory,             // 记忆回忆
    DailyChat,          // 日常闲聊
    Event,              // 事件处理
    Neighbor,           // 邻里关系
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingDecision {
    pub decision_id: Uuid,
    pub decision_type: DecisionType,
    pub description: String,
    pub options: Vec<DecisionOption>,
    pub deadline: DateTime<Utc>,
    pub urgency: DecisionUrgency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecisionType {
    TravelDestination,  // 选择旅行目的地
    EventResponse,      // 事件响应
    RecipeExperiment,   // 实验策略
    CustomerIssue,      // 顾客问题
    SupplyOrder,        // 订单决策
    FacilityUpgrade,    // 设施升级选择
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: Uuid,
    pub label: String,
    pub description: String,
    pub consequences: Vec<PredictedConsequence>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedConsequence {
    pub consequence_type: String,
    pub description: String,
    pub probability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecisionUrgency {
    Low,        // 24小时内
    Medium,     // 12小时内
    High,       // 4小时内
    Critical,   // 1小时内
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueStatistics {
    pub total_messages: u32,
    pub player_messages: u32,
    pub panpan_messages: u32,
    pub commands_sent: u32,
    pub commands_executed: u32,
    pub commands_rejected: u32,
    pub decisions_made: u32,
    pub average_response_time_seconds: f32,
    pub last_message_at: Option<DateTime<Utc>>,
}
```

#### 5.10.4 对话流程管理

```rust
/// 对话系统管理器
pub struct DialogueManager {
    context: DialogueContext,
    llm_manager: Arc<LlmManager>,
    delay_calculator: Arc<CommunicationSystem>,
}

impl DialogueManager {
    /// 处理玩家消息
    pub async fn process_player_message(
        &mut self,
        content: String,
        message_type: MessageType,
    ) -> Result<DialogueMessage, GameError> {
        // 1. 创建消息
        let message = DialogueMessage {
            id: Uuid::new_v4(),
            save_id: self.context.save_id,
            sender: DialogueSender::Player {
                location: PlayerLocation::Mars,
            },
            content: content.clone(),
            timestamp: Utc::now(),
            message_type: message_type.clone(),
            status: MessageStatus::Sent,
            related_entity: None,
            llm_context: None,
        };

        // 2. 计算通信延迟
        let delay_minutes = self.delay_calculator.calculate_total_delay();
        let arrive_at = Utc::now() + Duration::minutes(delay_minutes as i64);

        // 3. 根据消息类型处理
        match message_type {
            MessageType::Command { .. } => {
                self.process_command_message(message.clone(), arrive_at).await
            }
            MessageType::Chat => {
                self.process_chat_message(message.clone(), arrive_at).await
            }
            MessageType::Query { .. } => {
                self.process_query_message(message.clone(), arrive_at).await
            }
            MessageType::DecisionResponse { .. } => {
                self.process_decision_response(message.clone()).await
            }
            _ => Ok(message),
        }
    }

    /// 处理指令类消息
    async fn process_command_message(
        &mut self,
        mut message: DialogueMessage,
        arrive_at: DateTime<Utc>,
    ) -> Result<DialogueMessage, GameError> {
        // 解析指令意图
        let parsed = self.parse_command_intent(&message.content).await?;

        // 更新消息类型
        if let MessageType::Command { command_type, .. } = &mut message.message_type {
            *command_type = parsed.command_type.clone();
        }
        if let MessageType::Command { parsed_intent, .. } = &mut message.message_type {
            *parsed_intent = Some(parsed.clone());
        }

        // 更新状态为传输中
        message.status = MessageStatus::InTransit { arrive_at };

        // 存入延迟队列
        self.queue_delayed_command(message.clone(), arrive_at);

        // 添加到历史
        self.context.history.push_back(message.clone());
        self.context.statistics.player_messages += 1;
        self.context.statistics.commands_sent += 1;

        Ok(message)
    }

    /// 处理聊天类消息
    async fn process_chat_message(
        &mut self,
        mut message: DialogueMessage,
        arrive_at: DateTime<Utc>,
    ) -> Result<DialogueMessage, GameError> {
        // 生成盼盼的响应
        let panpan_response = self.generate_panpan_chat_response(&message).await?;

        // 更新状态
        message.status = MessageStatus::Arrived;
        message.llm_context = Some(LlmContextData {
            prompt_tokens: panpan_response.prompt_tokens,
            response_tokens: panpan_response.response_tokens,
        });

        self.context.history.push_back(message.clone());
        self.context.statistics.player_messages += 1;

        Ok(message)
    }

    /// 生成盼盼的聊天响应
    async fn generate_panpan_chat_response(
        &mut self,
        player_message: &DialogueMessage,
    ) -> Result<PanpanResponse, GameError> {
        // 构建上下文
        let context = self.build_llm_context();

        // 调用 LLM
        let response = self.llm_manager
            .generate_chat_response(player_message, &context)
            .await?;

        // 创建盼盼的响应消息
        let panpan_message = DialogueMessage {
            id: Uuid::new_v4(),
            save_id: self.context.save_id,
            sender: DialogueSender::Panpan {
                mood: response.suggested_mood.clone(),
                location: Location::Shop,
            },
            content: response.content.clone(),
            timestamp: Utc::now(),
            message_type: MessageType::Chat,
            status: MessageStatus::Executed,
            related_entity: None,
            llm_context: Some(LlmContextData {
                prompt_tokens: response.prompt_tokens,
                response_tokens: response.response_tokens,
            }),
        };

        // 添加到历史
        self.context.history.push_back(panpan_message);
        self.context.statistics.panpan_messages += 1;

        Ok(response)
    }

    /// 解析指令意图
    async fn parse_command_intent(&self, content: &str) -> Result<ParsedIntentWithCommand, GameError> {
        let intent = self.llm_manager.parse_intent(content).await?;

        Ok(ParsedIntentWithCommand {
            command_type: intent.command_type,
            understood: intent.understood,
            intent: intent.intent,
            entities: intent.entities.into_iter().map(|e| ExtractedEntity {
                entity_type: e.entity_type,
                value: e.value,
                confidence: e.confidence,
            }).collect(),
            confidence: intent.confidence,
        })
    }

    /// 构建 LLM 上下文
    fn build_llm_context(&self) -> LlmPromptContext {
        LlmPromptContext {
            recent_history: self.context.history.iter().rev().take(10).cloned().collect(),
            active_topic: self.context.active_topic.clone(),
            pending_decisions: self.context.pending_decisions.clone(),
        }
    }

    /// 将指令加入延迟队列
    fn queue_delayed_command(&mut self, message: DialogueMessage, arrive_at: DateTime<Utc>) {
        // 实现延迟队列逻辑
    }
}

#[derive(Clone, Debug)]
pub struct LlmPromptContext {
    pub recent_history: Vec<DialogueMessage>,
    pub active_topic: Option<ActiveTopic>,
    pub pending_decisions: Vec<PendingDecision>,
}

#[derive(Clone, Debug)]
pub struct PanpanResponse {
    pub content: String,
    pub suggested_mood: Emotion,
    pub trust_change: i32,
    pub prompt_tokens: u32,
    pub response_tokens: u32,
}

#[derive(Clone, Debug)]
pub struct ParsedIntentWithCommand {
    pub command_type: CommandType,
    pub understood: bool,
    pub intent: String,
    pub entities: Vec<ExtractedEntity>,
    pub confidence: f32,
}
```

#### 5.10.5 对话数据库表

```sql
-- 对话消息表
CREATE TABLE dialogue_messages (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    sender_type TEXT NOT NULL,         -- Player/Panpan/System
    sender_detail TEXT NOT NULL,       -- JSON
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    message_type TEXT NOT NULL,        -- JSON
    status TEXT NOT NULL,
    related_entity TEXT,               -- JSON
    llm_context TEXT,                  -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

CREATE INDEX idx_dialogue_save ON dialogue_messages(save_id);
CREATE INDEX idx_dialogue_timestamp ON dialogue_messages(save_id, timestamp);
CREATE INDEX idx_dialogue_sender ON dialogue_messages(save_id, sender_type);

-- 待处理决策表
CREATE TABLE pending_decisions (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    decision_type TEXT NOT NULL,
    description TEXT NOT NULL,
    options TEXT NOT NULL,             -- JSON
    deadline TEXT NOT NULL,
    urgency TEXT NOT NULL,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    selected_option TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

CREATE INDEX idx_decisions_save ON pending_decisions(save_id);
CREATE INDEX idx_decisions_deadline ON pending_decisions(save_id, deadline);
```

#### 5.10.6 对话 API

```
# 发送消息给盼盼
POST /api/v1/saves/:id/dialogue/send
Body: {
    "content": "盼盼，今天店里生意怎么样？",
    "message_type": { "Chat": {} }
}
Response: {
    "message_id": "uuid",
    "status": "Sent",
    "estimated_arrival": "2024-01-15T10:30:00Z",
    "delay_minutes": 8
}

# 获取对话历史
GET /api/v1/saves/:id/dialogue/history
Query: ?limit=50&before=uuid
Response: {
    "messages": [DialogueMessage],
    "has_more": true
}

# 获取待处理决策
GET /api/v1/saves/:id/dialogue/decisions
Response: {
    "pending_decisions": [PendingDecision],
    "urgent_count": 2
}

# 做出决策
POST /api/v1/saves/:id/dialogue/decisions/:decision_id/respond
Body: {
    "selected_option": "uuid",
    "comment": "选这个吧，看起来不错"
}
Response: {
    "success": true,
    "panpan_reaction": "好的主人，我会按您的决定去做！"
}

# 获取对话统计
GET /api/v1/saves/:id/dialogue/statistics
Response: DialogueStatistics
```

---

## 六、星夜小馆系统设计

### 6.1 设计理念

星夜小馆是《味延廿载》中玩家继承并经营的地球老街小饭馆，由机器人盼盼远程管理。小馆系统与盼盼系统相辅相成，共同构成游戏的核心经营维度。

**核心设计原则**：
- 小馆属性与盼盼属性相互影响
- 设施状态决定经营能力上限
- 口碑系统驱动客流量变化
- 修复进度作为游戏进程指标

### 6.2 小馆基础信息

| 属性 | 说明 |
|------|------|
| **名称** | 星夜小馆 |
| **英文名** | Starry Night Bistro |
| **位置** | 地球·老街（老城区，临街） |
| **建筑年代** | 约1995年（祖父购得并改造） |
| **建筑结构** | 两层小楼，一楼餐厅+厨房，二楼储物+盼盼充电区，后院为菜地+工坊 |

### 6.3 设施系统

设施分为四大区域，每个区域有独立的完成度(0-100%)。完成度影响该区域的运营能力。

#### 6.3.1 区域与等级定义

```rust
/// 设施区域
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FacilityZone {
    Restaurant,  // 餐厅
    Kitchen,     // 厨房
    Backyard,    // 后院
    Workshop,    // 工坊
}

/// 区域等级（以餐厅为例）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneLevel {
    pub zone: FacilityZone,
    pub level: u32,                 // 1-5
    pub name: String,               // 等级名称
    pub reputation_cap: u32,        // 口碑上限
    pub unlocked_features: Vec<String>,
}

impl ZoneLevel {
    /// 餐厅等级定义
    pub fn restaurant_levels() -> Vec<Self> {
        vec![
            Self {
                zone: FacilityZone::Restaurant,
                level: 1,
                name: "破土重生".into(),
                reputation_cap: 30,
                unlocked_features: vec!["basic_service".into()],
            },
            Self {
                zone: FacilityZone::Restaurant,
                level: 2,
                name: "初具规模".into(),
                reputation_cap: 40,
                unlocked_features: vec!["new_menu_slot".into()],
            },
            Self {
                zone: FacilityZone::Restaurant,
                level: 3,
                name: "小有名气".into(),
                reputation_cap: 60,
                unlocked_features: vec!["extended_hours".into()],
            },
            Self {
                zone: FacilityZone::Restaurant,
                level: 4,
                name: "老街地标".into(),
                reputation_cap: 80,
                unlocked_features: vec!["special_events".into()],
            },
            Self {
                zone: FacilityZone::Restaurant,
                level: 5,
                name: "星夜明珠".into(),
                reputation_cap: 100,
                unlocked_features: vec!["hidden_customers".into(), "ending_clues".into()],
            },
        ]
    }
}
```

#### 6.3.2 子设施定义

```rust
/// 子设施定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubFacility {
    pub id: String,
    pub name: String,
    pub zone: FacilityZone,
    pub facility_type: FacilityType,
    pub level: u32,                 // 等级 1-5（部分设施最高3级）
    pub max_level: u32,
    pub condition: u32,             // 完好度 0-100
    pub is_functional: bool,        // 是否可用
    pub effect: FacilityEffect,     // 影响效果
    pub upgrade_cost: UpgradeCost,
    pub quantity: Option<u32>,      // 数量（如餐桌椅）
    pub max_quantity: Option<u32>,
}

/// 设施类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FacilityType {
    // 餐厅设施
    DiningTables,    // 餐桌椅 - 影响: 最大同时接待顾客数
    Lighting,        // 照明系统 - 影响: 深夜客流、停留时间
    Signboard,       // 招牌 - 影响: 新顾客发现率
    ClimateControl,  // 空调/暖气 - 影响: 季节客流
    CashierSystem,   // 收银系统 - 影响: 翻台率、财务准确性
    Decoration,      // 装饰与风格 - 影响: 氛围评分
    // 厨房设施
    Stove,           // 灶台
    OvenSteamer,     // 烤箱/蒸箱
    Refrigerator,    // 冰箱/冷柜
    Cookware,        // 厨具
    Ventilation,     // 通风系统
    // 后院设施
    VegetablePatch,  // 菜地
    Irrigation,      // 灌溉系统
    ToolShed,        // 工具房
    Greenhouse,      // 温室
    // 工坊设施
    Workbench,       // 工作台
    MaterialRack,    // 材料架
    RepairToolkit,   // 维修工具箱
}

/// 设施效果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FacilityEffect {
    pub effect_type: EffectType,
    pub base_value: f32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectType {
    MaxCustomers,      // 最大顾客数
    CookingSpeed,      // 烹饪速度
    StorageCapacity,   // 存储容量
    PlantingSlots,     // 种植槽位
    CraftingAbility,   // 制作能力
    AtmosphereScore,   // 氛围评分
    CustomerDiscovery, // 新顾客发现率
    TurnoverRate,      // 翻台率
    SeasonModifier,    // 季节修正
}
```

#### 6.3.3 餐厅子设施详情

| 子设施 | 初始状态 | 等级上限 | 影响 |
|--------|----------|----------|------|
| 餐桌椅 | 8套，仅3套可用 | 5级 | 最大同时接待顾客数（客容量） |
| 照明系统 | 灯泡损坏，仅2盏可用 | 5级 | 深夜客流、顾客停留时间、拍照打卡率 |
| 招牌 | 褪色破损 | 3级 | 新顾客发现率 |
| 空调/暖气 | 完全故障 | 3级 | 季节客流（夏/冬季若故障，客流-50%） |
| 收银系统 | 手动记账 | 3级 | 结账效率（翻台率）、财务准确性 |
| 装饰风格 | 墙壁斑驳 | 5级 | 氛围评分、特定顾客群体吸引力 |

#### 6.3.4 厨房子设施详情

厨房是星夜小馆的"心脏"，所有菜品在这里诞生，盼盼的实验也在这里进行。

**厨房等级系统**：

| 等级 | 名称 | 解锁条件 | 效果 |
|------|------|----------|------|
| 1级 | 勉强可用 | 初始状态 | 基础烹饪可行，效率低，容易失败 |
| 2级 | 整修一新 | 修复灶台、冰箱基本功能 | 烹饪成功率+10%，解锁基础菜谱升级 |
| 3级 | 功能齐全 | 烤箱/蒸箱修复，厨具基本齐全 | 可制作复杂菜品，实验速度+15% |
| 4级 | 专业厨房 | 通风系统升级，设备稳定 | 可同时进行两道菜烹饪，解锁特殊食材处理 |
| 5级 | 星夜后厨 | 全部设施顶级，智能监控 | 实验成功率+20%，解锁隐藏菜谱线索 |

**厨房子设施列表**：

| 子设施 | 初始状态 | 等级上限 | 影响 |
|--------|----------|----------|------|
| 灶台 | 锈蚀严重，点火困难 | 5级 | 烹饪速度、成功率、菜品品质上限 |
| 冰箱/冷柜 | 勉强运行，制冷不足 | 4级 | 食材保鲜时间、库存上限、特殊食材存储 |
| 烤箱/蒸箱 | 部分功能失灵 | 4级 | 可制作的菜品种类（烤、蒸类）、烘焙成功率 |
| 厨具 | 缺东少西 | 5级 | 烹饪效率、实验成功率、可同时制作的菜品数 |
| 通风系统 | 堵塞严重 | 3级 | 厨房环境、盼盼健康度、火灾风险 |
| 水槽 | 水龙头漏水 | 3级 | 清洗效率、食材处理速度、卫生状况 |
| 储物柜/货架 | 积灰，部分损坏 | 3级 | 食材存储容量、取用效率 |

**设施健康度影响**：

```rust
/// 厨房设施健康度影响
impl KitchenFacility {
    /// 灶台健康度影响
    pub fn stove_health_effect(&self) -> CookingModifier {
        if self.condition < 50 {
            CookingModifier {
                time_penalty: 0.5,     // 烹饪时间+50%
                failure_rate_bonus: 0.2, // 失败率+20%
            }
        } else {
            CookingModifier::default()
        }
    }

    /// 冰箱健康度影响
    pub fn refrigerator_health_effect(&self) -> StorageModifier {
        if self.condition < 60 {
            StorageModifier {
                spoilage_rate: 1.0,    // 食材腐败速度+100%
                food_safety_risk: 0.1,  // 食品安全事件概率+10%
            }
        } else {
            StorageModifier::default()
        }
    }

    /// 通风系统健康度影响
    pub fn ventilation_health_effect(&self) -> EnvironmentModifier {
        if self.condition < 40 {
            EnvironmentModifier {
                panpan_health_penalty: 1, // 盼盼每日健康度额外-1
                fire_risk_bonus: 0.05,    // 火灾风险+5%
            }
        } else {
            EnvironmentModifier::default()
        }
    }
}
```

#### 6.3.5 升级路径系统

```rust
/// 升级路径
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradePath {
    pub facility_type: FacilityType,
    pub from_level: u32,
    pub to_level: u32,
    pub cost: Decimal,              // 资金
    pub materials: Vec<MaterialCost>,
    pub time_days: u32,
    pub required_personnel: PersonnelType,
    pub unlocks: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialCost {
    pub material_type: MaterialType,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialType {
    Wood,           // 木材
    Fabric,         // 布料
    LightBulb,      // 灯泡
    OldPhoto,       // 老照片（需记忆碎片）
    RetroTile,      // 复古瓷砖（旅行带回）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonnelType {
    PanpanOnly,       // 盼盼独立完成
    NeedElectrician,  // 需要电工
    NeedCarpenter,    // 需要木匠
    NeedHelper,       // 需要帮工
    NeedNeighbor,     // 需要邻居帮助
}

/// 升级路径表（示例：餐桌椅）
impl UpgradePath {
    pub fn dining_tables_upgrades() -> Vec<Self> {
        vec![
            Self {
                facility_type: FacilityType::DiningTables,
                from_level: 1, to_level: 2,
                cost: Decimal::from(2000),
                materials: vec![],
                time_days: 1,
                required_personnel: PersonnelType::PanpanOnly,
                unlocks: None,
            },
            Self {
                facility_type: FacilityType::DiningTables,
                from_level: 2, to_level: 3,
                cost: Decimal::from(3000),
                materials: vec![MaterialCost { material_type: MaterialType::Wood, quantity: 5 }],
                time_days: 2,
                required_personnel: PersonnelType::PanpanOnly,
                unlocks: None,
            },
            Self {
                facility_type: FacilityType::DiningTables,
                from_level: 3, to_level: 4,
                cost: Decimal::from(5000),
                materials: vec![MaterialCost { material_type: MaterialType::Wood, quantity: 10 }],
                time_days: 3,
                required_personnel: PersonnelType::NeedCarpenter,
                unlocks: Some("solid_wood_tables".into()),
            },
            Self {
                facility_type: FacilityType::DiningTables,
                from_level: 4, to_level: 5,
                cost: Decimal::from(8000),
                materials: vec![],
                time_days: 5,
                required_personnel: PersonnelType::NeedCarpenter,
                unlocks: Some("theme_decoration".into()),
            },
        ]
    }
}
```

### 6.4 经营数据系统

#### 6.4.1 资金系统

```rust
/// 资金状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinancialState {
    pub funds: Decimal,              // 当前资金
    pub daily_income: Decimal,       // 今日收入
    pub daily_expense: Decimal,      // 今日支出
    pub total_income: Decimal,       // 累计收入
    pub total_expense: Decimal,      // 累计支出
}

impl FinancialState {
    /// 初始资金：祖父留下的存款
    pub fn initial() -> Self {
        Self {
            funds: Decimal::from(10000),
            daily_income: Decimal::ZERO,
            daily_expense: Decimal::ZERO,
            total_income: Decimal::ZERO,
            total_expense: Decimal::ZERO,
        }
    }
}
```

#### 6.4.2 顾客数据

```rust
/// 顾客统计
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerStats {
    pub daily_customers: u32,        // 今日客流量
    pub returning_customers: u32,    // 老顾客数量
    pub new_customer_rate: f32,      // 新顾客转化率
    pub avg_satisfaction: f32,       // 平均满意度 1-5
    pub customer_history: Vec<DailyCustomerRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyCustomerRecord {
    pub date: Date<Utc>,
    pub total: u32,
    pub new_customers: u32,
    pub returning_customers: u32,
    pub avg_satisfaction: f32,
}
```

#### 6.4.3 客流计算系统

```rust
/// 客流计算器
pub struct CustomerFlowCalculator;

impl CustomerFlowCalculator {
    /// 计算每日客流量
    ///
    /// 基础客流 = 口碑指数 × 0.5 + 季节系数 × 20 + 周末/节日加成
    /// 实际客流 = min(基础客流 × 氛围系数, 座位数 × 翻台率上限)
    pub fn calculate_daily_customers(
        reputation: f32,           // 口碑指数 0-100
        season: Season,
        is_weekend: bool,
        is_holiday: bool,
        atmosphere_index: &AtmosphereIndex,
        seating_capacity: u32,
        turnover_rate: f32,
        has_climate_control: bool,
    ) -> u32 {
        // 季节系数
        let season_mod = if has_climate_control {
            1.0  // 空调正常时不受季节影响
        } else {
            season.modifier()
        };

        // 基础客流
        let base_flow = reputation * 0.5
            + season_mod * 20.0
            + if is_weekend { 10.0 } else { 0.0 }
            + if is_holiday { 15.0 } else { 0.0 };

        // 氛围系数 (0-2倍)
        let atmosphere_mod = atmosphere_index.atmosphere_modifier();

        // 实际客流 = min(基础客流 × 氛围系数, 座位数 × 翻台率)
        let actual = (base_flow * atmosphere_mod).min(seating_capacity as f32 * turnover_rate);
        actual.max(0.0) as u32
    }
}

#[derive(Clone, Debug)]
pub enum Season {
    Spring,  // 系数 1.0
    Summer,  // 系数 0.8（无空调）
    Autumn,  // 系数 1.0
    Winter,  // 系数 0.7（无暖气）
}

impl Season {
    pub fn modifier(&self) -> f32 {
        match self {
            Season::Spring | Season::Autumn => 1.0,
            Season::Summer => 0.8,
            Season::Winter => 0.7,
        }
    }
}
```

### 6.5 厨房运营系统

#### 6.5.1 库存管理系统

```rust
/// 食材分类
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IngredientCategory {
    Basic,      // 基础食材（米、面、油、盐、酱油）- 可自动补货
    Fresh,      // 新鲜食材（蔬菜、肉类、海鲜）- 需采购或自产
    Special,    // 特殊食材（香料、特色酱料）- 旅行带回
    Premium,    // 珍贵食材（松露、和牛等）- 特殊事件获得
}

/// 食材信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: Uuid,
    pub name: String,
    pub category: IngredientCategory,
    pub quantity: u32,
    pub max_quantity: u32,
    pub freshness: Option<f32>,      // 新鲜度 0-1（仅新鲜食材）
    pub spoilage_rate: f32,          // 每日腐败率
    pub unit: String,
    pub unit_price: Decimal,
    pub auto_restock_threshold: Option<u32>,  // 自动补货阈值
    pub auto_restock_enabled: bool,
}

/// 库存容量
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryCapacity {
    pub current_types: u32,          // 当前食材种类数
    pub max_types: u32,              // 最大种类数（初始20，最大50）
    pub capacity_per_type: u32,      // 每种食材上限（初始10，最大50）
}

impl InventoryCapacity {
    /// 初始容量
    pub fn initial() -> Self {
        Self {
            current_types: 0,
            max_types: 20,
            capacity_per_type: 10,
        }
    }

    /// 根据冰箱和储物柜等级计算容量
    pub fn calculate_from_facilities(
        refrigerator_level: u32,
        storage_level: u32
    ) -> Self {
        Self {
            current_types: 0,
            max_types: 20 + (refrigerator_level - 1) * 10,
            capacity_per_type: 10 + (storage_level - 1) * 10,
        }
    }
}

/// 库存监控系统
pub struct InventoryMonitor {
    inventory: HashMap<Uuid, Ingredient>,
    capacity: InventoryCapacity,
}

impl InventoryMonitor {
    /// 每日盘点
    pub fn daily_check(&mut self) -> InventoryReport {
        let mut low_stock = Vec::new();
        let mut expired = Vec::new();
        let mut needs_restock = Vec::new();

        for (id, ingredient) in &self.inventory {
            // 检查低库存
            if ingredient.quantity < 5 {
                low_stock.push(ingredient.clone());
            }

            // 检查自动补货
            if ingredient.auto_restock_enabled {
                if let Some(threshold) = ingredient.auto_restock_threshold {
                    if ingredient.quantity < threshold {
                        needs_restock.push(ingredient.clone());
                    }
                }
            }

            // 检查新鲜度
            if let Some(freshness) = ingredient.freshness {
                if freshness < 0.2 {
                    expired.push(ingredient.clone());
                }
            }
        }

        InventoryReport {
            low_stock,
            expired,
            needs_restock,
        }
    }

    /// 更新食材新鲜度
    pub fn update_freshness(&mut self, refrigerator_condition: u32) {
        let spoilage_modifier = if refrigerator_condition >= 80 { 0.5 }
            else if refrigerator_condition >= 60 { 0.8 }
            else if refrigerator_condition >= 40 { 1.0 }
            else { 1.5 };

        for (_, ingredient) in &mut self.inventory {
            if let Some(ref mut freshness) = ingredient.freshness {
                *freshness -= ingredient.spoilage_rate * spoilage_modifier;
                *freshness = freshness.max(0.0);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryReport {
    pub low_stock: Vec<Ingredient>,
    pub expired: Vec<Ingredient>,
    pub needs_restock: Vec<Ingredient>,
}
```

#### 6.5.2 菜品制作流程

```rust
/// 烹饪队列
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookingQueue {
    pub slots: Vec<Option<CookingSlot>>,
    pub max_slots: u32,              // 受厨具等级影响（1-3）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookingSlot {
    pub recipe_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expected_complete: DateTime<Utc>,
    pub progress: f32,               // 0-1
    pub quality_prediction: f32,     // 预测品质
}

impl CookingQueue {
    /// 计算最大烹饪槽位
    pub fn calculate_max_slots(cookware_level: u32, ventilation_level: u32) -> u32 {
        let base = 1;
        let cookware_bonus = if cookware_level >= 4 { 1 } else { 0 };
        let ventilation_bonus = if ventilation_level >= 3 { 1 } else { 0 };
        (base + cookware_bonus + ventilation_bonus).min(3)
    }
}

/// 烹饪系统
pub struct CookingSystem;

impl CookingSystem {
    /// 计算烹饪时间
    pub fn calculate_cooking_time(
        base_time_minutes: u32,
        panpan_cooking_skill: u32,
        stove_condition: u32,
        cookware_level: u32,
    ) -> u32 {
        let skill_modifier = 1.0 - (panpan_cooking_skill as f32 * 0.03);
        let condition_modifier = if stove_condition >= 50 { 1.0 } else { 1.5 };
        let level_modifier = 1.0 - (cookware_level as f32 * 0.05);

        let result = base_time_minutes as f32 * skill_modifier * condition_modifier * level_modifier;
        result.max(1.0) as u32
    }

    /// 计算成功率
    pub fn calculate_success_rate(
        recipe_difficulty: f32,
        panpan_cooking_skill: u32,
        facilities_condition: f32,
        ingredient_freshness: Option<f32>,
        panpan_emotion: &Emotion,
    ) -> f32 {
        let base_rate = 0.7;
        let skill_bonus = panpan_cooking_skill as f32 * 0.02;
        let facility_bonus = (facilities_condition - 50.0) / 100.0;
        let freshness_bonus = ingredient_freshness.map(|f| (f - 0.5) * 0.1).unwrap_or(0.0);
        let emotion_bonus = match panpan_emotion {
            Emotion::Happy => 0.05,
            Emotion::Tired => -0.1,
            Emotion::Confused => -0.05,
            _ => 0.0,
        };

        (base_rate + skill_bonus + facility_bonus + freshness_bonus + emotion_bonus).clamp(0.1, 0.99)
    }

    /// 计算菜品品质
    pub fn calculate_dish_quality(
        recipe_base_quality: u32,
        ingredient_freshness: Option<f32>,
        panpan_cooking_skill: u32,
        facilities_condition: f32,
    ) -> u32 {
        let base = recipe_base_quality as f32;
        let freshness_mod = ingredient_freshness.unwrap_or(0.7) * 0.3;
        let skill_mod = (panpan_cooking_skill as f32 / 10.0) * 0.2;
        let facility_mod = (facilities_condition / 100.0) * 0.2;

        let quality = base * (1.0 + freshness_mod + skill_mod + facility_mod);
        quality.clamp(1.0, 5.0) as u32
    }
}

/// 菜品产出
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DishOutput {
    pub recipe_id: Uuid,
    pub quality: u32,                // 1-5星
    pub produced_at: DateTime<Utc>,
    pub ingredients_consumed: Vec<(Uuid, u32)>,
    pub sold: bool,
    pub sold_price: Option<Decimal>,
}
```

#### 6.5.3 实验研发系统

厨房也是盼盼进行新菜实验的场所。实验系统与盼盼的旅行收集和实验技能深度结合。

```rust
/// 实验状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperimentStatus {
    NotStarted,
    InProgress {
        current_iteration: u32,
        max_iterations: u32,
    },
    Completed,
    Failed,
}

/// 模糊菜谱（旅行带回）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VagueRecipe {
    pub id: Uuid,
    pub name: String,
    pub cuisine_type: String,
    pub description: String,         // 模糊描述
    pub source_location: String,     // 旅行来源
    pub estimated_ingredients: Vec<EstimatedIngredient>,
    pub estimated_difficulty: f32,   // 1-5
    pub required_iterations: u32,    // 需要的实验次数（3-10）
}

/// 预估食材用量
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstimatedIngredient {
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub estimated_min: u32,          // 预估最小量
    pub estimated_max: u32,          // 预估最大量
    pub actual_amount: Option<u32>,  // 确定量（实验成功后）
}

/// 实验记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub id: Uuid,
    pub vague_recipe_id: Uuid,
    pub iteration: u32,
    pub attempted_at: DateTime<Utc>,

    // 本次尝试的配方
    pub attempted_amounts: HashMap<Uuid, u32>,

    // 传感器反馈
    pub feedback: Vec<IngredientFeedback>,

    // 结果
    pub result: ExperimentResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngredientFeedback {
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub direction: AdjustmentDirection,
    pub confidence: f32,             // 反馈置信度
    pub suggested_adjustment: i32,   // 建议调整量（克/毫升）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdjustmentDirection {
    TooLittle,    // 偏少
    TooMuch,      // 偏多
    JustRight,    // 刚好
    Unknown,      // 不确定
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperimentResult {
    Success {
        final_recipe: PreciseRecipe,
    },
    ContinueNeeded {
        remaining_iterations: u32,
    },
    Failed {
        reason: String,
    },
}

/// 精确菜谱（实验成功后）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreciseRecipe {
    pub id: Uuid,
    pub vague_recipe_id: Uuid,
    pub name: String,
    pub cuisine_type: String,
    pub ingredients: Vec<PreciseIngredient>,
    pub cooking_time_minutes: u32,
    pub price: Decimal,
    pub cost: Decimal,
    pub base_quality: u32,
    pub unlock_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreciseIngredient {
    pub ingredient_id: Uuid,
    pub amount: u32,
    pub unit: String,
}

/// 实验系统
pub struct ExperimentSystem;

impl ExperimentSystem {
    /// 检查实验条件
    pub fn can_start_experiment(
        kitchen_facilities: &[SubFacility],
        inventory: &InventoryMonitor,
        vague_recipe: &VagueRecipe,
    ) -> Result<(), ExperimentError> {
        // 检查设施健康度
        for facility in kitchen_facilities {
            if facility.zone == FacilityZone::Kitchen && facility.condition < 70 {
                return Err(ExperimentError::FacilityConditionTooLow);
            }
        }

        // 检查食材是否充足
        for est in &vague_recipe.estimated_ingredients {
            if !inventory.has_sufficient_ingredient(est.ingredient_id, est.estimated_max) {
                return Err(ExperimentError::InsufficientIngredients);
            }
        }

        Ok(())
    }

    /// 计算实验反馈准确度
    pub fn calculate_feedback_accuracy(
        panpan_sensor_level: u32,
        panpan_experiment_skill: u32,
    ) -> f32 {
        let base_accuracy = 0.5;
        let sensor_bonus = panpan_sensor_level as f32 * 0.05;
        let skill_bonus = panpan_experiment_skill as f32 * 0.03;
        (base_accuracy + sensor_bonus + skill_bonus).min(0.95)
    }

    /// 计算所需实验次数
    pub fn calculate_required_iterations(
        recipe_difficulty: f32,
        panpan_experiment_skill: u32,
    ) -> u32 {
        let base = (recipe_difficulty * 2.0) as u32;
        let skill_reduction = (panpan_experiment_skill / 2) as u32;
        (base.max(3) - skill_reduction).max(3)
    }
}

#[derive(Debug)]
pub enum ExperimentError {
    FacilityConditionTooLow,
    InsufficientIngredients,
    RecipeAlreadyMastered,
}
```

#### 6.5.4 厨房清洁度与安全

```rust
/// 厨房卫生状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KitchenHygiene {
    pub cleanliness: u32,            // 清洁度 0-100
    pub last_cleaned: Option<DateTime<Utc>>,
    pub daily_decline_rate: f32,     // 每日下降率（烹饪越多越快）
    pub safety_risks: Vec<SafetyRisk>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SafetyRisk {
    FireHazard {
        probability: f32,            // 火灾概率
        cause: String,               // 原因（如通风不良、灶台老化）
    },
    FoodSafety {
        probability: f32,            // 食品安全事件概率
        cause: String,               // 原因（如冰箱故障、食材过期）
    },
}

impl KitchenHygiene {
    /// 每日清洁度下降
    pub fn daily_decline(&mut self, dishes_cooked: u32) {
        let base_decline = 5.0;
        let cooking_decline = dishes_cooked as f32 * 0.5;
        self.cleanliness = (self.cleanliness as f32 - base_decline - cooking_decline).max(0.0) as u32;
    }

    /// 执行清洁
    pub fn clean(&mut self, cleaner_skill: u32) {
        let base_restore = 30;
        let skill_bonus = cleaner_skill * 5;
        self.cleanliness = (self.cleanliness + base_restore + skill_bonus).min(100);
        self.last_cleaned = Some(Utc::now());
    }

    /// 计算安全风险
    pub fn calculate_safety_risks(
        &self,
        ventilation_condition: u32,
        refrigerator_condition: u32,
        stove_condition: u32,
    ) -> Vec<SafetyRisk> {
        let mut risks = Vec::new();

        // 火灾风险
        if ventilation_condition < 40 || stove_condition < 30 {
            let fire_prob = (100 - ventilation_condition as f32) / 500.0
                + (100 - stove_condition as f32) / 1000.0;
            risks.push(SafetyRisk::FireHazard {
                probability: fire_prob,
                cause: if ventilation_condition < 40 {
                    "通风系统堵塞".into()
                } else {
                    "灶台老化严重".into()
                },
            });
        }

        // 食品安全风险
        if self.cleanliness < 50 || refrigerator_condition < 50 {
            let food_prob = (100 - self.cleanliness as f32) / 200.0
                + (100 - refrigerator_condition as f32) / 200.0;
            risks.push(SafetyRisk::FoodSafety {
                probability: food_prob,
                cause: if self.cleanliness < 50 {
                    "厨房清洁度不足".into()
                } else {
                    "冰箱制冷异常".into()
                },
            });
        }

        risks
    }
}
```

### 6.6 后院种植系统

后院是星夜小馆的"绿洲"，既是食材来源，也是盼盼放松心情的地方。这里可以种植蔬菜、香料和花卉，收获的作物用于烹饪、装饰或赠送邻居。

#### 6.6.1 后院等级系统

```rust
/// 后院等级
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackyardLevel {
    pub level: u32,                 // 1-5
    pub name: String,
    pub unlocked_plots: u32,        // 解锁的菜地数量
    pub growth_speed_bonus: f32,    // 生长速度加成
    pub unlocked_crop_types: Vec<CropCategory>,
}

impl BackyardLevel {
    pub fn levels() -> Vec<Self> {
        vec![
            Self {
                level: 1,
                name: "荒芜初垦".into(),
                unlocked_plots: 1,
                growth_speed_bonus: 0.0,
                unlocked_crop_types: vec![CropCategory::Vegetable],
            },
            Self {
                level: 2,
                name: "菜畦成行".into(),
                unlocked_plots: 2,
                growth_speed_bonus: 0.1,
                unlocked_crop_types: vec![CropCategory::Vegetable, CropCategory::Herb],
            },
            Self {
                level: 3,
                name: "花果满园".into(),
                unlocked_plots: 3,
                growth_speed_bonus: 0.15,
                unlocked_crop_types: vec![CropCategory::Vegetable, CropCategory::Herb, CropCategory::Flower],
            },
            Self {
                level: 4,
                name: "四季常青".into(),
                unlocked_plots: 4,
                growth_speed_bonus: 0.2,
                unlocked_crop_types: vec![CropCategory::Vegetable, CropCategory::Herb, CropCategory::Flower, CropCategory::Special],
            },
            Self {
                level: 5,
                name: "星夜花园".into(),
                unlocked_plots: 5,
                growth_speed_bonus: 0.3,
                unlocked_crop_types: vec![CropCategory::Vegetable, CropCategory::Herb, CropCategory::Flower, CropCategory::Special, CropCategory::Exotic],
            },
        ]
    }
}
```

#### 6.6.2 后院子设施

```rust
/// 后院子设施类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BackyardFacilityType {
    VegetablePlot,    // 菜地
    Irrigation,       // 灌溉系统
    ToolShed,         // 工具房
    CompostArea,      // 堆肥区
    Greenhouse,       // 温室（可选）
}

/// 菜地
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VegetablePlot {
    pub id: Uuid,
    pub plot_number: u32,           // 1-5
    pub is_unlocked: bool,
    pub soil_level: u32,            // 1-3（普通/改良/肥沃黑土）
    pub fertility: u32,             // 肥力 0-100
    pub current_crop: Option<GrowingCrop>,
    pub needs_tilling: bool,        // 是否需要翻土
}

/// 生长中的作物
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowingCrop {
    pub crop_id: Uuid,
    pub crop_type: CropType,
    pub planted_at: DateTime<Utc>,
    pub current_stage: GrowthStage,
    pub stage_progress: f32,        // 当前阶段进度 0-1
    pub health: u32,                // 健康度 0-100（影响产量）
    pub watered_today: bool,
    pub fertilized_this_cycle: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GrowthStage {
    Sowing,       // 播种期（1天）
    Germinating,  // 发芽期（1-2天）
    Growing,      // 生长期（2-3天）
    Mature,       // 成熟期（可收获，2天后开始枯萎）
    Withering,    // 枯萎中（产量逐渐下降）
}

/// 灌溉系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrrigationSystem {
    pub level: u32,                 // 0-3（0=无，需手动浇水）
    pub condition: u32,             // 健康度 0-100
    pub is_automatic: bool,         // 是否自动浇水
    pub water_efficiency: f32,      // 浇水效率
}

impl IrrigationSystem {
    pub fn level_benefits(level: u32) -> Self {
        Self {
            level,
            condition: 100,
            is_automatic: level >= 3,
            water_efficiency: match level {
                0 => 0.5,   // 手动，效率低
                1 => 0.7,   // 简易水管
                2 => 0.9,   // 改良灌溉
                3 => 1.0,   // 智能滴灌
                _ => 1.0,
            },
        }
    }
}

/// 堆肥区
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompostArea {
    pub level: u32,                 // 0-2（0=无）
    pub materials: Vec<CompostMaterial>,
    pub processing_batch: Option<CompostBatch>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompostMaterial {
    pub material_type: String,      // 厨余类型
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompostBatch {
    pub started_at: DateTime<Utc>,
    pub ready_at: DateTime<Utc>,    // 堆肥需要3-5天
    pub expected_yield: u32,        // 预计产出肥料数量
}

/// 温室
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Greenhouse {
    pub is_built: bool,
    pub condition: u32,
    pub current_crops: Vec<GrowingCrop>,
    pub temperature_control: bool,  // 温度控制（反季节种植）
}
```

#### 6.6.3 作物系统

```rust
/// 作物类别
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CropCategory {
    Vegetable,    // 蔬菜（2-4天）：烹饪核心食材
    Herb,         // 香料（3-5天）：烹饪调味，工坊制作
    Flower,       // 花卉（4-7天）：装饰餐厅，赠送顾客
    Special,      // 特殊作物（不定）：与事件/记忆碎片相关
    Exotic,       // 异星作物（旅行带回）：稀有菜谱材料
}

/// 作物稀有度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CropRarity {
    Common,     // 普通：可购买，常见
    Uncommon,   // 少见：邻里赠送，部分旅行
    Rare,       // 稀有：特殊旅行，事件奖励
    Legendary,  // 传说：祖父遗产，隐藏事件
}

/// 作物定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropType {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: CropCategory,
    pub rarity: CropRarity,

    // 生长相关
    pub growth_stages: GrowthStages,   // 各阶段持续时间
    pub seasons: Vec<Season>,          // 适宜季节
    pub ideal_temperature: (i32, i32), // 理想温度范围（摄氏度）
    pub water_need: WaterNeed,         // 需水量等级
    pub fertilizer_need: FertilizerNeed, // 需肥量等级

    // 产量相关
    pub base_yield: u32,               // 基础产量
    pub max_yield: u32,                // 最大产量
    pub can_save_seed: bool,           // 是否可留种
    pub save_seed_skill_level: u32,    // 留种所需园艺等级

    // 经济相关
    pub seed_price: Decimal,           // 种子购买价格
    pub sale_price: Decimal,           // 售卖单价

    // 用途
    pub uses: Vec<CropUse>,
    pub related_recipes: Vec<Uuid>,    // 关联菜谱ID
    pub related_events: Vec<String>,   // 关联事件
}

/// 生长阶段定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowthStages {
    pub sowing_hours: u32,        // 播种期（小时）
    pub germinating_hours: u32,   // 发芽期（小时）
    pub growing_hours: u32,       // 生长期（小时）
    pub mature_hours: u32,        // 成熟期（可收获持续时间）
    pub withering_hours: u32,     // 枯萎期（产量逐渐下降）
}

impl GrowthStages {
    /// 总生长周期（小时）
    pub fn total_cycle(&self) -> u32 {
        self.sowing_hours + self.germinating_hours + self.growing_hours
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WaterNeed {
    Low,      // 少：每天0.5次（花卉为主）
    Medium,   // 中：每天1次（大部分作物）
    High,     // 高：每天2次（蔬菜为主）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FertilizerNeed {
    Low,      // 每周期消耗5肥力
    Medium,   // 每周期消耗10肥力
    High,     // 每周期消耗20肥力
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CropUse {
    Cooking {
        quality_bonus: f32,       // 烹饪品质加成
        freshness_decay: f32,     // 新鲜度对品质的影响
    },
    Gift {
        favor_bonus: u32,         // 好感度加成
        preferred_by: Vec<String>,// 喜欢此作物的角色
    },
    Decoration {
        atmosphere_bonus: u32,    // 氛围加成
        duration_days: u32,       // 持续天数
    },
    Sale {
        price: Decimal,           // 售价
    },
    Crafting {
        item_unlocks: Vec<String>,// 解锁的工坊配方
    },
}

/// 作物实例（收获后）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarvestedCrop {
    pub id: Uuid,
    pub crop_type_id: Uuid,
    pub name: String,
    pub quantity: u32,
    pub quality: u32,               // 1-5星（受种子品质+种植过程影响）
    pub freshness: f32,             // 0-1（随时间衰减）
    pub harvested_at: DateTime<Utc>,
    pub source_plot: Uuid,          // 收获自哪块菜地
}

impl HarvestedCrop {
    /// 计算当前品质（受新鲜度影响）
    pub fn current_quality(&self) -> u32 {
        let quality_decay = (1.0 - self.freshness) * 2.0; // 新鲜度影响
        (self.quality as f32 - quality_decay).max(1.0) as u32
    }

    /// 是否可用于烹饪
    pub fn can_cook(&self) -> bool {
        self.freshness > 0.3
    }

    /// 是否适合赠送
    pub fn can_gift(&self) -> bool {
        self.freshness > 0.5 && self.quality >= 3
    }
}
```

#### 6.6.3.1 种子系统详细设计

```rust
/// 种子定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Seed {
    pub id: Uuid,
    pub crop_type_id: Uuid,
    pub name: String,
    pub quality: u32,               // 1-5星（影响作物品质上限）
    pub origin: SeedOrigin,         // 产地/来源
    pub can_save: bool,             // 是否可留种
    pub quantity: u32,              // 数量
    pub obtained_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeedOrigin {
    Initial,                        // 初始赠送（祖父遗产）
    Purchased {                     // 购买
        shop_name: String,
        price: Decimal,
    },
    NeighborGift {                  // 邻居赠送
        neighbor_name: String,
        reason: String,             // 赠送原因
    },
    TravelBrought {                 // 旅行带回
        location: String,
        traveler_notes: String,     // 盼盼的旅行笔记
    },
    EventReward {                   // 事件奖励
        event_name: String,
    },
    SelfSaved {                     // 自留种
        from_harvest_id: Uuid,
        saved_at: DateTime<Utc>,
    },
}

/// 种子库存
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeedInventory {
    pub seeds: Vec<SeedStack>,
    pub capacity: u32,              // 容量（受工具房等级影响）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeedStack {
    pub crop_type_id: Uuid,
    pub name: String,
    pub seeds: Vec<Seed>,           // 同类种子按品质分组
    pub total_quantity: u32,
}

impl SeedInventory {
    /// 添加种子
    pub fn add_seed(&mut self, seed: Seed) -> Result<(), String> {
        if self.total_count() >= self.capacity {
            return Err("种子库存已满".to_string());
        }

        // 尝试合并到现有堆栈
        if let Some(stack) = self.seeds.iter_mut().find(|s| s.crop_type_id == seed.crop_type_id) {
            stack.seeds.push(seed.clone());
            stack.total_quantity += seed.quantity;
        } else {
            self.seeds.push(SeedStack {
                crop_type_id: seed.crop_type_id,
                name: seed.name.clone(),
                seeds: vec![seed.clone()],
                total_quantity: seed.quantity,
            });
        }

        Ok(())
    }

    /// 获取最佳品质种子
    pub fn get_best_seed(&self, crop_type_id: Uuid) -> Option<&Seed> {
        self.seeds
            .iter()
            .find(|s| s.crop_type_id == crop_type_id)
            .and_then(|stack| stack.seeds.iter().max_by_key(|s| s.quality))
    }

    fn total_count(&self) -> u32 {
        self.seeds.iter().map(|s| s.total_quantity).sum()
    }
}

/// 留种系统
pub struct SeedSavingSystem;

impl SeedSavingSystem {
    /// 尝试留种
    pub fn try_save_seed(
        harvest: &HarvestedCrop,
        gardening_level: u32,
        crop_type: &CropType,
    ) -> Result<Seed, SeedSaveError> {
        // 检查是否可留种
        if !crop_type.can_save_seed {
            return Err(SeedSaveError::CannotSaveSeed);
        }

        // 检查园艺等级
        if gardening_level < crop_type.save_seed_skill_level {
            return Err(SeedSaveError::SkillTooLow {
                required: crop_type.save_seed_skill_level,
                current: gardening_level,
            });
        }

        // 计算留种成功率和品质
        let success_rate = 0.5 + (gardening_level as f32 * 0.05);
        if rand::random::<f32>() > success_rate {
            return Err(SeedSaveError::Failed);
        }

        // 种子品质 = 收获品质 * (0.8~1.0)
        let seed_quality = (harvest.quality as f32 * (0.8 + rand::random::<f32>() * 0.2)).min(5.0) as u32;

        Ok(Seed {
            id: Uuid::new_v4(),
            crop_type_id: harvest.crop_type_id,
            name: format!("{}种子", harvest.name),
            quality: seed_quality,
            origin: SeedOrigin::SelfSaved {
                from_harvest_id: harvest.id,
                saved_at: Utc::now(),
            },
            can_save: true,
            quantity: 1,
            obtained_at: Utc::now(),
        })
    }
}

#[derive(Debug)]
pub enum SeedSaveError {
    CannotSaveSeed,
    SkillTooLow { required: u32, current: u32 },
    Failed,
}
```

#### 6.6.3.2 作物数据库定义

```sql
-- 作物类型表
CREATE TABLE crop_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    rarity TEXT NOT NULL,

    -- 生长参数
    sowing_hours INTEGER NOT NULL DEFAULT 6,
    germinating_hours INTEGER NOT NULL DEFAULT 12,
    growing_hours INTEGER NOT NULL DEFAULT 24,
    mature_hours INTEGER NOT NULL DEFAULT 48,
    withering_hours INTEGER NOT NULL DEFAULT 24,
    seasons TEXT NOT NULL,           -- JSON 数组
    ideal_temp_min INTEGER NOT NULL DEFAULT 15,
    ideal_temp_max INTEGER NOT NULL DEFAULT 30,
    water_need TEXT NOT NULL DEFAULT 'Medium',
    fertilizer_need TEXT NOT NULL DEFAULT 'Medium',

    -- 产量
    base_yield INTEGER NOT NULL DEFAULT 5,
    max_yield INTEGER NOT NULL DEFAULT 15,
    can_save_seed INTEGER NOT NULL DEFAULT 1,
    save_seed_skill_level INTEGER NOT NULL DEFAULT 4,

    -- 经济
    seed_price TEXT NOT NULL,
    sale_price TEXT NOT NULL,

    -- 用途
    uses TEXT NOT NULL,              -- JSON
    related_recipes TEXT,            -- JSON 数组
    related_events TEXT              -- JSON 数组
);

-- 初始作物数据
INSERT INTO crop_types (id, name, description, category, rarity,
    sowing_hours, germinating_hours, growing_hours, mature_hours, withering_hours,
    seasons, water_need, fertilizer_need, base_yield, max_yield, seed_price, sale_price, uses)
VALUES
-- 番茄
('tomato-001', '番茄', '红彤彤的番茄，是厨房的万能食材', 'Vegetable', 'Common',
    6, 12, 24, 48, 24,
    '["Spring","Summer","Autumn"]', 'Medium', 'Medium', 6, 12, '5.00', '3.00',
    '[{"Cooking":{"quality_bonus":0.1,"freshness_decay":0.15}},{"Gift":{"favor_bonus":5,"preferred_by":["李大爷"]}},{"Sale":{"price":"3.00"}}]'),

-- 薄荷
('mint-001', '薄荷', '清新的薄荷，泡茶调味两相宜', 'Herb', 'Common',
    8, 16, 36, 72, 36,
    '["Spring","Summer","Autumn"]', 'Low', 'Low', 4, 8, '8.00', '5.00',
    '[{"Cooking":{"quality_bonus":0.15,"freshness_decay":0.1}},{"Crafting":{"item_unlocks":["薄荷香皂","薄荷茶"]}}]'),

-- 茉莉
('jasmine-001', '茉莉', '芬芳的茉莉花，老街的记忆', 'Flower', 'Uncommon',
    12, 24, 72, 120, 48,
    '["Spring","Summer"]', 'Low', 'Low', 3, 6, '15.00', '10.00',
    '[{"Decoration":{"atmosphere_bonus":3,"duration_days":5}},{"Gift":{"favor_bonus":15,"preferred_by":["王奶奶"]}},{"Crafting":{"item_unlocks":["干花书签","茉莉花茶"]}}]'),

-- 祖父的玫瑰
('grandpa-rose-001', '祖父的玫瑰', '祖父生前最珍视的玫瑰，承载着深藏的记忆', 'Special', 'Legendary',
    24, 48, 120, 168, 72,
    '["Spring","Summer","Autumn"]', 'Medium', 'High', 2, 4, '0.00', '0.00',
    '[{"Decoration":{"atmosphere_bonus":8,"duration_days":7}},{"Gift":{"favor_bonus":30,"preferred_by":["王奶奶","张婶"]}}]');

-- 种子库存表
CREATE TABLE seed_inventory (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    crop_type_id TEXT NOT NULL,
    quality INTEGER NOT NULL DEFAULT 3,
    origin TEXT NOT NULL,            -- JSON
    quantity INTEGER NOT NULL DEFAULT 1,
    obtained_at TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (crop_type_id) REFERENCES crop_types(id)
);
```

#### 6.6.4 种植计算系统

```rust
/// 种植系统计算器
pub struct GardenCalculator;

impl GardenCalculator {
    /// 计算生长速度
    pub fn calculate_growth_speed(
        base_days: u32,
        soil_fertility: u32,
        backyard_level: u32,
        irrigation_level: u32,
        season_match: bool,
        has_greenhouse: bool,
    ) -> f32 {
        let base_speed = 1.0 / base_days as f32;

        // 肥力加成
        let fertility_mod = 1.0 + (soil_fertility as f32 - 50.0) / 100.0;

        // 后院等级加成
        let level_bonus = BackyardLevel::levels()
            .iter()
            .find(|l| l.level == backyard_level)
            .map(|l| 1.0 + l.growth_speed_bonus)
            .unwrap_or(1.0);

        // 灌溉加成
        let irrigation_mod = IrrigationSystem::level_benefits(irrigation_level).water_efficiency;

        // 季节不匹配惩罚
        let season_mod = if season_match { 1.0 } else { 0.5 };

        // 温室可消除季节惩罚
        let final_season_mod = if has_greenhouse { 1.0 } else { season_mod };

        base_speed * fertility_mod * level_bonus * irrigation_mod * final_season_mod
    }

    /// 计算产量
    pub fn calculate_yield(
        base_yield: u32,
        max_yield: u32,
        soil_fertility: u32,
        crop_health: u32,
        panpan_gardening_skill: u32,
    ) -> u32 {
        let fertility_factor = soil_fertility as f32 / 100.0;
        let health_factor = crop_health as f32 / 100.0;
        let skill_factor = 1.0 + panpan_gardening_skill as f32 * 0.05;

        let yield_range = max_yield - base_yield;
        let bonus = yield_range as f32 * fertility_factor * health_factor * skill_factor;

        (base_yield as f32 + bonus).min(max_yield as f32) as u32
    }

    /// 计算新鲜度衰减
    pub fn calculate_freshness_decay(
        current_freshness: f32,
        hours_elapsed: u32,
        in_refrigerator: bool,
        refrigerator_level: u32,
    ) -> f32 {
        let base_decay_rate = 0.02; // 每小时2%

        let decay_mod = if in_refrigerator {
            match refrigerator_level {
                1 => 0.8,
                2 => 0.5,
                3 => 0.3,
                4 => 0.2,
                _ => 1.0,
            }
        } else {
            1.0
        };

        let decay = base_decay_rate * decay_mod * hours_elapsed as f32;
        (current_freshness - decay).max(0.0)
    }
}
```

#### 6.6.5 后院管理操作

```rust
/// 后院状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackyardState {
    pub level: u32,
    pub plots: Vec<VegetablePlot>,
    pub irrigation: IrrigationSystem,
    pub tool_shed: Facility,
    pub compost_area: CompostArea,
    pub greenhouse: Option<Greenhouse>,
    pub seed_inventory: Vec<SeedStack>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeedStack {
    pub crop_type_id: Uuid,
    pub name: String,
    pub quantity: u32,
    pub source: SeedSource,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeedSource {
    Purchased,        // 购买
    SelfSaved,        // 自留种（需技能）
    TravelBrought,    // 旅行带回
    NeighborGift,     // 邻居赠送
    EventReward,      // 事件奖励
}

/// 后院操作
pub enum BackyardAction {
    Till { plot_id: Uuid },
    Plant { plot_id: Uuid, crop_type_id: Uuid },
    Water { plot_ids: Vec<Uuid> },
    Fertilize { plot_id: Uuid, fertilizer_type: FertilizerType },
    Harvest { plot_id: Uuid },
    RemoveWithered { plot_id: Uuid },
    BuildCompost,
    AddToCompost { material: CompostMaterial },
    CollectCompost,
    BuildGreenhouse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FertilizerType {
    Basic,      // 基础肥料（购买）
    Organic,    // 有机肥（堆肥区制作）
    Premium,    // 高级肥料（特殊事件获得）
}
```

#### 6.6.6 与盼盼系统的交互

```rust
/// 盼盼园艺技能模块（新增）
pub struct GardeningModule {
    pub level: u32,                 // 1-10
    pub condition: u32,             // 完好度 0-100
    pub experience: u32,
}

impl GardeningModule {
    /// 园艺技能效果
    pub fn gardening_effects(&self) -> GardeningEffects {
        GardeningEffects {
            planting_success_rate: 0.8 + self.level as f32 * 0.02,
            growth_speed_bonus: self.level as f32 * 0.02,
            yield_bonus: self.level as f32 * 0.05,
            can_save_seeds: self.level >= 5,
            pest_resistance: self.level as f32 * 0.03,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GardeningEffects {
    pub planting_success_rate: f32,  // 播种成功率
    pub growth_speed_bonus: f32,     // 生长速度加成
    pub yield_bonus: f32,            // 产量加成
    pub can_save_seeds: bool,        // 能否自留种
    pub pest_resistance: f32,        // 病虫害抵抗力
}

impl ShopSystem {
    /// 后院劳作能量消耗
    pub fn calculate_garden_energy_cost(action: &BackyardAction) -> u32 {
        match action {
            BackyardAction::Till { .. } => 10,
            BackyardAction::Plant { .. } => 5,
            BackyardAction::Water { plot_ids } => 3 * plot_ids.len() as u32,
            BackyardAction::Fertilize { .. } => 3,
            BackyardAction::Harvest { .. } => 5,
            BackyardAction::RemoveWithered { .. } => 3,
            _ => 0,
        }
    }
}
```

#### 6.6.7 病虫害系统

种植系统包含动态的病虫害机制，需要盼盼定期检查和处理。

```rust
/// 病虫害类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PestDisease {
    // 虫害
    Aphids,           // 蚜虫：吸食汁液，生长减缓
    Caterpillars,     // 毛虫：啃食叶片，健康度下降
    Mites,            // 螨虫：影响品质
    Whiteflies,       // 白粉虱：传播病害

    // 病害
    PowderyMildew,    // 白粉病：影响光合作用
    RootRot,          // 根腐病：过度浇水导致
    LeafSpot,         // 叶斑病：影响外观和健康
    Blight,           // 枯萎病：严重时可致死

    // 环境问题
    NutrientDeficiency, // 缺肥：生长缓慢
    Sunburn,            // 日灼：夏季强光
    Frostbite,          // 冻伤：冬季低温
}

/// 病虫害严重程度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Severity {
    None,           // 无
    Mild,           // 轻微（5% 影响）
    Moderate,       // 中等（15% 影响）
    Severe,         // 严重（30% 影响）
    Critical,       // 危急（50% 影响，可能死亡）
}

/// 作物健康状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropHealthStatus {
    pub current_issues: Vec<(PestDisease, Severity)>,
    pub last_check: DateTime<Utc>,
    pub treatment_history: Vec<TreatmentRecord>,
}

/// 治疗记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreatmentRecord {
    pub applied_at: DateTime<Utc>,
    pub treatment: Treatment,
    pub effectiveness: f32,
}

/// 治疗方法
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Treatment {
    // 物理方法
    ManualRemoval,      // 人工清除（耗能高）
    Pruning,            // 修剪病叶

    // 自然方法
    NeemOil,            // 印楝油（天然杀虫剂）
    SoapyWater,         // 肥皂水
    CompanionPlanting,  // 伴生种植（预防）

    // 化学方法（邻居提供）
    OrganicPesticide,   // 有机农药
    Fungicide,          // 杀菌剂

    // 环境调整
    AdjustWatering,     // 调整浇水
    ImproveDrainage,    // 改善排水
    AddNutrients,       // 补充营养
}

/// 病虫害管理器
pub struct PestDiseaseManager;

impl PestDiseaseManager {
    /// 检查作物健康（盼盼执行）
    pub fn check_crop_health(crop: &GrowingCrop, sensor_level: u32) -> CropHealthStatus {
        let detection_chance = 0.3 + sensor_level as f32 * 0.1;
        let mut detected_issues = Vec::new();

        // 根据作物状态随机检测问题
        if crop.health < 70 {
            if rand::random::<f32>() < detection_chance {
                detected_issues.push((PestDisease::NutrientDeficiency, Severity::Mild));
            }
        }

        // 检查是否有未处理的问题
        if let Some(existing) = &crop.health_status {
            for (issue, severity) in &existing.current_issues {
                // 问题可能恶化
                let worsening_chance = 0.1 * severity.multiplier();
                if rand::random::<f32>() < worsening_chance {
                    detected_issues.push((issue.clone(), severity.escalate()));
                } else {
                    detected_issues.push((issue.clone(), severity.clone()));
                }
            }
        }

        CropHealthStatus {
            current_issues: detected_issues,
            last_check: Utc::now(),
            treatment_history: crop.health_status.as_ref()
                .map(|h| h.treatment_history.clone())
                .unwrap_or_default(),
        }
    }

    /// 应用治疗
    pub fn apply_treatment(
        crop: &mut GrowingCrop,
        treatment: Treatment,
        gardening_level: u32,
    ) -> TreatmentResult {
        let base_effectiveness = treatment.base_effectiveness();
        let skill_bonus = gardening_level as f32 * 0.05;
        let final_effectiveness = (base_effectiveness + skill_bonus).min(0.95);

        // 移除或减轻问题
        crop.health_status.as_mut().map(|status| {
            status.current_issues.retain(|(_, severity)| {
                if rand::random::<f32>() < final_effectiveness {
                    // 治疗成功
                    false
                } else {
                    true
                }
            });
            status.treatment_history.push(TreatmentRecord {
                applied_at: Utc::now(),
                treatment: treatment.clone(),
                effectiveness: final_effectiveness,
            });
            status
        });

        // 恢复健康度
        crop.health = (crop.health + (final_effectiveness * 20.0) as u32).min(100);

        TreatmentResult {
            success: final_effectiveness > 0.5,
            effectiveness: final_effectiveness,
            energy_cost: treatment.energy_cost(),
        }
    }

    /// 随机发生病虫害（每日检查）
    pub fn random_pest_occurrence(
        crop: &GrowingCrop,
        season: Season,
        weather: Weather,
    ) -> Option<(PestDisease, Severity)> {
        // 基础发生概率
        let base_chance = 0.02;

        // 季节修正
        let season_mod = match season {
            Season::Spring => 1.2,  // 春季虫害多发
            Season::Summer => 1.5,  // 夏季虫害最严重
            Season::Autumn => 0.8,  // 秋季减少
            Season::Winter => 0.3,  // 冬季极少
        };

        // 天气修正
        let weather_mod = match weather {
            Weather::Rainy => 1.3,   // 雨天病害多发
            Weather::Stormy => 1.5,  // 暴风雨后更严重
            Weather::Sunny => 0.8,   // 晴天较少
            _ => 1.0,
        };

        let final_chance = base_chance * season_mod * weather_mod;

        if rand::random::<f32>() < final_chance {
            // 随机选择病虫害类型
            let pest = Self::random_pest_by_season(season);
            let severity = Self::random_severity();
            Some((pest, severity))
        } else {
            None
        }
    }

    fn random_pest_by_season(season: Season) -> PestDisease {
        match season {
            Season::Spring => {
                let options = [PestDisease::Aphids, PestDisease::Caterpillars, PestDisease::PowderyMildew];
                options[rand::random::<usize>() % options.len()].clone()
            }
            Season::Summer => {
                let options = [PestDisease::Mites, PestDisease::Whiteflies, PestDisease::Sunburn, PestDisease::Blight];
                options[rand::random::<usize>() % options.len()].clone()
            }
            Season::Autumn => {
                let options = [PestDisease::LeafSpot, PestDisease::RootRot];
                options[rand::random::<usize>() % options.len()].clone()
            }
            Season::Winter => {
                PestDisease::Frostbite
            }
        }
    }

    fn random_severity() -> Severity {
        let roll = rand::random::<f32>();
        if roll < 0.6 { Severity::Mild }
        else if roll < 0.85 { Severity::Moderate }
        else if roll < 0.95 { Severity::Severe }
        else { Severity::Critical }
    }
}

impl Severity {
    fn multiplier(&self) -> f32 {
        match self {
            Severity::None => 0.0,
            Severity::Mild => 1.0,
            Severity::Moderate => 1.5,
            Severity::Severe => 2.0,
            Severity::Critical => 3.0,
        }
    }

    fn escalate(&self) -> Self {
        match self {
            Severity::Mild => Severity::Moderate,
            Severity::Moderate => Severity::Severe,
            Severity::Severe => Severity::Critical,
            Severity::Critical => Severity::Critical,
            Severity::None => Severity::Mild,
        }
    }

    pub fn health_impact(&self) -> u32 {
        match self {
            Severity::None => 0,
            Severity::Mild => 5,
            Severity::Moderate => 15,
            Severity::Severe => 30,
            Severity::Critical => 50,
        }
    }

    pub fn growth_penalty(&self) -> f32 {
        match self {
            Severity::None => 0.0,
            Severity::Mild => 0.05,
            Severity::Moderate => 0.15,
            Severity::Severe => 0.30,
            Severity::Critical => 0.50,
        }
    }
}

impl Treatment {
    fn base_effectiveness(&self) -> f32 {
        match self {
            Treatment::ManualRemoval => 0.6,
            Treatment::Pruning => 0.5,
            Treatment::NeemOil => 0.75,
            Treatment::SoapyWater => 0.55,
            Treatment::CompanionPlanting => 0.4,  // 预防性质
            Treatment::OrganicPesticide => 0.85,
            Treatment::Fungicide => 0.9,
            Treatment::AdjustWatering => 0.7,
            Treatment::ImproveDrainage => 0.65,
            Treatment::AddNutrients => 0.8,
        }
    }

    fn energy_cost(&self) -> u32 {
        match self {
            Treatment::ManualRemoval => 15,
            Treatment::Pruning => 10,
            Treatment::NeemOil => 5,
            Treatment::SoapyWater => 3,
            Treatment::CompanionPlanting => 8,
            Treatment::OrganicPesticide => 5,
            Treatment::Fungicide => 5,
            Treatment::AdjustWatering => 2,
            Treatment::ImproveDrainage => 12,
            Treatment::AddNutrients => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreatmentResult {
    pub success: bool,
    pub effectiveness: f32,
    pub energy_cost: u32,
}
```

#### 6.6.8 天气与季节影响

```rust
/// 天气对种植的影响详情
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenWeatherEffect {
    pub growth_modifier: f32,       // 生长速度修正
    pub water_need_modifier: f32,   // 水分需求修正
    pub pest_risk_modifier: f32,    // 病虫害风险修正
    pub health_drain: u32,          // 健康度消耗
    pub auto_actions: Vec<AutoAction>, // 自动触发的操作
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AutoAction {
    SkipWatering,      // 跳过浇水（雨天）
    EmergencyCover,    // 紧急遮盖（暴风雨）
    MoveToGreenhouse,  // 移入温室（严寒）
    ExtraWatering,     // 额外浇水（高温）
}

impl Weather {
    /// 天气对种植的详细影响
    pub fn garden_effect(&self) -> GardenWeatherEffect {
        match self {
            Weather::Sunny => GardenWeatherEffect {
                growth_modifier: 1.1,
                water_need_modifier: 1.3,
                pest_risk_modifier: 0.8,
                health_drain: 0,
                auto_actions: vec![AutoAction::ExtraWatering],
            },
            Weather::Rainy => GardenWeatherEffect {
                growth_modifier: 0.95,
                water_need_modifier: 0.0,  // 不需要额外浇水
                pest_risk_modifier: 1.3,
                health_drain: 0,
                auto_actions: vec![AutoAction::SkipWatering],
            },
            Weather::Stormy => GardenWeatherEffect {
                growth_modifier: 0.7,
                water_need_modifier: 0.0,
                pest_risk_modifier: 1.5,
                health_drain: 10,
                auto_actions: vec![AutoAction::SkipWatering, AutoAction::EmergencyCover],
            },
            Weather::Snowy => GardenWeatherEffect {
                growth_modifier: 0.3,
                water_need_modifier: 0.5,
                pest_risk_modifier: 0.2,
                health_drain: 15,
                auto_actions: vec![AutoAction::MoveToGreenhouse],
            },
            _ => GardenWeatherEffect {
                growth_modifier: 1.0,
                water_need_modifier: 1.0,
                pest_risk_modifier: 1.0,
                health_drain: 0,
                auto_actions: vec![],
            },
        }
    }
}

/// 季节对种植的影响
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeasonEffect {
    pub growth_modifier: f32,
    pub available_crops: Vec<CropCategory>,
    pub pest_frequency: f32,
    pub special_events: Vec<String>,
}

impl Season {
    pub fn garden_effect(&self) -> SeasonEffect {
        match self {
            Season::Spring => SeasonEffect {
                growth_modifier: 1.1,
                available_crops: vec![CropCategory::Vegetable, CropCategory::Herb, CropCategory::Flower],
                pest_frequency: 1.2,
                special_events: vec!["春耕节".into(), "花朝节".into()],
            },
            Season::Summer => SeasonEffect {
                growth_modifier: 1.0,
                available_crops: vec![CropCategory::Vegetable, CropCategory::Herb, CropCategory::Flower, CropCategory::Exotic],
                pest_frequency: 1.5,
                special_events: vec!["夏日祭".into()],
            },
            Season::Autumn => SeasonEffect {
                growth_modifier: 1.2,  // 秋季最适合种植
                available_crops: vec![CropCategory::Vegetable, CropCategory::Herb],
                pest_frequency: 0.8,
                special_events: vec!["丰收节".into(), "中秋节".into()],
            },
            Season::Winter => SeasonEffect {
                growth_modifier: 0.5,
                available_crops: vec![],  // 需温室
                pest_frequency: 0.3,
                special_events: vec!["冬至".into()],
            },
        }
    }
}
```

#### 6.6.9 自动化种植系统

随着后院等级提升，可以解锁自动化功能。

```rust
/// 自动化种植系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenAutomation {
    pub auto_watering: bool,        // 自动浇水（灌溉系统3级）
    pub auto_fertilizing: bool,     // 自动施肥（需智能系统）
    pub auto_pest_check: bool,      // 自动虫害检测（传感器4级）
    pub auto_harvest: bool,         // 自动收获（需高级机器人）
    pub schedule: GardenSchedule,   // 种植计划
}

/// 种植计划
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenSchedule {
    pub daily_tasks: Vec<ScheduledTask>,
    pub crop_rotation: Vec<CropRotationEntry>,
    pub seasonal_plan: SeasonalPlan,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub task_type: ScheduledTaskType,
    pub preferred_time: u32,        // 小时 (0-23)
    pub auto_execute: bool,         // 是否自动执行
    pub last_executed: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScheduledTaskType {
    MorningCheck,      // 早间检查
    Watering,          // 浇水
    HealthInspection,  // 健康检查
    EveningHarvest,    // 傍晚收获
    WeeklyFertilize,   // 每周施肥
}

/// 轮作计划（保持土壤肥力）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropRotationEntry {
    pub plot_id: Uuid,
    pub crop_history: Vec<Uuid>,    // 历史种植的作物ID
    pub next_recommended: Option<CropCategory>,
}

/// 季度种植计划
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeasonalPlan {
    pub season: Season,
    pub planned_crops: Vec<PlannedCrop>,
    pub estimated_harvest: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlannedCrop {
    pub crop_type_id: Uuid,
    pub plot_assignment: Uuid,
    pub plant_date: DateTime<Utc>,
    pub expected_harvest: DateTime<Utc>,
}

/// 自动化效果
impl GardenAutomation {
    /// 检查是否可以自动执行某任务
    pub fn can_auto_execute(&self, task: &ScheduledTaskType, backyard_level: u32) -> bool {
        match task {
            ScheduledTaskType::Watering => {
                self.auto_watering && backyard_level >= 3
            }
            ScheduledTaskType::HealthInspection => {
                self.auto_pest_check && backyard_level >= 4
            }
            ScheduledTaskType::EveningHarvest => {
                self.auto_harvest && backyard_level >= 5
            }
            ScheduledTaskType::WeeklyFertilize => {
                self.auto_fertilizing && backyard_level >= 4
            }
            ScheduledTaskType::MorningCheck => {
                backyard_level >= 2
            }
        }
    }

    /// 计算自动化节省的能量
    pub fn energy_savings(&self, task: &ScheduledTaskType) -> u32 {
        match task {
            ScheduledTaskType::Watering if self.auto_watering => 5,
            ScheduledTaskType::HealthInspection if self.auto_pest_check => 8,
            ScheduledTaskType::EveningHarvest if self.auto_harvest => 5,
            ScheduledTaskType::WeeklyFertilize if self.auto_fertilizing => 3,
            _ => 0,
        }
    }

    /// 生成每日任务列表
    pub fn generate_daily_tasks(&self, backyard: &BackyardState) -> Vec<GardenTask> {
        let mut tasks = Vec::new();
        let now = Utc::now();

        // 检查每个菜地
        for plot in &backyard.plots {
            if let Some(crop) = &plot.current_crop {
                // 检查是否需要浇水
                if !crop.watered_today && !self.auto_watering {
                    tasks.push(GardenTask {
                        task_type: BackyardAction::Water { plot_ids: vec![plot.id] },
                        priority: TaskPriority::High,
                        deadline: now + Duration::hours(12),
                        energy_cost: 3,
                    });
                }

                // 检查是否可以收获
                if crop.current_stage == GrowthStage::Mature {
                    tasks.push(GardenTask {
                        task_type: BackyardAction::Harvest { plot_id: plot.id },
                        priority: TaskPriority::Medium,
                        deadline: now + Duration::hours(48),
                        energy_cost: 5,
                    });
                }

                // 检查是否枯萎
                if crop.current_stage == GrowthStage::Withering {
                    tasks.push(GardenTask {
                        task_type: BackyardAction::RemoveWithered { plot_id: plot.id },
                        priority: TaskPriority::High,
                        deadline: now + Duration::hours(24),
                        energy_cost: 3,
                    });
                }
            } else if plot.needs_tilling {
                // 需要翻土
                tasks.push(GardenTask {
                    task_type: BackyardAction::Till { plot_id: plot.id },
                    priority: TaskPriority::Low,
                    deadline: now + Duration::days(3),
                    energy_cost: 10,
                });
            }
        }

        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        tasks
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenTask {
    pub task_type: BackyardAction,
    pub priority: TaskPriority,
    pub deadline: DateTime<Utc>,
    pub energy_cost: u32,
}
```

#### 6.6.10 种植任务调度

盼盼的种植任务由 LLM 驱动，根据当前状态生成决策。

```rust
/// 种植系统任务调度器
pub struct GardenScheduler {
    automation: GardenAutomation,
    pending_tasks: VecDeque<GardenTask>,
    last_check: DateTime<Utc>,
}

impl GardenScheduler {
    /// 每小时更新（游戏时间）
    pub fn hourly_update(&mut self, backyard: &mut BackyardState, context: &PanpanContext) {
        let now = Utc::now();

        // 检查自动化任务
        for task in &self.automation.schedule.daily_tasks {
            if self.automation.can_auto_execute(&task.task_type, backyard.level) {
                if let Some(last) = task.last_executed {
                    if now - last >= Duration::hours(24) {
                        self.execute_auto_task(&task.task_type, backyard);
                    }
                }
            }
        }

        // 生成新任务
        let new_tasks = self.automation.generate_daily_tasks(backyard);
        for task in new_tasks {
            if !self.pending_tasks.iter().any(|t| t.task_type == task.task_type) {
                self.pending_tasks.push_back(task);
            }
        }

        // 更新作物生长
        self.update_crop_growth(backyard, context);

        self.last_check = now;
    }

    /// 执行自动化任务
    fn execute_auto_task(&self, task_type: &ScheduledTaskType, backyard: &mut BackyardState) {
        match task_type {
            ScheduledTaskType::Watering => {
                // 自动浇水所有需要浇水的菜地
                for plot in &mut backyard.plots {
                    if let Some(crop) = &mut plot.current_crop {
                        crop.watered_today = true;
                    }
                }
            }
            ScheduledTaskType::HealthInspection => {
                // 自动健康检查
                for plot in &mut backyard.plots {
                    if let Some(crop) = &mut plot.current_crop {
                        crop.health_status = Some(PestDiseaseManager::check_crop_health(
                            crop,
                            context.modules.sensor.level,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    /// 更新作物生长
    fn update_crop_growth(&self, backyard: &mut BackyardState, context: &PanpanContext) {
        let weather_effect = context.weather.garden_effect();
        let season_effect = context.season.garden_effect();

        for plot in &mut backyard.plots {
            if let Some(crop) = &mut plot.current_crop {
                // 计算生长进度
                let base_speed = GardenCalculator::calculate_growth_speed(
                    crop.crop_type.growth_days,
                    plot.fertility,
                    backyard.level,
                    backyard.irrigation.level,
                    crop.crop_type.seasons.contains(&context.season),
                    backyard.greenhouse.as_ref().map(|g| g.is_built).unwrap_or(false),
                );

                // 应用天气和季节修正
                let final_speed = base_speed
                    * weather_effect.growth_modifier
                    * season_effect.growth_modifier;

                // 更新生长阶段进度
                crop.stage_progress += final_speed / 24.0;  // 每小时进度

                // 检查阶段转换
                if crop.stage_progress >= 1.0 {
                    crop.stage_progress = 0.0;
                    crop.current_stage = crop.current_stage.next();
                }

                // 应用病虫害影响
                if let Some(status) = &crop.health_status {
                    for (_, severity) in &status.current_issues {
                        crop.health = crop.health.saturating_sub(severity.health_impact() / 24);
                    }
                }

                // 应用天气健康消耗
                crop.health = crop.health.saturating_sub(weather_effect.health_drain / 24);
            }
        }
    }

    /// LLM 决策：是否执行某项种植任务
    pub async fn decide_garden_action(
        &self,
        action: &BackyardAction,
        llm_manager: &LlmManager,
        context: &PanpanContext,
    ) -> GardenDecision {
        let prompt = format!(
            "当前后院状态：\n\
             - 后院等级：{}\n\
             - 活跃菜地：{} 块\n\
             - 待处理任务：{} 个\n\
             - 当前天气：{:?}\n\
             - 当前季节：{:?}\n\
             - 盼盼电量：{}%\n\
             - 盼盼心情：{:?}\n\n\
             拟执行操作：{:?}\n\n\
             请决定是否执行此操作，以及执行方式。",
            context.backyard.level,
            context.backyard.plots.iter().filter(|p| p.is_unlocked).count(),
            self.pending_tasks.len(),
            context.weather,
            context.season,
            context.energy.current,
            context.emotion,
            action,
        );

        // 调用 LLM 获取决策
        llm_manager.generate_garden_decision(&prompt).await
    }
}

impl GrowthStage {
    fn next(&self) -> Self {
        match self {
            GrowthStage::Sowing => GrowthStage::Germinating,
            GrowthStage::Germinating => GrowthStage::Growing,
            GrowthStage::Growing => GrowthStage::Mature,
            GrowthStage::Mature => GrowthStage::Withering,
            GrowthStage::Withering => GrowthStage::Withering,
        }
    }
}

/// 种植决策结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenDecision {
    pub should_execute: bool,
    pub priority: TaskPriority,
    pub modifications: Option<String>,
    pub response_to_player: String,
    pub mood_change: i32,
}
```

#### 6.6.11 种植数据存储

```sql
-- 种植相关数据库表

-- 作物定义表
CREATE TABLE crop_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    growth_days INTEGER NOT NULL,
    seasons TEXT NOT NULL,           -- JSON 数组
    base_yield INTEGER NOT NULL,
    max_yield INTEGER NOT NULL,
    seed_price TEXT NOT NULL,        -- Decimal as text
    uses TEXT NOT NULL,              -- JSON 数组
    description TEXT
);

-- 菜地表
CREATE TABLE vegetable_plots (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    plot_number INTEGER NOT NULL,
    is_unlocked INTEGER NOT NULL DEFAULT 0,
    soil_level INTEGER NOT NULL DEFAULT 1,
    fertility INTEGER NOT NULL DEFAULT 50,
    needs_tilling INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 生长中的作物表
CREATE TABLE growing_crops (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    plot_id TEXT NOT NULL,
    crop_type_id TEXT NOT NULL,
    planted_at TEXT NOT NULL,
    current_stage TEXT NOT NULL,
    stage_progress REAL NOT NULL DEFAULT 0.0,
    health INTEGER NOT NULL DEFAULT 100,
    watered_today INTEGER NOT NULL DEFAULT 0,
    fertilized_this_cycle INTEGER NOT NULL DEFAULT 0,
    health_status TEXT,              -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (plot_id) REFERENCES vegetable_plots(id),
    FOREIGN KEY (crop_type_id) REFERENCES crop_types(id)
);

-- 种子库存表
CREATE TABLE seed_inventory (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    crop_type_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (crop_type_id) REFERENCES crop_types(id)
);

-- 收获记录表
CREATE TABLE harvest_records (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    crop_type_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    quality INTEGER NOT NULL,
    harvested_at TEXT NOT NULL,
    plot_id TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 种植历史表（用于轮作）
CREATE TABLE planting_history (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    plot_id TEXT NOT NULL,
    crop_type_id TEXT NOT NULL,
    planted_at TEXT NOT NULL,
    harvested_at TEXT,
    yield_amount INTEGER,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 堆肥批次表
CREATE TABLE compost_batches (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ready_at TEXT NOT NULL,
    expected_yield INTEGER NOT NULL,
    materials TEXT NOT NULL,         -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 创建索引
CREATE INDEX idx_plots_save ON vegetable_plots(save_id);
CREATE INDEX idx_growing_crops_save ON growing_crops(save_id);
CREATE INDEX idx_growing_crops_plot ON growing_crops(plot_id);
CREATE INDEX idx_seeds_save ON seed_inventory(save_id);
CREATE INDEX idx_harvest_save ON harvest_records(save_id);
CREATE INDEX idx_planting_history_plot ON planting_history(plot_id);
```

#### 6.6.11.1 种植日志与报告系统

```rust
/// 种植日志条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenLogEntry {
    pub timestamp: DateTime<Utc>,
    pub entry_type: GardenLogType,
    pub plot_id: Option<Uuid>,
    pub crop_name: Option<String>,
    pub description: String,
    pub panpan_notes: Option<String>,  // 盼盼的备注
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GardenLogType {
    Planted,           // 播种
    Watered,           // 浇水
    Fertilized,        // 施肥
    Treated,           // 治疗
    Grew,              // 生长阶段变化
    Harvested,         // 收获
    Withered,          // 枯萎
    PestFound,         // 发现病虫害
    WeatherAlert,      // 天气预警
    Automation,        // 自动化操作
}

/// 种植报告生成器
pub struct GardenReportGenerator;

impl GardenReportGenerator {
    /// 生成每日种植简报
    pub fn generate_daily_report(
        backyard: &BackyardState,
        harvested_crops: &[HarvestedCrop],
        context: &PanpanContext,
    ) -> GardenDailyReport {
        let mut plot_reports = Vec::new();

        for (i, plot) in backyard.plots.iter().enumerate() {
            let report = if !plot.is_unlocked {
                PlotReport::Locked { plot_number: i + 1 }
            } else if let Some(crop) = &plot.current_crop {
                PlotReport::Growing {
                    plot_number: i + 1,
                    crop_name: crop.crop_type.name.clone(),
                    stage: crop.current_stage.clone(),
                    days_in_stage: crop.stage_progress,
                    health: crop.health,
                    watered: crop.watered_today,
                    needs_attention: crop.health < 70 || !crop.watered_today,
                }
            } else {
                PlotReport::Empty {
                    plot_number: i + 1,
                    fertility: plot.fertility,
                    needs_tilling: plot.needs_tilling,
                }
            };
            plot_reports.push(report);
        }

        // 生成建议
        let recommendations = Self::generate_recommendations(backyard, context);

        GardenDailyReport {
            date: Utc::now().date_naive(),
            plots: plot_reports,
            harvest_summary: Self::summarize_harvests(harvested_crops),
            seed_inventory: Self::summarize_seeds(&backyard.seed_inventory),
            recommendations,
            panpan_mood: context.emotion.clone(),
        }
    }

    /// 生成建议
    fn generate_recommendations(backyard: &BackyardState, context: &PanpanContext) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 检查即将成熟的作物
        for plot in &backyard.plots {
            if let Some(crop) = &plot.current_crop {
                if crop.current_stage == GrowthStage::Mature {
                    recommendations.push(format!(
                        "【紧急】菜地{}的{}已成熟，建议尽快收获！",
                        plot.plot_number, crop.crop_type.name
                    ));
                }
            }
        }

        // 检查季节适宜性
        for plot in &backyard.plots {
            if let Some(crop) = &plot.current_crop {
                if !crop.crop_type.seasons.contains(&context.season) && backyard.greenhouse.is_none() {
                    recommendations.push(format!(
                        "【注意】{}在当前季节生长缓慢，考虑建造温室",
                        crop.crop_type.name
                    ));
                }
            }
        }

        // 检查种子库存
        if backyard.seed_inventory.is_empty() {
            recommendations.push("种子库存不足，建议采购新种子".to_string());
        }

        // 检查肥力
        for plot in &backyard.plots {
            if plot.fertility < 30 && plot.is_unlocked {
                recommendations.push(format!(
                    "菜地{}肥力不足，建议施肥或休耕",
                    plot.plot_number
                ));
            }
        }

        recommendations
    }

    fn summarize_harvests(harvests: &[HarvestedCrop]) -> HarvestSummary {
        let mut by_type: HashMap<String, u32> = HashMap::new();
        let mut total_quality = 0u32;

        for harvest in harvests {
            *by_type.entry(harvest.name.clone()).or_insert(0) += harvest.quantity;
            total_quality += harvest.quality;
        }

        HarvestSummary {
            total_harvests: harvests.len() as u32,
            by_crop_type: by_type,
            average_quality: if !harvests.is_empty() {
                total_quality as f32 / harvests.len() as f32
            } else {
                0.0
            },
        }
    }

    fn summarize_seeds(seeds: &[SeedStack]) -> SeedSummary {
        let mut by_type: HashMap<String, u32> = HashMap::new();
        let mut rare_seeds = Vec::new();

        for stack in seeds {
            *by_type.entry(stack.name.clone()).or_insert(0) += stack.total_quantity;
            // 检查稀有种子
            if stack.seeds.iter().any(|s| matches!(s.origin, SeedOrigin::EventReward { .. } | SeedOrigin::Initial)) {
                rare_seeds.push(stack.name.clone());
            }
        }

        SeedSummary {
            total_varieties: seeds.len() as u32,
            by_crop_type: by_type,
            rare_seeds,
        }
    }
}

/// 菜地报告
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlotReport {
    Locked {
        plot_number: usize,
    },
    Empty {
        plot_number: usize,
        fertility: u32,
        needs_tilling: bool,
    },
    Growing {
        plot_number: usize,
        crop_name: String,
        stage: GrowthStage,
        days_in_stage: f32,
        health: u32,
        watered: bool,
        needs_attention: bool,
    },
}

/// 每日种植报告
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GardenDailyReport {
    pub date: NaiveDate,
    pub plots: Vec<PlotReport>,
    pub harvest_summary: HarvestSummary,
    pub seed_inventory: SeedSummary,
    pub recommendations: Vec<String>,
    pub panpan_mood: Emotion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarvestSummary {
    pub total_harvests: u32,
    pub by_crop_type: HashMap<String, u32>,
    pub average_quality: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeedSummary {
    pub total_varieties: u32,
    pub by_crop_type: HashMap<String, u32>,
    pub rare_seeds: Vec<String>,
}

/// 生成盼盼风格的报告文本
impl GardenDailyReport {
    pub fn to_panpan_message(&self, panpan_name: &str) -> String {
        let mut message = format!("【星夜小馆·后院种植报告】\n\n");

        // 菜地状态
        for plot in &self.plots {
            match plot {
                PlotReport::Locked { plot_number } => {
                    message.push_str(&format!("- 菜地{}：未解锁\n", plot_number));
                }
                PlotReport::Empty { plot_number, fertility, needs_tilling } => {
                    message.push_str(&format!(
                        "- 菜地{}：空闲（肥力{}%{}）\n",
                        plot_number,
                        fertility,
                        if *needs_tilling { "，需要翻土" } else { "" }
                    ));
                }
                PlotReport::Growing {
                    plot_number,
                    crop_name,
                    stage,
                    health,
                    watered,
                    needs_attention,
                    ..
                } => {
                    let stage_text = match stage {
                        GrowthStage::Sowing => "播种期",
                        GrowthStage::Germinating => "发芽期",
                        GrowthStage::Growing => "生长期",
                        GrowthStage::Mature => "成熟期【可收获】",
                        GrowthStage::Withering => "枯萎中",
                    };
                    let attention_mark = if *needs_attention { "⚠️" } else { "" };
                    let water_mark = if *watered { "" } else { "需浇水" };
                    message.push_str(&format!(
                        "- 菜地{}（{}）：{}，健康{}% {}{}\n",
                        plot_number,
                        crop_name,
                        stage_text,
                        health,
                        water_mark,
                        attention_mark
                    ));
                }
            }
        }

        // 收获摘要
        if self.harvest_summary.total_harvests > 0 {
            message.push_str("\n【今日收获】\n");
            for (crop, amount) in &self.harvest_summary.by_crop_type {
                message.push_str(&format!("- {} × {}\n", crop, amount));
            }
        }

        // 种子库存
        message.push_str("\n【种子库存】\n");
        for (seed, amount) in &self.seed_inventory.by_crop_type {
            message.push_str(&format!("- {}种子 × {}\n", seed, amount));
        }

        // 稀有种子提醒
        if !self.seed_inventory.rare_seeds.is_empty() {
            message.push_str("\n【珍贵种子】\n");
            for seed in &self.seed_inventory.rare_seeds {
                message.push_str(&format!("- {}（请妥善保管）\n", seed));
            }
        }

        // 建议
        if !self.recommendations.is_empty() {
            message.push_str("\n【建议】\n");
            for rec in &self.recommendations {
                message.push_str(&format!("• {}\n", rec));
            }
        }

        // 盼盼的结束语
        let closing = match self.panpan_mood {
            Emotion::Happy => "\n今天后院生机勃勃呢！🌸",
            Emotion::Tired => "\n照料菜地有点累，但看到作物成长很开心...💪",
            Emotion::Worried => "\n有些作物需要关注，请主人查看...🤔",
            _ => "\n我会继续照料后院的！🌱",
        };
        message.push_str(closing);

        message
    }
}
```

#### 6.6.11.2 收获后处理系统

```rust
/// 收获后处理系统
pub struct HarvestProcessor;

impl HarvestProcessor {
    /// 处理收获物
    pub fn process_harvest(
        harvest: HarvestedCrop,
        inventory: &mut CropInventory,
        kitchen: &mut KitchenState,
    ) -> HarvestProcessResult {
        let mut result = HarvestProcessResult::default();

        // 检查新鲜度，决定处理方式
        if harvest.freshness < 0.3 {
            // 新鲜度过低，只能堆肥
            result.went_to_compost = harvest.quantity;
            return result;
        }

        // 优先存入厨房库存（用于烹饪）
        let kitchen_capacity = kitchen.fresh_ingredient_capacity();
        let kitchen_used = kitchen.fresh_ingredients.values().sum::<u32>();

        if kitchen_used < kitchen_capacity {
            let to_kitchen = harvest.quantity.min(kitchen_capacity - kitchen_used);
            kitchen.add_fresh_ingredient(harvest.crop_type_id, to_kitchen, harvest.quality, harvest.freshness);
            result.went_to_kitchen = to_kitchen;

            if to_kitchen < harvest.quantity {
                // 剩余存入仓库
                let remaining = harvest.quantity - to_kitchen;
                inventory.add_crop(HarvestedCrop {
                    quantity: remaining,
                    ..harvest.clone()
                });
                result.went_to_storage = remaining;
            }
        } else {
            // 厨房已满，全部存入仓库
            inventory.add_crop(harvest.clone());
            result.went_to_storage = harvest.quantity;
        }

        result
    }

    /// 计算作物的最佳用途
    pub fn suggest_best_use(
        harvest: &HarvestedCrop,
        crop_type: &CropType,
        context: &GameContext,
    ) -> Vec<UseSuggestion> {
        let mut suggestions = Vec::new();

        // 烹饪建议
        if harvest.can_cook() {
            for recipe_id in &crop_type.related_recipes {
                if let Some(recipe) = context.recipes.get(recipe_id) {
                    if recipe.is_unlocked {
                        suggestions.push(UseSuggestion {
                            use_type: CropUseType::Cooking(recipe.name.clone()),
                            priority: 10 + harvest.quality as i32,
                            description: format!(
                                "用于【{}】，品质加成 +{}%",
                                recipe.name,
                                (harvest.current_quality() as f32 * 10.0) as u32
                            ),
                        });
                    }
                }
            }
        }

        // 赠送建议
        if harvest.can_gift() {
            for use in &crop_type.uses {
                if let CropUse::Gift { favor_bonus, preferred_by } = use {
                    for neighbor in preferred_by {
                        suggestions.push(UseSuggestion {
                            use_type: CropUseType::Gift(neighbor.clone()),
                            priority: 8 + *favor_bonus as i32 / 5,
                            description: format!(
                                "赠送给【{}】，好感度 +{}",
                                neighbor, favor_bonus
                            ),
                        });
                    }
                }
            }
        }

        // 装饰建议
        for use in &crop_type.uses {
            if let CropUse::Decoration { atmosphere_bonus, duration_days } = use {
                if harvest.freshness > 0.7 {
                    suggestions.push(UseSuggestion {
                        use_type: CropUseType::Decoration,
                        priority: 6,
                        description: format!(
                            "装饰餐厅，氛围 +{}，持续{}天",
                            atmosphere_bonus, duration_days
                        ),
                    });
                }
            }
        }

        // 售卖建议
        for use in &crop_type.uses {
            if let CropUse::Sale { price } = use {
                let final_price = *price * Decimal::from(harvest.quantity) * Decimal::from(harvest.quality);
                suggestions.push(UseSuggestion {
                    use_type: CropUseType::Sale,
                    priority: 3,
                    description: format!("在邻里市场出售，获得 ¥{}", final_price),
                });
            }
        }

        // 工坊制作建议
        for use in &crop_type.uses {
            if let CropUse::Crafting { item_unlocks } = use {
                for item in item_unlocks {
                    suggestions.push(UseSuggestion {
                        use_type: CropUseType::Crafting(item.clone()),
                        priority: 5,
                        description: format!("在工坊制作【{}】", item),
                    });
                }
            }
        }

        // 按优先级排序
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions
    }
}

#[derive(Default)]
pub struct HarvestProcessResult {
    pub went_to_kitchen: u32,
    pub went_to_storage: u32,
    pub went_to_compost: u32,
}

#[derive(Clone, Debug)]
pub struct UseSuggestion {
    pub use_type: CropUseType,
    pub priority: i32,
    pub description: String,
}

#[derive(Clone, Debug)]
pub enum CropUseType {
    Cooking(String),      // 菜谱名称
    Gift(String),         // 邻居名称
    Decoration,
    Sale,
    Crafting(String),     // 物品名称
}

/// 作物库存（仓库存储）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropInventory {
    pub items: Vec<InventoryCrop>,
    pub capacity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryCrop {
    pub crop: HarvestedCrop,
    pub stored_at: DateTime<Utc>,
}

impl CropInventory {
    /// 更新鲜度（每小时调用）
    pub fn update_freshness(&mut self, hours: u32) {
        for item in &mut self.items {
            item.crop.freshness = GardenCalculator::calculate_freshness_decay(
                item.crop.freshness,
                hours,
                false, // 仓库不在冰箱内
                0,
            );
        }

        // 移除完全腐烂的作物
        self.items.retain(|item| item.crop.freshness > 0.0);
    }

    /// 获取按类型分组的库存
    pub fn grouped_by_type(&self) -> HashMap<Uuid, Vec<&InventoryCrop>> {
        let mut grouped = HashMap::new();
        for item in &self.items {
            grouped
                .entry(item.crop.crop_type_id)
                .or_insert_with(Vec::new)
                .push(item);
        }
        grouped
    }

    /// 获取指定作物的最佳品质库存
    pub fn get_best_quality(&self, crop_type_id: Uuid) -> Option<&InventoryCrop> {
        self.items
            .iter()
            .filter(|item| item.crop.crop_type_id == crop_type_id)
            .max_by(|a, b| {
                let a_score = a.crop.quality as f32 * a.crop.freshness;
                let b_score = b.crop.quality as f32 * b.crop.freshness;
                a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
```

#### 6.6.11.3 与其他系统的联动

```rust
/// 种植系统与其他系统的联动管理
pub struct GardenIntegration;

impl GardenIntegration {
    /// 与厨房系统的联动
    pub fn kitchen_integration(
        fresh_ingredient: &HarvestedCrop,
        recipe: &Recipe,
    ) -> KitchenGardenBonus {
        // 新鲜度加成
        let freshness_bonus = if fresh_ingredient.freshness > 0.8 {
            0.15  // 极新鲜
        } else if fresh_ingredient.freshness > 0.6 {
            0.10  // 新鲜
        } else if fresh_ingredient.freshness > 0.4 {
            0.05  // 一般
        } else {
            0.0   // 不新鲜
        };

        // 自产加成（使用自己种植的食材）
        let homegrown_bonus = 0.05;

        // 品质加成
        let quality_bonus = (fresh_ingredient.quality - 3) as f32 * 0.03;

        KitchenGardenBonus {
            quality_bonus: freshness_bonus + homegrown_bonus + quality_bonus,
            special_description: if fresh_ingredient.quality >= 4 {
                Some(format!("使用了精心培育的{}", fresh_ingredient.name))
            } else {
                None
            },
        }
    }

    /// 与邻里系统的联动
    pub fn neighbor_integration(
        gift: &HarvestedCrop,
        neighbor: &Neighbor,
    ) -> NeighborGiftResult {
        // 检查邻居偏好
        let preference_bonus = neighbor.crop_preferences
            .iter()
            .find(|p| p.crop_type_id == gift.crop_type_id)
            .map(|p| p.favor_multiplier)
            .unwrap_or(1.0);

        // 计算好感度增加
        let base_favor = match gift.quality {
            5 => 20,
            4 => 15,
            3 => 10,
            2 => 5,
            _ => 2,
        };

        let final_favor = (base_favor as f32 * preference_bonus) as u32;

        // 检查是否触发特殊对话
        let special_dialogue = if gift.crop_type_id == neighbor.favorite_crop {
            Some(neighbor.favorite_crop_dialogue.clone())
        } else {
            None
        };

        NeighborGiftResult {
            favor_gained: final_favor,
            preference_bonus: preference_bonus > 1.0,
            special_dialogue,
            return_gift: if final_favor >= 15 {
                neighbor.generate_return_gift()
            } else {
                None
            },
        }
    }

    /// 与旅行系统的联动
    pub fn travel_integration(
        travel_destination: &TravelDestination,
    ) -> TravelSeedReward {
        // 根据目的地生成可能获得的种子
        let possible_seeds = match travel_destination.region {
            Region::Sichuan => vec![
                ("四川二荆条辣椒", CropRarity::Uncommon),
                ("汉源花椒", CropRarity::Rare),
            ],
            Region::Yunnan => vec![
                ("云南野生菌种", CropRarity::Rare),
                ("普洱茶树苗", CropRarity::Legendary),
            ],
            Region::Jiangnan => vec![
                ("江南水仙", CropRarity::Uncommon),
                ("苏州桂花", CropRarity::Rare),
            ],
            _ => vec![],
        };

        TravelSeedReward {
            possible_seeds,
            guaranteed_seeds: travel_destination.guaranteed_seed_rewards.clone(),
        }
    }

    /// 与记忆碎片系统的联动
    pub fn memory_integration(
        crop_type: &CropType,
        harvest: &HarvestedCrop,
    ) -> Option<MemoryTrigger> {
        // 特殊作物触发记忆
        for event in &crop_type.related_events {
            if event.starts_with("memory_") {
                return Some(MemoryTrigger {
                    memory_id: event[7..].to_string(), // 去掉 "memory_" 前缀
                    trigger_type: MemoryTriggerType::CropHarvest {
                        crop_name: crop_type.name.clone(),
                        quality: harvest.quality,
                    },
                    emotional_weight: if harvest.quality >= 4 { 1.2 } else { 1.0 },
                });
            }
        }

        // 祖父遗产作物特殊触发
        if crop_type.rarity == CropRarity::Legendary {
            return Some(MemoryTrigger {
                memory_id: format!("grandpa_{}", crop_type.id),
                trigger_type: MemoryTriggerType::SpecialCrop {
                    crop_name: crop_type.name.clone(),
                },
                emotional_weight: 1.5,
            });
        }

        None
    }

    /// 与工坊系统的联动
    pub fn workshop_integration(
        crop: &HarvestedCrop,
        crop_type: &CropType,
    ) -> Vec<WorkshopRecipe> {
        let mut recipes = Vec::new();

        for use in &crop_type.uses {
            if let CropUse::Crafting { item_unlocks } = use {
                for item_name in item_unlocks {
                    // 检查是否满足品质要求
                    let quality_ok = match item_name.as_str() {
                        "干花书签" => crop.quality >= 3,
                        "薄荷香皂" => crop.quality >= 2,
                        "茉莉花茶" => crop.quality >= 4,
                        _ => true,
                    };

                    if quality_ok {
                        recipes.push(WorkshopRecipe {
                            name: item_name.clone(),
                            required_crop: crop_type.name.clone(),
                            required_quality: crop.quality,
                            craft_time_hours: match item_name.as_str() {
                                "干花书签" => 24,
                                "薄荷香皂" => 48,
                                "茉莉花茶" => 12,
                                _ => 24,
                            },
                        });
                    }
                }
            }
        }

        recipes
    }
}

#[derive(Clone, Debug)]
pub struct KitchenGardenBonus {
    pub quality_bonus: f32,
    pub special_description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NeighborGiftResult {
    pub favor_gained: u32,
    pub preference_bonus: bool,
    pub special_dialogue: Option<String>,
    pub return_gift: Option<GiftItem>,
}

#[derive(Clone, Debug)]
pub struct TravelSeedReward {
    pub possible_seeds: Vec<(&'static str, CropRarity)>,
    pub guaranteed_seeds: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MemoryTrigger {
    pub memory_id: String,
    pub trigger_type: MemoryTriggerType,
    pub emotional_weight: f32,
}

#[derive(Clone, Debug)]
pub enum MemoryTriggerType {
    CropHarvest { crop_name: String, quality: u32 },
    SpecialCrop { crop_name: String },
}

#[derive(Clone, Debug)]
pub struct WorkshopRecipe {
    pub name: String,
    pub required_crop: String,
    pub required_quality: u32,
    pub craft_time_hours: u32,
}
```

#### 6.6.12 种植相关 API

```
# 后院种植系统 API

## 获取后院状态
GET /api/v1/saves/:id/backyard
Response: BackyardState

## 更新后院等级
PATCH /api/v1/saves/:id/backyard/level
Body: { "new_level": 3 }
Response: BackyardState

## 菜地操作

# 获取菜地列表
GET /api/v1/saves/:id/backyard/plots
Response: Vec<VegetablePlot>

# 翻土
POST /api/v1/saves/:id/backyard/plots/:plot_id/till
Response: { "success": true, "energy_cost": 10 }

# 播种
POST /api/v1/saves/:id/backyard/plots/:plot_id/plant
Body: { "crop_type_id": "uuid", "seed_source": "inventory" }
Response: { "success": true, "energy_cost": 5, "estimated_harvest": "2024-..." }

# 浇水
POST /api/v1/saves/:id/backyard/plots/water
Body: { "plot_ids": ["uuid1", "uuid2"] }
Response: { "success": true, "energy_cost": 6, "watered_count": 2 }

# 施肥
POST /api/v1/saves/:id/backyard/plots/:plot_id/fertilize
Body: { "fertilizer_type": "organic" }
Response: { "success": true, "energy_cost": 3, "fertility_bonus": 20 }

# 收获
POST /api/v1/saves/:id/backyard/plots/:plot_id/harvest
Response: {
    "success": true,
    "crop_name": "小白菜",
    "quantity": 8,
    "quality": 4,
    "energy_cost": 5
}

# 清除枯萎作物
DELETE /api/v1/saves/:id/backyard/plots/:plot_id/crop
Response: { "success": true, "energy_cost": 3 }

## 病虫害管理

# 检查作物健康
GET /api/v1/saves/:id/backyard/plots/:plot_id/health
Response: CropHealthStatus

# 治疗作物
POST /api/v1/saves/:id/backyard/plots/:plot_id/treat
Body: { "treatment": "neem_oil" }
Response: TreatmentResult

## 种子管理

# 获取种子库存
GET /api/v1/saves/:id/backyard/seeds
Response: Vec<SeedStack>

# 购买种子
POST /api/v1/saves/:id/backyard/seeds/purchase
Body: { "crop_type_id": "uuid", "quantity": 5 }
Response: { "success": true, "total_cost": "15.00" }

# 自留种（需要园艺技能5级）
POST /api/v1/saves/:id/backyard/seeds/save
Body: { "from_harvest_id": "uuid" }
Response: { "success": true, "seeds_obtained": 3 }

## 堆肥系统

# 获取堆肥状态
GET /api/v1/saves/:id/backyard/compost
Response: CompostArea

# 添加堆肥材料
POST /api/v1/saves/:id/backyard/compost/add
Body: { "material_type": "vegetable_scraps", "quantity": 3 }
Response: { "success": true, "current_fill": 45 }

# 收集完成的堆肥
POST /api/v1/saves/:id/backyard/compost/collect
Response: { "success": true, "fertilizer_obtained": 5 }

## 温室系统

# 获取温室状态
GET /api/v1/saves/:id/backyard/greenhouse
Response: Option<Greenhouse>

# 建造温室
POST /api/v1/saves/:id/backyard/greenhouse/build
Response: { "success": true, "cost": "500.00" }

## 自动化设置

# 获取自动化配置
GET /api/v1/saves/:id/backyard/automation
Response: GardenAutomation

# 更新自动化配置
PATCH /api/v1/saves/:id/backyard/automation
Body: { "auto_watering": true }
Response: GardenAutomation

# 设置种植计划
POST /api/v1/saves/:id/backyard/schedule
Body: GardenSchedule
Response: { "success": true }

## 种植任务

# 获取待处理任务
GET /api/v1/saves/:id/backyard/tasks
Response: Vec<GardenTask>

# 执行任务
POST /api/v1/saves/:id/backyard/tasks/execute
Body: { "task_id": "uuid" }
Response: { "success": true, "energy_cost": 5 }

## 统计数据

# 获取种植统计
GET /api/v1/saves/:id/backyard/statistics
Response: {
    "total_harvests": 45,
    "total_crops_grown": 280,
    "most_grown_crop": "小白菜",
    "average_yield": 7.2,
    "successful_harvest_rate": 0.85
}

# 获取种植历史
GET /api/v1/saves/:id/backyard/history
Query: ?limit=30&offset=0
Response: Vec<PlantingHistoryEntry>
```

### 6.7 工坊系统

工坊是星夜小馆的"百宝箱"，位于后院旁的工具房里。这里存放着各种工具和材料，盼盼可以在这里制作小物件、维修设备、甚至研发新的工具。

#### 6.7.1 工坊等级系统

```rust
/// 工坊等级
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkshopLevel {
    pub level: u32,                 // 1-5
    pub name: String,
    pub crafting_speed_bonus: f32,  // 制作速度加成
    pub success_rate_bonus: f32,    // 成功率加成
    pub unlocked_recipes: Vec<String>,
}

impl WorkshopLevel {
    pub fn levels() -> Vec<Self> {
        vec![
            Self {
                level: 1,
                name: "杂物小间".into(),
                crafting_speed_bonus: 0.0,
                success_rate_bonus: 0.0,
                unlocked_recipes: vec!["basic_items".into()],
            },
            Self {
                level: 2,
                name: "工具小屋".into(),
                crafting_speed_bonus: 0.15,
                success_rate_bonus: 0.05,
                unlocked_recipes: vec!["basic_items".into(), "medium_items".into()],
            },
            Self {
                level: 3,
                name: "手艺工坊".into(),
                crafting_speed_bonus: 0.25,
                success_rate_bonus: 0.1,
                unlocked_recipes: vec!["basic_items".into(), "medium_items".into(), "advanced_items".into()],
            },
            Self {
                level: 4,
                name: "创意工间".into(),
                crafting_speed_bonus: 0.35,
                success_rate_bonus: 0.15,
                unlocked_recipes: vec!["basic_items".into(), "medium_items".into(), "advanced_items".into(), "special_decorations".into()],
            },
            Self {
                level: 5,
                name: "星夜工坊".into(),
                crafting_speed_bonus: 0.5,
                success_rate_bonus: 0.2,
                unlocked_recipes: vec!["basic_items".into(), "medium_items".into(), "advanced_items".into(), "special_decorations".into(), "hidden_recipes".into()],
            },
        ]
    }
}
```

#### 6.7.2 工坊子设施

```rust
/// 工坊子设施类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkshopFacilityType {
    Workbench,       // 工作台
    MaterialRack,    // 材料架
    ToolWall,        // 工具墙
    RepairZone,      // 维修区
    PowerLighting,   // 电源与照明
}

/// 工作台
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workbench {
    pub level: u32,                 // 1-5
    pub condition: u32,             // 健康度 0-100
    pub has_lighting: bool,         // 是否有台灯
    pub has_measuring_tools: bool,  // 是否有测量工具
    pub is_smart: bool,             // 是否智能升降
}

impl Workbench {
    /// 工作台效果
    pub fn effects(&self) -> WorkbenchEffects {
        WorkbenchEffects {
            crafting_speed_mod: 1.0 + self.level as f32 * 0.1,
            complexity_cap: self.level * 2,  // 可制作物品复杂度上限
            precision_bonus: if self.has_measuring_tools { 0.1 } else { 0.0 },
        }
    }

    /// 健康度影响
    pub fn condition_penalty(&self) -> CraftingPenalty {
        if self.condition < 50 {
            CraftingPenalty {
                time_penalty: 0.3,      // 制作时间+30%
                failure_rate_bonus: 0.15, // 失败率+15%
            }
        } else {
            CraftingPenalty::default()
        }
    }
}

/// 材料架
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialRack {
    pub level: u32,                 // 1-4
    pub condition: u32,
    pub total_capacity: u32,        // 总容量
    pub categories: Vec<MaterialCategory>,
    pub has_moisture_protection: bool, // 防潮功能
}

impl MaterialRack {
    pub fn calculate_capacity(level: u32) -> u32 {
        match level {
            1 => 50,
            2 => 100,
            3 => 200,
            4 => 300,
            _ => 50,
        }
    }
}

/// 工具墙
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolWall {
    pub level: u32,                 // 1-4
    pub condition: u32,
    pub has_pegboard: bool,         // 洞洞板
    pub has_power_tools_zone: bool, // 电动工具区
    pub tool_access_speed: f32,     // 取用速度系数
}

impl ToolWall {
    pub fn calculate_tool_speed(&self) -> f32 {
        let base = 1.0;
        let pegboard_bonus = if self.has_pegboard { 0.15 } else { 0.0 };
        let power_bonus = if self.has_power_tools_zone { 0.2 } else { 0.0 };
        base + self.level as f32 * 0.05 + pegboard_bonus + power_bonus
    }
}

/// 维修区
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepairZone {
    pub level: u32,                 // 0-3（0=无）
    pub has_lifting_platform: bool, // 升降台
    pub max_equipment_weight: u32,  // 最大设备重量(kg)
}

/// 电源与照明
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerLighting {
    pub level: u32,                 // 1-3
    pub socket_count: u32,
    pub lighting_quality: f32,      // 0-1
    pub can_use_power_tools: bool,
}
```

#### 6.7.3 制作系统

```rust
/// 可制作物品分类
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CraftableCategory {
    DailyConsumable,   // 日常消耗品（筷子、杯垫、抹布）
    RepairTool,        // 维修工具（把手、螺丝配件）
    Decoration,        // 装饰物品（干花书签、木雕小摆件）
    Gift,              // 礼品（手工香皂、小盆栽）
    SpecialItem,       // 特殊物品（祖父相框、星夜招牌灯）
    KitchenTool,       // 厨具（新锅柄、砧板）
    GardenTool,        // 园艺工具（花架、温室零件）
}

/// 制作配方
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingRecipe {
    pub id: Uuid,
    pub name: String,
    pub category: CraftableCategory,
    pub complexity: u32,            // 复杂度 1-10
    pub required_workbench_level: u32,
    pub required_materials: Vec<MaterialRequirement>,
    pub base_crafting_time_minutes: u32,
    pub base_success_rate: f32,
    pub output: CraftingOutput,
    pub unlock_method: RecipeUnlockMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialRequirement {
    pub material_type: MaterialType,
    pub quantity: u32,
    pub quality_requirement: Option<u32>, // 品质要求
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingOutput {
    pub item_id: Uuid,
    pub item_name: String,
    pub quantity: u32,
    pub quality_range: (u32, u32),  // 品质范围
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeUnlockMethod {
    Initial,              // 初始自带
    WorkshopLevel(u32),   // 工坊升级解锁
    NeighborEvent(String),// 邻居事件
    TravelBrought(String),// 旅行带回
    MemoryFragment(Uuid), // 记忆碎片
    SkillUnlock(u32),     // 技能等级解锁
}

/// 制作队列
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingQueue {
    pub slots: Vec<Option<CraftingSlot>>,
    pub max_slots: u32,             // 受工作台等级影响
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingSlot {
    pub recipe_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expected_complete: DateTime<Utc>,
    pub progress: f32,
    pub materials_consumed: bool,
    pub focus_mode: bool,           // 专注模式（盼盼不执行其他任务）
}

/// 制作系统计算器
pub struct CraftingCalculator;

impl CraftingCalculator {
    /// 计算制作时间
    pub fn calculate_crafting_time(
        base_time_minutes: u32,
        workbench_level: u32,
        tool_wall_level: u32,
        panpan_crafting_skill: u32,
        workshop_level: u32,
    ) -> u32 {
        let base = base_time_minutes as f32;

        // 工作台加速
        let workbench_mod = 1.0 - workbench_level as f32 * 0.1;

        // 工具墙加速
        let tool_mod = 1.0 - tool_wall_level as f32 * 0.05;

        // 技能加速
        let skill_mod = 1.0 - panpan_crafting_skill as f32 * 0.03;

        // 工坊等级加速
        let level_bonus = WorkshopLevel::levels()
            .iter()
            .find(|l| l.level == workshop_level)
            .map(|l| 1.0 - l.crafting_speed_bonus)
            .unwrap_or(1.0);

        let result = base * workbench_mod * tool_mod * skill_mod * level_bonus;
        result.max(1.0) as u32
    }

    /// 计算成功率
    pub fn calculate_success_rate(
        base_rate: f32,
        workbench_condition: u32,
        tool_wall_level: u32,
        panpan_crafting_skill: u32,
        panpan_emotion: &Emotion,
        workshop_level: u32,
    ) -> f32 {
        let condition_penalty = if workbench_condition < 50 { -0.15 } else { 0.0 };
        let tool_bonus = tool_wall_level as f32 * 0.03;
        let skill_bonus = panpan_crafting_skill as f32 * 0.02;

        let emotion_mod = match panpan_emotion {
            Emotion::Happy => 0.05,
            Emotion::Tired => -0.1,
            Emotion::Confused => -0.05,
            _ => 0.0,
        };

        let level_bonus = WorkshopLevel::levels()
            .iter()
            .find(|l| l.level == workshop_level)
            .map(|l| l.success_rate_bonus)
            .unwrap_or(0.0);

        (base_rate + condition_penalty + tool_bonus + skill_bonus + emotion_mod + level_bonus).clamp(0.1, 0.99)
    }

    /// 计算制作品质
    pub fn calculate_output_quality(
        base_range: (u32, u32),
        material_quality: f32,
        panpan_crafting_skill: u32,
        workbench_level: u32,
    ) -> u32 {
        let skill_factor = panpan_crafting_skill as f32 / 10.0;
        let material_factor = material_quality;
        let workbench_factor = workbench_level as f32 / 5.0;

        let quality_bonus = (skill_factor + material_factor + workbench_factor) / 3.0;

        let min = base_range.0 as f32;
        let max = base_range.1 as f32;
        let range = max - min;

        (min + range * quality_bonus).clamp(min, max) as u32
    }
}
```

#### 6.7.4 材料管理

```rust
/// 材料类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialType {
    // 基础材料
    Wood,            // 木材
    Metal,           // 金属件
    Fabric,          // 布料
    Plastic,         // 塑料
    Glass,           // 玻璃
    Ceramic,         // 陶瓷

    // 天然材料
    Flower,          // 花卉（后院）
    Herb,            // 香料（后院）
    Seed,            // 种子

    // 特殊材料
    SpecialWood,     // 特殊木材（旅行带回）
    ExoticFabric,    // 异国布料（旅行带回）
    RareStone,       // 稀有石头（旅行带回）
    MemoryItem,      // 记忆物品（祖父遗物）
}

/// 材料来源
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialSource {
    Purchased,       // 采购
    BackyardHarvest, // 后院收获
    KitchenLeftover, // 厨房边角料
    NeighborGift,    // 邻居赠送
    TravelBrought,   // 旅行带回
    Disassembly,     // 拆解回收
}

/// 材料库存
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialInventory {
    pub materials: HashMap<MaterialType, MaterialStack>,
    pub total_capacity: u32,
    pub used_capacity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialStack {
    pub material_type: MaterialType,
    pub quantity: u32,
    pub quality: f32,               // 平均品质
    pub freshness: Option<f32>,     // 新鲜度（天然材料）
    pub source: MaterialSource,
    pub acquired_at: DateTime<Utc>,
}

/// 材料管理系统
impl MaterialInventory {
    /// 检查材料是否充足
    pub fn has_sufficient_materials(&self, requirements: &[MaterialRequirement]) -> bool {
        requirements.iter().all(|req| {
            self.materials.get(&req.material_type)
                .map(|stack| stack.quantity >= req.quantity)
                .unwrap_or(false)
        })
    }

    /// 消耗材料
    pub fn consume_materials(&mut self, requirements: &[MaterialRequirement]) -> Result<()> {
        // 先检查是否充足
        if !self.has_sufficient_materials(requirements) {
            return Err(Error::InsufficientMaterials);
        }

        // 消耗材料
        for req in requirements {
            if let Some(stack) = self.materials.get_mut(&req.material_type) {
                stack.quantity -= req.quantity;
                self.used_capacity -= req.quantity;
            }
        }

        Ok(())
    }

    /// 添加材料
    pub fn add_material(&mut self, material: MaterialStack) -> Result<()> {
        let quantity = material.quantity;
        if self.used_capacity + quantity > self.total_capacity {
            return Err(Error::InsufficientCapacity);
        }

        self.materials
            .entry(material.material_type.clone())
            .and_modify(|existing| {
                existing.quantity += material.quantity;
            })
            .or_insert(material);

        self.used_capacity += quantity;
        Ok(())
    }
}
```

#### 6.7.5 维修功能

工坊也是盼盼维修小馆设备的主要场所。

```rust
/// 设备维修请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepairRequest {
    pub facility_id: Uuid,
    pub facility_type: FacilityType,
    pub current_condition: u32,
    pub target_condition: u32,
    pub estimated_time_minutes: u32,
    pub required_materials: Vec<MaterialRequirement>,
    pub success_rate: f32,
    pub panpan_can_perform: bool,   // 盼盼是否能独立完成
    pub needs_external_help: bool,  // 是否需要外部帮助
}

/// 维修系统
pub struct RepairSystem;

impl RepairSystem {
    /// 评估维修需求
    pub fn assess_repair_needs(
        facility: &SubFacility,
        repair_zone_level: u32,
        panpan_repair_skill: u32,
    ) -> RepairRequest {
        let damage_level = 100 - facility.condition;
        let can_repair = damage_level as u32 <= panpan_repair_skill * 10 + repair_zone_level * 20;

        RepairRequest {
            facility_id: facility.id.clone(),
            facility_type: facility.facility_type.clone(),
            current_condition: facility.condition,
            target_condition: 100,
            estimated_time_minutes: Self::calculate_repair_time(damage_level, panpan_repair_skill),
            required_materials: Self::calculate_materials_needed(&facility.facility_type, damage_level),
            success_rate: Self::calculate_repair_success_rate(damage_level, panpan_repair_skill, repair_zone_level),
            panpan_can_perform: can_repair && repair_zone_level > 0,
            needs_external_help: !can_repair,
        }
    }

    /// 执行维修
    pub fn perform_repair(
        facility: &mut SubFacility,
        materials: &mut MaterialInventory,
        panpan_repair_skill: u32,
    ) -> RepairResult {
        let success_rate = Self::calculate_repair_success_rate(
            100 - facility.condition,
            panpan_repair_skill,
            1, // repair_zone_level
        );

        if rand::random::<f32>() < success_rate {
            facility.condition = 100;
            RepairResult::Success { restored_condition: 100 }
        } else {
            // 失败时部分恢复
            let partial_restore = 10 + panpan_repair_skill as u32 * 2;
            facility.condition = (facility.condition + partial_restore).min(100);
            RepairResult::PartialSuccess { restored_condition: facility.condition }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RepairResult {
    Success { restored_condition: u32 },
    PartialSuccess { restored_condition: u32 },
    Failed { reason: String },
}
```

#### 6.7.6 与盼盼系统的交互

```rust
impl ShopSystem {
    /// 工坊制作能量消耗
    pub fn calculate_crafting_energy_cost(recipe: &CraftingRecipe) -> u32 {
        match recipe.category {
            CraftableCategory::DailyConsumable => 5,
            CraftableCategory::RepairTool => 8,
            CraftableCategory::Decoration => 10,
            CraftableCategory::Gift => 8,
            CraftableCategory::SpecialItem => 15,
            CraftableCategory::KitchenTool => 12,
            CraftableCategory::GardenTool => 10,
        }
    }

    /// 维修能量消耗
    pub fn calculate_repair_energy_cost(damage_level: u32) -> u32 {
        10 + damage_level / 10
    }
}

/// 盼盼维修模块效果扩展
impl MobilityModule {
    /// 维修能力
    pub fn repair_capabilities(&self) -> RepairCapabilities {
        RepairCapabilities {
            can_repair_basic: self.level >= 1,
            can_repair_complex: self.level >= 3,
            can_craft_parts: self.level >= 5,
            repair_speed_bonus: self.level as f32 * 0.1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RepairCapabilities {
    pub can_repair_basic: bool,
    pub can_repair_complex: bool,
    pub can_craft_parts: bool,
    pub repair_speed_bonus: f32,
}
```

### 6.8 菜品体系

#### 6.5.1 菜谱分类

```rust
/// 菜谱来源
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeSource {
    Inherited,        // 祖父传承（初期部分损坏，需修复）
    TravelDiscovered, // 旅行发现（需实验）
    Innovative,       // 创新改良（需高创新倾向触发）
}

/// 菜谱状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeStatus {
    Damaged,    // 损坏（需修复）
    Vague,      // 模糊（需实验）
    Precise,    // 精确（可制作）
    Mastered,   // 掌握（品质提升）
}

/// 菜谱
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub id: Uuid,
    pub name: String,
    pub cuisine_type: String,        // 菜系
    pub source: RecipeSource,
    pub status: RecipeStatus,
    pub ingredients: Vec<IngredientUsage>,
    pub cooking_time_minutes: u32,
    pub price: Decimal,
    pub cost: Decimal,
    pub base_quality: u32,           // 1-5
    pub is_on_menu: bool,
}
```

#### 6.5.2 菜单系统

```rust
/// 每日菜单
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyMenu {
    pub date: Date<Utc>,
    pub available_recipes: Vec<Uuid>,
    pub daily_specials: Vec<Uuid>,   // 今日推荐（1-3道）
}
```

#### 6.5.3 研发树

```rust
/// 研发线索
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchClue {
    pub id: Uuid,
    pub cuisine_type: String,
    pub discovered: bool,
    pub discovered_at: Option<DateTime<Utc>>,
    pub travel_destination: Option<String>,  // 关联的旅行目的地
    pub unlocked_recipes: Vec<Uuid>,
    pub prerequisites: Vec<Uuid>,    // 前置线索
}
```

### 6.9 口碑指数系统

口碑是小馆的综合声誉，由多个子项加权计算得出，影响客流量和新顾客转化。

```rust
/// 口碑系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationSystem {
    pub food_score: u32,             // 菜品口碑 0-100
    pub service_score: u32,          // 服务口碑 0-100
    pub environment_score: u32,      // 环境口碑 0-100
    pub neighborhood_score: u32,     // 邻里口碑 0-100
    pub loyalty_score: u32,          // 老顾客情感 0-100
}

impl ReputationSystem {
    /// 计算综合口碑指数
    ///
    /// 口碑 = 菜品×40% + 服务×20% + 环境×15% + 邻里×15% + 老顾客×10%
    pub fn calculate_total(&self) -> f32 {
        let weights = ReputationWeights::default();
        (self.food_score as f32 * weights.food
            + self.service_score as f32 * weights.service
            + self.environment_score as f32 * weights.environment
            + self.neighborhood_score as f32 * weights.neighborhood
            + self.loyalty_score as f32 * weights.loyalty)
    }

    /// 口碑等级（每10分一档）
    pub fn reputation_level(&self) -> u32 {
        (self.calculate_total() / 10.0) as u32
    }

    /// 客流量系数
    pub fn customer_modifier(&self) -> f32 {
        let total = self.calculate_total();
        if total >= 80.0 { 1.2 }
        else if total >= 60.0 { 1.0 }
        else if total >= 40.0 { 0.8 }
        else { 0.6 }
    }

    /// 初始状态
    pub fn initial() -> Self {
        Self {
            food_score: 30,
            service_score: 20,
            environment_score: 15,
            neighborhood_score: 20,
            loyalty_score: 10,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReputationWeights {
    pub food: f32,          // 40%
    pub service: f32,       // 20%
    pub environment: f32,   // 15%
    pub neighborhood: f32,  // 15%
    pub loyalty: f32,       // 10%
}

impl Default for ReputationWeights {
    fn default() -> Self {
        Self {
            food: 0.40,
            service: 0.20,
            environment: 0.15,
            neighborhood: 0.15,
            loyalty: 0.10,
        }
    }
}
```

### 6.10 环境氛围系统

```rust
/// 环境氛围指数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtmosphereIndex {
    pub lighting_comfort: f32,      // 照明舒适度 0-1
    pub temperature_comfort: f32,   // 温度舒适度 0-1
    pub cleanliness: f32,           // 清洁度 0-1
    pub decoration_taste: f32,      // 装饰品味 0-1
    pub music_score: f32,           // 音乐评分 0-1
}

impl AtmosphereIndex {
    /// 计算综合氛围指数 0-100
    ///
    /// 氛围指数 = (照明×25% + 温度×20% + 清洁×20% + 装饰×20% + 音乐×15%) × 100
    pub fn calculate_total(&self) -> f32 {
        (self.lighting_comfort * 0.25
            + self.temperature_comfort * 0.20
            + self.cleanliness * 0.20
            + self.decoration_taste * 0.20
            + self.music_score * 0.15) * 100.0
    }

    /// 氛围系数（用于客流计算，0-2倍）
    pub fn atmosphere_modifier(&self) -> f32 {
        self.calculate_total() / 50.0
    }

    /// 初始状态
    pub fn initial() -> Self {
        Self {
            lighting_comfort: 0.2,   // 灯光昏暗
            temperature_comfort: 0.3, // 空调故障
            cleanliness: 0.3,        // 较差
            decoration_taste: 0.2,   // 装饰陈旧
            music_score: 0.0,        // 无音乐
        }
    }
}

/// 环境氛围管理
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtmosphereSystem {
    pub index: AtmosphereIndex,
    pub decoration_style: DecorationStyle,
    pub music_enabled: bool,
    pub music_playlist: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecorationStyle {
    OldNostalgic,    // 老旧怀旧
    Simple,          // 简约
    Cozy,            // 温馨
    Artistic,        // 文艺
}
```

### 6.11 顾客满意度计算

```rust
/// 顾客满意度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerSatisfaction {
    pub dish_quality: f32,          // 菜品品质 1-5
    pub service_attitude: f32,      // 服务态度 0-5
    pub environment_score: f32,     // 环境氛围 0-5
}

impl CustomerSatisfaction {
    /// 计算综合满意度 1-5星
    ///
    /// 满意度 = 菜品品质×50% + 服务态度×30% + 环境氛围×20%
    pub fn calculate(&self) -> f32 {
        self.dish_quality * 0.5
            + self.service_attitude * 0.3
            + self.environment_score * 0.2
    }

    /// 从氛围指数计算环境评分
    pub fn from_atmosphere(atmosphere: &AtmosphereIndex) -> f32 {
        atmosphere.calculate_total() / 20.0  // 0-5
    }
}
```

### 6.12 修复与升级进度

```rust
/// 区域修复进度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestorationProgress {
    pub zone: FacilityZone,
    pub completion: u32,             // 0-100
    pub milestones: Vec<Milestone>,
    pub unlocked_features: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_completion: u32,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub reward: Option<MilestoneReward>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneReward {
    pub reward_type: RewardType,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RewardType {
    MemoryFragment,   // 记忆碎片
    NewRecipe,        // 新菜谱
    FeatureUnlock,    // 功能解锁
    StatBoost,        // 属性提升
}

/// 初始修复进度
impl RestorationProgress {
    pub fn initial_all() -> Vec<Self> {
        vec![
            Self {
                zone: FacilityZone::Restaurant,
                completion: 20,
                milestones: vec![
                    Milestone {
                        id: "restaurant_basic".into(),
                        name: "基本营业".into(),
                        description: "恢复4张餐桌".into(),
                        required_completion: 25,
                        is_completed: false,
                        completed_at: None,
                        reward: Some(MilestoneReward {
                            reward_type: RewardType::FeatureUnlock,
                            value: "basic_service".into(),
                        }),
                    },
                ],
                unlocked_features: vec![],
            },
            Self {
                zone: FacilityZone::Kitchen,
                completion: 15,
                milestones: vec![],
                unlocked_features: vec![],
            },
            Self {
                zone: FacilityZone::Backyard,
                completion: 10,
                milestones: vec![],
                unlocked_features: vec![],
            },
            Self {
                zone: FacilityZone::Workshop,
                completion: 5,
                milestones: vec![],
                unlocked_features: vec![],
            },
        ]
    }
}
```

### 6.13 与盼盼系统的交互

| 盼盼属性 | 影响小馆的方面 |
|----------|----------------|
| 厨房模块等级 | 菜品品质、烹饪成功率 |
| 社交模块等级 | 顾客满意度、邻里关系 |
| 移动模块等级 | 设施修复速度、能否自行维修 |
| 记忆模块等级 | 祖传菜谱修复速度 |
| 传感器模块等级 | 实验成功率（新菜研发） |
| 性格-经营风格 | 定价策略、折扣倾向 |
| 性格-创新倾向 | 新菜推出频率、融合菜研发 |
| 信任度 | 主动提出的修复建议质量 |
| 情绪状态 | 工作效率、顾客互动质量 |

```rust
/// 小馆系统与盼盼系统的交互计算
impl ShopSystem {
    /// 计算盼盼模块对小馆的影响
    pub fn calculate_panpan_effects(&self, panpan: &PanpanState) -> ShopEffects {
        ShopEffects {
            cooking_bonus: panpan.modules.kitchen.effectiveness(),
            service_bonus: panpan.modules.social.effectiveness(),
            repair_speed: panpan.modules.mobility.effectiveness(),
            experiment_success_rate: panpan.modules.sensor.effectiveness(),
        }
    }
}
```

### 6.14 旅行系统

旅行是盼盼收集新菜谱、探索世界、追寻祖父足迹的重要方式。盼盼会不定期离开小馆，前往地球各地，带回模糊菜谱、特殊材料和旅行记忆。

#### 6.14.1 旅行触发系统

```rust
/// 旅行触发条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelTrigger {
    pub method: TravelTriggerMethod,
    pub conditions: TravelConditions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TravelTriggerMethod {
    PanpanProposal { suggested_destination: Option<DestinationId> },
    PlayerCommand { destination: DestinationId },
    PlayerRandomChoice,  // 让盼盼随机选择
}

/// 旅行条件检查
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelConditions {
    pub shop_stable: bool,           // 无紧急任务
    pub inventory_sufficient: bool,  // 库存充足
    pub panpan_health: u32,          // 健康度≥80
    pub panpan_energy: u32,          // 能量≥80
    pub trust_level: u32,            // 信任度≥60
    pub cooldown_expired: bool,      // 冷却时间已过
}

impl TravelConditions {
    /// 检查盼盼是否可以主动提议旅行
    pub fn can_panpan_propose(&self) -> bool {
        self.shop_stable
            && self.inventory_sufficient
            && self.panpan_health >= 80
            && self.panpan_energy >= 80
            && self.trust_level >= 60
            && self.cooldown_expired
    }
}

/// 旅行准备
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelPreparation {
    pub destination: DestinationId,
    pub started_at: DateTime<Utc>,
    pub expected_departure: DateTime<Utc>,  // 1小时后
    pub is_cancelled: bool,
}

impl TravelPreparation {
    /// 准备时间（现实时间）
    pub const PREPARATION_DURATION_HOURS: i64 = 1;

    pub fn new(destination: DestinationId) -> Self {
        let now = Utc::now();
        Self {
            destination,
            started_at: now,
            expected_departure: now + chrono::Duration::hours(Self::PREPARATION_DURATION_HOURS),
            is_cancelled: false,
        }
    }
}
```

#### 6.14.2 目的地系统

```rust
/// 目的地定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Destination {
    pub id: DestinationId,
    pub name: String,
    pub region: Region,
    pub country: Country,
    pub unlock_status: UnlockStatus,

    /// 旅行配置
    pub travel_days: u32,            // 现实天数（1-3）
    pub recipe_pool: Vec<RecipePoolEntry>,
    pub material_pool: Vec<MaterialPoolEntry>,
    pub event_pool: Vec<TravelEventTemplate>,

    /// 目的地特色
    pub cuisine_style: String,       // 菜系风格
    pub description: String,
    pub landmark: Option<String>,    // 地标
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DestinationId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Region {
    DomesticNear,      // 国内邻近（1天）
    DomesticFar,       // 国内较远（2天）
    InternationalNear, // 国外邻近（2天）
    InternationalFar,  // 国外远距离（3天）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnlockStatus {
    Initial,                        // 初始解锁
    MemoryFragmentUnlock(Uuid),     // 记忆碎片解锁
    SkillUnlock(u32),               // 技能等级解锁
    EventUnlock(String),            // 事件解锁
    Locked,                         // 未解锁
}

/// 菜谱池条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipePoolEntry {
    pub vague_recipe_id: Uuid,
    pub recipe_name: String,
    pub rarity: RecipeRarity,
    pub weight: u32,                // 抽取权重
    pub requires_event: Option<String>, // 需要特定事件才能获得
}

/// 材料池条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialPoolEntry {
    pub material_type: MaterialType,
    pub material_name: String,
    pub quantity_range: (u32, u32),
    pub weight: u32,
}

/// 目的地示例数据
impl Destination {
    pub fn initial_destinations() -> Vec<Self> {
        vec![
            Self {
                id: DestinationId(Uuid::new_v4()),
                name: "成都".into(),
                region: Region::DomesticNear,
                country: Country::China,
                unlock_status: UnlockStatus::Initial,
                travel_days: 1,
                recipe_pool: vec![
                    RecipePoolEntry {
                        vague_recipe_id: Uuid::new_v4(),
                        recipe_name: "麻婆豆腐".into(),
                        rarity: RecipeRarity::Common,
                        weight: 30,
                        requires_event: None,
                    },
                ],
                material_pool: vec![
                    MaterialPoolEntry {
                        material_type: MaterialType::Special,
                        material_name: "花椒".into(),
                        quantity_range: (3, 8),
                        weight: 50,
                    },
                ],
                event_pool: vec![],
                cuisine_style: "川菜（麻辣）".into(),
                description: "天府之国，美食之都".into(),
                landmark: Some("宽窄巷子".into()),
            },
            Self {
                id: DestinationId(Uuid::new_v4()),
                name: "西安".into(),
                region: Region::DomesticNear,
                country: Country::China,
                unlock_status: UnlockStatus::Initial,
                travel_days: 1,
                recipe_pool: vec![],
                material_pool: vec![],
                event_pool: vec![],
                cuisine_style: "西北面食".into(),
                description: "古都长安，面食天堂".into(),
                landmark: Some("城墙".into()),
            },
            Self {
                id: DestinationId(Uuid::new_v4()),
                name: "广州".into(),
                region: Region::DomesticNear,
                country: Country::China,
                unlock_status: UnlockStatus::Initial,
                travel_days: 1,
                recipe_pool: vec![],
                material_pool: vec![],
                event_pool: vec![],
                cuisine_style: "粤菜（清淡）".into(),
                description: "食在广州，早茶之都".into(),
                landmark: Some("沙面".into()),
            },
        ]
    }
}
```

#### 6.14.3 旅行冷却系统

```rust
/// 旅行冷却管理
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelCooldown {
    pub last_travel_end: Option<DateTime<Utc>>,
    pub base_cooldown_hours: u32,    // 基础冷却时间24小时
}

impl TravelCooldown {
    pub const BASE_COOLDOWN_HOURS: u32 = 24;
    pub const MIN_COOLDOWN_HOURS: u32 = 12;

    /// 计算冷却结束时间
    pub fn calculate_cooldown_end(
        &self,
        panpan_travel_skill: u32,
    ) -> Option<DateTime<Utc>> {
        self.last_travel_end.map(|end| {
            // 每级技能减少1小时冷却
            let reduction = panpan_travel_skill.min(12) as u32;
            let cooldown = (self.base_cooldown_hours - reduction).max(Self::MIN_COOLDOWN_HOURS);
            end + chrono::Duration::hours(cooldown as i64)
        })
    }

    /// 检查冷却是否已过
    pub fn is_cooldown_expired(&self, panpan_travel_skill: u32) -> bool {
        match self.calculate_cooldown_end(panpan_travel_skill) {
            Some(end) => Utc::now() >= end,
            None => true,  // 从未旅行过
        }
    }
}
```

#### 6.14.4 旅行中事件系统

```rust
/// 旅行事件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelEvent {
    pub id: Uuid,
    pub event_type: TravelEventType,
    pub description: String,
    pub image_url: Option<String>,   // 事件图片

    /// 玩家决策
    pub choices: Vec<TravelChoice>,
    pub player_decision: Option<TravelChoice>,
    pub decision_deadline: DateTime<Utc>,  // 决策截止时间（现实12小时）

    /// 事件结果
    pub result: Option<TravelEventResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TravelEventType {
    LearningOpportunity,   // 学习机会：遇到当地老厨师
    MarketDiscovery,       // 市场奇遇：发现稀有材料
    Trouble,               // 麻烦事：迷路/设备故障
    GrandfatherTrail,      // 祖父足迹：发现祖父曾到访
    UnexpectedJoy,         // 意外之喜：偶遇美食节
    LocalEncounter,        // 当地相遇：认识新朋友
    WeatherChallenge,      // 天气挑战：影响行程
}

/// 旅行选择
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelChoice {
    pub id: Uuid,
    pub text: String,               // 选择文本
    pub description: String,        // 选择描述
    pub potential_outcomes: Vec<PotentialOutcome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PotentialOutcome {
    pub outcome_type: OutcomeType,
    pub probability: f32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OutcomeType {
    RecipeGain { recipe_id: Uuid },
    MaterialGain { material: MaterialType, quantity: u32 },
    TimeChange { hours: i32 },
    EnergyChange { percentage: i32 },
    MemoryFragment { fragment_id: Uuid },
    Nothing,
    Trouble { description: String },
}

/// 事件结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelEventResult {
    pub chosen_option: Uuid,
    pub actual_outcome: OutcomeType,
    pub narrative: String,          // 结果描述文本
    pub rewards: Vec<TravelReward>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelReward {
    pub reward_type: TravelRewardType,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TravelRewardType {
    VagueRecipe(Uuid),
    Material(MaterialType),
    MemoryFragment(Uuid),
    Experience(u32),
    Souvenir(Uuid),
}

/// 事件触发系统
pub struct TravelEventSystem;

impl TravelEventSystem {
    /// 计算旅行中事件数量
    pub fn calculate_event_count(
        base_events: u32,
        panpan_travel_skill: u32,
        destination_region: &Region,
    ) -> u32 {
        let skill_bonus = panpan_travel_skill / 3;
        let region_modifier = match destination_region {
            Region::DomesticNear => 0,
            Region::DomesticFar => 1,
            Region::InternationalNear => 1,
            Region::InternationalFar => 2,
        };
        (base_events + skill_bonus + region_modifier).clamp(1, 5)
    }

    /// 随机选择事件
    pub fn select_events(
        destination: &Destination,
        count: u32,
    ) -> Vec<TravelEventTemplate> {
        let mut rng = rand::thread_rng();
        destination.event_pool
            .choose_multiple(&mut rng, count as usize)
            .cloned()
            .collect()
    }
}
```

#### 6.14.5 旅行收获系统

```rust
/// 旅行收获
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelHarvest {
    pub recipes: Vec<VagueRecipe>,
    pub materials: Vec<MaterialStack>,
    pub memory_fragments: Vec<MemoryFragment>,
    pub souvenirs: Vec<Souvenir>,
    pub travel_log: TravelLog,
}

/// 模糊菜谱
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VagueRecipe {
    pub id: Uuid,
    pub name: String,
    pub cuisine_style: String,
    pub source_destination: DestinationId,
    pub acquired_at: DateTime<Utc>,

    /// 模糊状态
    pub description: String,         // 盼盼的描述（模糊）
    pub estimated_ingredients: Vec<EstimatedIngredient>,
    pub estimated_difficulty: f32,   // 1-5
    pub required_experiments: u32,   // 需要的实验次数

    /// 稀有度
    pub rarity: RecipeRarity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeRarity {
    Common,      // 普通，实验3-5次
    Uncommon,    // 少见，实验5-7次
    Rare,        // 稀有，实验7-10次
    Legendary,   // 传说，实验10+次
}

/// 纪念品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Souvenir {
    pub id: Uuid,
    pub name: String,
    pub source_destination: DestinationId,
    pub description: String,
    pub uses: Vec<SouvenirUse>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SouvenirUse {
    GiftToCustomer { favor_bonus: u32 },
    GiftToNeighbor { trust_bonus: u32 },
    Decoration { atmosphere_bonus: u32 },
    Sell { price: Decimal },
}

/// 旅行日志
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelLog {
    pub id: Uuid,
    pub destination: DestinationId,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,

    /// 日志条目
    pub entries: Vec<TravelLogEntry>,
    pub photos: Vec<String>,         // 照片URL

    /// 总结
    pub summary: String,
    pub panpan_mood: Emotion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelLogEntry {
    pub day: u32,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub photo_url: Option<String>,
    pub event_reference: Option<Uuid>,
}

/// 收获计算器
pub struct HarvestCalculator;

impl HarvestCalculator {
    /// 计算旅行收获
    pub fn calculate_harvest(
        destination: &Destination,
        events: &[TravelEventResult],
        panpan_travel_skill: u32,
    ) -> TravelHarvest {
        // 基础菜谱数量：1-3个
        let base_recipes = 1 + rand::thread_rng().gen_range(0..3);
        let skill_bonus = (panpan_travel_skill / 5) as usize;
        let recipe_count = (base_recipes + skill_bonus).min(3);

        // 从菜谱池抽取
        let recipes = Self::draw_recipes(&destination.recipe_pool, recipe_count, events);

        // 计算材料收获
        let materials = Self::draw_materials(&destination.material_pool, events);

        // 记忆碎片（必得1个，祖父足迹事件+1）
        let mut memory_fragments = vec![Self::generate_travel_memory(destination)];
        for event in events {
            if let OutcomeType::MemoryFragment { fragment_id } = &event.actual_outcome {
                memory_fragments.push(MemoryFragment::from_id(*fragment_id));
            }
        }

        // 纪念品
        let souvenirs = Self::generate_souvenirs(destination, events);

        TravelHarvest {
            recipes,
            materials,
            memory_fragments,
            souvenirs,
            travel_log: TravelLog::default(),
        }
    }
}
```

#### 6.14.6 旅行对盼盼的影响

```rust
/// 旅行影响
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelEffects {
    pub energy_cost: u32,            // 能量消耗
    pub final_energy: u32,           // 最终能量（20%-50%）
    pub mood_change: MoodChange,      // 情绪变化
    pub skill_experience: u32,       // 技能经验
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MoodChange {
    Happy { reason: String },        // 顺利且收获丰富
    Excited { reason: String },      // 特别收获
    Tired { reason: String },        // 遭遇麻烦
    Calm,                            // 收获平平
    Nostalgic { reason: String },    // 触发祖父记忆
}

impl TravelEffects {
    /// 计算旅行影响
    pub fn calculate(
        travel_days: u32,
        events: &[TravelEventResult],
        harvest: &TravelHarvest,
    ) -> Self {
        // 基础能量消耗（每天约20%）
        let base_cost = travel_days * 20;
        // 事件影响
        let event_cost = events.iter()
            .filter_map(|e| match &e.actual_outcome {
                OutcomeType::EnergyChange { percentage } => Some(*percentage as u32),
                _ => None,
            })
            .sum::<u32>();
        // 旅途充电（旅馆充电，消耗比工作略高）
        let net_cost = (base_cost + event_cost).saturating_sub(travel_days * 15);
        // 最终能量范围20%-50%
        let final_energy = (100 - net_cost).clamp(20, 50);

        // 情绪判断
        let mood_change = Self::determine_mood(events, harvest);

        // 技能经验
        let base_exp = travel_days * 10;
        let event_exp = events.len() as u32 * 5;
        let skill_experience = base_exp + event_exp;

        Self {
            energy_cost: net_cost,
            final_energy,
            mood_change,
            skill_experience,
        }
    }

    fn determine_mood(events: &[TravelEventResult], harvest: &TravelHarvest) -> MoodChange {
        // 检查是否有祖父足迹
        let has_grandfather_memory = events.iter().any(|e| {
            matches!(e.actual_outcome, OutcomeType::MemoryFragment { .. })
        });

        if has_grandfather_memory {
            return MoodChange::Nostalgic {
                reason: "发现了祖父的足迹".into(),
            };
        }

        // 检查收获丰富度
        let recipe_count = harvest.recipes.len();
        let has_rare = harvest.recipes.iter().any(|r| r.rarity >= RecipeRarity::Rare);

        if has_rare && recipe_count >= 2 {
            MoodChange::Excited {
                reason: "收获满满，还找到了稀有菜谱！".into(),
            }
        } else if recipe_count >= 2 || harvest.materials.len() >= 3 {
            MoodChange::Happy {
                reason: "这次旅行收获颇丰".into(),
            }
        } else if events.iter().any(|e| matches!(e.actual_outcome, OutcomeType::Trouble { .. })) {
            MoodChange::Tired {
                reason: "旅途有些波折".into(),
            }
        } else {
            MoodChange::Calm
        }
    }
}
```

#### 6.14.7 玩家通信系统

```rust
/// 旅行通信
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelCommunication {
    pub travel_id: Uuid,
    pub messages: Vec<TravelMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelMessage {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub message_type: TravelMessageType,
    pub content: String,
    pub photo_url: Option<String>,
    pub requires_response: bool,
    pub response_deadline: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageSender {
    Panpan,
    Player,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TravelMessageType {
    StatusUpdate,       // 状态更新
    EventReport,        // 事件报告
    DecisionRequest,    // 决策请求
    DecisionResponse,   // 决策响应
    DailyLog,           // 每日日志
    ReturnAnnouncement, // 返程通知
}

/// 通信延迟模拟
pub struct CommunicationDelay;

impl CommunicationDelay {
    /// 计算消息延迟（模拟火星-地球通信）
    /// 返回消息到达时间
    pub fn calculate_arrival(
        sent_at: DateTime<Utc>,
        base_delay_minutes: u32,
    ) -> DateTime<Utc> {
        // 添加随机抖动（±2分钟）
        let jitter = rand::thread_rng().gen_range(-2..=2);
        sent_at + chrono::Duration::minutes((base_delay_minutes as i64) + jitter)
    }
}
```

#### 6.14.8 旅行API

```
# 发起旅行
POST /api/v1/saves/:id/travels
{
  "destination_id": "uuid",
  "method": "player_command" | "panpan_proposal" | "random"
}

# 取消旅行准备
DELETE /api/v1/saves/:id/travels/current

# 获取旅行状态
GET /api/v1/saves/:id/travels/current

# 获取旅行历史
GET /api/v1/saves/:id/travels

# 获取旅行详情
GET /api/v1/saves/:id/travels/:travel_id

# 回复旅行事件
POST /api/v1/saves/:id/travels/current/events/:event_id/respond
{
  "choice_id": "uuid"
}

# 获取旅行日志
GET /api/v1/saves/:id/travels/:travel_id/log

# 获取目的地列表
GET /api/v1/saves/:id/destinations

# 解锁目的地
POST /api/v1/saves/:id/destinations/:destination_id/unlock
{
  "unlock_method": "memory_fragment" | "skill" | "event",
  "unlock_key": "uuid or event_name"
}
```

### 6.15 顾客系统

顾客是星夜小馆的"常驻民"，每一位老顾客都承载着老街的记忆，也与祖父有着千丝万缕的联系。玩家通过盼盼的间接观察了解他们，并通过事件选择影响他们的故事走向。

#### 6.15.1 顾客基础属性

```rust
/// 顾客基础信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: CustomerId,
    pub name: String,               // 如"李大爷"、"王奶奶"
    pub full_name: String,          // 全名，如"李建国"
    pub avatar_url: Option<String>, // 像素风格头像
    pub identity: String,           // 身份，如"退休机械师"

    /// 出现规律
    pub appearance_schedule: Vec<AppearanceSchedule>,
    pub favorite_dishes: Vec<RecipeId>,

    /// 属性
    pub personality: CustomerPersonality,
    pub favorability: u32,          // 好感度 0-100
    pub status: CustomerStatus,

    /// 与祖父关系
    pub grandfather_relation: GrandfatherRelation,

    /// 解锁内容
    pub unlocked_stories: Vec<String>,
    pub unlocked_memories: Vec<MemoryFragmentId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppearanceSchedule {
    pub day_of_week: Option<DayOfWeek>,  // None = 每天
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerPersonality {
    Cheerful,    // 开朗
    Reserved,    // 寡言
    Enthusiastic, // 热心
    Picky,       // 挑剔
    Gentle,      // 温和
    Introverted, // 内向
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerStatus {
    Normal,
    Sick,
    Traveling,
    Worried,      // 有心事
    Celebrating,  // 庆祝中
}

/// 与祖父的关系
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrandfatherRelation {
    pub relation_type: RelationType,
    pub description: String,
    pub memory_unlock_threshold: u32,  // 解锁记忆所需好感度
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RelationType {
    OldFriend,      // 老友
    Neighbor,       // 邻居
    Beneficiary,    // 受恩惠者
    FamilyFriend,   // 世交
    RegularCustomer, // 老顾客
    SecretConnection, // 秘密联系（剧情相关）
}
```

#### 6.15.2 顾客类型系统

```rust
/// 顾客类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerType {
    Regular(RegularCustomer),    // 固定老顾客
    Floating(FloatingCustomer),  // 流动顾客
    Special(SpecialCustomer),    // 特殊顾客
}

/// 固定老顾客
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegularCustomer {
    pub customer: Customer,
    pub is_backbone: bool,          // 是否为"班底"顾客
    pub introduction_count: u32,    // 介绍新客人数
}

/// 流动顾客
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatingCustomer {
    pub customer: Customer,
    pub floating_type: FloatingType,
    pub potential_regular: bool,    // 是否可能成为老顾客
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FloatingType {
    Traveler,          // 旅行者
    FoodBlogger,       // 美食博主
    NewResident,       // 老街新移民
    MysteriousVisitor, // 神秘访客
}

/// 特殊顾客
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecialCustomer {
    pub customer: Customer,
    pub appearance_condition: SpecialCondition,
    pub story_role: StoryRole,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpecialCondition {
    MemoryFragmentsComplete,  // 记忆碎片集齐
    HighTrustEvent,           // 高信任度特殊事件
    EndingRelated,            // 与结局相关
    HiddenAchievement,        // 隐藏成就解锁
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoryRole {
    TruthRevealer,    // 揭示真相（父亲）
    FinalMessenger,   // 最后信息传递者（祖父幻影）
    SecretKeeper,     // 秘密守护者
}

/// 初始固定老顾客
impl RegularCustomer {
    pub fn initial_customers() -> Vec<Self> {
        vec![
            // 李大爷
            Self {
                customer: Customer {
                    id: CustomerId(Uuid::new_v4()),
                    name: "李大爷".into(),
                    full_name: "李建国".into(),
                    avatar_url: None,
                    identity: "退休机械师，70岁".into(),
                    appearance_schedule: vec![AppearanceSchedule {
                        day_of_week: None,  // 每天
                        start_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                        end_time: NaiveTime::from_hms_opt(8, 30, 0).unwrap(),
                    }],
                    favorite_dishes: vec![],  // 豆浆、油条
                    personality: CustomerPersonality::Enthusiastic,
                    favorability: 30,
                    status: CustomerStatus::Normal,
                    grandfather_relation: GrandfatherRelation {
                        relation_type: RelationType::OldFriend,
                        description: "三十年老友，常一起下棋、钓鱼".into(),
                        memory_unlock_threshold: 60,
                    },
                    unlocked_stories: vec![],
                    unlocked_memories: vec![],
                },
                is_backbone: true,
                introduction_count: 0,
            },
            // 王奶奶
            Self {
                customer: Customer {
                    id: CustomerId(Uuid::new_v4()),
                    name: "王奶奶".into(),
                    full_name: "王秀英".into(),
                    avatar_url: None,
                    identity: "花店老板娘，65岁".into(),
                    appearance_schedule: vec![AppearanceSchedule {
                        day_of_week: None,
                        start_time: NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
                        end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                    }],
                    favorite_dishes: vec![],
                    personality: CustomerPersonality::Gentle,
                    favorability: 25,
                    status: CustomerStatus::Normal,
                    grandfather_relation: GrandfatherRelation {
                        relation_type: RelationType::Neighbor,
                        description: "邻居，常与祖父交换花种和菜种".into(),
                        memory_unlock_threshold: 50,
                    },
                    unlocked_stories: vec![],
                    unlocked_memories: vec![],
                },
                is_backbone: true,
                introduction_count: 0,
            },
            // 老周
            Self {
                customer: Customer {
                    id: CustomerId(Uuid::new_v4()),
                    name: "老周".into(),
                    full_name: "周文远".into(),
                    avatar_url: None,
                    identity: "自由撰稿人，45岁".into(),
                    appearance_schedule: vec![AppearanceSchedule {
                        day_of_week: None,
                        start_time: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                        end_time: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                    }],
                    favorite_dishes: vec![],
                    personality: CustomerPersonality::Introverted,
                    favorability: 20,
                    status: CustomerStatus::Normal,
                    grandfather_relation: GrandfatherRelation {
                        relation_type: RelationType::Beneficiary,
                        description: "祖父是他忠实的读者，常鼓励他写作".into(),
                        memory_unlock_threshold: 55,
                    },
                    unlocked_stories: vec![],
                    unlocked_memories: vec![],
                },
                is_backbone: true,
                introduction_count: 0,
            },
            // 小美
            Self {
                customer: Customer {
                    id: CustomerId(Uuid::new_v4()),
                    name: "小美".into(),
                    full_name: "林小美".into(),
                    avatar_url: None,
                    identity: "年轻白领，28岁".into(),
                    appearance_schedule: vec![
                        AppearanceSchedule {
                            day_of_week: Some(DayOfWeek::Tuesday),
                            start_time: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                            end_time: NaiveTime::from_hms_opt(20, 30, 0).unwrap(),
                        },
                        AppearanceSchedule {
                            day_of_week: Some(DayOfWeek::Thursday),
                            start_time: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                            end_time: NaiveTime::from_hms_opt(20, 30, 0).unwrap(),
                        },
                    ],
                    favorite_dishes: vec![],
                    personality: CustomerPersonality::Cheerful,
                    favorability: 15,
                    status: CustomerStatus::Normal,
                    grandfather_relation: GrandfatherRelation {
                        relation_type: RelationType::Beneficiary,
                        description: "祖父曾匿名资助她读完大学".into(),
                        memory_unlock_threshold: 60,
                    },
                    unlocked_stories: vec![],
                    unlocked_memories: vec![],
                },
                is_backbone: false,
                introduction_count: 0,
            },
        ]
    }
}
```

#### 6.15.3 顾客好感度系统

```rust
/// 好感度等级
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FavorabilityLevel {
    Ordinary,       // 0-20：普通顾客
    Regular,        // 21-40：熟客
    Loyal,          // 41-60：忠实顾客
    Friend,         // 61-80：老街朋友
    Family,         // 81-100：亲人般的存在
}

impl FavorabilityLevel {
    pub fn from_value(value: u32) -> Self {
        match value {
            0..=20 => Self::Ordinary,
            21..=40 => Self::Regular,
            41..=60 => Self::Loyal,
            61..=80 => Self::Friend,
            _ => Self::Family,
        }
    }

    /// 获取等级效果描述
    pub fn effects(&self) -> FavorabilityEffects {
        match self {
            Self::Ordinary => FavorabilityEffects {
                visit_frequency_mod: 1.0,
                can_trigger_event: false,
                can_unlock_memory: false,
                introduces_new_customers: false,
            },
            Self::Regular => FavorabilityEffects {
                visit_frequency_mod: 1.2,
                can_trigger_event: false,
                can_unlock_memory: false,
                introduces_new_customers: false,
            },
            Self::Loyal => FavorabilityEffects {
                visit_frequency_mod: 1.5,
                can_trigger_event: true,
                can_unlock_memory: false,
                introduces_new_customers: true,
            },
            Self::Friend => FavorabilityEffects {
                visit_frequency_mod: 1.8,
                can_trigger_event: true,
                can_unlock_memory: true,
                introduces_new_customers: true,
            },
            Self::Family => FavorabilityEffects {
                visit_frequency_mod: 2.0,
                can_trigger_event: true,
                can_unlock_memory: true,
                introduces_new_customers: true,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FavorabilityEffects {
    pub visit_frequency_mod: f32,
    pub can_trigger_event: bool,
    pub can_unlock_memory: bool,
    pub introduces_new_customers: bool,
}

/// 好感度变化来源
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FavorabilitySource {
    FavoriteDish { quality: f32 },     // 菜品合口味，品质1-5
    SpecialCare { event_id: Uuid },    // 特殊关怀事件
    RememberPreference,                 // 记住偏好
    Gift { gift_type: GiftType },      // 赠送礼物
    HelpInNeed { event_id: Uuid },     // 困难时帮助
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GiftType {
    Souvenir,          // 旅行纪念品
    HandmadeItem,      // 工坊自制物品
    FreshProduce,      // 后院新鲜食材
    SpecialDish,       // 特制菜品
}

/// 好感度计算器
pub struct FavorabilityCalculator;

impl FavorabilityCalculator {
    /// 计算好感度变化
    pub fn calculate_change(source: &FavorabilitySource) -> i32 {
        match source {
            FavorabilitySource::FavoriteDish { quality } => {
                // 品质1-5，对应+1~3
                ((quality - 1.0) / 2.0 + 1.0) as i32
            }
            FavorabilitySource::SpecialCare { .. } => 5,
            FavorabilitySource::RememberPreference => 1,
            FavorabilitySource::Gift { gift_type } => {
                match gift_type {
                    GiftType::Souvenir => 10,
                    GiftType::HandmadeItem => 15,
                    GiftType::FreshProduce => 5,
                    GiftType::SpecialDish => 8,
                }
            }
            FavorabilitySource::HelpInNeed { .. } => 15,
        }
    }

    /// 应用好感度变化
    pub fn apply_change(current: u32, change: i32) -> u32 {
        if change >= 0 {
            (current + change as u32).min(100)
        } else {
            current.saturating_sub((-change) as u32)
        }
    }
}
```

#### 6.15.4 顾客事件系统

```rust
/// 顾客事件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerEvent {
    pub id: Uuid,
    pub customer_id: CustomerId,
    pub event_type: CustomerEventType,
    pub trigger_condition: EventTriggerCondition,

    /// 事件内容
    pub title: String,
    pub description: String,
    pub panpan_observation: String,  // 盼盼的观察描述

    /// 玩家选择
    pub choices: Vec<CustomerEventChoice>,
    pub player_choice: Option<Uuid>,
    pub deadline: DateTime<Utc>,

    /// 结果
    pub result: Option<CustomerEventResult>,
    pub is_resolved: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerEventType {
    PersonalStory,     // 个人故事
    HelpRequest,       // 求助
    Sharing,           // 分享
    Crisis,            // 危机
    Surprise,          // 惊喜
    Birthday,          // 生日
    Festival,          // 节日
    MemoryUnlock,      // 记忆解锁
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventTriggerCondition {
    FavorabilityThreshold { threshold: u32 },
    SpecificDate { month: u32, day: u32 },
    ShopMilestone { milestone: String },
    RandomChance { probability: f32 },
    Combined { conditions: Vec<EventTriggerCondition> },
}

/// 事件选择
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerEventChoice {
    pub id: Uuid,
    pub text: String,
    pub description: String,

    /// 潜在影响
    pub effects: Vec<PotentialEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PotentialEffect {
    pub effect_type: CustomerEventEffectType,
    pub probability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerEventEffectType {
    FavorabilityChange { amount: i32 },
    MemoryUnlock { fragment_id: Uuid },
    ReputationChange { amount: i32 },
    PersonalityShift { axis: PersonalityAxis, delta: i32 },
    ShopAtmosphereChange { amount: i32 },
    NewCustomerIntroduced,
    DecorationUnlock { decoration_id: Uuid },
}

/// 事件结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerEventResult {
    pub chosen_choice: Uuid,
    pub narrative: String,
    pub actual_effects: Vec<ActualEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActualEffect {
    pub effect_type: CustomerEventEffectType,
    pub value_before: serde_json::Value,
    pub value_after: serde_json::Value,
}

/// 顾客事件示例
impl CustomerEvent {
    /// 李大爷的钓鱼记忆
    pub fn fishing_memory_event(customer_id: CustomerId) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            event_type: CustomerEventType::MemoryUnlock,
            trigger_condition: EventTriggerCondition::FavorabilityThreshold { threshold: 60 },
            title: "钓鱼的早晨".into(),
            description: "李大爷今天格外健谈，聊起了和祖父一起钓鱼的往事...".into(),
            panpan_observation: "李大爷的眼睛亮了起来，他看着窗外，好像看到了很多年前的那个早晨。".into(),
            choices: vec![
                CustomerEventChoice {
                    id: Uuid::new_v4(),
                    text: "静静聆听".into(),
                    description: "让李大爷继续讲下去，不打断他的回忆".into(),
                    effects: vec![
                        PotentialEffect {
                            effect_type: CustomerEventEffectType::MemoryUnlock {
                                fragment_id: Uuid::new_v4(),
                            },
                            probability: 1.0,
                        },
                        PotentialEffect {
                            effect_type: CustomerEventEffectType::FavorabilityChange { amount: 5 },
                            probability: 1.0,
                        },
                    ],
                },
                CustomerEventChoice {
                    id: Uuid::new_v4(),
                    text: "询问那道菜".into(),
                    description: "问李大爷祖父当年做的那道菜".into(),
                    effects: vec![
                        PotentialEffect {
                            effect_type: CustomerEventEffectType::MemoryUnlock {
                                fragment_id: Uuid::new_v4(),
                            },
                            probability: 1.0,
                        },
                        PotentialEffect {
                            effect_type: CustomerEventEffectType::FavorabilityChange { amount: 8 },
                            probability: 1.0,
                        },
                    ],
                },
            ],
            player_choice: None,
            deadline: Utc::now() + chrono::Duration::hours(12),
            result: None,
            is_resolved: false,
        }
    }
}
```

#### 6.15.5 顾客与记忆碎片关联

```rust
/// 顾客记忆碎片定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerMemory {
    pub customer_id: CustomerId,
    pub memory_title: String,
    pub unlock_condition: MemoryUnlockCondition,
    pub content: String,
    pub related_recipe: Option<RecipeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryUnlockCondition {
    FavorabilityOnly { threshold: u32 },
    FavorabilityAndEvent { threshold: u32, event_type: String },
    FavorabilityAndAction { threshold: u32, action: String },
}

/// 初始顾客记忆碎片
impl CustomerMemory {
    pub fn initial_memories() -> Vec<Self> {
        vec![
            // 李大爷
            Self {
                customer_id: CustomerId(Uuid::nil()), // 关联实际ID
                memory_title: "钓鱼的早晨".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityAndEvent {
                    threshold: 60,
                    event_type: "chat".into(),
                },
                content: "祖父和李大爷年轻时一起钓鱼，祖父用钓到的鱼做了一道让李大爷至今难忘的菜。".into(),
                related_recipe: None,
            },
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "最后的棋局".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityOnly { threshold: 80 },
                content: "祖父离世前一天，和李大爷下了一盘棋。李大爷说，那天祖父一直在笑。".into(),
                related_recipe: None,
            },
            // 王奶奶
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "花与种子".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityAndAction {
                    threshold: 50,
                    action: "gift_flower".into(),
                },
                content: "祖父教王奶奶种花，两人交换过一包神秘的种子。".into(),
                related_recipe: None,
            },
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "祖父的秘密花园".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityOnly { threshold: 75 },
                content: "祖父在后院藏了一盆珍贵的花，那是他为祖母种的。".into(),
                related_recipe: None,
            },
            // 老周
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "写作的鼓励".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityAndEvent {
                    threshold: 55,
                    event_type: "book_published".into(),
                },
                content: "祖父曾鼓励老周坚持写作，说'小馆永远给你留个位置'。".into(),
                related_recipe: None,
            },
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "书里的小馆".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityOnly { threshold: 80 },
                content: "老周出版的第一本书，扉页写着'献给小馆的林伯'。小说中以小馆为原型。".into(),
                related_recipe: None,
            },
            // 小美
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "上学的路费".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityAndEvent {
                    threshold: 60,
                    event_type: "promotion".into(),
                },
                content: "祖父以'老邻居'名义每年资助小美学费，直到她大学毕业。".into(),
                related_recipe: None,
            },
            Self {
                customer_id: CustomerId(Uuid::nil()),
                memory_title: "谢谢你，爷爷".into(),
                unlock_condition: MemoryUnlockCondition::FavorabilityOnly { threshold: 85 },
                content: "小美终于知道真相，泪流满面。她说，是祖父让她相信这世界上有善良。".into(),
                related_recipe: None,
            },
        ]
    }
}
```

#### 6.15.6 顾客图鉴系统

```rust
/// 顾客图鉴
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerCompendium {
    pub entries: Vec<CompendiumEntry>,
    pub total_unlocked: u32,
    pub total_discovered: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompendiumEntry {
    pub customer_id: CustomerId,
    pub discovery_date: DateTime<Utc>,

    /// 头像清晰度（随好感度提升）
    pub avatar_clarity: AvatarClarity,

    /// 基本信息
    pub basic_info_unlocked: bool,
    pub story_progress: StoryProgress,

    /// 记忆碎片
    pub unlocked_memories: Vec<MemoryFragmentId>,
    pub locked_memories: Vec<String>,  // 仅显示标题
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AvatarClarity {
    Blurry,      // 模糊（好感度 < 20）
    Recognizable, // 可辨认（20-50）
    Clear,       // 清晰（50-80）
    Detailed,    // 精细（80+）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryProgress {
    pub total_chapters: u32,
    pub unlocked_chapters: u32,
    pub current_chapter: Option<String>,
}

impl CompendiumEntry {
    /// 根据好感度计算头像清晰度
    pub fn calculate_clarity(favorability: u32) -> AvatarClarity {
        match favorability {
            0..=20 => AvatarClarity::Blurry,
            21..=50 => AvatarClarity::Recognizable,
            51..=80 => AvatarClarity::Clear,
            _ => AvatarClarity::Detailed,
        }
    }
}
```

#### 6.15.7 顾客日报系统

```rust
/// 顾客日报
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerDailyReport {
    pub date: Date<Utc>,
    pub total_visitors: u32,
    pub regular_customers: u32,
    pub new_customers: u32,

    /// 顾客访问记录
    pub visit_records: Vec<CustomerVisitRecord>,

    /// 特殊提醒
    pub reminders: Vec<CustomerReminder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerVisitRecord {
    pub customer_id: CustomerId,
    pub customer_name: String,
    pub arrival_time: DateTime<Utc>,
    pub ordered_dishes: Vec<String>,
    pub mood: CustomerMood,
    pub observation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CustomerMood {
    Happy,
    Normal,
    Thoughtful,
    Anxious,
    Sad,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerReminder {
    pub reminder_type: ReminderType,
    pub customer_id: Option<CustomerId>,
    pub customer_name: Option<String>,
    pub message: String,
    pub suggested_action: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReminderType {
    Birthday,
    SpecialDate,
    MoodChange,
    MilestoneNear,
    MemoryUnlockAvailable,
    NewCustomerPotential,
}

/// 日报生成器
pub struct DailyReportGenerator;

impl DailyReportGenerator {
    pub fn generate(
        visits: &[CustomerVisitRecord],
        customers: &[Customer],
        date: Date<Utc>,
    ) -> CustomerDailyReport {
        let regular_count = visits.iter()
            .filter(|v| customers.iter().any(|c| c.id == v.customer_id))
            .count() as u32;

        let reminders = Self::generate_reminders(customers, date);

        CustomerDailyReport {
            date,
            total_visitors: visits.len() as u32,
            regular_customers: regular_count,
            new_customers: (visits.len() as u32).saturating_sub(regular_count),
            visit_records: visits.to_vec(),
            reminders,
        }
    }

    fn generate_reminders(customers: &[Customer], date: Date<Utc>) -> Vec<CustomerReminder> {
        let mut reminders = Vec::new();

        // 检查即将到来的生日
        for customer in customers {
            // 假设生日存储在某处
            // if tomorrow is birthday, add reminder
        }

        reminders
    }
}
```

#### 6.15.8 顾客API

```
# 获取顾客列表
GET /api/v1/saves/:id/customers
?type=regular|floating|special
&favorability_min=0
&favorability_max=100

# 获取顾客详情
GET /api/v1/saves/:id/customers/:customer_id

# 获取顾客图鉴
GET /api/v1/saves/:id/customers/compendium

# 获取顾客事件
GET /api/v1/saves/:id/customers/:customer_id/events

# 回复顾客事件
POST /api/v1/saves/:id/customers/:customer_id/events/:event_id/respond
{
  "choice_id": "uuid"
}

# 赠送礼物给顾客
POST /api/v1/saves/:id/customers/:customer_id/gift
{
  "gift_type": "souvenir" | "handmade_item" | "fresh_produce" | "special_dish",
  "item_id": "uuid"
}

# 获取顾客日报
GET /api/v1/saves/:id/customers/daily-report?date=2024-01-15

# 获取顾客记忆碎片
GET /api/v1/saves/:id/customers/:customer_id/memories
```

#### 6.15.9 与盼盼系统的交互

| 盼盼属性 | 顾客相关影响 |
|----------|--------------|
| **社交技能** | 影响盼盼观察顾客情绪、记录偏好的准确性；高技能能更早察觉顾客心事 |
| **记忆容量** | 影响盼盼能记住多少顾客的细节（如生日、喜好） |
| **信任度** | 高信任度时，盼盼会主动提醒玩家"今天是李大爷生日"或"王奶奶看起来不开心" |
| **情绪** | 盼盼情绪好时，与顾客互动更愉快，间接提升顾客满意度 |
| **旅行技能** | 旅行带回的纪念品可赠送顾客，大幅提升好感度 |

### 6.16 菜谱与实验系统

菜谱是星夜小馆的灵魂，每一道菜都承载着祖父的记忆或盼盼旅途的见闻。游戏中的菜谱分为两类：**祖父传承菜谱**（初始拥有部分，但可能模糊或损坏）和**旅行实验菜谱**（盼盼带回模糊描述，通过实验研发）。

#### 6.16.1 菜谱基础属性

```rust
/// 菜谱定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,               // 菜名
    pub cuisine_style: CuisineStyle, // 菜系
    pub source: RecipeSource,       // 来源
    pub status: RecipeStatus,       // 状态

    /// 描述信息
    pub description: String,
    pub story: Option<String>,      // 故事背景

    /// 配方信息（研发成功后填充）
    pub ingredients: Vec<IngredientUsage>,
    pub cooking_time_minutes: u32,  // 制作时间

    /// 经济属性
    pub base_price: Decimal,        // 基础售价
    pub current_price: Decimal,     // 当前售价（玩家可调整）

    /// 品质属性
    pub base_quality: f32,          // 基础品质 1-5星
    pub rarity: RecipeRarity,       // 稀有度

    /// 记忆关联
    pub memory_fragment: Option<MemoryFragmentId>,

    /// 解锁与统计
    pub unlocked_at: Option<DateTime<Utc>>,
    pub times_served: u32,          // 售出次数
    pub average_rating: f32,        // 平均评分
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CuisineStyle {
    Sichuan,      // 川菜
    Cantonese,    // 粤菜
    HomeStyle,    // 家常菜
    Northwest,    // 西北菜
    Jiangnan,     // 江南菜
    Northeast,    // 东北菜
    Japanese,     // 日料
    Western,      // 西餐
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeSource {
    GrandfatherInheritance { clarity: InheritanceClarity },  // 祖父传承
    TravelAcquired { destination: DestinationId },          // 旅行获得
    EventAcquired { event_id: Uuid },                       // 事件获得
    PlayerCreation,                                          // 玩家创新
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InheritanceClarity {
    Clear,         // 清晰
    PartiallyFuzzy, // 部分模糊
    Damaged,       // 损坏
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeStatus {
    Vague,         // 模糊（未研发）
    InResearch,    // 研发中
    Researched,    // 已研发
    Mastered,      // 精通（可优化）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngredientUsage {
    pub ingredient_type: IngredientType,
    pub ingredient_name: String,
    pub amount: f32,                // 用量（克或份）
    pub unit: String,               // 单位
    pub weight: f32,                // 在配方中的重要性权重
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeRarity {
    Common,      // 普通
    Uncommon,    // 少见
    Rare,        // 稀有
    Legendary,   // 传说
}
```

#### 6.16.2 模糊菜谱系统

```rust
/// 模糊菜谱
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VagueRecipe {
    pub id: Uuid,
    pub name: String,
    pub source_destination: Option<DestinationId>,
    pub acquired_at: DateTime<Utc>,

    /// 模糊描述
    pub description: String,
    pub chef_notes: Option<String>,  // 老厨师的备注

    /// 已知信息
    pub known_ingredients: Vec<KnownIngredient>,
    pub clues: Vec<RecipeClue>,

    /// 研发难度
    pub estimated_difficulty: f32,   // 1-5
    pub required_experiments: u32,   // 预估需要的实验次数

    /// 稀有度
    pub rarity: RecipeRarity,
}

/// 已知食材信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnownIngredient {
    pub ingredient_type: IngredientType,
    pub ingredient_name: String,
    pub estimated_amount: Option<EstimatedAmount>,
    pub is_required: bool,
}

/// 预估用量
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstimatedAmount {
    pub min: f32,
    pub max: f32,
    pub confidence: f32,            // 置信度 0-100%
    pub unit: String,
}

/// 配方线索
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeClue {
    pub clue_type: ClueType,
    pub content: String,
    pub reliability: f32,           // 可靠性 0-100%
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClueType {
    Ratio { ingredient1: String, ingredient2: String, ratio: String },
    KeyPoint { description: String },
    Warning { description: String },
    Secret { description: String },
}

/// 模糊菜谱示例
impl VagueRecipe {
    pub fn mapo_tofu_vague() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "麻婆豆腐".into(),
            source_destination: None,  // 成都
            acquired_at: Utc::now(),
            description: "麻辣鲜香的川菜，豆腐嫩滑，牛肉末香酥。需要辣椒和花椒，但用量不详。老厨师说最重要的是'麻辣平衡'。".into(),
            chef_notes: Some("老厨师说最重要的是'麻辣平衡'".into()),
            known_ingredients: vec![
                KnownIngredient {
                    ingredient_type: IngredientType::Main,
                    ingredient_name: "豆腐".into(),
                    estimated_amount: Some(EstimatedAmount {
                        min: 250.0,
                        max: 350.0,
                        confidence: 80.0,
                        unit: "克".into(),
                    }),
                    is_required: true,
                },
                KnownIngredient {
                    ingredient_type: IngredientType::Main,
                    ingredient_name: "牛肉末".into(),
                    estimated_amount: Some(EstimatedAmount {
                        min: 40.0,
                        max: 60.0,
                        confidence: 70.0,
                        unit: "克".into(),
                    }),
                    is_required: true,
                },
                KnownIngredient {
                    ingredient_type: IngredientType::Seasoning,
                    ingredient_name: "辣椒".into(),
                    estimated_amount: Some(EstimatedAmount {
                        min: 10.0,
                        max: 25.0,
                        confidence: 40.0,
                        unit: "克".into(),
                    }),
                    is_required: true,
                },
                KnownIngredient {
                    ingredient_type: IngredientType::Seasoning,
                    ingredient_name: "花椒".into(),
                    estimated_amount: None,
                    is_required: true,
                },
                KnownIngredient {
                    ingredient_type: IngredientType::Seasoning,
                    ingredient_name: "豆瓣酱".into(),
                    estimated_amount: None,
                    is_required: true,
                },
            ],
            clues: vec![
                RecipeClue {
                    clue_type: ClueType::Ratio {
                        ingredient1: "辣椒".into(),
                        ingredient2: "花椒".into(),
                        ratio: "1:1".into(),
                    },
                    content: "辣椒和花椒可能1:1".into(),
                    reliability: 50.0,
                },
            ],
            estimated_difficulty: 3.0,
            required_experiments: 5,
            rarity: RecipeRarity::Common,
        }
    }
}
```

#### 6.16.3 实验研发流程

```rust
/// 实验会话
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentSession {
    pub id: Uuid,
    pub vague_recipe_id: Uuid,
    pub started_at: DateTime<Utc>,

    /// 当前配方状态
    pub current_recipe: ExperimentRecipe,
    pub experiment_count: u32,

    /// 实验记录
    pub history: Vec<ExperimentRecord>,

    /// 状态
    pub status: ExperimentStatus,
}

/// 实验配方（研发中的配方）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRecipe {
    pub ingredients: Vec<ExperimentIngredient>,
    pub cooking_method: Option<String>,
    pub confidence_score: f32,      // 整体置信度
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentIngredient {
    pub ingredient_name: String,
    pub current_amount: f32,
    pub ideal_range: Option<(f32, f32)>,  // 隐藏的理想范围
    pub confidence: f32,
    pub status: IngredientStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IngredientStatus {
    TooLow,
    TooHigh,
    Good,
    Unknown,
}

/// 单次实验记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub experiment_number: u32,
    pub timestamp: DateTime<Utc>,
    pub recipe_used: ExperimentRecipe,
    pub feedback: SensorFeedback,
    pub player_instruction: Option<PlayerInstruction>,
    pub quality_score: f32,         // 本次品质评分
}

/// 传感器反馈
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorFeedback {
    pub ingredient_feedbacks: Vec<IngredientFeedback>,
    pub cooking_feedback: Option<CookingFeedback>,
    pub overall_assessment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngredientFeedback {
    pub ingredient_name: String,
    pub feedback_type: FeedbackType,
    pub suggested_adjustment: Option<f32>,  // 建议调整量
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeedbackType {
    TooMuch { amount: f32 },        // 多了X克
    TooLittle { amount: f32 },      // 少了X克
    JustRight,                      // 刚好
    Uncertain,                      // 不确定
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookingFeedback {
    pub aspect: CookingAspect,
    pub feedback: String,
    pub suggested_action: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CookingAspect {
    Heat,         // 火候
    Timing,       // 时间
    Technique,    // 技法
    Sequence,     // 顺序
}

/// 玩家指导
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerInstruction {
    FollowFeedback,                 // 按反馈调整
    ManualAdjust { ingredient: String, direction: AdjustmentDirection },
    KeepCurrent,                    // 保持原样再试
    Skip,                           // 跳过本次
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdjustmentDirection {
    Increase { amount: f32 },
    Decrease { amount: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperimentStatus {
    InProgress,
    Success { final_recipe: Recipe },
    Abandoned,
}

/// 实验启动条件检查
pub struct ExperimentConditionChecker;

impl ExperimentConditionChecker {
    /// 检查是否可以启动实验
    pub fn check_conditions(
        vague_recipe: &VagueRecipe,
        kitchen_facilities: &[Facility],
        pantry: &Pantry,
        panpan: &PanpanState,
    ) -> Result<(), ExperimentConditionError> {
        // 厨房设施健康度检查
        let avg_health = kitchen_facilities.iter()
            .map(|f| f.condition)
            .sum::<u32>() / kitchen_facilities.len() as u32;
        if avg_health < 70 {
            return Err(ExperimentConditionError::KitchenHealthTooLow(avg_health));
        }

        // 食材库存检查
        for ingredient in &vague_recipe.known_ingredients {
            if ingredient.is_required {
                let stock = pantry.get_stock(&ingredient.ingredient_name);
                if stock < 3 {
                    return Err(ExperimentConditionError::InsufficientIngredients(
                        ingredient.ingredient_name.clone(),
                    ));
                }
            }
        }

        // 盼盼能量检查
        if panpan.energy < 30 {
            return Err(ExperimentConditionError::PanpanEnergyTooLow(panpan.energy));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ExperimentConditionError {
    KitchenHealthTooLow(u32),
    InsufficientIngredients(String),
    PanpanEnergyTooLow(u32),
}
```

#### 6.16.4 传感器模型与成功判定

```rust
/// 传感器模型
pub struct SensorModel;

impl SensorModel {
    /// 计算传感器误差率
    /// 误差率 = (100 - 盼盼实验技能)/200 + (100 - 设备健康度)/200
    /// 范围 0~0.5
    pub fn calculate_error_rate(
        panpan_experiment_skill: u32,
        avg_equipment_health: u32,
    ) -> f32 {
        let skill_error = (100 - panpan_experiment_skill) as f32 / 200.0;
        let equipment_error = (100 - avg_equipment_health) as f32 / 200.0;
        (skill_error + equipment_error).min(0.5)
    }

    /// 生成反馈（带误差）
    pub fn generate_feedback(
        actual_amount: f32,
        ideal_range: (f32, f32),
        error_rate: f32,
    ) -> IngredientFeedback {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 真实差距
        let real_diff = if actual_amount < ideal_range.0 {
            ideal_range.0 - actual_amount  // 少了
        } else if actual_amount > ideal_range.1 {
            actual_amount - ideal_range.1  // 多了
        } else {
            0.0  // 刚好
        };

        // 添加误差抖动
        let jitter = rng.gen_range(-error_rate..=error_rate) * real_diff.abs();
        let perceived_diff = (real_diff + jitter).max(0.0);

        let (feedback_type, suggested) = if real_diff.abs() < 0.1 {
            (FeedbackType::JustRight, None)
        } else if actual_amount < ideal_range.0 {
            (FeedbackType::TooLittle { amount: perceived_diff }, Some(perceived_diff))
        } else {
            (FeedbackType::TooMuch { amount: perceived_diff }, Some(-perceived_diff))
        };

        // 置信度随误差率降低
        let confidence = (1.0 - error_rate) * 100.0;

        IngredientFeedback {
            ingredient_name: String::new(),  // 由调用者填充
            feedback_type,
            suggested_adjustment: suggested,
            confidence,
        }
    }
}

/// 综合评分计算
pub struct QualityScorer;

impl QualityScorer {
    /// 计算综合评分
    /// 综合评分 = Σ(各食材偏差百分比 × 权重) + 火候评分 + 其他因素
    pub fn calculate_score(
        recipe: &ExperimentRecipe,
        cooking_bonus: f32,
    ) -> f32 {
        let mut total_score = 100.0;

        for ingredient in &recipe.ingredients {
            if let Some((min, max)) = ingredient.ideal_range {
                let mid = (min + max) / 2.0;
                let tolerance = (max - min) / 2.0;

                // 偏差百分比
                let deviation = ((ingredient.current_amount - mid).abs() - tolerance).max(0.0);
                let deviation_percent = if mid > 0.0 { deviation / mid * 100.0 } else { 0.0 };

                // 扣分 = 偏差百分比 × 权重
                let deduction = deviation_percent * ingredient.confidence / 100.0;
                total_score -= deduction;
            }
        }

        // 加上火候加成
        total_score += cooking_bonus;

        total_score.max(0.0).min(100.0)
    }

    /// 判断是否成功（≥95分）
    pub fn is_success(score: f32) -> bool {
        score >= 95.0
    }
}

/// 配方收敛检测
pub struct ConvergenceChecker;

impl ConvergenceChecker {
    /// 检查食材是否连续两次"合适"
    pub fn check_ingredient_converged(
        history: &[ExperimentRecord],
        ingredient_name: &str,
    ) -> bool {
        if history.len() < 2 {
            return false;
        }

        let last_two: Vec<_> = history.iter().rev().take(2).collect();
        last_two.iter().all(|record| {
            record.feedback.ingredient_feedbacks.iter()
                .filter(|f| f.ingredient_name == ingredient_name)
                .all(|f| matches!(f.feedback_type, FeedbackType::JustRight))
        })
    }

    /// 检查所有食材是否收敛
    pub fn check_all_converged(
        history: &[ExperimentRecord],
        ingredients: &[ExperimentIngredient],
    ) -> bool {
        ingredients.iter().all(|ing| {
            Self::check_ingredient_converged(history, &ing.ingredient_name)
        })
    }
}
```

#### 6.16.5 实验影响因素

```rust
/// 实验影响因素
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentFactors {
    /// 盼盼因素
    pub panpan_experiment_skill: u32,  // 影响初始配方准确度、反馈误差率
    pub panpan_cooking_skill: u32,     // 影响火候评分
    pub panpan_energy: u32,            // 影响实验效率
    pub panpan_mood: Emotion,          // 影响成功率

    /// 设备因素
    pub stove_health: u32,             // 灶台健康度
    pub oven_health: u32,              // 烤箱健康度
    pub avg_facility_health: u32,      // 平均健康度

    /// 食材因素
    pub ingredient_freshness: f32,     // 平均新鲜度

    /// 玩家因素
    pub player_guidance_quality: f32,  // 玩家指导质量（0-1）

    /// 随机因素
    pub luck_factor: f32,              // 幸运因素（小概率灵感迸发）
}

/// 初始配方生成器
pub struct InitialRecipeGenerator;

impl InitialRecipeGenerator {
    /// 根据盼盼技能和模糊菜谱生成初始配方
    pub fn generate(
        vague_recipe: &VagueRecipe,
        panpan_skill: u32,
        similar_recipe_experience: u32,
    ) -> ExperimentRecipe {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // 技能加成：技能越高，初始配方越接近理想
        let skill_factor = panpan_skill as f32 / 100.0;
        let experience_factor = similar_recipe_experience as f32 / 10.0;
        let accuracy = (skill_factor + experience_factor * 0.1).min(0.9);

        let ingredients = vague_recipe.known_ingredients.iter()
            .map(|known| {
                let base_amount = known.estimated_amount.as_ref()
                    .map(|e| (e.min + e.max) / 2.0)
                    .unwrap_or(50.0);

                // 添加随机偏差，技能高则偏差小
                let max_deviation = (1.0 - accuracy) * base_amount;
                let deviation = rng.gen_range(-max_deviation..=max_deviation);
                let current_amount = (base_amount + deviation).max(1.0);

                ExperimentIngredient {
                    ingredient_name: known.ingredient_name.clone(),
                    current_amount,
                    ideal_range: None,  // 隐藏，不显示给玩家
                    confidence: known.estimated_amount.as_ref()
                        .map(|e| e.confidence)
                        .unwrap_or(30.0),
                    status: IngredientStatus::Unknown,
                }
            })
            .collect();

        ExperimentRecipe {
            ingredients,
            cooking_method: None,
            confidence_score: accuracy * 100.0,
        }
    }
}
```

#### 6.16.6 菜谱品质与价值

```rust
/// 菜谱品质计算
pub struct RecipeQualityCalculator;

impl RecipeQualityCalculator {
    /// 计算研发成功后的菜谱品质
    pub fn calculate_quality(
        recipe_rarity: &RecipeRarity,
        average_experiment_score: f32,
        panpan_cooking_skill: u32,
    ) -> f32 {
        // 基础品质（基于稀有度）
        let base = match recipe_rarity {
            RecipeRarity::Common => 2.5,
            RecipeRarity::Uncommon => 3.0,
            RecipeRarity::Rare => 3.5,
            RecipeRarity::Legendary => 4.0,
        };

        // 实验过程评分加成（0-0.5）
        let experiment_bonus = (average_experiment_score - 95.0) / 10.0;

        // 烹饪技能加成（0-0.5）
        let skill_bonus = panpan_cooking_skill as f32 / 200.0;

        (base + experiment_bonus + skill_bonus).clamp(1.0, 5.0)
    }

    /// 计算建议售价
    pub fn calculate_base_price(
        quality: f32,
        cooking_time_minutes: u32,
        ingredient_cost: Decimal,
        rarity: &RecipeRarity,
    ) -> Decimal {
        let quality_multiplier = quality / 3.0;  // 以3星为基准
        let time_multiplier = 1.0 + (cooking_time_minutes as f32 / 60.0) * 0.2;
        let rarity_multiplier = match rarity {
            RecipeRarity::Common => 1.0,
            RecipeRarity::Uncommon => 1.2,
            RecipeRarity::Rare => 1.5,
            RecipeRarity::Legendary => 2.0,
        };

        ingredient_cost * Decimal::from(quality_multiplier * time_multiplier * rarity_multiplier)
    }
}

/// 菜谱优化系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeOptimization {
    pub recipe_id: RecipeId,
    pub current_quality: f32,
    pub optimization_attempts: u32,
    pub best_score: f32,
}

impl RecipeOptimization {
    /// 已研发的菜谱可以重复实验优化
    pub fn can_optimize(current_quality: f32) -> bool {
        current_quality < 5.0  // 未达到满星
    }

    /// 尝试优化
    pub fn attempt_optimization(
        &mut self,
        experiment_score: f32,
    ) -> Option<f32> {
        self.optimization_attempts += 1;

        // 如果本次评分高于最佳记录，提升品质
        if experiment_score > self.best_score {
            let improvement = (experiment_score - self.best_score) / 100.0;
            let new_quality = (self.current_quality + improvement).min(5.0);
            self.current_quality = new_quality;
            self.best_score = experiment_score;
            return Some(new_quality);
        }
        None
    }
}
```

#### 6.16.7 菜谱图鉴与记忆关联

```rust
/// 菜谱图鉴
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeCompendium {
    pub entries: Vec<RecipeCompendiumEntry>,
    pub total_unlocked: u32,
    pub by_cuisine: HashMap<CuisineStyle, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeCompendiumEntry {
    pub recipe_id: RecipeId,
    pub name: String,
    pub cuisine_style: CuisineStyle,
    pub source: RecipeSource,
    pub rarity: RecipeRarity,

    /// 解锁状态
    pub status: RecipeStatus,
    pub unlocked_at: Option<DateTime<Utc>>,

    /// 记忆碎片关联
    pub memory_fragment: Option<MemoryFragmentId>,
    pub memory_unlocked: bool,

    /// 统计数据
    pub times_served: u32,
    pub average_rating: f32,
    pub best_quality: f32,
}

/// 菜谱记忆关联
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeMemoryLink {
    pub recipe_id: RecipeId,
    pub memory_type: RecipeMemoryType,
    pub unlock_condition: RecipeMemoryCondition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeMemoryType {
    GrandfatherStory { title: String },     // 祖父故事
    TravelMemory { destination: String },   // 旅行记忆
    SpecialEnding { ending_type: String },  // 结局线索
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecipeMemoryCondition {
    OnResearch,                              // 研发成功时解锁
    OnServe { count: u32 },                  // 售出指定次数后解锁
    OnQuality { min_quality: f32 },          // 品质达到指定值后解锁
}
```

#### 6.16.8 实验日志系统

```rust
/// 实验日志
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentLog {
    pub session_id: Uuid,
    pub recipe_name: String,
    pub entries: Vec<LogEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub experiment_number: u32,
    pub timestamp: DateTime<Utc>,

    /// 当前配方
    pub current_recipe: String,     // 格式化的配方字符串

    /// 传感器反馈摘要
    pub feedback_summary: Vec<String>,

    /// 盼盼建议
    pub panpan_suggestion: String,

    /// 玩家选择
    pub player_choice: Option<String>,

    /// 品质评分
    pub quality_score: f32,
}

impl ExperimentLog {
    /// 格式化日志条目
    pub fn format_entry(&self, entry: &LogEntry) -> String {
        format!(
            "【实验日志：{}（第{}次）】\n\
             - 当前配方：{}\n\
             - 传感器反馈：\n  {}\n\
             - 盼盼建议：{}\n\
             - 玩家选择：{}\n\
             - 品质评分：{:.1}分",
            self.recipe_name,
            entry.experiment_number,
            entry.current_recipe,
            entry.feedback_summary.join("\n  "),
            entry.panpan_suggestion,
            entry.player_choice.as_deref().unwrap_or("待决定"),
            entry.quality_score,
        )
    }
}

/// 示例日志生成
impl ExperimentLog {
    pub fn mapo_tofu_example() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            recipe_name: "麻婆豆腐".into(),
            entries: vec![LogEntry {
                experiment_number: 3,
                timestamp: Utc::now(),
                current_recipe: "豆腐300g，牛肉末50g，辣椒15g，花椒8g，豆瓣酱20g，蒜苗20g".into(),
                feedback_summary: vec![
                    "辣椒：稍少（建议+2g）".into(),
                    "花椒：合适".into(),
                    "豆瓣酱：稍多（建议-3g）".into(),
                    "火候：稍欠（灶台温度偏低）".into(),
                ],
                panpan_suggestion: "增加辣椒至17g，减少豆瓣酱至17g，注意灶台火力。".into(),
                player_choice: Some("采纳建议".into()),
                quality_score: 82.5,
            }],
        }
    }
}
```

#### 6.16.9 菜谱与实验API

```
# 获取菜谱列表
GET /api/v1/saves/:id/recipes
?status=vague|in_research|researched|mastered
&cuisine_style=sichuan|cantonese|...
&rarity=common|uncommon|rare|legendary

# 获取菜谱详情
GET /api/v1/saves/:id/recipes/:recipe_id

# 获取模糊菜谱详情
GET /api/v1/saves/:id/vague-recipes/:vague_id

# 启动实验
POST /api/v1/saves/:id/experiments
{
  "vague_recipe_id": "uuid"
}

# 获取实验状态
GET /api/v1/saves/:id/experiments/current

# 执行一次实验
POST /api/v1/saves/:id/experiments/current/run
{
  "instruction": "follow_feedback" | "manual_adjust" | "keep_current" | "skip",
  "manual_adjustments": [
    { "ingredient": "辣椒", "direction": "increase", "amount": 2.0 }
  ]
}

# 获取实验历史
GET /api/v1/saves/:id/experiments/:session_id/history

# 放弃实验
DELETE /api/v1/saves/:id/experiments/current

# 优化已研发菜谱
POST /api/v1/saves/:id/recipes/:recipe_id/optimize

# 获取菜谱图鉴
GET /api/v1/saves/:id/recipes/compendium

# 调整菜谱售价
PATCH /api/v1/saves/:id/recipes/:recipe_id/price
{
  "new_price": 38.00
}
```

#### 6.16.10 与盼盼系统的交互

| 盼盼属性 | 菜谱实验相关影响 |
|----------|------------------|
| **实验技能** | 影响初始配方准确度、传感器反馈误差率、实验收敛速度 |
| **烹饪技能** | 影响火候控制、最终菜谱品质、优化上限 |
| **能量** | 实验消耗能量，能量不足时实验效率降低 |
| **情绪** | 心情好时有小概率"灵感迸发"，可能一次成功 |
| **信任度** | 高信任度可解锁"自动迭代"模式，盼盼自行连续实验 |
| **旅行技能** | 旅行带回更多模糊菜谱，且描述更详细 |

### 6.17 天气系统

天气系统影响客流、种植和特殊事件的触发。

#### 6.17.1 天气类型与效果

```rust
/// 天气状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherState {
    pub current_weather: Weather,
    pub forecast: Vec<WeatherForecast>,
    pub last_update: DateTime<Utc>,
    pub season: Season,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Weather {
    Sunny,        // 晴天：客流+10%，后院生长+20%
    Cloudy,       // 多云：正常
    Rainy,        // 雨天：客流-15%，后院自动浇水
    Stormy,       // 暴风雨：客流-30%，可能损坏设施
    Snowy,        // 下雪：客流-20%，特殊节日加成
    HeatWave,     // 酷暑：客流-10%，空调需求增
    ColdSnap,     // 寒潮：客流-10%，暖气需求增
    Foggy,        // 大雾：旅行延迟+20%
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub date: Date<Utc>,
    pub weather: Weather,
    pub confidence: f32,            // 预报准确度
}

impl Weather {
    /// 天气对客流的影响
    pub fn customer_modifier(&self, has_climate_control: bool) -> f32 {
        if has_climate_control {
            match self {
                Weather::Sunny => 1.1,
                Weather::Cloudy => 1.0,
                Weather::Rainy => 0.9,
                Weather::Stormy => 0.85,
                Weather::Snowy => 0.9,
                Weather::HeatWave => 1.0,   // 有空调则不受影响
                Weather::ColdSnap => 1.0,
                Weather::Foggy => 0.95,
            }
        } else {
            match self {
                Weather::Sunny => 1.1,
                Weather::Cloudy => 1.0,
                Weather::Rainy => 0.85,
                Weather::Stormy => 0.7,
                Weather::Snowy => 0.8,
                Weather::HeatWave => 0.75,
                Weather::ColdSnap => 0.8,
                Weather::Foggy => 0.95,
            }
        }
    }

    /// 天气对种植的影响
    pub fn garden_effect(&self) -> GardenWeatherEffect {
        match self {
            Weather::Sunny => GardenWeatherEffect {
                growth_modifier: 1.2,
                water_need_multiplier: 1.5,
                damage_risk: 0.0,
            },
            Weather::Rainy => GardenWeatherEffect {
                growth_modifier: 1.0,
                water_need_multiplier: 0.0,  // 自动浇水
                damage_risk: 0.0,
            },
            Weather::Stormy => GardenWeatherEffect {
                growth_modifier: 0.5,
                water_need_multiplier: 0.0,
                damage_risk: 0.2,  // 20% 概率损坏作物
            },
            Weather::Snowy => GardenWeatherEffect {
                growth_modifier: 0.3,
                water_need_multiplier: 0.5,
                damage_risk: 0.1,
            },
            _ => GardenWeatherEffect {
                growth_modifier: 1.0,
                water_need_multiplier: 1.0,
                damage_risk: 0.0,
            },
        }
    }

    /// 天气对旅行的影响
    pub fn travel_effect(&self) -> TravelWeatherEffect {
        match self {
            Weather::Stormy => TravelWeatherEffect {
                delay_hours: 12,
                can_cancel: true,
                risk_level: 0.3,
            },
            Weather::Foggy => TravelWeatherEffect {
                delay_hours: 4,
                can_cancel: true,
                risk_level: 0.1,
            },
            _ => TravelWeatherEffect::default(),
        }
    }
}

/// 天气生成器
pub struct WeatherGenerator;

impl WeatherGenerator {
    /// 根据季节生成天气
    pub fn generate_weather(season: &Season) -> Weather {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..100);

        match season {
            Season::Spring => match roll {
                0..=40 => Weather::Cloudy,
                41..=70 => Weather::Rainy,
                71..=90 => Weather::Sunny,
                _ => Weather::Foggy,
            },
            Season::Summer => match roll {
                0..=30 => Weather::Sunny,
                31..=50 => Weather::Cloudy,
                51..=70 => Weather::Rainy,
                71..=85 => Weather::HeatWave,
                _ => Weather::Stormy,
            },
            Season::Autumn => match roll {
                0..=35 => Weather::Sunny,
                36..=60 => Weather::Cloudy,
                61..=80 => Weather::Rainy,
                _ => Weather::Foggy,
            },
            Season::Winter => match roll {
                0..=30 => Weather::Cloudy,
                31..=50 => Weather::Snowy,
                51..=70 => Weather::ColdSnap,
                71..=90 => Weather::Sunny,
                _ => Weather::Stormy,
            },
        }
    }
}
```

#### 6.17.2 天气 API

```
# 获取当前天气
GET /api/v1/saves/:id/weather

# 获取天气预报（7天）
GET /api/v1/saves/:id/weather/forecast?days=7
```

### 6.18 节假日系统

中国传统节假日影响客流、菜品需求和特殊事件。

#### 6.18.1 节假日定义

```rust
/// 节假日系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FestivalSystem {
    pub current_festival: Option<Festival>,
    pub upcoming_festivals: Vec<Festival>,
    pub festival_history: Vec<FestivalRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Festival {
    pub id: FestivalId,
    pub name: String,
    pub festival_type: FestivalType,
    pub start_date: LunarDate,      // 农历日期
    pub duration_days: u32,
    pub effects: FestivalEffects,
    pub special_recipes: Vec<RecipeId>,
    pub special_events: Vec<FestivalEvent>,
    pub traditional_foods: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FestivalType {
    Traditional,   // 传统节日（春节、中秋等）
    Modern,        // 现代节日（劳动节、国庆等）
    Solar,         // 节气（立春、冬至等）
    Local,         // 地方性节日
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LunarDate {
    pub month: u8,
    pub day: u8,
    pub is_leap_month: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FestivalEffects {
    pub customer_bonus: f32,        // 客流加成
    pub price_bonus: f32,           // 可提价幅度
    pub special_decorations: bool,  // 是否解锁特殊装饰
    pub memory_unlock_chance: f32,  // 记忆碎片解锁概率加成
    pub panpan_mood_bonus: f32,     // 盼盼心情加成
}

/// 节假日定义
impl FestivalSystem {
    pub fn traditional_festivals() -> Vec<Festival> {
        vec![
            // 春节
            Festival {
                id: FestivalId::new_v4(),
                name: "春节".into(),
                festival_type: FestivalType::Traditional,
                start_date: LunarDate { month: 1, day: 1, is_leap_month: false },
                duration_days: 7,
                effects: FestivalEffects {
                    customer_bonus: 0.5,
                    price_bonus: 0.2,
                    special_decorations: true,
                    memory_unlock_chance: 0.3,
                    panpan_mood_bonus: 0.2,
                },
                special_recipes: vec![],  // 饺子、年糕、汤圆
                special_events: vec![],   // 团圆饭事件
                traditional_foods: vec!["饺子".into(), "年糕".into(), "鱼".into()],
            },
            // 元宵节
            Festival {
                id: FestivalId::new_v4(),
                name: "元宵节".into(),
                festival_type: FestivalType::Traditional,
                start_date: LunarDate { month: 1, day: 15, is_leap_month: false },
                duration_days: 1,
                effects: FestivalEffects {
                    customer_bonus: 0.3,
                    price_bonus: 0.1,
                    special_decorations: true,
                    memory_unlock_chance: 0.2,
                    panpan_mood_bonus: 0.1,
                },
                special_recipes: vec![],  // 汤圆
                special_events: vec![],
                traditional_foods: vec!["汤圆".into()],
            },
            // 端午节
            Festival {
                id: FestivalId::new_v4(),
                name: "端午节".into(),
                festival_type: FestivalType::Traditional,
                start_date: LunarDate { month: 5, day: 5, is_leap_month: false },
                duration_days: 1,
                effects: FestivalEffects {
                    customer_bonus: 0.25,
                    price_bonus: 0.1,
                    special_decorations: true,
                    memory_unlock_chance: 0.15,
                    panpan_mood_bonus: 0.1,
                },
                special_recipes: vec![],  // 粽子
                special_events: vec![],
                traditional_foods: vec!["粽子".into()],
            },
            // 中秋节
            Festival {
                id: FestivalId::new_v4(),
                name: "中秋节".into(),
                festival_type: FestivalType::Traditional,
                start_date: LunarDate { month: 8, day: 15, is_leap_month: false },
                duration_days: 3,
                effects: FestivalEffects {
                    customer_bonus: 0.35,
                    price_bonus: 0.15,
                    special_decorations: true,
                    memory_unlock_chance: 0.25,
                    panpan_mood_bonus: 0.15,
                },
                special_recipes: vec![],  // 月饼
                special_events: vec![],
                traditional_foods: vec!["月饼".into(), "桂花糕".into()],
            },
            // 重阳节
            Festival {
                id: FestivalId::new_v4(),
                name: "重阳节".into(),
                festival_type: FestivalType::Traditional,
                start_date: LunarDate { month: 9, day: 9, is_leap_month: false },
                duration_days: 1,
                effects: FestivalEffects {
                    customer_bonus: 0.2,
                    price_bonus: 0.1,
                    special_decorations: true,
                    memory_unlock_chance: 0.2,
                    panpan_mood_bonus: 0.1,
                },
                special_recipes: vec![],  // 重阳糕
                special_events: vec![],
                traditional_foods: vec!["重阳糕".into(), "菊花茶".into()],
            },
        ]
    }

    pub fn solar_terms() -> Vec<SolarTerm> {
        vec![
            SolarTerm { name: "立春".into(), effects: SeasonTransitionEffect::SpringStart },
            SolarTerm { name: "春分".into(), effects: SeasonTransitionEffect::DayNightEqual },
            SolarTerm { name: "立夏".into(), effects: SeasonTransitionEffect::SummerStart },
            SolarTerm { name: "夏至".into(), effects: SeasonTransitionEffect::LongestDay },
            SolarTerm { name: "立秋".into(), effects: SeasonTransitionEffect::AutumnStart },
            SolarTerm { name: "秋分".into(), effects: SeasonTransitionEffect::DayNightEqual },
            SolarTerm { name: "立冬".into(), effects: SeasonTransitionEffect::WinterStart },
            SolarTerm { name: "冬至".into(), effects: SeasonTransitionEffect::ShortestDay },
        ]
    }
}

/// 农历转换器
pub struct LunarCalendarConverter;

impl LunarCalendarConverter {
    /// 将公历日期转换为农历日期
    pub fn solar_to_lunar(date: Date<Utc>) -> LunarDate {
        // 使用农历算法库进行转换
        // 实际实现需要引用 lunarcalendar 库或类似实现
        todo!("集成农历转换库")
    }

    /// 将农历日期转换为公历日期
    pub fn lunar_to_solar(lunar: &LunarDate, year: i32) -> Date<Utc> {
        todo!("集成农历转换库")
    }

    /// 检查今天是否是某个节日
    pub fn check_festival(date: Date<Utc>, festivals: &[Festival]) -> Option<&Festival> {
        let lunar = Self::solar_to_lunar(date);
        festivals.iter().find(|f| {
            f.start_date.month == lunar.month && f.start_date.day == lunar.day
        })
    }
}
```

#### 6.18.2 节假日 API

```
# 获取当前节日
GET /api/v1/saves/:id/festivals/current

# 获取即将到来的节日
GET /api/v1/saves/:id/festivals/upcoming?months=3

# 获取节日历史记录
GET /api/v1/saves/:id/festivals/history
```

### 6.19 邻里系统

邻里系统是玩家与老街居民互动的重要途径，通过互助、交易和社交来获取资源和信息。

#### 6.19.1 邻居定义

```rust
/// 邻里系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborhoodSystem {
    pub neighbors: Vec<Neighbor>,
    pub mutual_aid_points: u32,     // 互助积分
    pub community_reputation: u32,  // 社区声望 0-100
    pub active_requests: Vec<NeighborRequest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighbor {
    pub id: NeighborId,
    pub name: String,
    pub age: u32,
    pub profession: NeighborProfession,
    pub personality: NeighborPersonality,

    /// 关系
    pub relationship: u32,          // 关系值 0-100
    pub interaction_count: u32,
    pub last_interaction: Option<DateTime<Utc>>,

    /// 能力
    pub skills: Vec<NeighborSkill>,
    pub available_help: Vec<HelpType>,
    pub trade_options: Vec<TradeOption>,

    /// 背景
    pub backstory: String,
    pub connection_to_grandfather: Option<String>,
    pub schedule: NeighborSchedule,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NeighborProfession {
    Electrician,      // 电工
    Carpenter,        // 木匠
    Gardener,         // 园艺师
    Fisherman,        // 渔夫
    Butcher,          // 屠夫
    Baker,            // 面包师
    Teacher,          // 老师
    RetiredChef,      // 退休厨师
    Mechanic,         // 机械师
    Herbalist,        // 草药师
    Photographer,     // 摄影师
    StreetVendor,     // 街边小贩
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborSkill {
    pub skill_type: SkillType,
    pub level: u32,                 // 1-10
    pub special_bonuses: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HelpType {
    FacilityRepair {
        facility_types: Vec<FacilityType>,
        discount: f32,
        time_reduction: f32,
    },
    MaterialSupply {
        material_type: MaterialType,
        quantity_per_week: u32,
        quality_bonus: f32,
    },
    SkillTeaching {
        skill: SkillType,
        max_level: u32,
        duration_hours: u32,
    },
    RecipeHint {
        recipe_category: String,
        hint_quality: f32,
    },
    EmergencyHelp {
        help_type: EmergencyType,
        cooldown_days: u32,
    },
    InformationSharing {
        info_type: InfoType,
        reliability: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeOption {
    pub id: Uuid,
    pub name: String,
    pub give: ResourceSpec,
    pub receive: ResourceSpec,
    pub daily_limit: Option<u32>,
    pub relationship_required: u32,
    pub available_days: Vec<DayOfWeek>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub resource_type: ResourceType,
    pub quantity: u32,
    pub quality: Option<u32>,
}

/// 初始邻居
impl Neighbor {
    pub fn initial_neighbors() -> Vec<Self> {
        vec![
            // 李大爷 - 退休机械师
            Self {
                id: NeighborId::new_v4(),
                name: "李建国".into(),
                age: 70,
                profession: NeighborProfession::Electrician,
                personality: NeighborPersonality::Cheerful,
                relationship: 40,  // 初始认识
                interaction_count: 5,
                last_interaction: None,
                skills: vec![
                    NeighborSkill { skill_type: SkillType::Repair, level: 8, special_bonuses: vec!["old_appliances".into()] },
                    NeighborSkill { skill_type: SkillType::Fishing, level: 6, special_bonuses: vec![] },
                ],
                available_help: vec![
                    HelpType::FacilityRepair {
                        facility_types: vec![FacilityType::Electrical],
                        discount: 0.3,
                        time_reduction: 0.5,
                    },
                ],
                trade_options: vec![],
                backstory: "在工厂工作了四十年，退休后闲不住，喜欢帮邻居修修补补。".into(),
                connection_to_grandfather: Some("祖父的老朋友，常一起下棋钓鱼".into()),
                schedule: NeighborSchedule::default(),
            },
            // 王奶奶 - 花店老板
            Self {
                id: NeighborId::new_v4(),
                name: "王秀英".into(),
                age: 65,
                profession: NeighborProfession::Gardener,
                personality: NeighborPersonality::Gentle,
                relationship: 35,
                interaction_count: 3,
                last_interaction: None,
                skills: vec![
                    NeighborSkill { skill_type: SkillType::Gardening, level: 9, special_bonuses: vec!["flowers".into()] },
                    NeighborSkill { skill_type: SkillType::Herbalism, level: 5, special_bonuses: vec![] },
                ],
                available_help: vec![
                    HelpType::MaterialSupply {
                        material_type: MaterialType::Seeds,
                        quantity_per_week: 5,
                        quality_bonus: 0.2,
                    },
                ],
                trade_options: vec![
                    TradeOption {
                        id: Uuid::new_v4(),
                        name: "花种交换".into(),
                        give: ResourceSpec { resource_type: ResourceType::Vegetables, quantity: 3, quality: None },
                        receive: ResourceSpec { resource_type: ResourceType::Seeds, quantity: 2, quality: Some(80) },
                        daily_limit: Some(1),
                        relationship_required: 30,
                        available_days: vec![DayOfWeek::Tuesday, DayOfWeek::Friday],
                    },
                ],
                backstory: "经营花店三十年，对花草了如指掌。".into(),
                connection_to_grandfather: Some("祖父常向她请教种植问题".into()),
                schedule: NeighborSchedule::default(),
            },
            // 老周 - 自由撰稿人
            Self {
                id: NeighborId::new_v4(),
                name: "周文远".into(),
                age: 45,
                profession: NeighborProfession::Photographer,
                personality: NeighborPersonality::Introverted,
                relationship: 25,
                interaction_count: 2,
                last_interaction: None,
                skills: vec![
                    NeighborSkill { skill_type: SkillType::Writing, level: 7, special_bonuses: vec![] },
                    NeighborSkill { skill_type: SkillType::Photography, level: 6, special_bonuses: vec!["food".into()] },
                ],
                available_help: vec![
                    HelpType::InformationSharing {
                        info_type: InfoType::FoodTrends,
                        reliability: 0.8,
                    },
                ],
                trade_options: vec![],
                backstory: "自由撰稿人，喜欢在小馆写作。".into(),
                connection_to_grandfather: Some("祖父鼓励他坚持写作".into()),
                schedule: NeighborSchedule::default(),
            },
        ]
    }
}
```

#### 6.19.2 邻里互动系统

```rust
/// 邻里互动请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborRequest {
    pub id: Uuid,
    pub neighbor_id: NeighborId,
    pub request_type: NeighborRequestType,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub status: RequestStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NeighborRequestType {
    HelpNeeded { help_type: String },
    TradeOffer { trade: TradeOption },
    SocialVisit,
    Emergency { urgency: u32 },
}

/// 邻里互动响应
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborInteraction {
    pub id: Uuid,
    pub neighbor_id: NeighborId,
    pub interaction_type: InteractionType,
    pub timestamp: DateTime<Utc>,
    pub outcome: InteractionOutcome,
    pub relationship_change: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InteractionType {
    HelpGiven,
    HelpReceived,
    TradeCompleted,
    SocialChat,
    GiftExchange,
    Conflict,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionOutcome {
    pub success: bool,
    pub narrative: String,
    pub rewards: Vec<Reward>,
    pub unlocks: Vec<String>,
}
```

#### 6.19.3 邻里 API

```
# 获取邻居列表
GET /api/v1/saves/:id/neighbors

# 获取邻居详情
GET /api/v1/saves/:id/neighbors/:neighbor_id

# 与邻居互动
POST /api/v1/saves/:id/neighbors/:neighbor_id/interact
{
  "interaction_type": "help_request",
  "help_type": "facility_repair",
  "facility_id": "uuid"
}

# 完成交易
POST /api/v1/saves/:id/neighbors/:neighbor_id/trade
{
  "trade_option_id": "uuid"
}

# 赠送礼物
POST /api/v1/saves/:id/neighbors/:neighbor_id/gift
{
  "item_type": "souvenir",
  "item_id": "uuid"
}

# 获取邻里请求
GET /api/v1/saves/:id/neighbors/requests
```

### 6.20 供应商系统

供应商系统管理食材和材料的采购渠道。

#### 6.20.1 供应商定义

```rust
/// 供应商系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupplierSystem {
    pub suppliers: Vec<Supplier>,
    pub active_contracts: Vec<SupplyContract>,
    pub order_history: Vec<SupplyOrder>,
    pub unlocked_suppliers: Vec<SupplierId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Supplier {
    pub id: SupplierId,
    pub name: String,
    pub supplier_type: SupplierType,
    pub description: String,

    /// 供应能力
    pub available_ingredients: Vec<IngredientOffering>,
    pub min_order_quantity: u32,
    pub max_order_quantity: u32,

    /// 条件
    pub reliability: u32,           // 可靠性 0-100（影响按时交付概率）
    pub price_tier: PriceTier,
    pub quality_range: (u32, u32),  // 品质范围
    pub delivery_time_hours: u32,

    /// 解锁条件
    pub unlock_condition: Option<SupplierUnlockCondition>,
    pub relationship_required: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SupplierType {
    WholesaleMarket,   // 批发市场 - 价格低，量大，品质一般
    LocalFarmer,       // 本地农户 - 新鲜，价格中，供应不稳定
    PremiumSupplier,   // 高端供应商 - 品质高，价格高
    SpecialtyImporter, // 进口商 - 稀有食材
    TravelingMerchant, // 流动商贩 - 随机稀有食材
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PriceTier {
    Budget,      // 预算级：0.6x
    Standard,    // 标准级：1.0x
    Premium,     // 高级：1.5x
    Luxury,      // 奢华级：2.5x
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngredientOffering {
    pub ingredient_type: String,
    pub base_price: Decimal,
    pub quality: u32,               // 1-100
    pub available_quantity: u32,
    pub seasonal_availability: Vec<Season>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupplyContract {
    pub id: Uuid,
    pub supplier_id: SupplierId,
    pub ingredient_type: String,
    pub quantity_per_week: u32,
    pub negotiated_price: Decimal,
    pub start_date: Date<Utc>,
    pub end_date: Option<Date<Utc>>,
    pub discount: f32,              // 长期合同折扣
    pub reliability_bonus: u32,     // 长期合作可靠性提升
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SupplyOrder {
    pub id: Uuid,
    pub supplier_id: SupplierId,
    pub items: Vec<OrderItem>,
    pub total_cost: Decimal,
    pub ordered_at: DateTime<Utc>,
    pub expected_delivery: DateTime<Utc>,
    pub actual_delivery: Option<DateTime<Utc>>,
    pub status: OrderStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderItem {
    pub ingredient_type: String,
    pub quantity: u32,
    pub unit_price: Decimal,
    pub quality: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Delayed { reason: String },
    Cancelled,
}

/// 初始供应商
impl Supplier {
    pub fn initial_suppliers() -> Vec<Self> {
        vec![
            // 批发市场
            Self {
                id: SupplierId::new_v4(),
                name: "老街批发市场".into(),
                supplier_type: SupplierType::WholesaleMarket,
                description: "老街最大的食材批发市场，品种齐全，价格实惠。".into(),
                available_ingredients: vec![
                    IngredientOffering {
                        ingredient_type: "米".into(),
                        base_price: Decimal::from(3),
                        quality: 60,
                        available_quantity: 1000,
                        seasonal_availability: vec![],
                    },
                ],
                min_order_quantity: 10,
                max_order_quantity: 500,
                reliability: 85,
                price_tier: PriceTier::Budget,
                quality_range: (40, 70),
                delivery_time_hours: 24,
                unlock_condition: None,
                relationship_required: 0,
            },
            // 本地农户
            Self {
                id: SupplierId::new_v4(),
                name: "张记农场".into(),
                supplier_type: SupplierType::LocalFarmer,
                description: "城郊的有机农场，新鲜蔬菜直供。".into(),
                available_ingredients: vec![],
                min_order_quantity: 1,
                max_order_quantity: 50,
                reliability: 70,  // 天气影响供应
                price_tier: PriceTier::Standard,
                quality_range: (70, 90),
                delivery_time_hours: 12,
                unlock_condition: Some(SupplierUnlockCondition::RelationshipWithNeighbor { neighbor_name: "王奶奶".into() }),
                relationship_required: 50,
            },
        ]
    }
}
```

#### 6.20.2 供应商 API

```
# 获取供应商列表
GET /api/v1/saves/:id/suppliers
?type=wholesale|local|premium
&unlocked=true

# 获取供应商详情
GET /api/v1/saves/:id/suppliers/:supplier_id

# 下单
POST /api/v1/saves/:id/suppliers/:supplier_id/order
{
  "items": [
    { "ingredient_type": "番茄", "quantity": 20 }
  ]
}

# 签订长期合同
POST /api/v1/saves/:id/suppliers/:supplier_id/contract
{
  "ingredient_type": "番茄",
  "quantity_per_week": 50,
  "duration_weeks": 4
}

# 获取订单状态
GET /api/v1/saves/:id/suppliers/orders
&status=pending|delivered
```

### 6.21 成就系统

成就系统记录玩家的游戏进度和特殊成就。

#### 6.21.1 成就定义

```rust
/// 成就系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementSystem {
    pub definitions: Vec<AchievementDefinition>,
    pub unlocked: Vec<UnlockedAchievement>,
    pub progress: HashMap<String, AchievementProgress>,
    pub total_points: u32,
    pub display_title: Option<String>,  // 当前展示的头衔
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub condition: AchievementCondition,
    pub reward: AchievementReward,
    pub points: u32,
    pub hidden: bool,                // 是否为隐藏成就
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AchievementCategory {
    Business,      // 经营类：营收、客流相关
    Cooking,       // 烹饪类：菜谱、品质相关
    Social,        // 社交类：顾客、邻里相关
    Exploration,   // 探索类：旅行相关
    Story,         // 剧情类：记忆碎片相关
    Mastery,       // 精通类：技能满级
    Hidden,        // 隐藏成就
    TimeBased,     // 时间类：游戏时长、连续登录
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AchievementCondition {
    CustomersServed { count: u32 },
    RevenueEarned { amount: Decimal },
    RecipesMastered { count: u32 },
    PerfectExperiments { count: u32 },
    TravelsCompleted { count: u32 },
    DestinationsUnlocked { count: u32 },
    MemoriesCollected { count: u32 },
    CustomerMaxFavorability { count: u32 },
    NeighborRelationshipMax { count: u32 },
    FacilityMaxLevel { count: u32 },
    ModuleMaxLevel { count: u32 },
    PlayTime { hours: u32 },
    ConsecutiveDays { days: u32 },
    FestivalParticipated { count: u32 },
    // 复合条件
    And { conditions: Vec<AchievementCondition> },
    Or { conditions: Vec<AchievementCondition> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementReward {
    pub funds: Option<Decimal>,
    pub reputation_bonus: Option<u32>,
    pub unlock_feature: Option<String>,
    pub title: Option<String>,
    pub special_item: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlockedAchievement {
    pub achievement_id: String,
    pub unlocked_at: DateTime<Utc>,
    pub snapshot: GameSnapshot,     // 解锁时的游戏快照
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementProgress {
    pub achievement_id: String,
    pub current_value: u32,
    pub target_value: u32,
    pub percentage: f32,
}

/// 成就定义
impl AchievementSystem {
    pub fn all_achievements() -> Vec<AchievementDefinition> {
        vec![
            // 经营类
            AchievementDefinition {
                id: "first_customer".into(),
                name: "开门迎客".into(),
                description: "接待第一位顾客".into(),
                category: AchievementCategory::Business,
                condition: AchievementCondition::CustomersServed { count: 1 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(100)),
                    title: Some("小馆新人".into()),
                    ..Default::default()
                },
                points: 5,
                hidden: false,
                icon: None,
            },
            AchievementDefinition {
                id: "hundred_customers".into(),
                name: "客似云来".into(),
                description: "累计接待100位顾客".into(),
                category: AchievementCategory::Business,
                condition: AchievementCondition::CustomersServed { count: 100 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(500)),
                    reputation_bonus: Some(5),
                    ..Default::default()
                },
                points: 15,
                hidden: false,
                icon: None,
            },
            AchievementDefinition {
                id: "thousand_customers".into(),
                name: "门庭若市".into(),
                description: "累计接待1000位顾客".into(),
                category: AchievementCategory::Business,
                condition: AchievementCondition::CustomersServed { count: 1000 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(2000)),
                    reputation_bonus: Some(10),
                    title: Some("人气店长".into()),
                    ..Default::default()
                },
                points: 30,
                hidden: false,
                icon: None,
            },
            // 烹饪类
            AchievementDefinition {
                id: "first_recipe".into(),
                name: "初试身手".into(),
                description: "成功研发第一道菜谱".into(),
                category: AchievementCategory::Cooking,
                condition: AchievementCondition::RecipesMastered { count: 1 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(200)),
                    ..Default::default()
                },
                points: 10,
                hidden: false,
                icon: None,
            },
            AchievementDefinition {
                id: "master_chef".into(),
                name: "厨艺大师".into(),
                description: "成功研发20道菜谱".into(),
                category: AchievementCategory::Cooking,
                condition: AchievementCondition::RecipesMastered { count: 20 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(5000)),
                    title: Some("厨艺大师".into()),
                    ..Default::default()
                },
                points: 50,
                hidden: false,
                icon: None,
            },
            // 探索类
            AchievementDefinition {
                id: "first_travel".into(),
                name: "踏上旅途".into(),
                description: "完成第一次旅行".into(),
                category: AchievementCategory::Exploration,
                condition: AchievementCondition::TravelsCompleted { count: 1 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(300)),
                    ..Default::default()
                },
                points: 10,
                hidden: false,
                icon: None,
            },
            AchievementDefinition {
                id: "world_traveler".into(),
                name: "足迹天涯".into(),
                description: "解锁所有旅行目的地".into(),
                category: AchievementCategory::Exploration,
                condition: AchievementCondition::DestinationsUnlocked { count: 10 },
                reward: AchievementReward {
                    funds: Some(Decimal::from(10000)),
                    title: Some("足迹天涯".into()),
                    ..Default::default()
                },
                points: 50,
                hidden: false,
                icon: None,
            },
            // 剧情类
            AchievementDefinition {
                id: "memory_keeper".into(),
                name: "记忆守护者".into(),
                description: "收集所有祖父记忆碎片".into(),
                category: AchievementCategory::Story,
                condition: AchievementCondition::MemoriesCollected { count: 50 },
                reward: AchievementReward {
                    title: Some("记忆守护者".into()),
                    unlock_feature: Some("true_ending".into()),
                    ..Default::default()
                },
                points: 100,
                hidden: true,
                icon: None,
            },
            // 隐藏成就
            AchievementDefinition {
                id: "grandfathers_legacy".into(),
                name: "祖父的传承".into(),
                description: "???" // 隐藏描述
                    .into(),
                category: AchievementCategory::Hidden,
                condition: AchievementCondition::And {
                    conditions: vec![
                        AchievementCondition::MemoriesCollected { count: 50 },
                        AchievementCondition::RecipesMastered { count: 30 },
                        AchievementCondition::CustomerMaxFavorability { count: 4 },
                    ],
                },
                reward: AchievementReward {
                    title: Some("祖父的传人".into()),
                    special_item: Some("grandfather_notebook".into()),
                    ..Default::default()
                },
                points: 200,
                hidden: true,
                icon: None,
            },
        ]
    }
}
```

#### 6.21.2 成就 API

```
# 获取所有成就
GET /api/v1/saves/:id/achievements
&category=business|cooking|...
&include_hidden=false

# 获取成就进度
GET /api/v1/saves/:id/achievements/progress

# 获取已解锁成就
GET /api/v1/saves/:id/achievements/unlocked

# 设置展示头衔
POST /api/v1/saves/:id/achievements/title
{
  "achievement_id": "master_chef"
}
```

### 6.22 教程系统

教程系统引导新手玩家熟悉游戏机制。

#### 6.22.1 教程流程

```rust
/// 教程系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialSystem {
    pub is_enabled: bool,
    pub is_active: bool,
    pub current_step: Option<TutorialStep>,
    pub completed_steps: Vec<String>,
    pub skipped: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub sequence: u32,
    pub title: String,
    pub description: String,
    pub detailed_text: Option<String>,

    /// UI 引导
    pub highlight_element: Option<String>,
    pub highlight_position: Option<HighlightPosition>,
    pub arrow_direction: Option<ArrowDirection>,

    /// 触发条件
    pub required_action: TutorialAction,
    pub auto_advance: bool,         // 完成动作后是否自动进入下一步
    pub skip_allowed: bool,

    /// 奖励
    pub completion_reward: Option<TutorialReward>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TutorialAction {
    ViewSection { section: String },
    SendCommand { hint: String, pattern: Option<String> },
    WaitDuration { minutes: u32 },
    CompleteTask { task_type: String },
    ReachCondition { condition: String },
    ClickElement { element_id: String },
    ReadMessage { message_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialReward {
    pub funds: Option<Decimal>,
    pub unlock_feature: Option<String>,
    pub hint: Option<String>,
}

/// 教程流程定义
impl TutorialSystem {
    pub fn tutorial_flow() -> Vec<TutorialStep> {
        vec![
            // 步骤 1: 欢迎
            TutorialStep {
                id: "welcome".into(),
                sequence: 1,
                title: "来自地球的消息".into(),
                description: "你收到了祖父留下的机器人盼盼发来的消息...".into(),
                detailed_text: Some("祖父去世后，他留下的机器人盼盼一直守着星夜小馆。今天，盼盼终于联系上了远在火星的你。".into()),
                highlight_element: None,
                highlight_position: None,
                arrow_direction: None,
                required_action: TutorialAction::ReadMessage { message_id: "intro_1".into() },
                auto_advance: false,
                skip_allowed: true,
                completion_reward: None,
            },
            // 步骤 2: 查看盼盼状态
            TutorialStep {
                id: "view_panpan".into(),
                sequence: 2,
                title: "认识盼盼".into(),
                description: "查看盼盼的当前状态".into(),
                detailed_text: Some("盼盼是你的得力助手。点击查看它的状态面板，了解它的能量、情绪和能力。".into()),
                highlight_element: Some("panpan_status_panel".into()),
                highlight_position: Some(HighlightPosition::Left),
                arrow_direction: Some(ArrowDirection::Right),
                required_action: TutorialAction::ViewSection { section: "panpan".into() },
                auto_advance: true,
                skip_allowed: true,
                completion_reward: Some(TutorialReward {
                    funds: Some(Decimal::from(100)),
                    hint: Some("小馆的启动资金！".into()),
                }),
            },
            // 步骤 3: 发送第一条指令
            TutorialStep {
                id: "first_command".into(),
                sequence: 3,
                title: "发送指令".into(),
                description: "尝试告诉盼盼'查看小馆状态'".into(),
                detailed_text: Some("由于你在火星，与盼盼的通信会有延迟。发送指令后需要等待一段时间才能收到回复。".into()),
                highlight_element: Some("command_input".into()),
                highlight_position: Some(HighlightPosition::Top),
                arrow_direction: Some(ArrowDirection::Down),
                required_action: TutorialAction::SendCommand {
                    hint: "查看小馆状态".into(),
                    pattern: Some("查看.*状态|状态".into()),
                },
                auto_advance: false,
                skip_allowed: true,
                completion_reward: None,
            },
            // 步骤 4: 理解通信延迟
            TutorialStep {
                id: "understand_delay".into(),
                sequence: 4,
                title: "通信延迟".into(),
                description: "等待指令到达...".into(),
                detailed_text: Some("火星与地球的距离决定了通信延迟。当前延迟约 X 分钟。升级盼盼的通信模块可以减少延迟。".into()),
                highlight_element: Some("delay_indicator".into()),
                highlight_position: Some(HighlightPosition::Bottom),
                arrow_direction: Some(ArrowDirection::Up),
                required_action: TutorialAction::WaitDuration { minutes: 1 },  // 教程中缩短等待
                auto_advance: true,
                skip_allowed: true,
                completion_reward: None,
            },
            // 步骤 5: 查看小馆状态
            TutorialStep {
                id: "view_shop".into(),
                sequence: 5,
                title: "星夜小馆".into(),
                description: "查看小馆的当前状况".into(),
                detailed_text: Some("祖父的小馆已经有些年头了。设施老化，需要修缮。你可以逐步升级各个区域。".into()),
                highlight_element: Some("shop_panel".into()),
                highlight_position: None,
                arrow_direction: None,
                required_action: TutorialAction::ViewSection { section: "shop".into() },
                auto_advance: true,
                skip_allowed: true,
                completion_reward: None,
            },
            // 步骤 6: 教程完成
            TutorialStep {
                id: "complete".into(),
                sequence: 6,
                title: "开始经营".into(),
                description: "你已经掌握了基本操作，开始你的小馆经营之旅吧！".into(),
                detailed_text: Some("提示：多与盼盼交流，它会主动提出建议。探索各个系统，收集祖父的记忆碎片，揭开小馆的故事。".into()),
                highlight_element: None,
                highlight_position: None,
                arrow_direction: None,
                required_action: TutorialAction::ReachCondition { condition: "user_acknowledged".into() },
                auto_advance: false,
                skip_allowed: false,
                completion_reward: Some(TutorialReward {
                    funds: Some(Decimal::from(500)),
                    unlock_feature: Some("travel_system".into()),
                    hint: Some("旅行系统已解锁！盼盼可以出发旅行收集新菜谱了。".into()),
                }),
            },
        ]
    }
}
```

#### 6.22.2 教程 API

```
# 获取教程状态
GET /api/v1/saves/:id/tutorial

# 开始教程
POST /api/v1/saves/:id/tutorial/start

# 跳过教程
POST /api/v1/saves/:id/tutorial/skip

# 完成当前步骤
POST /api/v1/saves/:id/tutorial/advance

# 重置教程
POST /api/v1/saves/:id/tutorial/reset
```

### 6.23 统计与数据系统

统计系统记录和分析游戏数据。

#### 6.23.1 游戏统计

```rust
/// 游戏统计系统
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStatistics {
    // 经营统计
    pub total_customers_served: u64,
    pub total_dishes_sold: u64,
    pub total_revenue: Decimal,
    pub total_expenses: Decimal,
    pub best_selling_dish: Option<RecipeId>,
    pub best_day_revenue: Option<(Date<Utc>, Decimal)>,

    // 烹饪统计
    pub dishes_cooked: HashMap<RecipeId, u64>,
    pub perfect_dishes: u64,
    pub failed_dishes: u64,
    pub experiments_conducted: u32,
    pub experiments_succeeded: u32,

    // 旅行统计
    pub total_travels: u32,
    pub destinations_visited: Vec<DestinationId>,
    pub recipes_found: u32,
    pub rare_materials_found: u32,

    // 社交统计
    pub customers_at_max_favorability: u32,
    pub neighbors_at_max_relationship: u32,
    pub memories_unlocked: u32,

    // 时间统计
    pub play_time_hours: f32,
    pub in_game_days: u32,
    pub commands_sent: u32,

    // 每日记录
    pub daily_records: VecDeque<DailyStatistics>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyStatistics {
    pub date: Date<Utc>,
    pub customers: u32,
    pub revenue: Decimal,
    pub expenses: Decimal,
    pub dishes_served: HashMap<String, u32>,
    pub average_satisfaction: f32,
    pub events_triggered: u32,
    pub weather: Weather,
    pub festival: Option<String>,
}

/// 统计分析
impl GameStatistics {
    /// 计算平均日营收
    pub fn average_daily_revenue(&self) -> Decimal {
        if self.in_game_days == 0 {
            return Decimal::ZERO;
        }
        self.total_revenue / Decimal::from(self.in_game_days)
    }

    /// 计算营收趋势（最近7天）
    pub fn revenue_trend(&self) -> RevenueTrend {
        let recent: Vec<_> = self.daily_records.iter().rev().take(7).collect();
        if recent.len() < 2 {
            return RevenueTrend::InsufficientData;
        }

        let first_half: Decimal = recent.iter().skip(3).map(|d| d.revenue).sum();
        let second_half: Decimal = recent.iter().take(3).map(|d| d.revenue).sum();

        if second_half > first_half * Decimal::from(110) / 100 {
            RevenueTrend::Growing
        } else if second_half < first_half * Decimal::from(90) / 100 {
            RevenueTrend::Declining
        } else {
            RevenueTrend::Stable
        }
    }

    /// 获取最受欢迎的菜品
    pub fn most_popular_dishes(&self, count: usize) -> Vec<(RecipeId, u64)> {
        let mut dishes: Vec<_> = self.dishes_cooked.iter().map(|(k, v)| (*k, *v)).collect();
        dishes.sort_by(|a, b| b.1.cmp(&a.1));
        dishes.into_iter().take(count).collect()
    }
}

pub enum RevenueTrend {
    Growing,
    Stable,
    Declining,
    InsufficientData,
}
```

#### 6.23.2 统计 API

```
# 获取总体统计
GET /api/v1/saves/:id/statistics

# 获取每日统计
GET /api/v1/saves/:id/statistics/daily?days=30

# 获取营收图表数据
GET /api/v1/saves/:id/statistics/revenue-chart?period=week|month

# 获取菜品销售排行
GET /api/v1/saves/:id/statistics/dish-ranking?limit=10

# 获取顾客分析
GET /api/v1/saves/:id/statistics/customer-analysis

# 导出统计数据
GET /api/v1/saves/:id/statistics/export?format=csv|json
```

---

## 七、时间系统与加速模式

### 7.1 时间同步策略

**正常模式**：
- 游戏时间与地球现实时间 1:1 同步
- 营业时间：地球东八区 7:00 - 24:00
- 通信延迟：基于真实火星-地球距离计算（4-24分钟）

**测试模式**：
- 可配置时间加速倍率（默认 10 倍）
- 1 分钟现实时间 = 10 分钟游戏时间
- 便于快速测试旅行、种植、事件等长周期功能

### 7.2 配置与切换

```toml
# config/default.toml
[time]
# 时间倍率：1 = 正常，10 = 测试默认
acceleration = 1

# 时区设置
timezone = "Asia/Shanghai"  # 东八区

# 营业时间
business_hours_start = 7    # 7:00
business_hours_end = 24     # 24:00 (次日0点)
```

```rust
/// 时间系统
pub struct TimeSystem {
    /// 加速倍率（1 = 正常）
    acceleration: u32,
    /// 游戏基准时间点
    epoch: DateTime<Utc>,
    /// 真实开始时间
    real_start: Instant,
}

impl TimeSystem {
    /// 获取当前游戏时间
    pub fn now(&self) -> DateTime<Utc> {
        let elapsed = self.real_start.elapsed();
        let accelerated = elapsed * self.acceleration;
        self.epoch + chrono::Duration::from_std(accelerated).unwrap()
    }

    /// 计算通信延迟（考虑加速）
    pub fn calculate_arrival_time(&self, delay_minutes: u32) -> DateTime<Utc> {
        let now = self.now();
        // 延迟也要受加速影响
        let accelerated_delay = delay_minutes / self.acceleration;
        now + chrono::Duration::minutes(accelerated_delay as i64)
    }
}
```

### 7.3 API 支持

```
# 运行时修改加速倍率（调试用）
PATCH /api/v1/config
{
  "time_acceleration": 10
}

# 获取当前时间状态
GET /api/v1/time
{
  "earth_time": "2024-03-15T14:30:00+08:00",
  "game_time": "2024-03-15T14:30:00+08:00",
  "acceleration": 10,
  "communication_delay_minutes": 12
}
```

---

## 八、图像资源（后续迭代）

**当前阶段决策**：暂不实现 Kitty 图像功能，专注核心游戏逻辑。

**后续迭代计划**：
- 第一版使用文字描述 + ASCII 艺术替代图片
- 图像功能在后端核心稳定后单独开发
- 预留图像资源的 API 接口设计

```
# 预留接口（暂不实现）
GET /api/v1/saves/:id/images/shop     # 小馆监控图
GET /api/v1/saves/:id/images/travel/:travel_id  # 旅行照片
```

---

## 九、数据模型设计

### 8.1 设计理念

**不使用 SaveState 聚合模型**：由于使用 SQLite 作为持久化存储，不需要内存中的大聚合对象。每个实体独立操作，更新时直接写入数据库。所有数据表通过 `save_id` 关联到存档。

**架构优势**：
- 避免内存与数据库双重维护
- 程序重启后数据自动恢复
- 简化数据同步逻辑
- SQLite 性能对单机游戏足够

### 8.2 核心实体

```rust
// 存档元数据（对应 saves 表）
struct Save {
    id: Uuid,
    name: String,
    player_name: String,
    created_at: DateTime<Utc>,
    last_played: DateTime<Utc>,
    play_time_seconds: u64,
    chapter: u32,
}

// 盼盼状态（对应 panpan_states 表，通过 save_id 关联）
struct PanpanState {
    save_id: Uuid,                    // 外键，关联存档
    name: String,
    model: String,                    // HSL-2005
    manufacture_date: DateTime<Utc>,  // 制造日期

    // 核心属性
    personality: Personality,         // 性格轴（3维）
    trust_level: u32,                 // 信任度 0-100
    emotion: Emotion,                 // 当前情绪
    energy: EnergySystem,             // 能量系统

    // 状态
    location: Location,               // 店内/旅行中
    current_state: ActivityState,     // 空闲/工作/旅行中/实验中/维修中/充电中/休眠
    current_task: Option<Task>,
}

// 模块状态（对应 modules 表，通过 save_id 关联）
struct Module {
    id: Uuid,
    save_id: Uuid,                    // 外键
    module_type: ModuleType,
    level: u32,                       // 等级 1-10
    condition: u32,                   // 完好度 0-100
    experience: u32,                  // 经验值
    is_functional: bool,
}

// 性格轴（存储为 JSON）
struct Personality {
    business_style: u32,    // 理性(0) <-> 感性(100)，初始50
    innovation: u32,        // 保守(0) <-> 创新(100)，初始50
    independence: u32,      // 服从(0) <-> 自主(100)，初始50
}

// 能量系统
struct EnergySystem {
    current: u32,            // 当前能量 0-100
    max: u32,                // 最大能量
    charge_rate: u32,        // 充电速度
    is_charging: bool,
}

// 情绪
enum Emotion {
    Happy, Calm, Tired, Confused, Worried, Lonely, Excited,
}

// 活动状态
enum ActivityState {
    Idle, Working, Traveling, Experimenting, Repairing, Charging, Sleeping,
}

// 玩家指令（对应 command_queue 表）
struct Command {
    id: Uuid,
    save_id: Uuid,                    // 外键
    content: String,
    created_at: DateTime<Utc>,
    arrive_at: DateTime<Utc>,
    status: CommandStatus,
    result: Option<CommandResult>,
}

// 小馆状态（对应 shop_states 表）
struct ShopState {
    save_id: Uuid,                    // 外键

    // 基础信息
    name: String,
    english_name: String,
    location: String,
    is_open: bool,

    // 设施系统
    facilities: Vec<SubFacility>,     // 所有子设施
    restoration_progress: Vec<RestorationProgress>,  // 各区域修复进度

    // 经营数据
    finance: FinancialState,          // 资金状态
    customer_stats: CustomerStats,    // 顾客统计

    // 菜品体系
    recipes: Vec<Recipe>,             // 已有菜谱
    daily_menu: Option<DailyMenu>,    // 今日菜单
    research_clues: Vec<ResearchClue>,// 研发线索

    // 口碑与环境
    reputation: ReputationSystem,     // 口碑系统（5维度）
    atmosphere: AtmosphereSystem,     // 环境氛围
}

// 旅行记录（对应 travels 表）
struct Travel {
    id: Uuid,
    save_id: Uuid,                    // 外键
    destination: String,
    started_at: DateTime<Utc>,
    expected_return: DateTime<Utc>,
    status: TravelStatus,
    recipes_found: Vec<VagueRecipe>,  // JSON 存储
    log_entries: Vec<TravelLogEntry>, // JSON 存储
}

// 实验记录（对应 experiments 表）
struct Experiment {
    id: Uuid,
    save_id: Uuid,                    // 外键
    recipe_id: Uuid,
    attempts: Vec<ExperimentAttempt>, // JSON 存储
    status: ExperimentStatus,
    final_recipe: Option<PreciseRecipe>,
}

// 记忆碎片（对应 memory_fragments 表）
struct MemoryFragment {
    id: Uuid,
    save_id: Uuid,                    // 外键
    title: String,
    content: String,
    fragment_type: MemoryType,
    unlocked: bool,
    unlock_condition: UnlockCondition,
    unlocked_at: Option<DateTime<Utc>>,
    is_read: bool,
}

enum MemoryType {
    Grandfather, Customer, Travel, Experiment, Emotional,
}
```

#### 8.2.1 记忆碎片内容存储设计

记忆碎片是游戏叙事的核心，每个碎片包含丰富的内容，用于揭示祖父的过去和星夜小馆的历史。

```rust
/// 记忆碎片完整定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryFragmentDefinition {
    pub id: Uuid,
    pub fragment_number: u32,         // 碎片编号（1-50）
    pub title: String,                // 碎片标题
    pub fragment_type: MemoryTypeDetail,

    // 内容
    pub content: MemoryContent,

    // 解锁条件
    pub unlock_conditions: Vec<UnlockCondition>,

    // 关联信息
    pub related_entities: RelatedEntities,

    // 元数据
    pub emotional_tone: EmotionalTone,
    pub importance: MemoryImportance,
    pub estimated_unlock_chapter: u32,
}

/// 记忆碎片内容
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryContent {
    /// 主叙事文本（盼盼的回忆）
    pub narrative: String,

    /// 附加描述（场景描述）
    pub scene_description: Option<String>,

    /// 祖父的原话（如果有）
    pub grandfather_quote: Option<String>,

    /// 相关的感官记忆
    pub sensory_memories: Vec<SensoryMemory>,

    /// 碎片解锁时的盼盼备注
    pub panpan_reaction: String,

    /// 解锁后解锁的额外信息
    pub unlocked_knowledge: Option<String>,
}

/// 感官记忆（增强沉浸感）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensoryMemory {
    pub sense_type: SenseType,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SenseType {
    Visual,    // 视觉：颜色、光影
    Auditory,  // 听觉：声音、音乐
    Olfactory, // 嗅觉：气味
    Gustatory, // 味觉：味道
    Tactile,   // 触觉：温度、质感
}

/// 记忆碎片类型（详细）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryTypeDetail {
    /// 关于祖父的记忆
    Grandfather { period: GrandfatherPeriod },

    /// 关于小馆的记忆
    ShopHistory { year_range: (u32, u32) },

    /// 关于老街的记忆
    OldStreet { location: String },

    /// 关于菜谱的记忆
    RecipeStory { recipe_id: Uuid },

    /// 关于顾客的记忆
    CustomerStory { customer_type: String },

    /// 盼盼自身的记忆
    PanpanSelf { created_year: u32 },

    /// 旅行收获的记忆
    TravelMemory { destination: String },

    /// 家庭秘密
    FamilySecret { secret_level: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GrandfatherPeriod {
    Youth,         // 青年时期（1960s-1970s）
    MiddleAge,     // 中年时期（1980s-1990s）
    LateYears,     // 晚年时期（2000s-2020s）
    BeforePassing, // 临终前
}

/// 解锁条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnlockCondition {
    /// 里程碑达成
    MilestoneCompleted { milestone_id: String },

    /// 特定事件触发
    EventTriggered { event_id: String },

    /// 菜谱研发成功
    RecipeMastered { recipe_id: Uuid },

    /// 顾客好感度达标
    CustomerBondReached { customer_type: String, level: u32 },

    /// 旅行到特定地点
    TravelCompleted { destination: String },

    /// 种植收获特定作物
    CropHarvested { crop_id: Uuid, quality: u32 },

    /// 信任度达标
    TrustLevelReached { level: u32 },

    /// 游玩时长
    PlayTimeReached { hours: u32 },

    /// 特定日期（纪念日等）
    SpecificDate { month: u32, day: u32 },

    /// 前置碎片解锁
    PreviousFragmentUnlocked { fragment_ids: Vec<Uuid> },

    /// 组合条件
    All(Vec<UnlockCondition>),
    Any(Vec<UnlockCondition>),
}

/// 关联实体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedEntities {
    pub recipes: Vec<Uuid>,
    pub customers: Vec<String>,
    pub locations: Vec<String>,
    pub items: Vec<String>,
    pub other_fragments: Vec<Uuid>,
}

/// 情感基调
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmotionalTone {
    Warm,        // 温馨：家庭团聚、日常幸福
    Nostalgic,   // 怀旧：追忆往昔、岁月流逝
    Bittersweet, // 苦乐参半：离别、成长
    Melancholy,  // 伤感：失去、遗憾
    Joyful,      // 欢快：庆祝、成功
    Mysterious,  // 神秘：秘密、发现
    Hopeful,     // 充满希望：新的开始
    Peaceful,    // 平和：宁静、满足
}

/// 记忆重要性
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryImportance {
    Trivial,    // 琐碎：背景补充
    Normal,     // 普通：日常记忆
    Important,  // 重要：关键情节
    Critical,   // 关键：核心叙事
    Hidden,     // 隐藏：特殊解锁
}

/// 记忆碎片解锁状态（运行时）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryUnlockState {
    pub fragment_id: Uuid,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub unlock_method: Option<String>,
    pub viewed: bool,
    pub view_count: u32,
}
```

#### 记忆碎片内容数据示例

```rust
impl MemoryFragmentDefinition {
    /// 初始碎片定义
    pub fn initial_fragments() -> Vec<Self> {
        vec![
            // 碎片01：初始解锁 - 晨光中的身影
            Self {
                id: Uuid::nil(),
                fragment_number: 1,
                title: "晨光中的身影".into(),
                fragment_type: MemoryTypeDetail::Grandfather {
                    period: GrandfatherPeriod::LateYears,
                },
                content: MemoryContent {
                    narrative: "祖父……每天清晨会坐在靠窗的位置……喝一杯茶……看着老街慢慢醒来。他总说，这是他最喜欢的时间。".into(),
                    scene_description: Some("晨光透过老旧的玻璃窗洒进来，尘埃在光线中缓缓漂浮。祖父的身影在逆光中显得有些模糊。".into()),
                    grandfather_quote: Some("小远，你看这条街，每家店都有自己的故事。我们的店，就是为了让人们记住家的味道。".into()),
                    sensory_memories: vec![
                        SensoryMemory {
                            sense_type: SenseType::Olfactory,
                            description: "淡淡的茉莉花香，那是祖父每天必泡的茶".into(),
                        },
                        SensoryMemory {
                            sense_type: SenseType::Auditory,
                            description: "老街渐渐苏醒的声音——卷帘门拉起、自行车铃声".into(),
                        },
                    ],
                    panpan_reaction: "这段记忆……虽然模糊，但感觉很温暖。祖父很喜欢这家店。".into(),
                    unlocked_knowledge: None,
                },
                unlock_conditions: vec![UnlockCondition::MilestoneCompleted {
                    milestone_id: "first_connection".into(),
                }],
                related_entities: RelatedEntities {
                    recipes: vec![],
                    customers: vec![],
                    locations: vec!["星夜小馆".into()],
                    items: vec![],
                    other_fragments: vec![],
                },
                emotional_tone: EmotionalTone::Nostalgic,
                importance: MemoryImportance::Important,
                estimated_unlock_chapter: 1,
            },

            // 碎片05：第一位老顾客
            Self {
                id: Uuid::nil(),
                fragment_number: 5,
                title: "街坊邻居".into(),
                fragment_type: MemoryTypeDetail::CustomerStory {
                    customer_type: "elderly_neighbor".into(),
                },
                content: MemoryContent {
                    narrative: "第一位顾客是李大爷……他以前每天来吃早饭……祖父总会给他多盛一碗粥。".into(),
                    scene_description: Some("李大爷拄着拐杖，慢慢走进店里。他的眼睛亮了起来，像是看到了久违的老朋友。".into()),
                    grandfather_quote: Some("老李啊，这是你爱吃的皮蛋瘦肉粥，我多放了点姜丝。".into()),
                    sensory_memories: vec![
                        SensoryMemory {
                            sense_type: SenseType::Gustatory,
                            description: "热腾腾的粥，带着姜丝的微微辛辣".into(),
                        },
                    ],
                    panpan_reaction: "李大爷……我记得他。他总是笑眯眯的，还会给我讲他年轻时的故事。".into(),
                    unlocked_knowledge: Some("解锁：李大爷的背景故事".into()),
                },
                unlock_conditions: vec![UnlockCondition::CustomerBondReached {
                    customer_type: "李大爷".into(),
                    level: 2,
                }],
                related_entities: RelatedEntities {
                    recipes: vec![],
                    customers: vec!["李大爷".into()],
                    locations: vec![],
                    items: vec![],
                    other_fragments: vec![],
                },
                emotional_tone: EmotionalTone::Warm,
                importance: MemoryImportance::Normal,
                estimated_unlock_chapter: 1,
            },

            // 碎片08：旅行的记忆
            Self {
                id: Uuid::nil(),
                fragment_number: 8,
                title: "成都的味道".into(),
                fragment_type: MemoryTypeDetail::TravelMemory {
                    destination: "成都".into(),
                },
                content: MemoryContent {
                    narrative: "祖父年轻时也喜欢旅行，他曾去成都学做麻婆豆腐……这是他最自豪的菜。".into(),
                    scene_description: Some("成都的街头，空气中弥漫着麻辣的香气。祖父站在一家老店门前，认真地看着老师傅翻炒。".into()),
                    grandfather_quote: Some("川菜的精髓在于'麻'和'辣'的平衡。花椒要够香，辣椒要够红，但最重要的是——要让客人感受到厨师的用心。".into()),
                    sensory_memories: vec![
                        SensoryMemory {
                            sense_type: SenseType::Olfactory,
                            description: "花椒的麻香、豆瓣酱的醇厚".into(),
                        },
                        SensoryMemory {
                            sense_type: SenseType::Tactile,
                            description: "铁锅传来的热浪，汗水顺着脸颊滑落".into(),
                        },
                    ],
                    panpan_reaction: "原来祖父的麻婆豆腐是在成都学的！难怪那么正宗。我……想去成都看看。".into(),
                    unlocked_knowledge: Some("解锁：成都旅行目的地".into()),
                },
                unlock_conditions: vec![UnlockCondition::TravelCompleted {
                    destination: "成都".into(),
                }],
                related_entities: RelatedEntities {
                    recipes: vec![],
                    customers: vec![],
                    locations: vec!["成都".into()],
                    items: vec![],
                    other_fragments: vec![],
                },
                emotional_tone: EmotionalTone::Joyful,
                importance: MemoryImportance::Important,
                estimated_unlock_chapter: 2,
            },

            // 碎片25：家庭秘密（关键碎片）
            Self {
                id: Uuid::nil(),
                fragment_number: 25,
                title: "未寄出的信".into(),
                fragment_type: MemoryTypeDetail::FamilySecret {
                    secret_level: 2,
                },
                content: MemoryContent {
                    narrative: "在修理盼盼时，我在他体内发现了一封从未寄出的信。那是祖父写给远在火星的我的信……".into(),
                    scene_description: Some("一封泛黄的信封，字迹已经有些模糊。信封上写着'火星殖民地·奥林帕斯基地·林远收'。".into()),
                    grandfather_quote: Some("小远，爷爷知道你在火星很忙，回不来。没关系，爷爷会把小馆一直开着，等你回来的那天。这封信可能永远寄不出去，但爷爷想让你知道——星夜小馆，永远是你的家。".into()),
                    sensory_memories: vec![
                        SensoryMemory {
                            sense_type: SenseType::Tactile,
                            description: "纸张已经变得脆弱，小心翼翼地展开".into(),
                        },
                    ],
                    panpan_reaction: "这封信……祖父一直藏在我体内。我……我不知道该说什么。主人，对不起，我应该早点告诉你的。".into(),
                    unlocked_knowledge: Some("解锁：祖父的真实心意、家庭背景完整信息".into()),
                },
                unlock_conditions: vec![UnlockCondition::All(vec![
                    UnlockCondition::TrustLevelReached { level: 80 },
                    UnlockCondition::MilestoneCompleted { milestone_id: "shop_fully_restored".into() },
                    UnlockCondition::PreviousFragmentUnlocked { fragment_ids: vec![] },
                ])],
                related_entities: RelatedEntities {
                    recipes: vec![],
                    customers: vec![],
                    locations: vec![],
                    items: vec!["祖父的信".into()],
                    other_fragments: vec![],
                },
                emotional_tone: EmotionalTone::Bittersweet,
                importance: MemoryImportance::Critical,
                estimated_unlock_chapter: 4,
            },
        ]
    }
}
```

#### 记忆碎片数据库表

```sql
-- 记忆碎片定义表（全局配置）
CREATE TABLE memory_fragment_definitions (
    id TEXT PRIMARY KEY,
    fragment_number INTEGER NOT NULL UNIQUE,
    title TEXT NOT NULL,
    fragment_type TEXT NOT NULL,        -- JSON
    content TEXT NOT NULL,              -- JSON (MemoryContent)
    unlock_conditions TEXT NOT NULL,    -- JSON
    related_entities TEXT NOT NULL,     -- JSON
    emotional_tone TEXT NOT NULL,
    importance TEXT NOT NULL,
    estimated_unlock_chapter INTEGER NOT NULL
);

-- 玩家记忆碎片状态表（存档相关）
CREATE TABLE memory_fragment_states (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    fragment_id TEXT NOT NULL,
    unlocked INTEGER NOT NULL DEFAULT 0,
    unlocked_at TEXT,
    unlock_method TEXT,
    viewed INTEGER NOT NULL DEFAULT 0,
    view_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(save_id, fragment_id),
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (fragment_id) REFERENCES memory_fragment_definitions(id)
);

CREATE INDEX idx_memory_states_save ON memory_fragment_states(save_id);
CREATE INDEX idx_memory_states_unlocked ON memory_fragment_states(save_id, unlocked);
```

#### 记忆碎片 API

```
# 获取所有记忆碎片状态
GET /api/v1/saves/:id/memories
Response: {
    "total": 50,
    "unlocked": 12,
    "fragments": [
        {
            "id": "uuid",
            "fragment_number": 1,
            "title": "晨光中的身影",
            "unlocked": true,
            "viewed": true,
            "importance": "Important",
            "emotional_tone": "Nostalgic"
        }
    ]
}

# 获取单个记忆碎片详情
GET /api/v1/saves/:id/memories/:fragment_id
Response: MemoryFragmentDefinition

# 标记记忆碎片为已读
POST /api/v1/saves/:id/memories/:fragment_id/view
Response: { "success": true }

# 获取记忆碎片解锁进度
GET /api/v1/saves/:id/memories/progress
Response: {
    "by_chapter": {
        "1": { "unlocked": 8, "total": 15 },
        "2": { "unlocked": 4, "total": 12 },
        "3": { "unlocked": 0, "total": 10 },
        "4": { "unlocked": 0, "total": 8 },
        "5": { "unlocked": 0, "total": 5 }
    },
    "by_type": {
        "Grandfather": { "unlocked": 3, "total": 10 },
        "ShopHistory": { "unlocked": 4, "total": 8 },
        "RecipeStory": { "unlocked": 2, "total": 12 },
        "FamilySecret": { "unlocked": 0, "total": 5 }
    },
    "next_unlock_hint": "继续提升李大爷的好感度以解锁新记忆"
}
```

### 8.3 仓储层设计

```rust
/// 仓储 trait - 每个实体有自己的仓储
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn get_by_save(&self, save_id: Uuid) -> Result<T>;
    async fn save(&self, entity: &T) -> Result<()>;
}

/// 盼盼仓储
#[async_trait]
pub trait PanpanRepository: Repository<PanpanState> {
    async fn get_modules(&self, save_id: Uuid) -> Result<Vec<Module>>;
    async fn update_module(&self, module: &Module) -> Result<()>;
    async fn update_trust(&self, save_id: Uuid, new_value: u32) -> Result<()>;
    async fn update_emotion(&self, save_id: Uuid, emotion: Emotion) -> Result<()>;
}

/// 小馆仓储
#[async_trait]
pub trait ShopRepository: Repository<ShopState> {
    async fn update_funds(&self, save_id: Uuid, amount: Decimal) -> Result<()>;
    async fn update_reputation(&self, save_id: Uuid, value: f32) -> Result<()>;
}

/// 旅行仓储
#[async_trait]
pub trait TravelRepository {
    async fn get_active(&self, save_id: Uuid) -> Result<Option<Travel>>;
    async fn get_all(&self, save_id: Uuid) -> Result<Vec<Travel>>;
    async fn save(&self, travel: &Travel) -> Result<()>;
}
```

### 8.4 数据库 Schema（SQLite）

**设计说明**：
- 所有数据表通过 `save_id` 关联到 `saves` 表
- 不使用 JSON 存储完整游戏状态，每个实体独立存储
- 模块使用独立表而非 JSON，便于细粒度查询和更新

```sql
-- 存档元数据表（核心表）
CREATE TABLE saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    player_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_played TEXT NOT NULL,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    chapter INTEGER NOT NULL DEFAULT 1
);

-- 指令队列表
CREATE TABLE command_queue (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    arrive_at TEXT NOT NULL,
    status TEXT NOT NULL,
    result TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 盼盼状态表
CREATE TABLE panpan_states (
    save_id TEXT PRIMARY KEY,
    -- 基础信息
    name TEXT NOT NULL DEFAULT '盼盼',
    model TEXT NOT NULL DEFAULT 'HSL-2005',
    manufacture_date TEXT NOT NULL,

    -- 核心属性
    personality TEXT NOT NULL,      -- JSON: {business_style, innovation, independence}
    trust_level INTEGER NOT NULL DEFAULT 50,
    emotion TEXT NOT NULL DEFAULT 'Calm',
    energy_current INTEGER NOT NULL DEFAULT 60,
    energy_max INTEGER NOT NULL DEFAULT 100,
    energy_charge_rate INTEGER NOT NULL DEFAULT 15,
    is_charging INTEGER NOT NULL DEFAULT 0,

    -- 当前状态
    location TEXT NOT NULL DEFAULT 'Shop',
    current_state TEXT NOT NULL DEFAULT 'Idle',
    current_task TEXT,

    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 模块详情表（独立表，便于细粒度操作）
CREATE TABLE modules (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    module_type TEXT NOT NULL,       -- Communication/Memory/Sensor/Mobility/Battery/Kitchen/Social
    level INTEGER NOT NULL DEFAULT 1,
    condition INTEGER NOT NULL DEFAULT 50,
    experience INTEGER NOT NULL DEFAULT 0,
    is_functional INTEGER NOT NULL DEFAULT 1,
    UNIQUE(save_id, module_type),
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 模块升级历史表
CREATE TABLE module_upgrades (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    module_type TEXT NOT NULL,
    from_level INTEGER NOT NULL,
    to_level INTEGER NOT NULL,
    upgraded_at TEXT NOT NULL,
    cost TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 情绪历史表（用于追踪情绪变化）
CREATE TABLE emotion_history (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    emotion TEXT NOT NULL,
    triggered_at TEXT NOT NULL,
    trigger_reason TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 信任度变化历史表
CREATE TABLE trust_history (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    change_value INTEGER NOT NULL,
    reason TEXT NOT NULL,
    changed_at TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 小馆状态表
-- 小馆状态表（主表）
CREATE TABLE shop_states (
    save_id TEXT PRIMARY KEY,

    -- 基础信息
    name TEXT NOT NULL DEFAULT '星夜小馆',
    english_name TEXT NOT NULL DEFAULT 'Starry Night Bistro',
    location TEXT NOT NULL DEFAULT '地球·老街',
    is_open INTEGER NOT NULL DEFAULT 0,

    -- 经营数据（JSON 存储）
    finance TEXT NOT NULL,           -- FinancialState JSON
    customer_stats TEXT NOT NULL,    -- CustomerStats JSON

    -- 口碑系统（JSON 存储）
    reputation TEXT NOT NULL,        -- ReputationSystem JSON

    -- 环境氛围（JSON 存储）
    atmosphere TEXT NOT NULL,        -- AtmosphereSystem JSON

    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 子设施表
CREATE TABLE shop_facilities (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    facility_id TEXT NOT NULL,       -- 设施标识（如 dining_tables, stove）
    zone TEXT NOT NULL,              -- Restaurant/Kitchen/Backyard/Workshop
    facility_type TEXT NOT NULL,     -- FacilityType
    name TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    max_level INTEGER NOT NULL DEFAULT 5,
    condition INTEGER NOT NULL DEFAULT 50,
    is_functional INTEGER NOT NULL DEFAULT 1,
    effect TEXT NOT NULL,            -- FacilityEffect JSON
    quantity INTEGER,
    max_quantity INTEGER,
    UNIQUE(save_id, facility_id),
    FOREIGN KEY (save_id) REFERENCES shop_states(save_id)
);

-- 升级路径表
CREATE TABLE facility_upgrade_paths (
    id TEXT PRIMARY KEY,
    facility_type TEXT NOT NULL,
    from_level INTEGER NOT NULL,
    to_level INTEGER NOT NULL,
    cost TEXT NOT NULL,              -- Decimal
    materials TEXT NOT NULL,         -- Vec<MaterialCost> JSON
    time_days INTEGER NOT NULL,
    required_personnel TEXT NOT NULL,-- PersonnelType
    unlocks TEXT,
    UNIQUE(facility_type, from_level, to_level)
);

-- 升级记录表
CREATE TABLE facility_upgrades (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    facility_id TEXT NOT NULL,
    from_level INTEGER NOT NULL,
    to_level INTEGER NOT NULL,
    cost TEXT NOT NULL,
    upgraded_at TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 修复进度表
CREATE TABLE shop_restoration (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    zone TEXT NOT NULL,
    completion INTEGER NOT NULL DEFAULT 0,
    milestones TEXT NOT NULL,        -- Vec<Milestone> JSON
    unlocked_features TEXT NOT NULL, -- Vec<String> JSON
    UNIQUE(save_id, zone),
    FOREIGN KEY (save_id) REFERENCES shop_states(save_id)
);

-- 菜谱表
CREATE TABLE shop_recipes (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    name TEXT NOT NULL,
    cuisine_type TEXT NOT NULL,
    source TEXT NOT NULL,            -- Inherited/TravelDiscovered/Innovative
    status TEXT NOT NULL,            -- Damaged/Vague/Precise/Mastered
    ingredients TEXT NOT NULL,       -- Vec<IngredientUsage> JSON
    cooking_time_minutes INTEGER NOT NULL,
    price TEXT NOT NULL,
    cost TEXT NOT NULL,
    base_quality INTEGER NOT NULL DEFAULT 3,
    is_on_menu INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES shop_states(save_id)
);

-- 每日菜单表
CREATE TABLE shop_daily_menus (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    date TEXT NOT NULL,
    available_recipes TEXT NOT NULL, -- Vec<Uuid> JSON
    daily_specials TEXT NOT NULL,    -- Vec<Uuid> JSON
    UNIQUE(save_id, date),
    FOREIGN KEY (save_id) REFERENCES shop_states(save_id)
);

-- 研发线索表
CREATE TABLE shop_research_clues (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    cuisine_type TEXT NOT NULL,
    discovered INTEGER NOT NULL DEFAULT 0,
    discovered_at TEXT,
    travel_destination TEXT,
    unlocked_recipes TEXT NOT NULL,  -- Vec<Uuid> JSON
    prerequisites TEXT NOT NULL,     -- Vec<Uuid> JSON
    FOREIGN KEY (save_id) REFERENCES shop_states(save_id)
);

-- 旅行记录表
CREATE TABLE travels (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    destination TEXT NOT NULL,
    started_at TEXT NOT NULL,
    expected_return TEXT NOT NULL,
    status TEXT NOT NULL,
    recipes_found TEXT,  -- JSON
    log_entries TEXT,    -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 实验记录表
CREATE TABLE experiments (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    attempts TEXT NOT NULL,  -- JSON
    status TEXT NOT NULL,
    final_recipe TEXT,       -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 记忆碎片表
CREATE TABLE memory_fragments (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    fragment_type TEXT NOT NULL,  -- Grandfather/Customer/Travel/Experiment/Emotional
    unlocked INTEGER NOT NULL,
    unlock_condition TEXT NOT NULL,  -- JSON
    unlocked_at TEXT,
    is_read INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 事件日志表
CREATE TABLE event_logs (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    content TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- ==================== 新增系统表 ====================

-- 天气状态表
CREATE TABLE weather_states (
    save_id TEXT PRIMARY KEY,
    current_weather TEXT NOT NULL,
    season TEXT NOT NULL,
    last_update TEXT NOT NULL,
    forecast TEXT NOT NULL,          -- Vec<WeatherForecast> JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 节假日状态表
CREATE TABLE festival_states (
    save_id TEXT PRIMARY KEY,
    current_festival TEXT,           -- Festival JSON
    upcoming_festivals TEXT NOT NULL,-- Vec<Festival> JSON
    festival_history TEXT NOT NULL,  -- Vec<FestivalRecord> JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 邻居表
CREATE TABLE neighbors (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    profession TEXT NOT NULL,
    personality TEXT NOT NULL,
    relationship INTEGER NOT NULL DEFAULT 0,
    interaction_count INTEGER NOT NULL DEFAULT 0,
    last_interaction TEXT,
    skills TEXT NOT NULL,            -- Vec<NeighborSkill> JSON
    available_help TEXT NOT NULL,    -- Vec<HelpType> JSON
    trade_options TEXT NOT NULL,     -- Vec<TradeOption> JSON
    backstory TEXT NOT NULL,
    connection_to_grandfather TEXT,
    schedule TEXT NOT NULL,          -- NeighborSchedule JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 邻里互动记录表
CREATE TABLE neighbor_interactions (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    neighbor_id TEXT NOT NULL,
    interaction_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    outcome TEXT NOT NULL,           -- InteractionOutcome JSON
    relationship_change INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (neighbor_id) REFERENCES neighbors(id)
);

-- 邻里请求表
CREATE TABLE neighbor_requests (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    neighbor_id TEXT NOT NULL,
    request_type TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL,
    deadline TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (neighbor_id) REFERENCES neighbors(id)
);

-- 供应商表
CREATE TABLE suppliers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    supplier_type TEXT NOT NULL,
    description TEXT NOT NULL,
    available_ingredients TEXT NOT NULL, -- Vec<IngredientOffering> JSON
    min_order_quantity INTEGER NOT NULL,
    max_order_quantity INTEGER NOT NULL,
    reliability INTEGER NOT NULL,
    price_tier TEXT NOT NULL,
    quality_range_min INTEGER NOT NULL,
    quality_range_max INTEGER NOT NULL,
    delivery_time_hours INTEGER NOT NULL,
    unlock_condition TEXT,           -- SupplierUnlockCondition JSON
    relationship_required INTEGER NOT NULL DEFAULT 0
);

-- 已解锁供应商表
CREATE TABLE unlocked_suppliers (
    save_id TEXT NOT NULL,
    supplier_id TEXT NOT NULL,
    unlocked_at TEXT NOT NULL,
    PRIMARY KEY (save_id, supplier_id),
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
);

-- 供应合同表
CREATE TABLE supply_contracts (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    supplier_id TEXT NOT NULL,
    ingredient_type TEXT NOT NULL,
    quantity_per_week INTEGER NOT NULL,
    negotiated_price TEXT NOT NULL,  -- Decimal
    start_date TEXT NOT NULL,
    end_date TEXT,
    discount REAL NOT NULL,
    reliability_bonus INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
);

-- 供应订单表
CREATE TABLE supply_orders (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    supplier_id TEXT NOT NULL,
    items TEXT NOT NULL,             -- Vec<OrderItem> JSON
    total_cost TEXT NOT NULL,        -- Decimal
    ordered_at TEXT NOT NULL,
    expected_delivery TEXT NOT NULL,
    actual_delivery TEXT,
    status TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
);

-- 成就表
CREATE TABLE achievements (
    id TEXT PRIMARY KEY,             -- 成就定义 ID
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    condition TEXT NOT NULL,         -- AchievementCondition JSON
    reward TEXT NOT NULL,            -- AchievementReward JSON
    points INTEGER NOT NULL,
    hidden INTEGER NOT NULL DEFAULT 0,
    icon TEXT
);

-- 已解锁成就表
CREATE TABLE unlocked_achievements (
    save_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TEXT NOT NULL,
    snapshot TEXT NOT NULL,          -- GameSnapshot JSON
    PRIMARY KEY (save_id, achievement_id),
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (achievement_id) REFERENCES achievements(id)
);

-- 成就进度表
CREATE TABLE achievement_progress (
    save_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    current_value INTEGER NOT NULL,
    target_value INTEGER NOT NULL,
    percentage REAL NOT NULL,
    PRIMARY KEY (save_id, achievement_id),
    FOREIGN KEY (save_id) REFERENCES saves(id),
    FOREIGN KEY (achievement_id) REFERENCES achievements(id)
);

-- 教程状态表
CREATE TABLE tutorial_states (
    save_id TEXT PRIMARY KEY,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 0,
    current_step TEXT,
    completed_steps TEXT NOT NULL,   -- Vec<String> JSON
    skipped INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 游戏统计表
CREATE TABLE game_statistics (
    save_id TEXT PRIMARY KEY,
    -- 经营统计
    total_customers_served INTEGER NOT NULL DEFAULT 0,
    total_dishes_sold INTEGER NOT NULL DEFAULT 0,
    total_revenue TEXT NOT NULL,     -- Decimal
    total_expenses TEXT NOT NULL,    -- Decimal
    best_selling_dish TEXT,
    best_day_revenue_date TEXT,
    best_day_revenue_amount TEXT,
    -- 烹饪统计
    dishes_cooked TEXT NOT NULL,     -- HashMap<RecipeId, u64> JSON
    perfect_dishes INTEGER NOT NULL DEFAULT 0,
    failed_dishes INTEGER NOT NULL DEFAULT 0,
    experiments_conducted INTEGER NOT NULL DEFAULT 0,
    experiments_succeeded INTEGER NOT NULL DEFAULT 0,
    -- 旅行统计
    total_travels INTEGER NOT NULL DEFAULT 0,
    destinations_visited TEXT NOT NULL, -- Vec<DestinationId> JSON
    recipes_found INTEGER NOT NULL DEFAULT 0,
    rare_materials_found INTEGER NOT NULL DEFAULT 0,
    -- 社交统计
    customers_at_max_favorability INTEGER NOT NULL DEFAULT 0,
    neighbors_at_max_relationship INTEGER NOT NULL DEFAULT 0,
    memories_unlocked INTEGER NOT NULL DEFAULT 0,
    -- 时间统计
    play_time_hours REAL NOT NULL DEFAULT 0,
    in_game_days INTEGER NOT NULL DEFAULT 0,
    commands_sent INTEGER NOT NULL DEFAULT 0,
    -- 每日记录（最近30天）
    daily_records TEXT NOT NULL,     -- VecDeque<DailyStatistics> JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- ==================== 索引设计 ====================

-- 指令队列索引
CREATE INDEX idx_commands_save_arrive ON command_queue(save_id, arrive_at);
CREATE INDEX idx_commands_status ON command_queue(status);

-- 模块索引
CREATE INDEX idx_modules_save_type ON modules(save_id, module_type);

-- 设施索引
CREATE INDEX idx_facilities_save_zone ON shop_facilities(save_id, zone);
CREATE INDEX idx_facilities_type ON shop_facilities(facility_type);

-- 菜谱索引
CREATE INDEX idx_recipes_save_status ON recipes(save_id, status);
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine_style);

-- 记忆碎片索引
CREATE INDEX idx_memory_save_unlocked ON memory_fragments(save_id, unlocked);
CREATE INDEX idx_memory_type ON memory_fragments(fragment_type);

-- 事件日志索引
CREATE INDEX idx_events_save_type ON event_logs(save_id, event_type);
CREATE INDEX idx_events_time ON event_logs(occurred_at);

-- 顾客索引
CREATE INDEX idx_customers_save_favorability ON customers(save_id, favorability);

-- 邻居索引
CREATE INDEX idx_neighbors_save_relationship ON neighbors(save_id, relationship);

-- 订单索引
CREATE INDEX idx_orders_status ON supply_orders(status);
CREATE INDEX idx_orders_expected ON supply_orders(expected_delivery);

-- 成就索引
CREATE INDEX idx_achievements_category ON achievements(category);
CREATE INDEX idx_unlocked_save ON unlocked_achievements(save_id);
```

---

## 十、API 设计

### 10.0 健康检查 API

健康检查 API 用于 systemd 监控和运维管理，无需认证。

```
# ==================== 健康检查 ====================

# 基础健康检查（systemd 使用）
GET /health
Response: {
    "status": "healthy" | "degraded" | "unhealthy",
    "uptime_seconds": 3600,
    "version": "1.0.0"
}

# 详细健康检查（运维使用）
GET /health/detail
Response: {
    "status": "healthy",
    "uptime_seconds": 3600,
    "version": "1.0.0",
    "components": {
        "database": {
            "status": "healthy",
            "latency_ms": 5,
            "connections": 2
        },
        "llm": {
            "status": "healthy",
            "provider": "ollama",
            "model": "qwen2.5:7b",
            "last_success": "2024-01-15T10:30:00Z"
        },
        "memory": {
            "status": "healthy",
            "used_mb": 150,
            "available_mb": 500
        }
    },
    "active_saves": 3,
    "active_connections": 1
}

# 就绪检查（Kubernetes/负载均衡使用）
GET /ready
Response: {
    "ready": true,
    "checks": {
        "database": true,
        "llm": true,
        "config": true
    }
}

# 存活检查
GET /live
Response: {
    "alive": true
}
```

#### 健康状态定义

| 状态 | 说明 | systemd 处理 |
|------|------|-------------|
| `healthy` | 所有组件正常 | 继续运行 |
| `degraded` | 部分非关键组件异常（如 LLM 不可用但有降级） | 继续运行，记录警告 |
| `unhealthy` | 关键组件异常（数据库损坏等） | 重启服务 |

#### 健康检查实现

```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ComponentStatus,
    pub uptime_seconds: u64,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<ComponentsHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_saves: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_connections: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentsHealth {
    pub database: DatabaseHealth,
    pub llm: LlmHealth,
    pub memory: MemoryHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: ComponentStatus,
    pub latency_ms: u64,
    pub connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmHealth {
    pub status: ComponentStatus,
    pub provider: String,
    pub model: String,
    pub last_success: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub status: ComponentStatus,
    pub used_mb: u64,
    pub available_mb: u64,
}

/// 健康检查服务
pub struct HealthChecker {
    start_time: Instant,
    db_pool: SqlitePool,
    llm_manager: Arc<LlmManager>,
    config: Arc<Config>,
}

impl HealthChecker {
    /// 基础健康检查
    pub async fn check_basic(&self) -> HealthStatus {
        let db_health = self.check_database().await;

        let status = if db_health.status == ComponentStatus::Unhealthy {
            ComponentStatus::Unhealthy
        } else if db_health.status == ComponentStatus::Degraded {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Healthy
        };

        HealthStatus {
            status,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: self.config.version.clone(),
            components: None,
            active_saves: None,
            active_connections: None,
        }
    }

    /// 详细健康检查
    pub async fn check_detail(&self) -> HealthStatus {
        let database = self.check_database().await;
        let llm = self.check_llm().await;
        let memory = self.check_memory().await;

        // 综合判断状态
        let status = if database.status == ComponentStatus::Unhealthy {
            ComponentStatus::Unhealthy
        } else if llm.status == ComponentStatus::Unhealthy {
            // LLM 不可用但有降级策略，标记为 degraded
            ComponentStatus::Degraded
        } else if database.status == ComponentStatus::Degraded
            || llm.status == ComponentStatus::Degraded
            || memory.status == ComponentStatus::Degraded
        {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Healthy
        };

        HealthStatus {
            status,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: self.config.version.clone(),
            components: Some(ComponentsHealth { database, llm, memory }),
            active_saves: Some(self.count_active_saves().await),
            active_connections: Some(self.count_active_connections().await),
        }
    }

    /// 检查数据库健康
    async fn check_database(&self) -> DatabaseHealth {
        let start = std::time::Instant::now();

        // 执行简单查询测试连接
        match sqlx::query("SELECT 1").fetch_one(&self.db_pool).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                let status = if latency < 100 {
                    ComponentStatus::Healthy
                } else if latency < 500 {
                    ComponentStatus::Degraded
                } else {
                    ComponentStatus::Unhealthy
                };

                DatabaseHealth {
                    status,
                    latency_ms: latency,
                    connections: self.db_pool.size() as u32,
                }
            }
            Err(e) => {
                tracing::error!("Database health check failed: {}", e);
                DatabaseHealth {
                    status: ComponentStatus::Unhealthy,
                    latency_ms: 0,
                    connections: 0,
                }
            }
        }
    }

    /// 检查 LLM 服务健康
    async fn check_llm(&self) -> LlmHealth {
        match self.llm_manager.health_check().await {
            Ok(info) => LlmHealth {
                status: ComponentStatus::Healthy,
                provider: info.provider,
                model: info.model,
                last_success: Some(info.last_success),
            },
            Err(e) => {
                tracing::warn!("LLM health check failed: {}", e);
                LlmHealth {
                    status: ComponentStatus::Degraded, // 降级但可用
                    provider: self.llm_manager.provider_name().to_string(),
                    model: self.llm_manager.model_name().to_string(),
                    last_success: self.llm_manager.last_success_time(),
                }
            }
        }
    }

    /// 检查内存使用
    fn check_memory(&self) -> MemoryHealth {
        let used = self.get_memory_usage();
        let available = self.get_available_memory();

        let status = if used < available * 80 / 100 {
            ComponentStatus::Healthy
        } else if used < available * 95 / 100 {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Unhealthy
        };

        MemoryHealth {
            status,
            used_mb: used / 1024 / 1024,
            available_mb: available / 1024 / 1024,
        }
    }

    /// 就绪检查
    pub async fn check_ready(&self) -> ReadyStatus {
        let db_ready = self.check_database().await.status != ComponentStatus::Unhealthy;
        let llm_ready = self.llm_manager.is_available().await;
        let config_ready = self.config.is_valid();

        ReadyStatus {
            ready: db_ready && config_ready,
            checks: ChecksStatus {
                database: db_ready,
                llm: llm_ready,
                config: config_ready,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyStatus {
    pub ready: bool,
    pub checks: ChecksStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksStatus {
    pub database: bool,
    pub llm: bool,
    pub config: bool,
}

// API 路由
pub fn health_routes() -> Router<HealthChecker> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/health/detail", get(health_detail_handler))
        .route("/ready", get(ready_handler))
        .route("/live", get(live_handler))
}

async fn health_handler(
    State(checker): State<HealthChecker>,
) -> Json<HealthStatus> {
    Json(checker.check_basic().await)
}

async fn health_detail_handler(
    State(checker): State<HealthChecker>,
) -> Json<HealthStatus> {
    Json(checker.check_detail().await)
}

async fn ready_handler(
    State(checker): State<HealthChecker>,
) -> Json<ReadyStatus> {
    Json(checker.check_ready().await)
}

async fn live_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "alive": true }))
}
```

#### systemd 配置

```ini
# /etc/systemd/system/flavors-game.service
[Unit]
Description=Flavors Across Two Decades Game Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/flavors-game
Restart=on-failure
RestartSec=10

# 健康检查配置
ExecStartPre=/usr/bin/curl -f http://localhost:8080/ready || exit 1
ExecReload=/bin/kill -HUP $MAINPID

# 资源限制
LimitNOFILE=65536
MemoryMax=512M

[Install]
WantedBy=multi-user.target
```

### 10.1 HTTP REST API

```
# ==================== 存档管理 ====================
POST   /api/v1/saves                      # 创建新存档
GET    /api/v1/saves                      # 获取存档列表
GET    /api/v1/saves/:id                  # 获取存档状态
DELETE /api/v1/saves/:id                  # 删除存档

# ==================== 指令系统 ====================
POST   /api/v1/saves/:id/commands         # 发送指令
GET    /api/v1/saves/:id/commands         # 获取指令历史

# ==================== 盼盼系统 ====================
GET    /api/v1/saves/:id/panpan                   # 获取盼盼完整状态
GET    /api/v1/saves/:id/panpan/modules           # 获取模块详情
POST   /api/v1/saves/:id/panpan/modules/:type/upgrade   # 升级模块
POST   /api/v1/saves/:id/panpan/modules/:type/repair    # 修复模块
GET    /api/v1/saves/:id/panpan/emotion           # 获取当前情绪
GET    /api/v1/saves/:id/panpan/trust             # 获取信任度
POST   /api/v1/saves/:id/panpan/charge            # 开始/停止充电
GET    /api/v1/saves/:id/panpan/memories          # 获取记忆碎片

# ==================== 小馆基础 ====================
GET    /api/v1/saves/:id/shop                      # 获取小馆完整状态
PATCH  /api/v1/saves/:id/shop                      # 更新小馆基础信息
POST   /api/v1/saves/:id/shop/open                 # 开店营业
POST   /api/v1/saves/:id/shop/close                # 打烊休息

# ==================== 设施管理 ====================
GET    /api/v1/saves/:id/shop/facilities           # 获取所有设施
GET    /api/v1/saves/:id/shop/facilities/:zone     # 获取指定区域设施
POST   /api/v1/saves/:id/shop/facilities/:id/upgrade  # 升级设施
POST   /api/v1/saves/:id/shop/facilities/:id/repair   # 修复设施

# ==================== 修复进度 ====================
GET    /api/v1/saves/:id/shop/restoration          # 获取修复进度
GET    /api/v1/saves/:id/shop/restoration/:zone    # 获取指定区域进度
GET    /api/v1/saves/:id/shop/milestones           # 获取所有里程碑

# ==================== 财务管理 ====================
GET    /api/v1/saves/:id/shop/finance              # 获取财务状态
GET    /api/v1/saves/:id/shop/finance/daily        # 获取今日财务
GET    /api/v1/saves/:id/shop/finance/history      # 获取财务历史

# ==================== 顾客管理 ====================
GET    /api/v1/saves/:id/shop/customers            # 获取顾客统计
GET    /api/v1/saves/:id/shop/customers/today      # 获取今日客流

# ==================== 菜谱管理 ====================
GET    /api/v1/saves/:id/shop/recipes              # 获取所有菜谱
GET    /api/v1/saves/:id/shop/recipes/available    # 获取可用菜谱
POST   /api/v1/saves/:id/shop/recipes              # 添加新菜谱
PATCH  /api/v1/saves/:id/shop/recipes/:recipe_id   # 更新菜谱状态

# ==================== 菜单管理 ====================
GET    /api/v1/saves/:id/shop/menu                 # 获取今日菜单
POST   /api/v1/saves/:id/shop/menu                 # 设置今日菜单
POST   /api/v1/saves/:id/shop/menu/specials        # 设置今日推荐

# ==================== 口碑环境 ====================
GET    /api/v1/saves/:id/shop/reputation           # 获取口碑状态
GET    /api/v1/saves/:id/shop/atmosphere           # 获取环境氛围
PATCH  /api/v1/saves/:id/shop/atmosphere           # 更新环境设置
POST   /api/v1/saves/:id/shop/atmosphere/clean     # 执行清洁

# ==================== 研发线索 ====================
GET    /api/v1/saves/:id/shop/research             # 获取研发线索
POST   /api/v1/saves/:id/shop/research/:clue_id/discover  # 发现线索

# ==================== 天气系统 ====================
GET    /api/v1/saves/:id/weather                    # 获取当前天气
GET    /api/v1/saves/:id/weather/forecast           # 获取天气预报
GET    /api/v1/saves/:id/weather/season             # 获取当前季节

# ==================== 节假日系统 ====================
GET    /api/v1/saves/:id/festivals                  # 获取节日列表
GET    /api/v1/saves/:id/festivals/today            # 获取今日节日
GET    /api/v1/saves/:id/festivals/upcoming         # 获取即将到来的节日

# ==================== 邻里系统 ====================
GET    /api/v1/saves/:id/neighbors                  # 获取邻居列表
GET    /api/v1/saves/:id/neighbors/:neighbor_id     # 获取邻居详情
POST   /api/v1/saves/:id/neighbors/:neighbor_id/interact  # 与邻居互动
POST   /api/v1/saves/:id/neighbors/:neighbor_id/gift      # 赠送礼物
GET    /api/v1/saves/:id/neighbors/:neighbor_id/events    # 获取邻居事件

# ==================== 供应商系统 ====================
GET    /api/v1/saves/:id/suppliers                  # 获取供应商列表
GET    /api/v1/saves/:id/suppliers/:supplier_id     # 获取供应商详情
POST   /api/v1/saves/:id/suppliers/:supplier_id/order     # 下订单
GET    /api/v1/saves/:id/suppliers/orders           # 获取订单列表
PATCH  /api/v1/saves/:id/suppliers/orders/:order_id # 更新订单状态

# ==================== 成就系统 ====================
GET    /api/v1/saves/:id/achievements               # 获取成就列表
GET    /api/v1/saves/:id/achievements/unlocked      # 获取已解锁成就
POST   /api/v1/saves/:id/achievements/:achievement_id/claim  # 领取成就奖励
GET    /api/v1/saves/:id/achievements/progress      # 获取成就进度

# ==================== 教程系统 ====================
GET    /api/v1/saves/:id/tutorial                   # 获取教程状态
POST   /api/v1/saves/:id/tutorial/complete/:step    # 完成教程步骤
POST   /api/v1/saves/:id/tutorial/skip              # 跳过教程
GET    /api/v1/saves/:id/tutorial/hint              # 获取当前上下文提示

# ==================== 统计系统 ====================
GET    /api/v1/saves/:id/statistics                 # 获取统计概览
GET    /api/v1/saves/:id/statistics/finance         # 获取财务统计
GET    /api/v1/saves/:id/statistics/customers       # 获取客流统计
GET    /api/v1/saves/:id/statistics/dishes          # 获取菜品统计
GET    /api/v1/saves/:id/statistics/trends          # 获取趋势数据

# ==================== 其他系统 ====================
GET    /api/v1/saves/:id/travels           # 获取旅行记录
GET    /api/v1/saves/:id/experiments       # 获取实验记录
GET    /api/v1/saves/:id/reports           # 获取盼盼简报
GET    /api/v1/saves/:id/events            # 获取事件日志
```

### 10.2 WebSocket API

```rust
// 连接: ws://host/api/v1/saves/:id/ws

// 客户端 -> 服务端
enum ClientMessage {
    SendCommand { content: String },
    RequestSync,
}

// 服务端 -> 客户端
enum ServerMessage {
    CommandArrived { command_id: Uuid },
    CommandExecuted { command_id: Uuid, result: CommandResult },
    NewReport { report: Report },
    EventOccurred { event: GameEvent },
    PanpanStatusUpdate { status: PanpanState },
    TimeSync { earth_time: DateTime<Utc>, delay_minutes: u32 },
    // 新系统消息
    WeatherChanged { weather: WeatherState },
    FestivalStarted { festival: Festival },
    FestivalEnded { festival_id: String },
    NeighborEvent { neighbor_id: String, event: NeighborEvent },
    OrderDelivered { order: SupplyOrder },
    AchievementUnlocked { achievement: Achievement },
    TutorialStepCompleted { step: String, next_step: Option<String> },
    StatisticsUpdated { category: StatCategory, data: StatData },
}
```

---

## 十一、通信延迟系统实现

### 11.1 延迟计算（与模块关联）

通信延迟由两部分组成：
1. **基础延迟**：火星-地球物理距离决定（不可改变）
2. **模块延迟**：盼盼通信模块等级决定（可升级改善）

```rust
pub struct CommunicationSystem {
    /// 火星-地球距离（万公里）
    distance: f64,
    /// 光速（万公里/分钟）
    light_speed: f64,
    /// 通信模块引用
    comm_module: Module,
}

impl CommunicationSystem {
    /// 计算物理基础延迟（光速限制，不可改变）
    pub fn calculate_base_delay(&self) -> u32 {
        let delay = self.distance / self.light_speed;
        // 加上网络处理延迟（1-3分钟）
        let processing_delay = 1.0 + rand::random::<f64>() * 2.0;
        (delay + processing_delay).ceil() as u32
    }

    /// 计算模块附加延迟（可通过升级减少）
    pub fn calculate_module_penalty(&self) -> u32 {
        let base_penalty = match self.comm_module.level {
            1 => rand::thread_rng().gen_range(15..=20),
            2 => rand::thread_rng().gen_range(10..=15),
            3 => rand::thread_rng().gen_range(5..=10),
            4 => rand::thread_rng().gen_range(3..=6),
            5 => rand::thread_rng().gen_range(2..=4),
            _ => 20,
        };
        // 完好度影响
        (base_penalty as f32 * self.comm_module.condition) as u32
    }

    /// 计算总通信延迟
    pub fn calculate_total_delay(&self) -> u32 {
        self.calculate_base_delay() + self.calculate_module_penalty()
    }

    /// 获取延迟描述文本
    pub fn get_delay_description(&self) -> String {
        let level = self.comm_module.level;
        let condition = self.comm_module.condition;

        match level {
            1 => format!(
                "通信模块严重故障（完好度: {:.0}%），信号极差，大量数据丢失。总延迟: {} 分钟",
                condition * 100.0,
                self.calculate_total_delay()
            ),
            2 => format!(
                "通信模块老化（完好度: {:.0}%），信号不稳定。总延迟: {} 分钟",
                condition * 100.0,
                self.calculate_total_delay()
            ),
            3 => format!(
                "通信模块正常（完好度: {:.0}%）。总延迟: {} 分钟",
                condition * 100.0,
                self.calculate_total_delay()
            ),
            4 => format!(
                "通信模块良好（完好度: {:.0}%），信号稳定。总延迟: {} 分钟",
                condition * 100.0,
                self.calculate_total_delay()
            ),
            5 => format!(
                "通信模块优秀（完好度: {:.0}%），量子通信原型！总延迟: {} 分钟",
                condition * 100.0,
                self.calculate_total_delay()
            ),
            _ => "未知状态".to_string(),
        }
    }
}
```

### 11.2 延迟示意

| 游戏阶段 | 通信模块等级 | 基础延迟 | 模块延迟 | 总延迟 |
|---------|------------|---------|---------|--------|
| 初期 | 1级 (30%完好) | 4-6 分钟 | 5-6 分钟 | **9-12 分钟** |
| 中期 | 3级 (80%完好) | 4-6 分钟 | 4-8 分钟 | **8-14 分钟** |
| 后期 | 5级 (100%完好) | 4-6 分钟 | 2-4 分钟 | **6-10 分钟** |

> 注：基础延迟会随火星-地球距离变化（4-24分钟），模块延迟叠加其上。

### 11.3 指令队列管理

```rust
pub struct CommandQueue {
    pending: VecDeque<PendingCommand>,
    delay_calculator: CommunicationDelay,
}

impl CommandQueue {
    /// 接收新指令
    pub fn receive_command(&mut self, command: Command) {
        let delay = self.delay_calculator.calculate_delay();
        let arrive_at = Utc::now() + Duration::minutes(delay as i64);

        self.pending.push_back(PendingCommand {
            command,
            arrive_at,
        });
    }

    /// 每帧检查并执行到达的指令
    pub fn process_arrived_commands(&mut self) -> Vec<Command> {
        let now = Utc::now();
        let mut arrived = Vec::new();

        while let Some(pending) = self.pending.front() {
            if pending.arrive_at <= now {
                arrived.push(self.pending.pop_front().unwrap().command);
            } else {
                break;
            }
        }

        arrived
    }
}
```

---

## 十二、项目目录结构

```
backend/
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
│   ├── lib.rs
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── saves.rs
│   │   │   ├── commands.rs
│   │   │   ├── panpan.rs
│   │   │   └── reports.rs
│   │   └── websocket.rs
│   │
│   ├── core/
│   │   ├── mod.rs
│   │   ├── engine.rs          # 游戏引擎主循环
│   │   ├── command.rs         # 指令处理
│   │   ├── event.rs           # 事件系统
│   │   ├── time.rs            # 时间系统
│   │   └── delay.rs           # 通信延迟
│   │
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── panpan/
│   │   │   ├── mod.rs
│   │   │   ├── modules.rs     # 模块系统（整合技能）
│   │   │   ├── personality.rs # 性格轴系统
│   │   │   ├── emotion.rs     # 情绪系统
│   │   │   ├── energy.rs      # 能量系统
│   │   │   ├── trust.rs       # 信任度系统
│   │   │   └── state.rs       # 状态管理
│   │   ├── shop.rs            # 小馆经营系统
│   │   ├── travel.rs          # 旅行收集系统
│   │   ├── recipe_lab.rs      # 实验研发系统
│   │   ├── memory.rs          # 记忆碎片系统
│   │   ├── garden.rs          # 后院种植系统
│   │   ├── customer.rs        # 顾客系统
│   │   └── event.rs           # 事件系统
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── player.rs
│   │   ├── save.rs
│   │   ├── panpan/
│   │   │   ├── mod.rs
│   │   │   ├── module.rs         # 模块定义
│   │   │   ├── personality.rs    # 性格轴
│   │   │   ├── emotion.rs        # 情绪系统
│   │   │   ├── energy.rs         # 能量系统
│   │   │   ├── trust.rs          # 信任度
│   │   │   └── state.rs          # 状态定义
│   │   ├── shop/
│   │   │   ├── mod.rs            # 小馆系统主模块
│   │   │   ├── facility.rs       # 设施管理
│   │   │   ├── restoration.rs    # 修复进度管理
│   │   │   ├── finance.rs        # 财务管理
│   │   │   ├── customer.rs       # 顾客管理
│   │   │   ├── recipe.rs         # 菜谱管理
│   │   │   ├── menu.rs           # 菜单管理
│   │   │   ├── reputation.rs     # 口碑计算
│   │   │   ├── atmosphere.rs     # 环境氛围
│   │   │   └── research.rs       # 研发树管理
│   │   ├── travel.rs
│   │   ├── recipe_lab.rs
│   │   ├── memory.rs
│   │   ├── garden.rs
│   │   └── event.rs
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── saves.rs
│   │   │   ├── commands.rs
│   │   │   └── game_state.rs
│   │   └── migrations.rs
│   │
│   └── utils/
│       ├── mod.rs
│       └── time.rs
│
└── assets/
    ├── recipes/               # 初始菜谱数据
    ├── memories/              # 记忆碎片内容
    ├── events/                # 事件定义
    └── images/                # Kitty 图像资源
```

---

## 十三、开发阶段规划

### Phase 1: 基础框架（第1-2周）
- [ ] 项目初始化，配置 Cargo.toml
- [ ] 实现配置管理模块
- [ ] 实现数据库连接和迁移
- [ ] 实现 HTTP API 框架
- [ ] 实现 WebSocket 连接

### Phase 2: 核心系统（第3-4周）
- [ ] 实现时间系统和通信延迟
- [ ] 实现指令队列系统
- [ ] 实现事件系统基础
- [ ] 实现游戏引擎主循环

### Phase 3: 游戏子系统（第5-8周）
- [ ] 实现盼盼系统（状态、性格）
- [ ] 实现小馆经营系统
- [ ] 实现旅行收集系统
- [ ] 实现实验研发系统
- [ ] 实现记忆碎片系统
- [ ] 实现后院种植系统
- [ ] 实现顾客系统
- [ ] 实现事件系统（完整版）

### Phase 4: 整合与测试（第9-10周）
- [ ] 子系统整合
- [ ] 实现完整游戏循环
- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 性能优化

### Phase 5: 部署（第11周）
- [ ] systemd 服务配置
- [ ] 生产环境配置
- [ ] 部署文档

---

## 十四、验证方案

### 14.1 开发阶段验证
1. 使用 `cargo test` 运行单元测试
2. 使用 `curl` 或 Postman 测试 HTTP API
3. 使用 `wscat` 测试 WebSocket 连接

### 14.2 功能验证
1. 创建新存档，验证数据库写入
2. 发送指令，验证延迟队列工作正常
3. 触发事件，验证事件系统响应
4. 模拟时间流逝，验证各子系统状态更新

### 14.3 性能验证
1. 压力测试 API 端点
2. 验证长时间运行稳定性
3. 验证内存使用情况

---

## 十五、已确认的设计决策

| 问题 | 决策 |
|------|------|
| 盼盼 AI 决策 | 接入 Ollama（第一版），盼盼所有行为由 AI 控制 |
| 存档机制 | 多存档，后端提供 CRUD API，前端管理存档选择 |
| **数据持久化** | 不使用 SaveState 聚合模型，所有实体独立存储，通过 save_id 关联，实时写入 SQLite |
| 图像资源 | 暂不实现，后续迭代，第一版使用文字描述 |
| 时间控制 | 正常模式 1:1 同步，测试模式可配置加速倍率（默认 10 倍）|
| **模块系统** | 7 个模块，等级 1-10，整合技能和健康度（完好度 0-100）|
| **通信延迟** | 基础延迟（物理）+ 模块延迟（可升级），随盼盼升级而降低 |
| **性格轴** | 3 维，范围 0-100，初始 50 |
| **信任度** | 0-100，初始 50，影响记忆恢复和主动行为 |
| **情绪系统** | 完整实现 7 种情绪（开心/平静/疲惫/困惑/担忧/孤独/兴奋）|
| **能量系统** | 0-100，不同活动消耗不同，可充电 |
| **记忆容量** | 初始 100，最大 500，5 种碎片类型 |
| **小馆区域** | 4 大区域（餐厅/厨房/后院/工坊），每区域独立等级 1-5 |
| **设施数量** | 20+ 子设施，每个有等级（1-5）和完好度（0-100%）|
| **口碑系统** | 5 维度加权计算（菜品 40%、服务 20%、环境 15%、邻里 15%、老顾客 10%）|
| **氛围指数** | 5 子项加权（照明 25%、温度 20%、清洁 20%、装饰 20%、音乐 15%）|
| **客流计算** | 口碑 + 季节 + 氛围 + 座位翻台率综合计算 |
| **满意度计算** | 菜品 50% + 服务 30% + 环境 20% |
| **菜品体系** | 3 种来源（传承/旅行/创新），4 种状态（损坏/模糊/精确/掌握）|
| **升级系统** | 每个设施有独立升级路径，需资金/材料/时间/人员 |
| **里程碑** | 每区域有里程碑系统，完成解锁奖励 |
| **天气系统** | 4 种天气类型（晴/雨/雪/阴），4 季节循环，影响客流和种植 |
| **节假日系统** | 内置中国传统节日，特殊事件和客流加成，顾客行为变化 |
| **邻里系统** | 5+ 邻居角色，好感度 0-100，互助事件，可提供服务和材料 |
| **供应商系统** | 3 类供应商（食材/设备/杂货），品质/价格/配送时间权衡选择 |
| **成就系统** | 5 大类别（经营/探索/社交/烹饪/收集），隐藏成就和里程碑成就 |
| **教程系统** | 5 阶段引导（基础/进阶/高级/专家/隐藏），可跳过，上下文感知提示 |
| **统计系统** | 7 类统计数据（财务/客流/顾客/菜品/运营/里程碑/趋势），支持可视化 |
| **数据库索引** | 为高频查询字段建立索引（save_id、时间戳、类型字段等） |
| **人员管理** | 无员工系统，盼盼独立管理所有功能，体现机器人主角特色 |
| **种植系统** | 后院 5 级等级，5 块菜地，5 类作物（蔬菜/香料/花卉/特殊/异星） |
| **作物生长** | 5 阶段生长（播种/发芽/生长/成熟/枯萎），受季节/天气/肥力影响 |
| **病虫害系统** | 12 种病虫害类型，5 级严重程度，10 种治疗方法 |
| **种植自动化** | 后院 3 级解锁自动浇水，4 级解锁自动虫检，5 级解锁自动收获 |
| **园艺模块** | 盼盼园艺技能 1-10 级，影响播种成功率/生长速度/产量/自留种能力 |

---

## 十六、下一步

确认以上设计方案后，将按以下顺序实现：
1. 初始化 Cargo 项目
2. 实现基础配置和数据库模块
3. 实现 HTTP API 框架
4. 逐步实现各子系统
