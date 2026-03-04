//! 记忆碎片定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::UnlockCondition;

/// 记忆碎片类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryFragmentType {
    /// 故事碎片（主线剧情）
    Story,
    /// 角色碎片（邻居/顾客相关）
    Character,
    /// 菜谱碎片（菜品来源故事）
    Recipe,
    /// 地点碎片（旅行记忆）
    Place,
    /// 秘密碎片（隐藏内容）
    Secret,
}

impl MemoryFragmentType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            MemoryFragmentType::Story => "故事",
            MemoryFragmentType::Character => "角色",
            MemoryFragmentType::Recipe => "菜谱",
            MemoryFragmentType::Place => "地点",
            MemoryFragmentType::Secret => "秘密",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            MemoryFragmentType::Story => "📖",
            MemoryFragmentType::Character => "👤",
            MemoryFragmentType::Recipe => "📜",
            MemoryFragmentType::Place => "🗺️",
            MemoryFragmentType::Secret => "🔮",
        }
    }
}

/// 记忆碎片稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryRarity {
    /// 普通
    Common,
    /// 稀有
    Rare,
    /// 史诗
    Epic,
    /// 传说
    Legendary,
}

impl MemoryRarity {
    /// 获取稀有度名称
    pub fn name(&self) -> &str {
        match self {
            MemoryRarity::Common => "普通",
            MemoryRarity::Rare => "稀有",
            MemoryRarity::Epic => "史诗",
            MemoryRarity::Legendary => "传说",
        }
    }

    /// 获取颜色代码
    pub fn color(&self) -> &str {
        match self {
            MemoryRarity::Common => "#808080",
            MemoryRarity::Rare => "#4A90D9",
            MemoryRarity::Epic => "#A335EE",
            MemoryRarity::Legendary => "#FF8000",
        }
    }

    /// 获取记忆点数奖励
    pub fn memory_points(&self) -> u32 {
        match self {
            MemoryRarity::Common => 10,
            MemoryRarity::Rare => 25,
            MemoryRarity::Epic => 50,
            MemoryRarity::Legendary => 100,
        }
    }
}

/// 记忆碎片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// 唯一 ID
    pub id: Uuid,
    /// 碎片 ID（模板 ID）
    pub fragment_id: String,
    /// 碎片名称
    pub name: String,
    /// 碎片描述
    pub description: String,
    /// 碎片类型
    pub fragment_type: MemoryFragmentType,
    /// 稀有度
    pub rarity: MemoryRarity,
    /// 记忆内容
    pub content: Option<super::MemoryContent>,
    /// 解锁条件
    pub unlock_conditions: Vec<UnlockCondition>,
    /// 是否已解锁
    pub is_unlocked: bool,
    /// 是否已阅读
    pub is_read: bool,
    /// 解锁时间
    pub unlocked_at: Option<DateTime<Utc>>,
    /// 关联的角色 ID（如果是角色碎片）
    pub related_character_id: Option<String>,
    /// 关联的菜谱 ID（如果是菜谱碎片）
    pub related_recipe_id: Option<String>,
    /// 关联的地点 ID（如果是地点碎片）
    pub related_place_id: Option<String>,
    /// 章节编号（用于故事碎片排序）
    pub chapter: u32,
    /// 排序权重
    pub sort_order: u32,
}

