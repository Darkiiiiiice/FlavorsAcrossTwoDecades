# Phase 2: 核心系统（第3-4周）

## 开发目标

- [ ] 实现时间系统和通信延迟
- [ ] 实现指令队列系统
- [ ] 实现事件系统基础
- [ ] 实现游戏引擎主循环

---

## 一、LLM 集成架构（盼盼 AI 决策系统）

### 1.1 设计理念

盼盼的所有行为决策由 LLM 驱动，包括：
- 解读玩家指令并决定执行方式
- 自主决定是否发起旅行
- 处理突发事件时的应对策略
- 与顾客的互动方式
- 实验研发时的策略选择
- 生成简报和日志的语气风格

### 1.2 LLM 服务抽象层

```rust
use async_trait::async_trait;
use futures::Stream;

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

### 1.3 Prompt 模板系统

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

### 1.4 决策流程

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

### 1.5 决策类型与 Prompt

| 决策场景 | Prompt 模板 | 输出格式 |
|---------|------------|---------|
| 玩家指令响应 | `command_decision.hbs` | JSON（执行计划+回复）|
| 自主行动 | `autonomous_action.hbs` | JSON（行动+理由）|
| 事件处理 | `event_response.hbs` | JSON（选择+影响）|
| 旅行决策 | `travel_decision.hbs` | JSON（目的地+理由）|
| 实验策略 | `experiment_strategy.hbs` | JSON（调整建议）|
| 简报生成 | `daily_report.hbs` | 自然语言文本 |
| 旅行日志 | `travel_log.hbs` | 自然语言文本 |

### 1.6 上下文管理

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

### 1.7 Ollama 配置

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

### 1.8 错误处理与降级

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

## 二、多存档系统设计

### 2.1 架构说明

**职责划分**：
- **前端**：负责存档的选择、创建、切换（通过命令参数）
- **后端**：提供存档的 CRUD API，管理数据持久化

### 2.2 存档数据模型

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

### 2.3 存档 API

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

### 2.4 数据库设计

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

### 2.5 数据持久化机制

**实时持久化**：由于不使用内存聚合模型，所有状态变更直接写入 SQLite，实现自动持久化：
- 每次状态变更直接写入对应数据库表
- 无需定时保存或手动保存
- 程序重启后数据自动恢复

**需要定期更新的数据**：
- `last_played` 时间戳（每次连接时更新）
- `play_time_seconds` 游戏时长（WebSocket 连接期间累加）

---

## 三、时间系统与通信延迟

### 3.1 时间同步策略

```rust
use chrono::{DateTime, Utc, TimeZone, FixedOffset};

/// 游戏时间系统
pub struct TimeSystem {
    /// 地球时区（东八区）
    earth_timezone: FixedOffset,
    /// 测试模式加速倍率（正常为1，测试时可设为10）
    time_scale: u32,
    /// 是否启用加速模式
    accelerated_mode: bool,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            earth_timezone: FixedOffset::east_opt(8 * 3600).unwrap(), // UTC+8
            time_scale: 1,
            accelerated_mode: false,
        }
    }

    /// 获取当前地球时间（东八区）
    pub fn earth_time(&self) -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&self.earth_timezone)
    }

    /// 切换加速模式
    pub fn toggle_accelerated_mode(&mut self, enabled: bool) {
        self.accelerated_mode = enabled;
        self.time_scale = if enabled { 10 } else { 1 };
    }

    /// 计算游戏内经过的时间（考虑加速）
    pub fn elapsed_game_time(&self, real_seconds: u64) -> u64 {
        real_seconds * self.time_scale as u64
    }
}
```

### 3.2 通信延迟计算

```rust
/// 火星-地球通信延迟计算
pub struct CommunicationDelay {
    /// 基础物理延迟（秒）
    base_delay_seconds: u32,
    /// 模块附加延迟（秒）
    module_delay_seconds: u32,
}

impl CommunicationDelay {
    /// 计算总延迟
    pub fn total_delay(&self) -> u32 {
        self.base_delay_seconds + self.module_delay_seconds
    }

    /// 根据模块等级更新延迟
    pub fn update_from_module(&mut self, communication_level: u32) {
        self.module_delay_seconds = match communication_level {
            1 => 45,
            2 => 40,
            3 => 35,
            4 => 30,
            5 => 25,
            6 => 20,
            7 => 15,
            8 => 10,
            9 => 5,
            10 => 1,
            _ => 45,
        };
    }
}
```

### 3.3 通信延迟示意

| 游戏阶段 | 通信模块等级 | 基础延迟 | 模块延迟 | 总延迟 |
|---------|-------------|---------|---------|--------|
| 初期 | 1 | 10s | 45s | 55s (55秒) |
| 初期 | 2 | 9s | 40s | 49s (49秒) |
| 初期 | 3 | 8s | 35s | 43s (43秒) |
| 初期 | 4 | 7s | 30s | 37s (37秒) |
| 初期 | 5 | 6s | 25s | 31s (31秒) |
| 初期 | 6 | 5s | 20s | 25s (25秒) |
| 初期 | 7 | 4s | 15s | 19s (19秒) |
| 初期 | 8 | 3s | 10s | 13s (13秒) |
| 初期 | 9 | 2s | 5s | 7s (7秒) |
| 初期 | 10 | 1s | 1s | 1s (1秒) |

---

## 四、指令队列系统

### 4.1 指令队列管理

```rust
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// 指令状态
#[derive(Debug, Clone)]
pub enum CommandStatus {
    Pending,          // 等待发送
    InTransit,        // 传输中
    Arrived,          // 已到达地球
    Processing,       // 盼盼处理中
    Completed,        // 已完成
    Failed(String),   // 失败
}

