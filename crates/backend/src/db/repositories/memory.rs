//! 记忆碎片仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::memory::MemoryFragment;
use crate::error::GameResult;

/// 记忆碎片仓储
pub struct MemoryRepository {
    pool: SqlitePool,
}

impl MemoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有记忆碎片
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<MemoryFragment>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存记忆碎片
    pub async fn save(&self, _fragment: &MemoryFragment) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
