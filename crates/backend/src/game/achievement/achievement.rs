//! 成就定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AchievementReward;

/// 成就类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementCategory {
    /// 经营成就
    Business,
    /// 探索成就
    Exploration,
    /// 社交成就
    Social,
    /// 烹饪成就
    Cooking,
    /// 收集成就
    Collection,
    /// 故事成就
    Story,
    /// 隐藏成就
    Hidden,
}

impl AchievementCategory {
    /// 获取类别名称
    pub fn name(&self) -> &str {
        match self {
            AchievementCategory::Business => "经营",
            AchievementCategory::Exploration => "探索",
            AchievementCategory::Social => "社交",
            AchievementCategory::Cooking => "烹饪",
            AchievementCategory::Collection => "收集",
            AchievementCategory::Story => "故事",
            AchievementCategory::Hidden => "隐藏",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            AchievementCategory::Business => "💼",
            AchievementCategory::Exploration => "🗺️",
            AchievementCategory::Social => "👥",
            AchievementCategory::Cooking => "🍳",
            AchievementCategory::Collection => "📦",
            AchievementCategory::Story => "📖",
            AchievementCategory::Hidden => "🔮",
        }
    }
}

/// 成就稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRarity {
    /// 普通
    Common,
    /// 稀有
    Rare,
    /// 史诗
    Epic,
    /// 传说
    Legendary,
}

impl AchievementRarity {
    /// 获取稀有度名称
    pub fn name(&self) -> &str {
        match self {
            AchievementRarity::Common => "普通",
            AchievementRarity::Rare => "稀有",
            AchievementRarity::Epic => "史诗",
            AchievementRarity::Legendary => "传说",
        }
    }

    /// 获取颜色
    pub fn color(&self) -> &str {
        match self {
            AchievementRarity::Common => "#808080",
            AchievementRarity::Rare => "#4A90D9",
            AchievementRarity::Epic => "#A335EE",
            AchievementRarity::Legendary => "#FF8000",
        }
    }

    /// 获取点数奖励
    pub fn points(&self) -> u32 {
        match self {
            AchievementRarity::Common => 10,
            AchievementRarity::Rare => 25,
            AchievementRarity::Epic => 50,
            AchievementRarity::Legendary => 100,
        }
    }
}

/// 成就状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementStatus {
    /// 未解锁
    Locked,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已领取奖励
    Claimed,
}

/// 成就条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementCondition {
    /// 条件类型
    pub condition_type: AchievementConditionType,
    /// 目标值
    pub target_value: u32,
    /// 当前进度
    pub current_value: u32,
    /// 条件描述
    pub description: String,
}

/// 成就条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementConditionType {
    /// 营业额
    Revenue,
    /// 口碑评分
    Reputation,
    /// 顾客数量
    CustomerCount,
    /// 旅行次数
    TravelCount,
    /// 目的地收集
    DestinationCollection,
    /// 邻居好感度
    NeighborAffinity,
    /// 菜谱数量
    RecipeCount,
    /// 菜品品质
    DishQuality,
    /// 记忆碎片收集
    MemoryFragmentCollection,
    /// 游戏天数
    GameDays,
    /// 特殊事件
    SpecialEvent,
}

impl AchievementCondition {
    /// 创建新条件
    pub fn new(condition_type: AchievementConditionType, target_value: u32, description: String) -> Self {
        Self {
            condition_type,
            target_value,
            current_value: 0,
            description,
        }
    }

    /// 更新进度
    pub fn update_progress(&mut self, value: u32) {
        self.current_value = value.min(self.target_value);
    }

    /// 增加进度
    pub fn add_progress(&mut self, amount: u32) {
        self.current_value = (self.current_value + amount).min(self.target_value);
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        self.current_value >= self.target_value
    }

    /// 获取进度百分比
    pub fn progress_percentage(&self) -> f32 {
        if self.target_value == 0 {
            return 100.0;
        }
        (self.current_value as f32 / self.target_value as f32 * 100.0).min(100.0)
    }
}

/// 成就
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    /// 成就 ID
    pub id: String,
    /// 成就名称
    pub name: String,
    /// 成就描述
    pub description: String,
    /// 类别
    pub category: AchievementCategory,
    /// 稀有度
    pub rarity: AchievementRarity,
    /// 状态
    pub status: AchievementStatus,
    /// 解锁条件
    pub conditions: Vec<AchievementCondition>,
    /// 奖励
    pub rewards: Vec<AchievementReward>,
    /// 图标
    pub icon: String,
    /// 是否隐藏（未解锁前不显示详情）
    pub is_hidden: bool,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 领取时间
    pub claimed_at: Option<DateTime<Utc>>,
}

