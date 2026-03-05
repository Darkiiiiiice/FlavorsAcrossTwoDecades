//! 记忆碎片仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::memory::MemoryFragment;
use crate::error::{DatabaseError, GameError, GameResult};

/// 记忆碎片仓储
pub struct MemoryRepository {
    pool: SqlitePool,
}

impl MemoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建记忆碎片
    pub async fn create(&self, fragment: &MemoryFragment) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO memory_fragments (id, save_id, fragment_type, title, content,
               is_unlocked, unlocked_at, trigger_condition)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(fragment.id.to_string())
        .bind(fragment.save_id.to_string())
        .bind(&fragment.fragment_type)
        .bind(&fragment.title)
        .bind(&fragment.content)
        .bind(fragment.is_unlocked as i64)
        .bind(fragment.unlocked_at.map(|t| t.to_rfc3339()))
        .bind(&fragment.trigger_condition)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找记忆碎片
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<MemoryFragment>> {
        let row = sqlx::query_as::<_, MemoryRow>(
            r#"SELECT id, save_id, fragment_type, title, content, is_unlocked, unlocked_at, trigger_condition
               FROM memory_fragments WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_fragment()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有记忆碎片
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<MemoryFragment>> {
        let rows = sqlx::query_as::<_, MemoryRow>(
            r#"SELECT id, save_id, fragment_type, title, content, is_unlocked, unlocked_at, trigger_condition
               FROM memory_fragments WHERE save_id = ? ORDER BY fragment_type, title"#
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_fragment()).collect()
    }

    /// 获取已解锁的记忆碎片
    pub async fn find_unlocked(&self, save_id: Uuid) -> GameResult<Vec<MemoryFragment>> {
        let rows = sqlx::query_as::<_, MemoryRow>(
            r#"SELECT id, save_id, fragment_type, title, content, is_unlocked, unlocked_at, trigger_condition
               FROM memory_fragments WHERE save_id = ? AND is_unlocked = 1
               ORDER BY unlocked_at DESC"#
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_fragment()).collect()
    }

    /// 更新记忆碎片（解锁）
    pub async fn update(&self, fragment: &MemoryFragment) -> GameResult<()> {
        sqlx::query(r#"UPDATE memory_fragments SET is_unlocked = ?, unlocked_at = ? WHERE id = ?"#)
            .bind(fragment.is_unlocked as i64)
            .bind(fragment.unlocked_at.map(|t| t.to_rfc3339()))
            .bind(fragment.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MemoryRow {
    id: String,
    save_id: String,
    fragment_type: String,
    title: String,
    content: String,
    is_unlocked: i64,
    unlocked_at: Option<String>,
    trigger_condition: String,
}

impl MemoryRow {
    fn into_fragment(self) -> GameResult<MemoryFragment> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        let unlocked_at = if let Some(t) = self.unlocked_at {
            Some(
                DateTime::parse_from_rfc3339(&t)
                    .map_err(|e| GameError::Validation {
                        details: format!("Invalid unlocked_at: {}", e),
                    })?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(MemoryFragment {
            id,
            save_id,
            fragment_type: self.fragment_type,
            title: self.title,
            content: self.content,
            is_unlocked: self.is_unlocked != 0,
            unlocked_at,
            trigger_condition: self.trigger_condition,
        })
    }
}
