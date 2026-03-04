# Phase 4: 整合与测试（第9-10周）

## 开发目标

- [ ] 子系统整合
- [ ] 实现完整游戏循环
- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 性能优化

---

## 一、数据模型设计

### 1.1 设计理念

**不使用 SaveState 聚合模型**：由于使用 SQLite 作为持久化存储，不需要内存中的大聚合对象。每个实体独立操作，更新时直接写入数据库。所有数据表通过 `save_id` 关联到存档。

**架构优势**：
- 避免内存与数据库双重维护
- 程序重启后数据自动恢复
- 简化数据同步逻辑
- SQLite 性能对单机游戏足够

### 1.2 核心实体

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

// 盼盼状态（对应 panpan_states 表）
struct PanpanState {
    save_id: Uuid,
    name: String,
    model: String,
    manufacture_date: DateTime<Utc>,
    personality: Personality,
    trust_level: u32,
    emotion: Emotion,
    energy: EnergySystem,
    location: Location,
    current_state: ActivityState,
    current_task: Option<Task>,
}

// 模块状态（对应 modules 表）
struct Module {
    id: Uuid,
    save_id: Uuid,
    module_type: ModuleType,
    level: u32,
    condition: u32,
    experience: u32,
    is_functional: bool,
}

// 性格轴（存储为 JSON）
struct Personality {
    business_style: u32,
    innovation: u32,
    independence: u32,
}

// 小馆状态（对应 shop_states 表）
struct ShopState {
    save_id: Uuid,
    name: String,
    funds: u64,
    reputation: f32,
    restaurant_level: u32,
    kitchen_level: u32,
    backyard_level: u32,
    workshop_level: u32,
}

// 设施状态（对应 facilities 表）
struct Facility {
    id: Uuid,
    save_id: Uuid,
    zone: FacilityZone,
    name: String,
    level: u32,
    condition: u32,
    upgrade_progress: Option<UpgradeProgress>,
}

// 菜地状态（对应 garden_plots 表）
struct GardenPlot {
    id: Uuid,
    save_id: Uuid,
    plot_number: u32,
    is_unlocked: bool,
    current_crop: Option<CropState>,
    fertility: u32,
    moisture: u32,
}

// 旅行状态（对应 travels 表）
struct Travel {
    id: Uuid,
    save_id: Uuid,
    destination: String,
    started_at: DateTime<Utc>,
    expected_return: DateTime<Utc>,
    status: TravelStatus,
    rewards: Option<TravelReward>,
}

// 记忆碎片（对应 memory_fragments 表）
struct MemoryFragment {
    id: Uuid,
    save_id: Uuid,
    fragment_type: MemoryType,
    title: String,
    content: String,
    is_unlocked: bool,
    unlocked_at: Option<DateTime<Utc>>,
    trigger_condition: String,
}

// 菜谱（对应 recipes 表）
struct Recipe {
    id: Uuid,
    save_id: Uuid,
    name: String,
    category: RecipeCategory,
    status: RecipeStatus,
    ingredients: Vec<IngredientAmount>,
    source: RecipeSource,
    unlock_condition: Option<String>,
}

// 顾客记录（对应 customers 表）
struct CustomerRecord {
    id: Uuid,
    save_id: Uuid,
    customer_type: String,
    name: String,
    favorability: u32,
    visit_count: u32,
    last_visit: DateTime<Utc>,
    preferences: Vec<String>,
}

// 指令记录（对应 commands 表）
struct Command {
    id: Uuid,
    save_id: Uuid,
    content: String,
    created_at: DateTime<Utc>,
    arrival_time: DateTime<Utc>,
    status: CommandStatus,
    result: Option<String>,
}

// 对话消息（对应 dialogues 表）
struct DialogueMessage {
    id: Uuid,
    save_id: Uuid,
    sender: DialogueSender,
    content: String,
    timestamp: DateTime<Utc>,
    message_type: MessageType,
    status: MessageStatus,
}
```

### 1.3 仓储层设计

```rust
use sqlx::SqlitePool;

/// 通用仓储 trait
#[async_trait]
pub trait Repository<T> {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>>;
    async fn find_by_save(&self, save_id: Uuid) -> Result<Vec<T>>;
    async fn save(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

/// 存档仓储
pub struct SaveRepository {
    pool: SqlitePool,
}

#[async_trait]
impl Repository<Save> for SaveRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Save>> {
        sqlx::query_as!(
            Save,
            "SELECT * FROM saves WHERE id = ?",
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(e.into()))
    }

    // ... 其他方法
}
```

### 1.4 数据库 Schema

```sql
-- 存档表
CREATE TABLE saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    player_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_played TEXT NOT NULL,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    chapter INTEGER NOT NULL DEFAULT 1
);

