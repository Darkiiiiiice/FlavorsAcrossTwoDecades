//! 指令仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::command::Command;
use crate::error::{DatabaseError, GameError, GameResult};
use crate::game::command::CommandStatus;

/// 指令仓储
pub struct CommandRepository {
    pool: SqlitePool,
}

impl CommandRepository {
    /// 创建新的指令仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建新指令
    pub async fn create(&self, command: &Command) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO commands (id, save_id, content, created_at, arrival_time, status, result)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(command.id.to_string())
        .bind(command.save_id.to_string())
        .bind(&command.content)
        .bind(command.created_at.to_rfc3339())
        .bind(command.arrival_time.to_rfc3339())
        .bind(Self::status_to_string(&command.status))
        .bind(&command.result)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找指令
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<Command>> {
        let row = sqlx::query_as::<_, CommandRow>(
            r#"SELECT id, save_id, content, created_at, arrival_time, status, result
               FROM commands WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_command()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有指令
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<Command>> {
        let rows = sqlx::query_as::<_, CommandRow>(
            r#"SELECT id, save_id, content, created_at, arrival_time, status, result
               FROM commands WHERE save_id = ? ORDER BY created_at DESC"#,
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_command()).collect()
    }

    /// 获取已到达但未完成的指令
    pub async fn find_arrived_commands(&self, save_id: Uuid) -> GameResult<Vec<Command>> {
        let now = Utc::now();
        let rows = sqlx::query_as::<_, CommandRow>(
            r#"SELECT id, save_id, content, created_at, arrival_time, status, result
               FROM commands
               WHERE save_id = ? AND arrival_time <= ? AND status = ?
               ORDER BY arrival_time ASC"#,
        )
        .bind(save_id.to_string())
        .bind(now.to_rfc3339())
        .bind(Self::status_to_string(&CommandStatus::Pending))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_command()).collect()
    }

    /// 更新指令状态
    pub async fn update_status(
        &self,
        id: Uuid,
        status: CommandStatus,
        result: Option<String>,
    ) -> GameResult<()> {
        sqlx::query(r#"UPDATE commands SET status = ?, result = ? WHERE id = ?"#)
            .bind(Self::status_to_string(&status))
            .bind(&result)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除指令
    pub async fn delete(&self, id: Uuid) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM commands WHERE id = ?"#)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 将状态转换为字符串
    fn status_to_string(status: &CommandStatus) -> String {
        match status {
            CommandStatus::Pending => "pending".to_string(),
            CommandStatus::InTransit => "in_transit".to_string(),
            CommandStatus::Arrived => "arrived".to_string(),
            CommandStatus::Processing => "processing".to_string(),
            CommandStatus::Completed => "completed".to_string(),
            CommandStatus::Failed(msg) => format!("failed:{}", msg),
        }
    }

    /// 从字符串解析状态
    fn string_to_status(s: &str) -> GameResult<CommandStatus> {
        if s.starts_with("failed:") {
            let msg = s.strip_prefix("failed:").unwrap_or("");
            return Ok(CommandStatus::Failed(msg.to_string()));
        }

        match s {
            "pending" => Ok(CommandStatus::Pending),
            "in_transit" => Ok(CommandStatus::InTransit),
            "arrived" => Ok(CommandStatus::Arrived),
            "processing" => Ok(CommandStatus::Processing),
            "completed" => Ok(CommandStatus::Completed),
            _ => Err(GameError::Validation {
                details: format!("Invalid command status: {}", s),
            }),
        }
    }
}

/// 指令数据库行
#[derive(sqlx::FromRow)]
struct CommandRow {
    id: String,
    save_id: String,
    content: String,
    created_at: String,
    arrival_time: String,
    status: String,
    result: Option<String>,
}

impl CommandRow {
    /// 将数据库行转换为 Command 模型
    fn into_command(self) -> GameResult<Command> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid created_at: {}", e),
            })?
            .with_timezone(&Utc);

        let arrival_time = DateTime::parse_from_rfc3339(&self.arrival_time)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid arrival_time: {}", e),
            })?
            .with_timezone(&Utc);

        let status = CommandRepository::string_to_status(&self.status)?;

        Ok(Command {
            id,
            save_id,
            content: self.content,
            created_at,
            arrival_time,
            status,
            result: self.result,
        })
    }
}
