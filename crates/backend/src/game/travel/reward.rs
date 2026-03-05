//! 旅行奖励定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 奖励类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelRewardType {
    /// 菜谱
    Recipe,
    /// 食材
    Ingredient,
    /// 记忆碎片
    MemoryFragment,
    /// 特殊物品
    SpecialItem,
    /// 经验值
    Experience,
    /// 金钱
    Money,
}

impl TravelRewardType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            TravelRewardType::Recipe => "菜谱",
            TravelRewardType::Ingredient => "食材",
            TravelRewardType::MemoryFragment => "记忆碎片",
            TravelRewardType::SpecialItem => "特殊物品",
            TravelRewardType::Experience => "经验值",
            TravelRewardType::Money => "金钱",
        }
    }
}

/// 旅行奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelReward {
    /// 奖励类型
    pub reward_type: TravelRewardType,
    /// 奖励名称
    pub name: String,
    /// 奖励描述
    pub description: String,
    /// 奖励值（如菜谱 ID、食材 ID、数量等）
    pub value: String,
}

impl TravelReward {
    /// 创建菜谱奖励
    pub fn recipe(recipe_id: &str, name: &str) -> Self {
        Self {
            reward_type: TravelRewardType::Recipe,
            name: name.to_string(),
            description: format!("从旅行中获得的菜谱：{}", name),
            value: recipe_id.to_string(),
        }
    }

    /// 创建食材奖励
    pub fn ingredient(ingredient_id: &str, name: &str, quantity: u32) -> Self {
        Self {
            reward_type: TravelRewardType::Ingredient,
            name: name.to_string(),
            description: format!("获得 {} x{}", name, quantity),
            value: format!("{}:{}", ingredient_id, quantity),
        }
    }

    /// 创建记忆碎片奖励
    pub fn memory_fragment(fragment_id: &str, name: &str) -> Self {
        Self {
            reward_type: TravelRewardType::MemoryFragment,
            name: name.to_string(),
            description: format!("解锁记忆碎片：{}", name),
            value: fragment_id.to_string(),
        }
    }

    /// 创建经验值奖励
    pub fn experience(amount: u32) -> Self {
        Self {
            reward_type: TravelRewardType::Experience,
            name: format!("{} 经验值", amount),
            description: format!("获得 {} 点经验值", amount),
            value: amount.to_string(),
        }
    }

    /// 创建金钱奖励
    pub fn money(amount: u64) -> Self {
        Self {
            reward_type: TravelRewardType::Money,
            name: format!("{} 元", amount),
            description: format!("获得 {} 元", amount),
            value: amount.to_string(),
        }
    }
}

/// 旅行照片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelPhoto {
    /// 唯一 ID
    pub id: Uuid,
    /// 目的地 ID
    pub destination_id: String,
    /// 目的地名称
    pub destination_name: String,
    /// 照片标题
    pub title: String,
    /// 照片描述
    pub description: String,
    /// 拍摄时间
    pub taken_at: DateTime<Utc>,
    /// 照片类型
    pub photo_type: PhotoType,
    /// 是否为精选
    pub is_featured: bool,
}

impl TravelPhoto {
    /// 创建新照片
    pub fn new(
        destination_id: String,
        destination_name: String,
        title: String,
        photo_type: PhotoType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            destination_id,
            destination_name,
            title,
            description: String::new(),
            taken_at: Utc::now(),
            photo_type,
            is_featured: false,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// 设为精选
    pub fn set_featured(&mut self) {
        self.is_featured = true;
    }
}

/// 照片类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhotoType {
    /// 风景照
    Scenery,
    /// 美食照
    Food,
    /// 人物照
    People,
    /// 街头照
    Street,
    /// 特写照
    CloseUp,
    /// 趣味照
    Fun,
}

impl PhotoType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            PhotoType::Scenery => "风景",
            PhotoType::Food => "美食",
            PhotoType::People => "人物",
            PhotoType::Street => "街头",
            PhotoType::CloseUp => "特写",
            PhotoType::Fun => "趣味",
        }
    }
}

/// 奖励生成器
pub struct RewardGenerator;

