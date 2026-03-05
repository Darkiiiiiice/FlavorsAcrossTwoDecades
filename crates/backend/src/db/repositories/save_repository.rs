//! 存档仓储

use chrono::{DateTime, Utc};
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
            r#"INSERT INTO saves (id, name, player_name, created_at, updated_at, play_time_seconds, chapter)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
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
        let row = sqlx::query_as::<_, SaveRow>(
            r#"SELECT id, name, player_name, created_at, updated_at, play_time_seconds, chapter
               FROM saves WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_save()?)),
            None => Ok(None),
        }
    }

    /// 获取所有存档
    pub async fn find_all(&self) -> GameResult<Vec<Save>> {
        let rows = sqlx::query_as::<_, SaveRow>(
            r#"SELECT id, name, player_name, created_at, updated_at, play_time_seconds, chapter
               FROM saves ORDER BY updated_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_save()).collect()
    }

    /// 更新存档
    pub async fn update(&self, save: &Save) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE saves SET name = ?, player_name = ?, updated_at = ?,
               play_time_seconds = ?, chapter = ? WHERE id = ?"#,
        )
        .bind(&save.name)
        .bind(&save.player_name)
        .bind(save.last_played.to_rfc3339())
        .bind(save.play_time_seconds as i64)
        .bind(save.chapter as i64)
        .bind(save.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除存档
    pub async fn delete(&self, id: Uuid) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM saves WHERE id = ?"#)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

/// 存档数据库行
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

impl SaveRow {
    /// 将数据库行转换为 Save 模型
    fn into_save(self) -> GameResult<Save> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid created_at: {}", e),
            })?
            .with_timezone(&Utc);

        let last_played = DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid updated_at: {}", e),
            })?
            .with_timezone(&Utc);

        Ok(Save {
            id,
            name: self.name,
            player_name: self.player_name.unwrap_or_default(),
            created_at,
            last_played,
            play_time_seconds: self.play_time_seconds as u64,
            chapter: self.chapter as u32,
        })
    }
}
