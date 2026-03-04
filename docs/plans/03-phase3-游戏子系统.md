# Phase 3: 游戏子系统（第5-8周）

## 开发目标

- [ ] 实现盼盼系统（状态、性格、情绪、能量、对话）
- [ ] 实现小馆经营系统（餐厅、厨房、后院、工坊）
- [ ] 实现旅行收集系统
- [ ] 实现实验研发系统
- [ ] 实现记忆碎片系统
- [ ] 实现后院种植系统
- [ ] 实现顾客系统
- [ ] 实现事件系统（完整版）
- [ ] 实现天气与节假日系统
- [ ] 实现邻里系统
- [ ] 实现成就与教程系统

---

## 一、盼盼完整属性系统

### 1.1 设计理念

盼盼是一个模块化的机器人，拥有完整的属性系统：
- **模块系统**：7个硬件模块，整合了技能功能，每个模块有等级和完好度
- **性格系统**：3维性格轴，影响决策倾向
- **信任度**：玩家与盼盼的关系深度
- **情绪系统**：7种情绪状态，影响工作效率和行为
- **能量系统**：电池续航管理

### 1.2 模块系统

```rust
/// 盼盼模块类型
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
    pub level: u32,              // 等级 1-10
    pub condition: u32,          // 完好度 0-100
    pub experience: u32,         // 经验值
    pub is_functional: bool,     // 是否可用
}
```

### 1.3 模块效果

| 模块 | 主要效果 | 升级影响 |
|------|---------|---------|
| 通信 | 通信延迟附加 | 降低延迟 55s->1s |
| 记忆 | 记忆容量/解锁速度 | 容量 100→500 |
| 传感器 | 实验误差 | 精度 ±30%→±2% |
| 移动 | 旅行速度/维修能力 | 速度 +50%→-40% |
| 电池 | 续航时间 | 4h→48h |
| 厨房 | 烹饪成功率/品质 | 成功率 50%→95% |
| 社交 | 顾客好感 | 效果 -50%→+40% |

### 1.4 性格轴系统

```rust
/// 性格轴
pub struct Personality {
    pub business_style: u32,    // 理性(0) <-> 感性(100)，初始50
    pub innovation: u32,        // 保守(0) <-> 创新(100)，初始50
    pub independence: u32,      // 服从(0) <-> 自主(100)，初始50
}
```

### 1.5 信任度系统

| 等级 | 范围 | 效果 |
|------|------|------|
| 陌生 | 0-20 | 记忆恢复×0.3，不主动提议 |
| 初识 | 21-40 | 记忆恢复×0.6，10%主动提议 |
| 一般 | 41-60 | 记忆恢复×1.0，30%主动提议 |
| 高度 | 61-80 | 记忆恢复×1.5，60%主动提议 |
| 完全 | 81-100 | 记忆恢复×2.0，90%主动提议 |

### 1.6 情绪系统

```rust
pub enum Emotion {
    Happy,      // 开心 - 工作速度×1.1，错误率×0.9
    Calm,       // 平静 - 正常状态
    Tired,      // 疲惫 - 工作速度×0.9，错误率×1.2
    Confused,   // 困惑 - 错误率×1.1，更倾向请示
    Worried,    // 担忧 - 主动提醒问题
    Lonely,     // 孤独 - 想引起注意
    Excited,    // 兴奋 - 旅行时间×0.9
}
```

### 1.7 能量系统

| 活动类型 | 能量消耗/小时 |
|---------|--------------|
| 待机 | 1 |
| 烹饪/实验 | 2 |
| 旅行 | 2 |
| 种植 | 4 |

### 1.8 对话系统设计

```rust
/// 对话消息
pub struct DialogueMessage {
    pub id: Uuid,
    pub save_id: Uuid,
    pub sender: DialogueSender,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub status: MessageStatus,
}

pub enum DialogueSender {
    Player,  // 玩家（火星）
    Panpan,  // 盼盼（地球）
}

pub enum MessageStatus {
    Sending,        // 发送中
    InTransit,      // 传输中（火星→地球）
    Delivered,      // 已到达地球
    Processing,     // 盼盼处理中
    Responded,      // 已回复
}
```

---

## 二、星夜小馆系统设计

### 2.1 小馆基础信息

| 属性 | 说明 |
|------|------|
| **名称** | 星夜小馆 |
| **位置** | 地球·老街 |
| **结构** | 两层小楼，一楼餐厅+厨房，二楼储物+盼盼充电区，后院菜地+工坊 |

### 2.2 设施区域

