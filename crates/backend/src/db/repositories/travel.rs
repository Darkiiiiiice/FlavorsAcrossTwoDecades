//! 旅行状态仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::travel::Travel;
use crate::error::GameResult;

/// 旅行状态仓储
pub struct TravelRepository {
    pool: SqlitePool,
}

impl TravelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有旅行记录
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<Travel>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存旅行记录
    pub async fn save(&self, _travel: &Travel) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
