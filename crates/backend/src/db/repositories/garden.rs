//! 菜地状态仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::garden::GardenPlot;
use crate::error::GameResult;

/// 菜地状态仓储
pub struct GardenRepository {
    pool: SqlitePool,
}

impl GardenRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有菜地
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<GardenPlot>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存菜地状态
    pub async fn save(&self, _plot: &GardenPlot) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