```rust
pub enum FacilityZone {
    Restaurant,  // 餐厅
    Kitchen,     // 厨房
    Backyard,    // 后院
    Workshop,    // 工坊
}

pub struct ZoneLevel {
    pub zone: FacilityZone,
    pub level: u32,                 // 1-64
    pub reputation_cap: u32,        // 口碑上限
    pub unlocked_features: Vec<String>,
}
```

### 2.3 区域等级效果

| 区域 | 1级 | 3级 | 5级 |
|------|-----|-----|-----|
| 餐厅 | 口碑上限30，基础服务 | 口碑上限50，新菜品槽 | 口碑上限80，VIP服务 |
| 厨房 | 基础烹饪 | 高级设备 | 专业厨房 |
| 后院 | 1块菜地 | 3块菜地 | 温室+自动灌溉 |
| 工坊 | 简单维修 | 高级制作 | 创意工坊 |

### 2.4 经营数据系统

```rust
/// 资金系统
pub struct Finance {
    pub cash: u64,           // 现金
    pub daily_revenue: u64,  // 今日收入
    pub daily_expenses: u64, // 今日支出
}

/// 口碑指数（加权计算）
pub fn calculate_reputation(shop: &ShopState) -> f32 {
    let dish_score = shop.dish_quality * 0.40;
    let service_score = shop.service_quality * 0.20;
    let environment_score = shop.environment_score * 0.15;
    let neighbor_score = shop.neighbor_relation * 0.15;
    let regular_score = shop.regular_customer_bonus * 0.10;
    dish_score + service_score + environment_score + neighbor_score + regular_score
}
```

### 2.5 厨房运营系统

```rust
/// 库存项
pub struct InventoryItem {
    pub ingredient_id: String,
    pub quantity: u32,
    pub quality: f32,        // 0-1，影响菜品品质
    pub freshness: f32,      // 0-1，随时间下降
    pub expiry_date: Option<DateTime<Utc>>,
}

/// 菜品制作
pub struct CookingTask {
    pub recipe_id: String,
    pub started_at: DateTime<Utc>,
    pub estimated_duration: Duration,
    pub status: CookingStatus,
}
```

---

## 三、后院种植系统

### 3.1 后院等级

| 等级 | 解锁内容 |
|------|---------|
| 1级 | 1块菜地，基础种植 |
| 2级 | 2块菜地，堆肥区 |
| 3级 | 3块菜地，自动浇水 |
| 4级 | 4块菜地，温室框架 |
| 5级 | 5块菜地，完整温室 |

### 3.2 作物系统

```rust
pub enum CropType {
    Vegetable,  // 蔬菜
    Herb,       // 香料
    Flower,     // 花卉
    Special,    // 特殊作物
    Alien,      // 异星植物
}

pub struct Crop {
    pub id: String,
    pub name: String,
    pub crop_type: CropType,
    pub growth_stages: GrowthStages,
    pub water_need: u32,
    pub fertilizer_need: u32,
    pub base_yield: u32,
    pub rarity: Rarity,
    pub seasons: Vec<Season>,
}
```

### 3.3 生长阶段

```rust
pub enum GrowthStage {
    Sowing,      // 播种期
    Germinating, // 发芽期
    Growing,     // 生长期
    Mature,      // 成熟期（可收获）
    Withering,   // 枯萎期
}
```

### 3.4 病虫害系统

| 类型 | 处理方法 |
|------|---------|
| 蚜虫 | 喷洒药水 |
| 霉菌 | 杀菌剂+通风 |
| 杂草 | 手工拔除 |
| 鸟雀 | 稻草人 |

### 3.5 种植 API

```
POST   /api/v1/saves/:id/garden/plots/:plot_id/plant  # 种植
POST   /api/v1/saves/:id/garden/plots/:plot_id/water  # 浇水
POST   /api/v1/saves/:id/garden/plots/:plot_id/fertilize # 施肥
POST   /api/v1/saves/:id/garden/plots/:plot_id/harvest # 收获
```

---

## 四、工坊系统

### 4.1 工坊等级

| 等级 | 名称 | 解锁内容 |
|------|------|---------|
| 1级 | 杂物小间 | 基础工具 |
| 2级 | 工具小屋 | 中等物品，制作速度+15% |
| 3级 | 手艺工坊 | 高级物品，维修复杂设备 |
| 4级 | 创意工间 | 特殊装饰品 |
| 5级 | 星夜工坊 | 隐藏配方 |

### 4.2 制作系统