/// 玩家指令
#[derive(Debug, Clone)]
pub struct Command {
    pub id: Uuid,
    pub save_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub arrival_time: DateTime<Utc>,
    pub status: CommandStatus,
    pub result: Option<String>,
}

/// 指令队列
pub struct CommandQueue {
    pending_commands: VecDeque<Command>,
    delay_calculator: CommunicationDelay,
}

impl CommandQueue {
    /// 添加新指令
    pub fn add_command(&mut self, save_id: Uuid, content: String) -> Command {
        let delay = self.delay_calculator.total_delay();
        let now = Utc::now();

        let command = Command {
            id: Uuid::new_v4(),
            save_id,
            content,
            created_at: now,
            arrival_time: now + chrono::Duration::seconds(delay as i64),
            status: CommandStatus::Pending,
            result: None,
        };

        self.pending_commands.push_back(command.clone());
        command
    }

    /// 处理已到达的指令
    pub fn process_arrived(&mut self) -> Vec<Command> {
        let now = Utc::now();
        let mut arrived = Vec::new();

        while let Some(cmd) = self.pending_commands.front() {
            if cmd.arrival_time <= now {
                let mut cmd = self.pending_commands.pop_front().unwrap();
                cmd.status = CommandStatus::Arrived;
                arrived.push(cmd);
            } else {
                break;
            }
        }

        arrived
    }
}
```

### 4.2 指令 API

```
POST   /api/v1/saves/:id/commands       # 发送新指令
GET    /api/v1/saves/:id/commands       # 获取指令列表
GET    /api/v1/saves/:id/commands/:cmd_id # 获取指令详情
```

---

## 五、事件系统基础

### 5.1 事件类型

```rust
/// 游戏事件类型
#[derive(Debug, Clone)]
pub enum GameEventType {
    // 时间事件
    DailyReport,         // 每日简报
    CropMature,          // 作物成熟
    TravelReturn,        // 旅行归来

    // 触发事件
    CustomerVisit,       // 顾客到访
    NeighborInteraction, // 邻居互动
    EquipmentFailure,    // 设备故障

    // 特殊事件
    Festival,            // 节日
    MemoryUnlock,        // 记忆解锁
    Achievement,         // 成就达成
}

/// 游戏事件
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub id: Uuid,
    pub save_id: Uuid,
    pub event_type: GameEventType,
    pub trigger_time: DateTime<Utc>,
    pub data: serde_json::Value,
    pub processed: bool,
}
```

### 5.2 事件分发器

```rust
/// 事件分发器
pub struct EventDispatcher {
    pending_events: Vec<GameEvent>,
}

impl EventDispatcher {
    /// 添加事件
    pub fn schedule(&mut self, event: GameEvent) {
        self.pending_events.push(event);
    }

    /// 处理到期事件
    pub fn process_due_events(&mut self) -> Vec<GameEvent> {
        let now = Utc::now();
        let mut due_events = Vec::new();

        self.pending_events.retain(|event| {
            if event.trigger_time <= now && !event.processed {
                due_events.push(event.clone());
                false // 移除已处理事件
            } else {
                true
            }
        });

        due_events
    }
}
```

---

## 六、游戏引擎主循环

### 6.1 主循环设计

```rust
use tokio::time::{interval, Duration};

/// 游戏引擎
pub struct GameEngine {
    time_system: TimeSystem,
    command_queue: CommandQueue,
    event_dispatcher: EventDispatcher,
    subsystems: Subsystems,
}

impl GameEngine {
    /// 主循环
    pub async fn run(&mut self) {
        let mut tick_interval = interval(Duration::from_secs(1));

        loop {
            tick_interval.tick().await;

            // 1. 处理时间更新
            self.time_system.tick();

            // 2. 处理到达的指令
            let arrived_commands = self.command_queue.process_arrived();
            for cmd in arrived_commands {
                self.process_command(cmd).await;
            }

            // 3. 处理到期事件
            let due_events = self.event_dispatcher.process_due_events();
            for event in due_events {
                self.process_event(event).await;
            }

            // 4. 更新子系统状态
            self.subsystems.tick().await;
        }
    }

    /// 处理指令
    async fn process_command(&mut self, command: Command) {
        // 1. 调用 LLM 获取盼盼决策
        // 2. 执行决策
        // 3. 更新状态
        // 4. 生成反馈
    }

    /// 处理事件
    async fn process_event(&mut self, event: GameEvent) {
        match event.event_type {
            GameEventType::DailyReport => {
                // 生成每日简报
            }
            GameEventType::CropMature => {
                // 处理作物成熟
            }
            // ... 其他事件类型
        }
    }
}
```

### 6.2 子系统集成

```rust
/// 子系统集合
pub struct Subsystems {
    panpan: PanpanSystem,
    shop: ShopSystem,
    travel: TravelSystem,
    garden: GardenSystem,
    kitchen: KitchenSystem,
    memory: MemorySystem,
    customer: CustomerSystem,
}

impl Subsystems {
    /// 每帧更新
    pub async fn tick(&mut self) {
        self.panpan.tick();
        self.shop.tick();
        self.garden.tick();
        self.kitchen.tick();
        self.travel.tick();
        self.memory.tick();
        self.customer.tick();
    }
}
```

---

## 七、配置与切换

```toml
# config/default.toml
[game]
time_scale = 1
accelerated_mode = false

[communication]
base_delay_seconds = 60
default_module_level = 1

[server]
host = "127.0.0.1"
port = 8080
