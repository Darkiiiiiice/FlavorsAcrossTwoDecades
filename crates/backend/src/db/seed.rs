//! 数据库种子数据初始化

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{DatabaseError, GameError, GameResult};

/// 默认游戏配置
const DEFAULT_CONFIGS: &[(&str, &str, &str)] = &[
    ("game_version", "1", "游戏版本号"),
    ("initial_money", "1000", "初始资金"),
    ("initial_energy", "100", "初始能量值"),
    ("garden_plots", "4", "初始菜地数量"),
    ("kitchen_slots", "2", "初始厨房设备槽位"),
    ("travel_cooldown", "3600", "旅行冷却时间（秒）"),
    ("auto_save_interval", "300", "自动存档间隔（秒）"),
    ("llm_communication_min_delay", "1", "LLM 通信最小延迟（秒）"),
    (
        "llm_communication_max_delay",
        "50",
        "LLM 通信最大延迟（秒）",
    ),
    ("crop_growth_base_time", "60", "作物基础生长时间（秒）"),
    ("dish_cooking_base_time", "30", "菜品基础烹饪时间（秒）"),
];

/// 检查并初始化种子数据
pub async fn initialize_seed_data(pool: &SqlitePool) -> GameResult<()> {
    // 检查是否已初始化
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM game_config")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            GameError::Database(DatabaseError::QueryFailed(format!(
                "Failed to check seed data status: {}",
                e
            )))
        })?;

    if count > 0 {
        tracing::info!("Seed data already initialized, skipping");
        return Ok(());
    }

    tracing::info!("Initializing seed data...");

    let now = Utc::now().to_rfc3339();

    // 插入默认配置
    for (key, value, description) in DEFAULT_CONFIGS {
        sqlx::query(
            "INSERT INTO game_config (key, value, description, updated_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(key)
        .bind(value)
        .bind(description)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| {
            GameError::Database(DatabaseError::WriteFailed(format!(
                "Failed to insert seed data for key '{}': {}",
                key, e
            )))
        })?;
    }

    tracing::info!(
        "Seed data initialized successfully with {} config entries",
        DEFAULT_CONFIGS.len()
    );
    Ok(())
}

/// 获取游戏配置值
pub async fn get_config(pool: &SqlitePool, key: &str) -> GameResult<Option<String>> {
    let result: Option<String> = sqlx::query_scalar("SELECT value FROM game_config WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            GameError::Database(DatabaseError::QueryFailed(format!(
                "Failed to get config '{}': {}",
                key, e
            )))
        })?;

    Ok(result)
}

/// 设置游戏配置值
pub async fn set_config(pool: &SqlitePool, key: &str, value: &str) -> GameResult<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO game_config (key, value, description, updated_at) VALUES (?1, ?2, '', ?3)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
    )
    .bind(key)
    .bind(value)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| {
        GameError::Database(DatabaseError::WriteFailed(format!(
            "Failed to set config '{}': {}",
            key, e
        )))
    })?;

    Ok(())
}
