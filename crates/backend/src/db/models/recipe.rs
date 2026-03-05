//! 菜谱数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::recipe::{RecipeCategory, RecipeSource, RecipeStatus};

/// 菜谱记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// 菜谱ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 名称
    pub name: String,
    /// 类别
    pub category: RecipeCategory,
    /// 状态
    pub status: RecipeStatus,
    /// 食材列表（JSON）
    pub ingredients: String,
    /// 来源
    pub source: RecipeSource,
    /// 解锁条件
    pub unlock_condition: Option<String>,
}

impl Recipe {
    /// 创建新菜谱
    pub fn new(
        save_id: Uuid,
        name: String,
        category: RecipeCategory,
        ingredients: String,
        source: RecipeSource,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            name,
            category,
            status: RecipeStatus::Fuzzy,
            ingredients,
            source,
            unlock_condition: None,
        }
    }
}