impl RewardGenerator {
    /// 根据目的地生成奖励
    pub fn generate_rewards(destination: &super::destination::Destination) -> Vec<TravelReward> {
        use rand::RngExt;

        let mut rng = rand::rng();
        let mut rewards = Vec::new();

        // 生成菜谱奖励
        if !destination.available_recipes.is_empty() {
            let recipe_count = rng
                .random_range(1..=2)
                .min(destination.available_recipes.len());
            let mut available = destination.available_recipes.clone();

            for _ in 0..recipe_count {
                if !available.is_empty() {
                    let idx = rng.random_range(0..available.len());
                    let recipe_id = available.remove(idx);
                    rewards.push(TravelReward::recipe(
                        &recipe_id,
                        &format!("菜谱：{}", recipe_id),
                    ));
                }
            }
        }

        // 生成食材奖励
        if !destination.special_ingredients.is_empty() {
            let ingredient_count = rng
                .random_range(1..=3)
                .min(destination.special_ingredients.len());
            let mut available = destination.special_ingredients.clone();

            for _ in 0..ingredient_count {
                if !available.is_empty() {
                    let idx = rng.random_range(0..available.len());
                    let ingredient_id = available.remove(idx);
                    let quantity = rng.random_range(1..=5);
                    rewards.push(TravelReward::ingredient(
                        &ingredient_id,
                        &format!("食材：{}", ingredient_id),
                        quantity,
                    ));
                }
            }
        }

        // 生成经验值奖励
        let exp = match destination.category {
            super::destination::DestinationCategory::Local => rng.random_range(10..=30),
            super::destination::DestinationCategory::Domestic => rng.random_range(30..=80),
            super::destination::DestinationCategory::International => rng.random_range(80..=150),
            super::destination::DestinationCategory::Special => rng.random_range(100..=200),
        };
        rewards.push(TravelReward::experience(exp));

        rewards
    }

    /// 生成旅行照片
    pub fn generate_photos(destination: &super::destination::Destination) -> Vec<TravelPhoto> {
        use rand::RngExt;

        let mut rng = rand::rng();
        let mut photos = Vec::new();

        let photo_count = rng.random_range(2..=5);

        let photo_types = [
            PhotoType::Scenery,
            PhotoType::Food,
            PhotoType::Street,
            PhotoType::CloseUp,
        ];

        for i in 0..photo_count {
            let photo_type = photo_types[rng.random_range(0..photo_types.len())];
            let title = format!("{}的{}照", destination.name, photo_type.name());

            let photo = TravelPhoto::new(
                destination.id.clone(),
                destination.name.clone(),
                title,
                photo_type,
            )
            .with_description(&format!(
                "在{}拍摄的第{}张照片",
                destination.name,
                i + 1
            ));

            photos.push(photo);
        }

        // 随机选择一张作为精选
        if !photos.is_empty() {
            let featured_idx = rng.random_range(0..photos.len());
            photos[featured_idx].set_featured();
        }

        photos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_reward() {
        let reward = TravelReward::recipe("mapo_tofu", "麻婆豆腐");

        assert_eq!(reward.reward_type, TravelRewardType::Recipe);
        assert_eq!(reward.name, "麻婆豆腐");
    }

    #[test]
    fn test_ingredient_reward() {
        let reward = TravelReward::ingredient("sichuan_pepper", "花椒", 5);

        assert_eq!(reward.reward_type, TravelRewardType::Ingredient);
        assert!(reward.value.contains("5"));
    }

    #[test]
    fn test_experience_reward() {
        let reward = TravelReward::experience(100);

        assert_eq!(reward.reward_type, TravelRewardType::Experience);
        assert_eq!(reward.value, "100");
    }

    #[test]
    fn test_money_reward() {
        let reward = TravelReward::money(500);

        assert_eq!(reward.reward_type, TravelRewardType::Money);
        assert_eq!(reward.value, "500");
    }

    #[test]
    fn test_photo_creation() {
        let photo = TravelPhoto::new(
            "chengdu".to_string(),
            "成都".to_string(),
            "宽窄巷子".to_string(),
            PhotoType::Scenery,
        );

        assert_eq!(photo.destination_id, "chengdu");
        assert_eq!(photo.photo_type, PhotoType::Scenery);
        assert!(!photo.is_featured);
    }

    #[test]
    fn test_photo_featured() {
        let mut photo = TravelPhoto::new(
            "test".to_string(),
            "测试".to_string(),
            "测试照片".to_string(),
            PhotoType::Food,
        );

        photo.set_featured();
        assert!(photo.is_featured);
    }
}
