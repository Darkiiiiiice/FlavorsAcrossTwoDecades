//! 菜谱仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::recipe::Recipe;
use crate::error::GameResult;

/// 菜谱仓储
pub struct RecipeRepository {
    pool: SqlitePool,
}

impl RecipeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 根据 save_id 查找所有菜谱
    pub async fn find_by_save_id(&self, _save_id: Uuid) -> GameResult<Vec<Recipe>> {
        // TODO: 实现数据库查询
        Ok(vec![])
    }

    /// 保存菜谱
    pub async fn save(&self, _recipe: &Recipe) -> GameResult<()> {
        // TODO: 实现数据库保存
        Ok(())
    }
}