impl Achievement {
    /// 创建新成就
    pub fn new(id: String, name: String, category: AchievementCategory, rarity: AchievementRarity) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            category,
            rarity,
            status: AchievementStatus::Locked,
            conditions: Vec::new(),
            rewards: Vec::new(),
            icon: "🏆".to_string(),
            is_hidden: false,
            completed_at: None,
            claimed_at: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 添加条件
    pub fn with_condition(mut self, condition: AchievementCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 添加奖励
    pub fn with_reward(mut self, reward: AchievementReward) -> Self {
        self.rewards.push(reward);
        self
    }

    /// 设置图标
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = icon;
        self
    }

    /// 设置为隐藏成就
    pub fn set_hidden(mut self) -> Self {
        self.is_hidden = true;
        self
    }

    /// 更新进度
    pub fn update_progress(&mut self, condition_type: AchievementConditionType, value: u32) {
        let all_locked = self.conditions.iter().all(|c| c.current_value == 0);

        for condition in &mut self.conditions {
            if condition.condition_type == condition_type {
                condition.update_progress(value);
            }
        }

        // 如果有任何进度，状态变为进行中
        if all_locked && self.conditions.iter().any(|c| c.current_value > 0) {
            self.status = AchievementStatus::InProgress;
        }

        // 检查是否全部完成
        if self.conditions.iter().all(|c| c.is_completed()) {
            self.status = AchievementStatus::Completed;
            self.completed_at = Some(Utc::now());
        }
    }

    /// 领取奖励
    pub fn claim_rewards(&mut self) -> Option<Vec<AchievementReward>> {
        if self.status == AchievementStatus::Completed {
            self.status = AchievementStatus::Claimed;
            self.claimed_at = Some(Utc::now());
            Some(self.rewards.clone())
        } else {
            None
        }
    }

    /// 获取总体进度
    pub fn overall_progress(&self) -> f32 {
        if self.conditions.is_empty() {
            return 0.0;
        }
        self.conditions.iter().map(|c| c.progress_percentage()).sum::<f32>()
            / self.conditions.len() as f32
    }

    /// 获取点数
    pub fn get_points(&self) -> u32 {
        self.rarity.points()
    }

    // ========== 预定义成就 ==========

