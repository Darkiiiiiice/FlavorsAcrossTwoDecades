-- 001_initial.sql
-- 初始化数据库表结构
--
-- ========== 天气数据表 ==========
CREATE TABLE IF NOT EXISTS weather (
    id INTEGER PRIMARY KEY,
    type INTEGER NOT NULL,
    temperature REAL NOT NULL,
    duration INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

-- ========== 游戏元数据表 ==========
CREATE TABLE IF NOT EXISTS game_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    player_name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);


-- ========== 盼盼状态表 ==========
CREATE TABLE IF NOT EXISTS panpan_states (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    model TEXT NOT NULL,
    manufacture_date TEXT NOT NULL,
    personality TEXT NOT NULL,
    trust_level INTEGER NOT NULL,
    emotion TEXT NOT NULL,
    energy_current INTEGER NOT NULL,
    energy_max INTEGER NOT NULL,
    location TEXT NOT NULL,
    current_state TEXT NOT NULL,
    current_task TEXT
);

-- ========== 模块表 ==========
CREATE TABLE IF NOT EXISTS modules (
    id TEXT PRIMARY KEY,
    module_type TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    experience INTEGER NOT NULL,
    is_functional INTEGER NOT NULL
);

-- ========== 小馆状态表 ==========
CREATE TABLE IF NOT EXISTS shop_states (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    funds INTEGER NOT NULL,
    reputation REAL NOT NULL,
    restaurant_level INTEGER NOT NULL,
    kitchen_level INTEGER NOT NULL,
    backyard_level INTEGER NOT NULL,
    workshop_level INTEGER NOT NULL
);

-- ========== 设施表 ==========
CREATE TABLE IF NOT EXISTS facilities (
    id TEXT PRIMARY KEY,
    zone TEXT NOT NULL,
    name TEXT NOT NULL,
    level INTEGER NOT NULL,
    condition INTEGER NOT NULL,
    upgrade_progress TEXT
);

-- ========== 菜地表 ==========
CREATE TABLE IF NOT EXISTS garden_plots (
    id TEXT PRIMARY KEY,
    plot_number INTEGER NOT NULL,
    is_unlocked INTEGER NOT NULL,
    current_crop TEXT,
    fertility INTEGER NOT NULL,
    moisture INTEGER NOT NULL
);

-- ========== 旅行表 ==========
CREATE TABLE IF NOT EXISTS travels (
    id TEXT PRIMARY KEY,
    destination TEXT NOT NULL,
    started_at TEXT NOT NULL,
    expected_return TEXT NOT NULL,
    status TEXT NOT NULL,
    rewards TEXT
);

-- ========== 记忆碎片表 ==========
CREATE TABLE IF NOT EXISTS memory_fragments (
    id TEXT PRIMARY KEY,
    fragment_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    is_unlocked INTEGER NOT NULL,
    unlocked_at TEXT,
    trigger_condition TEXT NOT NULL
);

-- ========== 菜谱表 ==========
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    ingredients TEXT NOT NULL,
    source TEXT NOT NULL,
    unlock_condition TEXT
);

-- ========== 顾客记录表 ==========
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    customer_type TEXT NOT NULL,
    name TEXT NOT NULL,
    favorability INTEGER NOT NULL,
    visit_count INTEGER NOT NULL,
    last_visit TEXT NOT NULL,
    preferences TEXT NOT NULL
);

-- ========== 指令表 ==========
CREATE TABLE IF NOT EXISTS commands (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    arrival_time TEXT NOT NULL,
    status TEXT NOT NULL,
    result TEXT
);

-- ========== 对话表 ==========
CREATE TABLE IF NOT EXISTS dialogues (
    id TEXT PRIMARY KEY,
    sender TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL
);

-- ========== 游戏配置表 ==========
CREATE TABLE IF NOT EXISTS game_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT NOT NULL
);

-- ========== 索引 ==========
CREATE INDEX IF NOT EXISTS idx_commands_arrival ON commands(arrival_time);
CREATE INDEX IF NOT EXISTS idx_garden_plots_number ON garden_plots(plot_number);