```rust
pub enum CraftCategory {
    Consumable,    // 日常消耗（筷子、杯垫）
    Tool,          // 维修工具
    Decoration,    // 装饰物品
    Gift,          // 礼品
    Special,       // 特殊物品
}

pub struct CraftRecipe {
    pub id: String,
    pub name: String,
    pub category: CraftCategory,
    pub materials: Vec<MaterialCost>,
    pub craft_time: Duration,
    pub required_workbench_level: u32,
}
```

---

## 五、旅行系统

### 5.1 旅行触发

```rust
pub struct TravelCondition {
    pub min_trust: u32,           // 最低信任度
    pub min_shop_stability: f32,  // 小馆稳定度
    pub min_energy: u32,          // 最低能量
    pub cooldown_hours: u32,      // 冷却时间
}
```

### 5.2 目的地系统

| 目的地 | 类型 | 收获 |
|--------|------|------|
| 成都 | 川菜 | 麻婆豆腐、回锅肉菜谱 |
| 西安 | 西北菜 | 羊肉泡馍、肉夹馍菜谱 |
| 巴黎 | 法餐 | 法式料理菜谱 |
| 京都 | 日料 | 日式料理菜谱 |

### 5.3 旅行收获

```rust
pub struct TravelReward {
    pub fuzzy_recipes: Vec<FuzzyRecipe>,  // 模糊菜谱
    pub special_ingredients: Vec<Ingredient>,
    pub memory_fragments: Vec<MemoryFragment>,
    pub photos: Vec<String>,              // 旅行照片
}
```

---

## 六、菜谱与实验系统

### 6.1 菜谱分类

| 来源 | 说明 |
|------|------|
| 传承 | 祖父留下的老菜谱 |
| 旅行 | 盼盼旅行带回 |
| 创新 | 盼盼实验研发 |

### 6.2 菜谱状态

```rust
pub enum RecipeStatus {
    Damaged,   // 损坏（需修复）
    Fuzzy,     // 模糊（需实验确定用量）
    Precise,   // 精确（可直接制作）
    Mastered,  // 掌握（有品质加成）
}
```

### 6.3 实验研发流程

```
1. 选择模糊菜谱
2. 预估配方用量
3. 进行实验烹饪
4. 传感器检测（味觉、嗅觉、质地）
5. 分析反馈（多了/少了/刚好）
6. 调整配方
7. 重复直到成功
```

### 6.4 传感器精度

| 传感器等级 | 实验误差 |
|-----------|---------|
| 1-2 | ±30% |
| 3-4 | ±20% |
| 5-6 | ±10% |
| 7-8 | ±5% |
| 9-10 | ±2% |

---

## 七、记忆碎片系统

### 7.1 碎片类型

```rust
pub enum MemoryType {
    Story,       // 故事碎片（主线）
    Character,   // 角色碎片（邻居）
    Recipe,      // 菜谱碎片（菜品来源）
    Place,       // 地点碎片（旅行）
    Secret,      // 秘密碎片（隐藏）
}
```

### 7.2 解锁条件

| 解锁方式 | 示例 |
|---------|------|
| 修复里程碑 | 重新开业 |
| 特定事件 | 祖父忌日 |
| 对话话题 | "祖父最喜欢什么菜" |
| 顾客互动 | 送王奶奶茉莉 |
| 旅行收获 | 成都旅行 |
| 菜谱研发 | 成功研发麻婆豆腐 |

### 7.3 记忆内容存储

```rust
pub struct MemoryContent {
    pub narrative: String,              // 叙事文本
    pub scene_description: Option<String>, // 场景描述
    pub grandfather_quote: Option<String>, // 祖父语录
    pub sensory_memories: Vec<SensoryMemory>, // 感官记忆
    pub panpan_reaction: String,        // 盼盼的反应
    pub unlocked_knowledge: Option<String>, // 解锁的知识
}

pub struct SensoryMemory {
    pub sense: Sense,
    pub description: String,
}

pub enum Sense {
    Visual,   // 视觉
    Auditory, // 听觉
    Olfactory,// 嗅觉
    Gustatory,// 味觉
    Tactile,  // 触觉
}
```

---

## 八、顾客系统

### 8.1 顾客类型

| 类型 | 特点 |
|------|------|
| 老街坊 | 稳定客源，有好感度 |
| 路人 | 随机出现，口碑影响 |
| 美食家 | 高要求，好评影响大 |
| 特殊顾客 | 剧情相关，解锁记忆 |

### 8.2 核心顾客