-- 盼盼状态表
CREATE TABLE panpan_states (
    save_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model TEXT NOT NULL,
    manufacture_date TEXT NOT NULL,
    personality TEXT NOT NULL,          -- JSON
    trust_level INTEGER NOT NULL,
    emotion TEXT NOT NULL,
    energy_current INTEGER NOT NULL,
    energy_max INTEGER NOT NULL,
    location TEXT NOT NULL,
    current_state TEXT NOT NULL,
    current_task TEXT,                  -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 模块表
CREATE TABLE modules (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    module_type TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    experience INTEGER NOT NULL,
    is_functional INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 小馆状态表
CREATE TABLE shop_states (
    save_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    funds INTEGER NOT NULL,
    reputation REAL NOT NULL,
    restaurant_level INTEGER NOT NULL,
    kitchen_level INTEGER NOT NULL,
    backyard_level INTEGER NOT NULL,
    workshop_level INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 设施表
CREATE TABLE facilities (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    zone TEXT NOT NULL,
    name TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    upgrade_progress TEXT,              -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 菜地表
CREATE TABLE garden_plots (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    plot_number INTEGER NOT NULL,
    is_unlocked INTEGER NOT NULL,
    current_crop TEXT,                  -- JSON
    fertility INTEGER NOT NULL,
    moisture INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 旅行表
CREATE TABLE travels (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    destination TEXT NOT NULL,
    started_at TEXT NOT NULL,
    expected_return TEXT NOT NULL,
    status TEXT NOT NULL,
    rewards TEXT,                       -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 记忆碎片表
CREATE TABLE memory_fragments (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    fragment_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    is_unlocked INTEGER NOT NULL,
    unlocked_at TEXT,
    trigger_condition TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 菜谱表
CREATE TABLE recipes (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    ingredients TEXT NOT NULL,          -- JSON
    source TEXT NOT NULL,
    unlock_condition TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 顾客记录表
CREATE TABLE customers (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    customer_type TEXT NOT NULL,
    name TEXT NOT NULL,
    favorability INTEGER NOT NULL,
    visit_count INTEGER NOT NULL,
    last_visit TEXT NOT NULL,
    preferences TEXT NOT NULL,          -- JSON
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 指令表
CREATE TABLE commands (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    arrival_time TEXT NOT NULL,
    status TEXT NOT NULL,
    result TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 对话表
CREATE TABLE dialogues (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id)
);

-- 创建索引
CREATE INDEX idx_modules_save ON modules(save_id);
CREATE INDEX idx_facilities_save ON facilities(save_id);
CREATE INDEX idx_garden_plots_save ON garden_plots(save_id);
CREATE INDEX idx_travels_save ON travels(save_id);
CREATE INDEX idx_memory_fragments_save ON memory_fragments(save_id);
CREATE INDEX idx_recipes_save ON recipes(save_id);
CREATE INDEX idx_customers_save ON customers(save_id);
CREATE INDEX idx_commands_save ON commands(save_id);
CREATE INDEX idx_commands_arrival ON commands(arrival_time);
CREATE INDEX idx_dialogues_save ON dialogues(save_id);
```

---

## 二、API 设计

### 2.1 HTTP REST API

```
# ========== 存档管理 ==========
POST   /api/v1/saves                    # 创建新存档
GET    /api/v1/saves                    # 获取存档列表
GET    /api/v1/saves/:id                # 获取存档详情
PATCH  /api/v1/saves/:id                # 更新存档元数据
DELETE /api/v1/saves/:id                # 删除存档
POST   /api/v1/saves/:id/autosave       # 触发自动保存
GET    /api/v1/saves/:id/export         # 导出存档
POST   /api/v1/saves/import             # 导入存档

# ========== 盼盼系统 ==========
GET    /api/v1/saves/:id/panpan         # 获取盼盼状态
PATCH  /api/v1/saves/:id/panpan         # 更新盼盼状态
GET    /api/v1/saves/:id/panpan/modules # 获取模块列表
PATCH  /api/v1/saves/:id/panpan/modules/:type # 更新模块

# ========== 指令系统 ==========
POST   /api/v1/saves/:id/commands       # 发送指令
GET    /api/v1/saves/:id/commands       # 获取指令列表
GET    /api/v1/saves/:id/commands/:cmd_id # 获取指令详情

# ========== 对话系统 ==========
POST   /api/v1/saves/:id/dialogues      # 发送消息
GET    /api/v1/saves/:id/dialogues      # 获取对话历史

# ========== 小馆系统 ==========
GET    /api/v1/saves/:id/shop           # 获取小馆状态
PATCH  /api/v1/saves/:id/shop           # 更新小馆状态
GET    /api/v1/saves/:id/shop/facilities # 获取设施列表
POST   /api/v1/saves/:id/shop/facilities/:facility_id/upgrade # 升级设施

# ========== 种植系统 ==========
GET    /api/v1/saves/:id/garden         # 获取后院状态
GET    /api/v1/saves/:id/garden/plots   # 获取菜地列表
POST   /api/v1/saves/:id/garden/plots/:plot_id/plant # 种植
POST   /api/v1/saves/:id/garden/plots/:plot_id/water # 浇水
POST   /api/v1/saves/:id/garden/plots/:plot_id/fertilize # 施肥
POST   /api/v1/saves/:id/garden/plots/:plot_id/harvest # 收获

# ========== 旅行系统 ==========
GET    /api/v1/saves/:id/travel         # 获取旅行状态
POST   /api/v1/saves/:id/travel/start   # 开始旅行
POST   /api/v1/saves/:id/travel/cancel  # 取消旅行
GET    /api/v1/saves/:id/travel/history # 旅行历史

# ========== 菜谱系统 ==========
GET    /api/v1/saves/:id/recipes        # 获取菜谱列表
POST   /api/v1/saves/:id/recipes/:recipe_id/experiment # 实验研发
POST   /api/v1/saves/:id/recipes/:recipe_id/cook # 烹饪

# ========== 记忆系统 ==========
GET    /api/v1/saves/:id/memories       # 获取记忆列表
POST   /api/v1/saves/:id/memories/:memory_id/unlock # 解锁记忆

# ========== 顾客系统 ==========
GET    /api/v1/saves/:id/customers      # 获取顾客列表
GET    /api/v1/saves/:id/customers/:customer_id # 获取顾客详情

# ========== 统计系统 ==========
GET    /api/v1/saves/:id/statistics     # 获取统计数据
GET    /api/v1/saves/:id/statistics/financial # 财务统计
GET    /api/v1/saves/:id/statistics/customer-analysis # 顾客分析

# ========== 健康检查 ==========
GET    /health                          # 完整健康检查
GET    /health/ready                    # 就绪检查
GET    /health/live                     # 存活检查

# ========== 时间系统 ==========
GET    /api/v1/time                     # 获取当前时间状态
PATCH  /api/v1/config                   # 运行时配置（调试用）
```

### 2.2 WebSocket API

```rust
/// WebSocket 消息类型
pub enum WsMessage {
    // 客户端 -> 服务端
    Subscribe { save_id: Uuid },
    Unsubscribe { save_id: Uuid },
    SendCommand { content: String },
    SendMessage { content: String },

    // 服务端 -> 客户端
    StateUpdate { entity: String, data: serde_json::Value },
    CommandArrived { command: Command },
    EventTriggered { event: GameEvent },
    DialogueMessage { message: DialogueMessage },
    DailyReport { report: DailyReport },
    Error { code: String, message: String },
}
```

---

## 三、测试策略

### 3.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_adjust() {
        let mut personality = Personality::new();
        personality.adjust(PersonalityAxis::BusinessStyle, 10);
        assert_eq!(personality.business_style, 60);

        personality.adjust(PersonalityAxis::BusinessStyle, -100);
        assert_eq!(personality.business_style, 0);
    }

    #[test]
    fn test_module_effectiveness() {
        let module = Module {
            module_type: ModuleType::Communication,
            level: 5,
            condition: 80,
            experience: 0,
            is_functional: true,
        };
        let effectiveness = module.effectiveness();
        assert!(effectiveness > 0.5);
    }

    #[tokio::test]
    async fn test_save_repository() {
        let pool = setup_test_db().await;
        let repo = SaveRepository::new(pool);

        let save = Save {
            id: Uuid::new_v4(),
            name: "测试存档".into(),
            player_name: "测试玩家".into(),
            created_at: Utc::now(),
            last_played: Utc::now(),
            play_time_seconds: 0,
            chapter: 1,
        };

        repo.save(&save).await.unwrap();
        let loaded = repo.find_by_id(save.id).await.unwrap();
        assert!(loaded.is_some());
    }
}
```

### 3.2 集成测试

```rust
#[tokio::test]
async fn test_full_game_loop() {
    // 1. 创建存档
    let save = create_test_save().await;

    // 2. 发送指令
    let command = send_command(&save.id, "查看盼盼状态").await;
    assert_eq!(command.status, CommandStatus::Pending);

    // 3. 等待指令到达
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 4. 处理指令
    process_commands().await;

    // 5. 验证状态更新
    let panpan = get_panpan_state(&save.id).await;
    assert!(panpan.is_some());
}
```

### 3.3 性能测试

```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let mut handles = vec![];

    for i in 0..100 {
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let response = client
                .get("http://localhost:8080/health")
                .send()
                .await
                .unwrap();
            assert!(response.status().is_success());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
```

---

## 四、性能优化

### 4.1 数据库优化

- 为高频查询字段建立索引
- 使用连接池管理数据库连接
- 批量写入减少事务次数

### 4.2 内存优化

- 使用 `Arc` 共享只读数据
- 避免不必要的数据克隆
- 使用 `Cow<str>` 处理字符串

### 4.3 异步优化

- 使用 `tokio::spawn` 并行处理独立任务
- 避免在异步代码中使用阻塞操作
- 合理设置超时时间

---

## 五、验证方案

### 5.1 开发阶段验证

1. 使用 `cargo test` 运行单元测试
2. 使用 `curl` 或 Postman 测试 HTTP API
3. 使用 `wscat` 测试 WebSocket 连接

### 5.2 功能验证

1. 创建新存档，验证数据库写入
2. 发送指令，验证延迟队列工作正常
3. 触发事件，验证事件系统响应
4. 模拟时间流逝，验证各子系统状态更新

### 5.3 性能验证

1. 压力测试 API 端点
2. 验证长时间运行稳定性
3. 验证内存使用情况
