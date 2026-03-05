//! 指令记录仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::command::Command;
use crate::error::GameResult;

/// 指令记录仓储
pub struct CommandRepository {
    pool: SqlitePool,
}

impl CommandRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有指令记录
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<Command>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存指令记录
    pub async fn save(&self, _command: &Command) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