| 顾客 | 背景 | 关联记忆 |
|------|------|---------|
| 李大爷 | 退休工人，祖父老友 | 祖父年轻时的事迹 |
| 王奶奶 | 邻居，喜欢种花 | 种植相关记忆 |
| 老周 | 自由撰稿人 | 文学与历史记忆 |
| 小美 | 年轻女孩，曾受资助 | 温情记忆 |

### 8.3 好感度系统

```rust
pub struct CustomerRelation {
    pub customer_id: String,
    pub favorability: u32,      // 0-100
    pub visit_count: u32,
    pub last_visit: DateTime<Utc>,
    pub unlocked_stories: Vec<String>,
}
```

---

## 九、天气与节假日系统

### 9.1 天气类型

| 天气 | 效果 |
|------|------|
| 晴天 | 正常客流 |
| 雨天 | 客流-20%，种植需防涝 |
| 雪天 | 客流-30%，供暖需求增加 |
| 阴天 | 客流-10% |

### 9.2 节假日效果

| 节日 | 效果 |
|------|------|
| 春节 | 客流+50%，团圆菜品需求 |
| 中秋 | 客流+30%，月饼相关 |
| 端午 | 客流+20%，粽子相关 |

---

## 十、邻里系统

### 10.1 邻居角色

| 邻居 | 能提供的帮助 |
|------|-------------|
| 王奶奶 | 花种、种植建议 |
| 李大爷 | 维修帮助、老故事 |
| 老张 | 食材批发 |
| 小刘 | 快递代收 |

### 10.2 互动系统

```rust
pub struct NeighborInteraction {
    pub neighbor_id: String,
    pub interaction_type: InteractionType,
    pub result: InteractionResult,
}

pub enum InteractionType {
    Gift,       // 赠送礼物
    Request,    // 请求帮助
    Chat,       // 闲聊
    Trade,      // 交易
}
```

---

## 十一、成就系统

### 11.1 成就类别

| 类别 | 示例 |
|------|------|
| 经营 | 营业额达标、口碑达标 |
| 探索 | 旅行次数、目的地收集 |
| 社交 | 邻居好感度、顾客数量 |
| 烹饪 | 菜谱收集、菜品品质 |
| 收集 | 记忆碎片、特殊物品 |

---

## 十二、教程系统

### 12.1 教程流程

1. 欢迎 - 介绍游戏背景
2. 查看盼盼状态 - 了解盼盼属性
3. 发送第一条指令 - 学习通信延迟
4. 查看小馆状态 - 了解经营界面
5. 完成教程 - 开始自由探索

### 12.2 上下文感知提示

根据玩家当前状态和操作，动态显示相关提示。

---

## 十三、统计系统

### 13.1 统计类别

| 类别 | 内容 |
|------|------|
| 财务 | 收入、支出、利润趋势 |
| 客流 | 每日客流、峰值时段 |
| 顾客 | 顾客类型分布、满意度 |
| 菜品 | 热门菜品、销售排行 |
| 运营 | 设施使用率、盼盼工作时长 |
| 里程碑 | 成就进度、解锁内容 |
| 趋势 | 口碑变化、好感度变化 |

---

## 十四、子系统联动关系

```
┌─────────────────────────────────────────────────────────────┐
│                       游戏子系统联动图                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐     提供食材      ┌─────────┐                  │
│  │ 后院种植 │ ───────────────→ │ 厨房运营 │                  │
│  └─────────┘                   └─────────┘                  │
│       │                             │                       │
│       │ 提供花卉                     │ 提供菜品              │
│       ↓                             ↓                       │
│  ┌─────────┐     提供装饰      ┌─────────┐                  │
│  │ 工坊制作 │ ───────────────→ │ 餐厅经营 │                  │
│  └─────────┘                   └─────────┘                  │
│       │                             │                       │
│       │ 提供礼物                     │ 产生顾客              │
│       ↓                             ↓                       │
│  ┌─────────┐     解锁记忆      ┌─────────┐                  │
│  │ 邻里系统 │ ───────────────→ │ 记忆碎片 │                  │
│  └─────────┘                   └─────────┘                  │
│       │                             ↑                       │
│       │ 获得种子                     │ 触发记忆              │
│       ↓                             │                       │
│  ┌─────────┐     带回菜谱      ┌─────────┐                  │
│  │ 旅行系统 │ ───────────────→ │ 菜谱实验 │                  │
│  └─────────┘                   └─────────┘                  │
│                                     │                       │
│                                     │ 新菜品                │
│                                     ↓                       │
│                              ┌─────────────┐                │
│                              │  盼盼系统   │                │
│                              │ (核心控制器) │                │
│                              └─────────────┘                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```
