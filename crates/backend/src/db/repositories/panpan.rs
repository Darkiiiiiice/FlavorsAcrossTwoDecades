//! 盼盼状态仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::panpan::{ModuleRecord, PanpanState};
use crate::error::{DatabaseError, GameError, GameResult};
use crate::game::panpan::{Emotion, Module, ModuleType, Personality};

/// 盼盼状态仓储
pub struct PanpanRepository {
    pool: SqlitePool,
}

impl PanpanRepository {
    /// 创建新的盼盼仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建盼盼状态
    pub async fn create(&self, state: &PanpanState) -> GameResult<()> {
        let personality_json = serde_json::to_string(&state.personality).map_err(|e| {
            GameError::Validation {
                details: format!("Failed to serialize personality: {}", e),
            }
        })?;

        sqlx::query(
            r#"INSERT INTO panpan_states (save_id, name, model, manufacture_date, personality,
               trust_level, emotion, energy_current, energy_max, location, current_state, current_task)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(state.save_id.to_string())
        .bind(&state.name)
        .bind(&state.model)
        .bind(state.manufacture_date.to_rfc3339())
        .bind(&personality_json)
        .bind(state.trust_level as i64)
        .bind(emotion_to_string(&state.emotion))
        .bind(state.energy_current as i64)
        .bind(state.energy_max as i64)
        .bind(&state.location)
        .bind(&state.current_state)
        .bind(&state.current_task)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据存档ID查找盼盼状态
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Option<PanpanState>> {
        let row = sqlx::query_as::<_, PanpanStateRow>(
            r#"SELECT save_id, name, model, manufacture_date, personality, trust_level,
               emotion, energy_current, energy_max, location, current_state, current_task
               FROM panpan_states WHERE save_id = ?"#
        )
        .bind(save_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_state()?)),
            None => Ok(None),
        }
    }

    /// 更新盼盼状态
    pub async fn update(&self, state: &PanpanState) -> GameResult<()> {
        let personality_json = serde_json::to_string(&state.personality).map_err(|e| {
            GameError::Validation {
                details: format!("Failed to serialize personality: {}", e),
            }
        })?;

        sqlx::query(
            r#"UPDATE panpan_states SET name = ?, model = ?, manufacture_date = ?,
               personality = ?, trust_level = ?, emotion = ?, energy_current = ?,
               energy_max = ?, location = ?, current_state = ?, current_task = ?
               WHERE save_id = ?"#
        )
        .bind(&state.name)
        .bind(&state.model)
        .bind(state.manufacture_date.to_rfc3339())
        .bind(&personality_json)
        .bind(state.trust_level as i64)
        .bind(emotion_to_string(&state.emotion))
        .bind(state.energy_current as i64)
        .bind(state.energy_max as i64)
        .bind(&state.location)
        .bind(&state.current_state)
        .bind(&state.current_task)
        .bind(state.save_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 创建或更新盼盼状态
    pub async fn upsert(&self, state: &PanpanState) -> GameResult<()> {
        let existing = self.find_by_save_id(state.save_id).await?;
        if existing.is_some() {
            self.update(state).await
        } else {
            self.create(state).await
        }
    }

    // ========== 模块管理 ==========

    /// 创建模块记录
    pub async fn create_module(&self, module: &ModuleRecord) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO modules (id, save_id, module_type, level, condition, experience, is_functional)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(module.id.to_string())
        .bind(module.save_id.to_string())
        .bind(&module.module_type)
        .bind(module.level as i64)
        .bind(module.condition as i64)
        .bind(module.experience as i64)
        .bind(module.is_functional as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 获取存档的所有模块
    pub async fn find_modules(&self, save_id: Uuid) -> GameResult<Vec<ModuleRecord>> {
        let rows = sqlx::query_as::<_, ModuleRow>(
            r#"SELECT id, save_id, module_type, level, condition, experience, is_functional
               FROM modules WHERE save_id = ?"#
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter()
            .map(|row| row.into_module_record())
            .collect()
    }

    /// 更新模块状态
    pub async fn update_module(&self, module: &ModuleRecord) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE modules SET level = ?, condition = ?, experience = ?, is_functional = ?
               WHERE id = ?"#
        )
        .bind(module.level as i64)
        .bind(module.condition as i64)
        .bind(module.experience as i64)
        .bind(module.is_functional as i64)
        .bind(module.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

// ========== 辅助函数和数据结构 ==========

/// 将情绪转换为字符串
fn emotion_to_string(emotion: &Emotion) -> String {
    match emotion {
        Emotion::Happy => "happy".to_string(),
        Emotion::Calm => "calm".to_string(),
        Emotion::Tired => "tired".to_string(),
        Emotion::Confused => "confused".to_string(),
        Emotion::Worried => "worried".to_string(),
        Emotion::Lonely => "lonely".to_string(),
        Emotion::Excited => "excited".to_string(),
    }
}

/// 从字符串解析情绪
fn string_to_emotion(s: &str) -> GameResult<Emotion> {
    match s {
        "happy" => Ok(Emotion::Happy),
        "calm" => Ok(Emotion::Calm),
        "tired" => Ok(Emotion::Tired),
        "confused" => Ok(Emotion::Confused),
        "worried" => Ok(Emotion::Worried),
        "lonely" => Ok(Emotion::Lonely),
        "excited" => Ok(Emotion::Excited),
        _ => Err(GameError::Validation {
            details: format!("Invalid emotion: {}", s),
        }),
    }
}

/// 盼盼状态数据库行
#[derive(sqlx::FromRow)]
struct PanpanStateRow {
    save_id: String,
    name: String,
    model: String,
    manufacture_date: String,
    personality: String,
    trust_level: i64,
    emotion: String,
    energy_current: i64,
    energy_max: i64,
    location: String,
    current_state: String,
    current_task: Option<String>,
}

impl PanpanStateRow {
    fn into_state(self) -> GameResult<PanpanState> {
        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| {
            GameError::Validation {
                details: format!("Invalid UUID: {}", e),
            }
        })?;

        let manufacture_date = DateTime::parse_from_rfc3339(&self.manufacture_date)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid manufacture_date: {}", e),
            })?
            .with_timezone(&Utc);

        let personality: Personality = serde_json::from_str(&self.personality).map_err(|e| {
            GameError::Validation {
                details: format!("Invalid personality JSON: {}", e),
            }
        })?;

        let emotion = string_to_emotion(&self.emotion)?;

        Ok(PanpanState {
            save_id,
            name: self.name,
            model: self.model,
            manufacture_date,
            personality,
            trust_level: self.trust_level as u32,
            emotion,
            energy_current: self.energy_current as u32,
            energy_max: self.energy_max as u32,
            location: self.location,
            current_state: self.current_state,
            current_task: self.current_task,
        })
    }
}

/// 模块数据库行
#[derive(sqlx::FromRow)]
struct ModuleRow {
    id: String,
    save_id: String,
    module_type: String,
    level: i64,
    condition: i64,
    experience: i64,
    is_functional: i64,
}

impl ModuleRow {
    fn into_module_record(self) -> GameResult<ModuleRecord> {
        let id = Uuid::parse_str(&self.id).map_err(|e| {
            GameError::Validation {
                details: format!("Invalid UUID: {}", e),
            }
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| {
            GameError::Validation {
                details: format!("Invalid save_id UUID: {}", e),
            }
        })?;

        Ok(ModuleRecord {
            id,
            save_id,
            module_type: self.module_type,
            level: self.level as u32,
            condition: self.condition as u32,
            experience: self.experience as u32,
            is_functional: self.is_functional != 0,
        })
    }
}
