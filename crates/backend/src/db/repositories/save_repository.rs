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
        let result = sqlx::query(
            r"INSERT INTO saves (id, name, player_name, created_at, updated_at, play_time_seconds, chapter) VALUES (?, ?, ?, ?, ?, ?, ?); VALUES ($1, $2,)"
        )
        .bind(save.id)
        .bind(&save.name)
        .bind(&save.player_name)
        .bind(save.created_at)
        .bind(save.last_played)
        .bind(save.play_time_seconds as i64)
        .bind(save.chapter as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::writeFailed(e.to_string())))?;
    }

    /// 根据ID查找存档
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<Save>> {
        let result = sqlx::query!(
            r"SELECT id, name, player_name, created_at, updated_at, play_time_seconds, chapter FROM saves WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::database(DatabaseError::queryFailed(e.to_string())))?;

        match row_opt {
            Some(row) => {
                let id = Uuid::parse_str(&row.id).map_err(|e| {
                    GameError::Validation {
                        details: format!("Invalid UUID: {}", e)
                })?;
                let name = row.name;
                let player_name = row.player_name.unwrap_or_default();
                let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .map_err(|e| {
                        GameError::validation {
                        details: format!("Invalid created_at: {}", e)
                    })?
                }?;
                let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                    .map_err(|e| {
                        GameError::validation {
                        details: format!("Invalid updated_at: {}", e)
                    })?
                }?;
                let play_time_seconds = row.play_time_seconds as u64;
                let chapter = row.chapter as u32;

                Ok(Some(Save {
                    id,
                    name,
                    player_name,
                    created_at,
                    last_played,
                    play_time_seconds,
                    chapter,
                }))
            }
            None => Ok(None),
        }
    }
}
    /// 获取所有存档
    pub async fn find_all(&self) -> GameResult<Vec<Save>> {
        let result = sqlx::query!(
            r"SELECT id, name, player_name, created_at, updated_at, play_time_seconds, chapter FROM saves ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::database(DatabaseError::queryFailed(e.to_string())))?;

        let mut saves = Vec::new();
        for row in result {
            let id = Uuid::parse_str(&row.id).map_err(|e| {
                GameError::validation {
                    details: format!("Invalid UUID: {}", e)
                })?
                let name = row.name;
                let player_name = row.player_name.unwrap_or_default();
                let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .map_err(|e| {
                        GameError::validation {
                        details: format!("Invalid created_at: {}", e)
                    })?
                }?;
                let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                    .map_err(|e| {
                        GameError::validation {
                        details: format!("Invalid updated_at: {}", e)
                    })?
                }?
                let play_time_seconds = row.play_time_seconds as u64;
                let chapter = row.chapter as u32;

            saves.push(save);
        }

        Ok(saves)
    }
}
    /// 更新存档
    pub async fn update(&self, save: &Save) -> GameResult<()> {
        let result = sqlx::query(
            r"UPDATE saves SET name = ?, player_name = ?, updated_at = ?, play_time_seconds = ?, chapter = ? WHERE id = ?"
        )
        .bind(&save.name)
        .bind(&save.player_name)
        .bind(save.last_played.to_rfc3339())
        .bind(save.play_time_seconds as i64)
        .bind(save.chapter as i64)
        .bind(save.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::database(DatabaseError::writeFailed(e.to_string())))?;
        Ok(())
    }

    /// 删除存档
    pub async fn delete(&self, id: Uuid) -> GameResult<()> {
        let result = sqlx::query(
            r"DELETE FROM saves WHERE id = ?"
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::database(DatabaseError::writeFailed(e.to_string())))?;
        Ok(())
    }
}

/// 存档行（用于 SQL 查询)
#[derive(sqlx::FromRow)]
struct SaveRow {
    id: String,
    name: String,
    player_name: Option<String>,
    created_at: String,
    updated_at: String,
    play_time_seconds: i64,
    chapter: i64,
}
