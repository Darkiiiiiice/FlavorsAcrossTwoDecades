-- 002_phase4_schema.sql
-- Phase 4: 细粒度数据模型 Schema

-- ========== 盼盼状态表 ==========
CREATE TABLE IF NOT EXISTS panpan_states (
    save_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model TEXT NOT NULL,
    manufacture_date TEXT NOT NULL,
    personality TEXT NOT NULL,          -- JSON: Personality
    trust_level INTEGER NOT NULL,
    emotion TEXT NOT NULL,
    energy_current INTEGER NOT NULL,
    energy_max INTEGER NOT NULL,
    location TEXT NOT NULL,
    current_state TEXT NOT NULL,
    current_task TEXT,                  -- JSON: Option<Task>
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 模块表 ==========
CREATE TABLE IF NOT EXISTS modules (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    module_type TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    experience INTEGER NOT NULL,
    is_functional INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 小馆状态表 ==========
CREATE TABLE IF NOT EXISTS shop_states (
    save_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    funds INTEGER NOT NULL,
    reputation REAL NOT NULL,
    restaurant_level INTEGER NOT NULL,
    kitchen_level INTEGER NOT NULL,
    backyard_level INTEGER NOT NULL,
    workshop_level INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 设施表 ==========
CREATE TABLE IF NOT EXISTS facilities (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    zone TEXT NOT NULL,
    name TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    upgrade_progress TEXT,              -- JSON: Option<UpgradeProgress>
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 菜地表 ==========
CREATE TABLE IF NOT EXISTS garden_plots (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    plot_number INTEGER NOT NULL,
    is_unlocked INTEGER NOT NULL,
    current_crop TEXT,                  -- JSON: Option<CropState>
    fertility INTEGER NOT NULL,
    moisture INTEGER NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 旅行表 ==========
CREATE TABLE IF NOT EXISTS travels (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    destination TEXT NOT NULL,
    started_at TEXT NOT NULL,
    expected_return TEXT NOT NULL,
    status TEXT NOT NULL,
    rewards TEXT,                       -- JSON: Option<TravelReward>
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 记忆碎片表 ==========
CREATE TABLE IF NOT EXISTS memory_fragments (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    fragment_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    is_unlocked INTEGER NOT NULL,
    unlocked_at TEXT,
    trigger_condition TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 菜谱表 ==========
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    ingredients TEXT NOT NULL,          -- JSON: Vec<IngredientAmount>
    source TEXT NOT NULL,
    unlock_condition TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 顾客记录表 ==========
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    customer_type TEXT NOT NULL,
    name TEXT NOT NULL,
    favorability INTEGER NOT NULL,
    visit_count INTEGER NOT NULL,
    last_visit TEXT NOT NULL,
    preferences TEXT NOT NULL,          -- JSON: Vec<String>
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 指令表 ==========
CREATE TABLE IF NOT EXISTS commands (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    arrival_time TEXT NOT NULL,
    status TEXT NOT NULL,
    result TEXT,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 对话表 ==========
CREATE TABLE IF NOT EXISTS dialogues (
    id TEXT PRIMARY KEY,
    save_id TEXT NOT NULL,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- ========== 创建索引 ==========
CREATE INDEX IF NOT EXISTS idx_modules_save ON modules(save_id);
CREATE INDEX IF NOT EXISTS idx_facilities_save ON facilities(save_id);
CREATE INDEX IF NOT EXISTS idx_garden_plots_save ON garden_plots(save_id);
CREATE INDEX IF NOT EXISTS idx_travels_save ON travels(save_id);
CREATE INDEX IF NOT EXISTS idx_memory_fragments_save ON memory_fragments(save_id);
CREATE INDEX IF NOT EXISTS idx_recipes_save ON recipes(save_id);
CREATE INDEX IF NOT EXISTS idx_customers_save ON customers(save_id);
CREATE INDEX IF NOT EXISTS idx_commands_save ON commands(save_id);
CREATE INDEX IF NOT EXISTS idx_commands_arrival ON commands(arrival_time);
CREATE INDEX IF NOT EXISTS idx_dialogues_save ON dialogues(save_id);
