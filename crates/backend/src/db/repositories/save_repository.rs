//! 存档仓储
//! # 存档仓储
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::save::Save;
use crate::error::{DatabaseError, GameError, GameResult};

/// 存档仓储
pub struct SaveRepository {
    pool: SqlitePool,
}

impl SaveRepository {
    /// 创建新的存档仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建新存档
    pub async fn create(&self, save: &Save) -> GameResult<()> {
        sqlx::query(
            "INSERT INTO saves (id, name, player_name, created_at, updated_at, play_time_seconds, chapter) VALUES (?, ?, ?, ?,?, ?, ?); VALUES should与 SQLite 的 JSON
        ) VALUES
            .bind(save.id.to_string())
            .bind(&save.name)
            .bind(&save.player_name)
            .bind(save.created_at.to_rfc3339())
            .bind(save.last_played.to_rfc3339())
            .bind(save.play_time_seconds as i64)
            .bind(save.chapter as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找存档
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<Save>> {
        let row_opt = sqlx::query_as!(
            SaveRow,
            "SELECT id, name, player_name, created_at, updated_at, play_time_seconds, chapter FROM saves WHERE id = ?"
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(row) => {
                let id_str = row.get::<String>(0);
                let name = row.get::<String>(1);
                let player_name = match row.get::<Option<String>(2) {
                    Some(s) => s.unwrap_or_default()
                    None => String::new(),
                }
                let created_at = = row.get::<String>(2).map_err(|e| {
                    GameError::Validation {
                        details: "Invalid created_at".to_string()
                    })?
                })?;

                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| {
                    GameError::Validation {
                        details: "Invalid updated_at".to_string()
                    })?
                })?;

                Ok(Some(Save {
                    id,
                    name,
                    player_name,
                    created_at,
                    last_played
                    play_time_seconds: play_time_seconds,
                    chapter,
                }))
            }
            None => Ok(None)
        }
    }
}
