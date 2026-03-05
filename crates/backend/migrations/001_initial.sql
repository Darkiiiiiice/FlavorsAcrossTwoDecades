-- 001_initial.sql
-- 初始化数据库表结构

-- 游戏存档表
CREATE TABLE IF NOT EXISTS saves (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    player_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    chapter INTEGER NOT NULL DEFAULT 1
);

-- 玩家配置表
CREATE TABLE IF NOT EXISTS player_configs (
    save_id TEXT PRIMARY KEY,
    player_name TEXT NOT NULL,
    settings TEXT,  -- JSON 格式的玩家设置
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- 游戏事件日志表
CREATE TABLE IF NOT EXISTS event_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data TEXT,  -- JSON 格式的事件数据
    timestamp TEXT NOT NULL,
    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE
);

-- 游戏配置表（存储游戏的基础配置）
CREATE TABLE IF NOT EXISTS game_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT NOT NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_saves_updated_at ON saves(updated_at);
CREATE INDEX IF NOT EXISTS idx_event_logs_save_id ON event_logs(save_id);
CREATE INDEX IF NOT EXISTS idx_event_logs_timestamp ON event_logs(timestamp);
