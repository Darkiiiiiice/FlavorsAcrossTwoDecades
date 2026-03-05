//! 对话消息仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::dialogue::DialogueMessage;
use crate::error::{DatabaseError, GameError, GameResult};

/// 对话消息仓储
pub struct DialogueRepository {
    pool: SqlitePool,
}

impl DialogueRepository {
    /// 创建新的对话仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建新消息
    pub async fn create(&self, message: &DialogueMessage) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO dialogues (id, save_id, sender, content, timestamp, message_type, status)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(message.id.to_string())
        .bind(message.save_id.to_string())
        .bind(&message.sender)
        .bind(&message.content)
        .bind(message.timestamp.to_rfc3339())
        .bind(&message.message_type)
        .bind(&message.status)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找消息
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<DialogueMessage>> {
        let row = sqlx::query_as::<_, DialogueRow>(
            r#"SELECT id, save_id, sender, content, timestamp, message_type, status
               FROM dialogues WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_message()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有消息
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<DialogueMessage>> {
        let rows = sqlx::query_as::<_, DialogueRow>(
            r#"SELECT id, save_id, sender, content, timestamp, message_type, status
               FROM dialogues WHERE save_id = ? ORDER BY timestamp ASC"#
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter()
            .map(|row| row.into_message())
            .collect()
    }

    /// 获取存档的最近N条消息
    pub async fn find_recent(&self, save_id: Uuid, limit: i64) -> GameResult<Vec<DialogueMessage>> {
        let rows = sqlx::query_as::<_, DialogueRow>(
            r#"SELECT id, save_id, sender, content, timestamp, message_type, status
               FROM dialogues WHERE save_id = ?
               ORDER BY timestamp DESC LIMIT ?"#
        )
        .bind(save_id.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        let mut messages: Vec<DialogueMessage> = rows
            .into_iter()
            .map(|row| row.into_message())
            .collect::<GameResult<Vec<_>>>()?;

        // 反转顺序，使最新的消息在最后
        messages.reverse();
        Ok(messages)
    }

    /// 删除消息
    pub async fn delete(&self, id: Uuid) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM dialogues WHERE id = ?"#)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除存档的所有消息
    pub async fn delete_by_save(&self, save_id: Uuid) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM dialogues WHERE save_id = ?"#)
            .bind(save_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

/// 对话消息数据库行
#[derive(sqlx::FromRow)]
struct DialogueRow {
    id: String,
    save_id: String,
    sender: String,
    content: String,
    timestamp: String,
    message_type: String,
    status: String,
}

impl DialogueRow {
    /// 将数据库行转换为 DialogueMessage 模型
    fn into_message(self) -> GameResult<DialogueMessage> {
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

        let timestamp = DateTime::parse_from_rfc3339(&self.timestamp)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid timestamp: {}", e),
            })?
            .with_timezone(&Utc);

        Ok(DialogueMessage {
            id,
            save_id,
            sender: self.sender,
            content: self.content,
            timestamp,
            message_type: self.message_type,
            status: self.status,
        })
    }
}
