//! 盼盼状态仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::panpan::PanpanState;
use crate::error::GameResult;

/// 盼盼状态仓储
pub struct PanpanRepository {
    pool: SqlitePool,
}

impl PanpanRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找盼盼状态
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Option<PanpanState>> {
        // TODO: 实现数据库查询
        Ok(None)
    }

    /// 保存盼盼状态
    pub async fn save(&self, _state: &PanpanState) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