impl MemoryFragment {
    /// 创建新记忆碎片
    pub fn new(
        fragment_id: String,
        name: String,
        fragment_type: MemoryFragmentType,
        rarity: MemoryRarity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            fragment_id,
            name,
            description: String::new(),
            fragment_type,
            rarity,
            content: None,
            unlock_conditions: Vec::new(),
            is_unlocked: false,
            is_read: false,
            unlocked_at: None,
            related_character_id: None,
            related_recipe_id: None,
            related_place_id: None,
            chapter: 0,
            sort_order: 0,
        }
    }

    /// 添加解锁条件
    pub fn add_unlock_condition(mut self, condition: UnlockCondition) -> Self {
        self.unlock_conditions.push(condition);
        self
    }

    /// 设置关联角色
    pub fn with_character(mut self, character_id: &str) -> Self {
        self.related_character_id = Some(character_id.to_string());
        self
    }

    /// 设置关联菜谱
    pub fn with_recipe(mut self, recipe_id: &str) -> Self {
        self.related_recipe_id = Some(recipe_id.to_string());
        self
    }

    /// 设置关联地点
    pub fn with_place(mut self, place_id: &str) -> Self {
        self.related_place_id = Some(place_id.to_string());
        self
    }

    /// 设置章节
    pub fn with_chapter(mut self, chapter: u32) -> Self {
        self.chapter = chapter;
        self
    }

    /// 设置排序权重
    pub fn with_sort_order(mut self, order: u32) -> Self {
        self.sort_order = order;
        self
    }

    /// 检查是否可以解锁
    pub fn can_unlock(&self) -> bool {
        !self.is_unlocked && self.unlock_conditions.iter().all(|c| c.is_satisfied)
    }

    /// 尝试解锁
    pub fn try_unlock(&mut self) -> bool {
        if self.can_unlock() {
            self.is_unlocked = true;
            self.unlocked_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// 标记为已读
    pub fn mark_as_read(&mut self) {
        if self.is_unlocked {
            self.is_read = true;
        }
    }

    /// 获取记忆点数
    pub fn get_memory_points(&self) -> u32 {
        if self.is_unlocked {
            self.rarity.memory_points()
        } else {
            0
        }
    }
}

/// 记忆碎片模板（用于定义可收集的记忆）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragmentTemplate {
    /// 碎片 ID
    pub fragment_id: String,
    /// 名称
    pub name: String,
    /// 类型
    pub fragment_type: MemoryFragmentType,
    /// 稀有度
    pub rarity: MemoryRarity,
    /// 解锁条件
    pub unlock_conditions: Vec<UnlockCondition>,
    /// 关联 ID
    pub related_id: Option<String>,
    /// 章节
    pub chapter: u32,
    /// 排序
    pub sort_order: u32,
}

impl MemoryFragmentTemplate {
    /// 创建碎片实例
    pub fn create_instance(&self) -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            self.fragment_id.clone(),
            self.name.clone(),
            self.fragment_type,
            self.rarity,
        );
        fragment.unlock_conditions = self.unlock_conditions.clone();
        fragment.chapter = self.chapter;
        fragment.sort_order = self.sort_order;

        match self.fragment_type {
            MemoryFragmentType::Character => {
                fragment.related_character_id = self.related_id.clone();
            }
            MemoryFragmentType::Recipe => {
                fragment.related_recipe_id = self.related_id.clone();
            }
            MemoryFragmentType::Place => {
                fragment.related_place_id = self.related_id.clone();
            }
            _ => {}
        }

        fragment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_creation() {
        let fragment = MemoryFragment::new(
            "story_001".to_string(),
            "初到小馆".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Common,
        );

        assert_eq!(fragment.fragment_id, "story_001");
        assert!(!fragment.is_unlocked);
        assert!(!fragment.is_read);
    }

    #[test]
    fn test_fragment_unlock() {
        let mut fragment = MemoryFragment::new(
            "test".to_string(),
            "测试".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Common,
        );

        // 没有解锁条件时可以直接解锁
        assert!(fragment.can_unlock());
        assert!(fragment.try_unlock());
        assert!(fragment.is_unlocked);
        assert!(fragment.unlocked_at.is_some());
    }

    #[test]
    fn test_fragment_with_condition() {
        let condition = UnlockCondition::trust_level(50);
        let mut fragment = MemoryFragment::new(
            "test".to_string(),
            "测试".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Rare,
        );
        fragment.unlock_conditions.push(condition);

        // 条件不满足，无法解锁
        fragment.unlock_conditions[0].is_satisfied = false;
        assert!(!fragment.can_unlock());
        assert!(!fragment.try_unlock());

        // 条件满足，可以解锁
        fragment.unlock_conditions[0].is_satisfied = true;
        assert!(fragment.can_unlock());
        assert!(fragment.try_unlock());
    }

    #[test]
    fn test_mark_as_read() {
        let mut fragment = MemoryFragment::new(
            "test".to_string(),
            "测试".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Common,
        );

        // 未解锁时无法标记已读
        fragment.mark_as_read();
        assert!(!fragment.is_read);

        // 解锁后可以标记
        fragment.try_unlock();
        fragment.mark_as_read();
        assert!(fragment.is_read);
    }

    #[test]
    fn test_memory_points() {
        let mut fragment = MemoryFragment::new(
            "test".to_string(),
            "测试".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Epic,
        );

        // 未解锁时没有记忆点数
        assert_eq!(fragment.get_memory_points(), 0);

        // 解锁后获得对应点数
        fragment.try_unlock();
        assert_eq!(fragment.get_memory_points(), 50);
    }

    #[test]
    fn test_rarity_points() {
        assert_eq!(MemoryRarity::Common.memory_points(), 10);
        assert_eq!(MemoryRarity::Rare.memory_points(), 25);
        assert_eq!(MemoryRarity::Epic.memory_points(), 50);
        assert_eq!(MemoryRarity::Legendary.memory_points(), 100);
    }
}
