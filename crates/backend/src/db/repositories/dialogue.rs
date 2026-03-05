//! 对话消息仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::dialogue::DialogueMessage;
use crate::error::GameResult;

/// 对话消息仓储
pub struct DialogueRepository {
    pool: SqlitePool,
}

impl DialogueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有对话消息
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<DialogueMessage>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存对话消息
    pub async fn save(&self, _message: &DialogueMessage) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