    /// 初次营业
    pub fn first_day_open() -> Self {
        Self::new(
            "first_day_open".to_string(),
            "初次营业".to_string(),
            AchievementCategory::Business,
            AchievementRarity::Common,
        )
        .with_description("完成小馆重新开业的第一天".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::GameDays,
            1,
            "营业1天".to_string(),
        ))
        .with_reward(AchievementReward {
            reward_type: super::AchievementRewardType::Money,
            amount: 100,
            description: "获得100元启动资金".to_string(),
        })
        .with_icon("🏪".to_string())
    }

    /// 小有名气
    pub fn gaining_fame() -> Self {
        Self::new(
            "gaining_fame".to_string(),
            "小有名气".to_string(),
            AchievementCategory::Business,
            AchievementRarity::Common,
        )
        .with_description("口碑评分达到60分".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::Reputation,
            60,
            "口碑达到60分".to_string(),
        ))
        .with_icon("⭐".to_string())
    }

    /// 口碑载道
    pub fn good_reputation() -> Self {
        Self::new(
            "good_reputation".to_string(),
            "口碑载道".to_string(),
            AchievementCategory::Business,
            AchievementRarity::Rare,
        )
        .with_description("口碑评分达到80分".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::Reputation,
            80,
            "口碑达到80分".to_string(),
        ))
        .with_icon("🌟".to_string())
    }

    /// 远近闻名
    pub fn famous() -> Self {
        Self::new(
            "famous".to_string(),
            "远近闻名".to_string(),
            AchievementCategory::Business,
            AchievementRarity::Epic,
        )
        .with_description("口碑评分达到95分".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::Reputation,
            95,
            "口碑达到95分".to_string(),
        ))
        .with_icon("💫".to_string())
    }

    /// 初次旅行
    pub fn first_travel() -> Self {
        Self::new(
            "first_travel".to_string(),
            "初次旅行".to_string(),
            AchievementCategory::Exploration,
            AchievementRarity::Common,
        )
        .with_description("完成第一次旅行".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::TravelCount,
            1,
            "完成1次旅行".to_string(),
        ))
        .with_icon("✈️".to_string())
    }

    /// 旅行达人
    pub fn travel_master() -> Self {
        Self::new(
            "travel_master".to_string(),
            "旅行达人".to_string(),
            AchievementCategory::Exploration,
            AchievementRarity::Rare,
        )
        .with_description("完成5次旅行".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::TravelCount,
            5,
            "完成5次旅行".to_string(),
        ))
        .with_icon("🌍".to_string())
    }

    /// 好客之道
    pub fn hospitality() -> Self {
        Self::new(
            "hospitality".to_string(),
            "好客之道".to_string(),
            AchievementCategory::Social,
            AchievementRarity::Common,
        )
        .with_description("接待100位顾客".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::CustomerCount,
            100,
            "接待100位顾客".to_string(),
        ))
        .with_icon("🤝".to_string())
    }

    /// 邻里和睦
    pub fn neighborhood_harmony() -> Self {
        Self::new(
            "neighborhood_harmony".to_string(),
            "邻里和睦".to_string(),
            AchievementCategory::Social,
            AchievementRarity::Rare,
        )
        .with_description("与3位邻居成为好友（好感度60+）".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::NeighborAffinity,
            3,
            "3位邻居好感度60+".to_string(),
        ))
        .with_icon("🏘️".to_string())
    }

    /// 初试厨艺
    pub fn first_dish() -> Self {
        Self::new(
            "first_dish".to_string(),
            "初试厨艺".to_string(),
            AchievementCategory::Cooking,
            AchievementRarity::Common,
        )
        .with_description("成功制作第一道菜".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::RecipeCount,
            1,
            "制作1道菜".to_string(),
        ))
        .with_icon("👨‍🍳".to_string())
    }

    /// 菜谱收集者
    pub fn recipe_collector() -> Self {
        Self::new(
            "recipe_collector".to_string(),
            "菜谱收集者".to_string(),
            AchievementCategory::Cooking,
            AchievementRarity::Rare,
        )
        .with_description("收集10份菜谱".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::RecipeCount,
            10,
            "收集10份菜谱".to_string(),
        ))
        .with_icon("📚".to_string())
    }

    /// 记忆碎片
    pub fn memory_collector() -> Self {
        Self::new(
            "memory_collector".to_string(),
            "记忆收集者".to_string(),
            AchievementCategory::Collection,
            AchievementRarity::Rare,
        )
        .with_description("收集5个记忆碎片".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::MemoryFragmentCollection,
            5,
            "收集5个记忆碎片".to_string(),
        ))
        .with_icon("🧩".to_string())
    }

    /// 祖父的遗产
    pub fn grandfathers_legacy() -> Self {
        Self::new(
            "grandfathers_legacy".to_string(),
            "祖父的遗产".to_string(),
            AchievementCategory::Story,
            AchievementRarity::Legendary,
        )
        .with_description("解锁所有故事类记忆碎片".to_string())
        .with_condition(AchievementCondition::new(
            AchievementConditionType::MemoryFragmentCollection,
            3,
            "收集所有故事碎片".to_string(),
        ))
        .with_icon("👑".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_creation() {
        let achievement = Achievement::first_day_open();
        assert_eq!(achievement.id, "first_day_open");
        assert_eq!(achievement.category, AchievementCategory::Business);
        assert_eq!(achievement.status, AchievementStatus::Locked);
    }

    #[test]
    fn test_achievement_progress() {
        let mut achievement = Achievement::first_day_open();

        achievement.update_progress(AchievementConditionType::GameDays, 1);

        assert_eq!(achievement.status, AchievementStatus::Completed);
        assert!(achievement.completed_at.is_some());
    }

    #[test]
    fn test_achievement_condition() {
        let mut condition = AchievementCondition::new(
            AchievementConditionType::CustomerCount,
            100,
            "接待100位顾客".to_string(),
        );

        assert!(!condition.is_completed());

        condition.update_progress(50);
        assert_eq!(condition.progress_percentage(), 50.0);

        condition.update_progress(100);
        assert!(condition.is_completed());
    }

    #[test]
    fn test_claim_rewards() {
        let mut achievement = Achievement::first_day_open();
        achievement.update_progress(AchievementConditionType::GameDays, 1);

        let rewards = achievement.claim_rewards();
        assert!(rewards.is_some());
        assert_eq!(achievement.status, AchievementStatus::Claimed);

        // 再次领取应该失败
        let rewards2 = achievement.claim_rewards();
        assert!(rewards2.is_none());
    }

    #[test]
    fn test_category_icons() {
        assert_eq!(AchievementCategory::Business.icon(), "💼");
        assert_eq!(AchievementCategory::Cooking.icon(), "🍳");
        assert_eq!(AchievementCategory::Hidden.icon(), "🔮");
    }
}
