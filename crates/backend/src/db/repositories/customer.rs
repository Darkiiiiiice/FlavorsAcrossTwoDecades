//! 顾客记录仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::customer::CustomerRecord;
use crate::error::GameResult;

/// 顾客记录仓储
pub struct CustomerRepository {
    pool: SqlitePool,
}

impl CustomerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有顾客记录
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<CustomerRecord>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存顾客记录
    pub async fn save(&self, _customer: &CustomerRecord) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
