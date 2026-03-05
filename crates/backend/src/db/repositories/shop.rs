//! 小馆状态仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::shop::ShopState;
use crate::error::GameResult;

/// 小馆状态仓储
pub struct ShopRepository {
    pool: SqlitePool,
}

impl ShopRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找小馆状态
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Option<ShopState>> {
        // TODO: 实现数据库查询
        Ok(None)
    }

    /// 保存小馆状态
    pub async fn save(&self, _state: &ShopState) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
