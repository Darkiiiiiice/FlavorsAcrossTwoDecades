#![allow(clippy::module_inception)]
//! 记忆碎片系统模块
//!
//! 管理玩家在游戏中收集的记忆碎片，逐步揭示小馆的历史和故事

mod fragment;
mod manager;
mod memory;

pub use fragment::{MemoryFragment, MemoryFragmentType, MemoryRarity};
pub use manager::MemoryManager;
pub use memory::{MemoryContent, Sense, SensoryMemory};

use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// 记忆解锁条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockCondition {
    /// 条件类型
    pub condition_type: UnlockConditionType,
    /// 条件描述
    pub description: String,
    /// 条件值
    pub value: String,
    /// 是否已满足
    pub is_satisfied: bool,
}

/// 解锁条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnlockConditionType {
    /// 信任度达到
    TrustLevel,
    /// 完成旅行
    TravelComplete,
    /// 制作菜品
    CookDish,
    /// 顾客互动
    CustomerInteraction,
    /// 特定日期
    SpecialDate,
    /// 对话话题
    DialogTopic,
    /// 修复里程碑
    RepairMilestone,
    /// 菜谱研发
    RecipeExperiment,
    /// 邻居好感
    NeighborAffinity,
    /// 组合条件
    Combined,
}

impl UnlockCondition {
    /// 创建信任度条件
    pub fn trust_level(level: u32) -> Self {
        Self {
            condition_type: UnlockConditionType::TrustLevel,
            description: format!("信任度达到 {}", level),
            value: level.to_string(),
            is_satisfied: false,
        }
    }

    /// 创建旅行完成条件
    pub fn travel_complete(destination_id: &str) -> Self {
        Self {
            condition_type: UnlockConditionType::TravelComplete,
            description: format!("完成 {} 的旅行", destination_id),
            value: destination_id.to_string(),
            is_satisfied: false,
        }
    }

    /// 制作菜品条件
    pub fn cook_dish(recipe_id: &str, count: u32) -> Self {
        Self {
            condition_type: UnlockConditionType::CookDish,
            description: format!("制作 {} 次菜品 {}", count, recipe_id),
            value: format!("{}:{}", recipe_id, count),
            is_satisfied: false,
        }
    }

    /// 特定日期条件
    pub fn special_date(month: u32, day: u32) -> Self {
        Self {
            condition_type: UnlockConditionType::SpecialDate,
            description: format!("在 {}/{} 访问", month, day),
            value: format!("{}/{}", month, day),
            is_satisfied: false,
        }
    }

    /// 顾客互动条件
    pub fn customer_interaction(customer_id: &str) -> Self {
        Self {
            condition_type: UnlockConditionType::CustomerInteraction,
            description: format!("与 {} 互动", customer_id),
            value: customer_id.to_string(),
            is_satisfied: false,
        }
    }

    /// 检查条件是否满足
    pub fn check(&mut self, context: &UnlockContext) -> bool {
        self.is_satisfied = match self.condition_type {
            UnlockConditionType::TrustLevel => {
                if let Ok(level) = self.value.parse::<u32>() {
                    context.trust_level >= level
                } else {
                    false
                }
            }
            UnlockConditionType::TravelComplete => {
                context.completed_destinations.contains(&self.value)
            }
            UnlockConditionType::CookDish => {
                // 解析 "recipe_id:count" 格式
                let parts: Vec<&str> = self.value.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(count) = parts[1].parse::<u32>() {
                        context.cooked_dishes.get(parts[0]).copied().unwrap_or(0) >= count
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            UnlockConditionType::CustomerInteraction => {
                context.customer_interactions.contains(&self.value)
            }
            UnlockConditionType::SpecialDate => {
                let parts: Vec<&str> = self.value.split('/').collect();
                if parts.len() == 2 {
                    if let (Ok(month), Ok(day)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                    {
                        context.current_date.month() == month && context.current_date.day() == day
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            UnlockConditionType::DialogTopic => context.discussed_topics.contains(&self.value),
            UnlockConditionType::RepairMilestone => context.repair_milestones.contains(&self.value),
            UnlockConditionType::RecipeExperiment => {
                context.completed_experiments.contains(&self.value)
            }
            UnlockConditionType::NeighborAffinity => {
                // 解析 "neighbor_id:level" 格式
                let parts: Vec<&str> = self.value.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(level) = parts[1].parse::<u32>() {
                        context
                            .neighbor_affinity
                            .get(parts[0])
                            .copied()
                            .unwrap_or(0)
                            >= level
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            UnlockConditionType::Combined => {
                // 组合条件需要特殊处理
                false
            }
        };
        self.is_satisfied
    }
}

/// 解锁上下文
#[derive(Debug, Clone, Default)]
pub struct UnlockContext {
    /// 当前信任度
    pub trust_level: u32,
    /// 已完成的目的地
    pub completed_destinations: Vec<String>,
    /// 已制作的菜品 (recipe_id -> count)
    pub cooked_dishes: std::collections::HashMap<String, u32>,
    /// 顾客互动
    pub customer_interactions: Vec<String>,
    /// 当前日期
    pub current_date: chrono::NaiveDate,
    /// 已讨论的话题
    pub discussed_topics: Vec<String>,
    /// 修复里程碑
    pub repair_milestones: Vec<String>,
    /// 已完成的实验
    pub completed_experiments: Vec<String>,
    /// 邻居好感度
    pub neighbor_affinity: std::collections::HashMap<String, u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_condition() {
        let mut condition = UnlockCondition::trust_level(50);
        let mut context = UnlockContext::default();

        context.trust_level = 30;
        assert!(!condition.check(&context));

        context.trust_level = 50;
        assert!(condition.check(&context));
    }

    #[test]
    fn test_travel_complete_condition() {
        let mut condition = UnlockCondition::travel_complete("chengdu");
        let mut context = UnlockContext::default();

        assert!(!condition.check(&context));

        context.completed_destinations.push("chengdu".to_string());
        assert!(condition.check(&context));
    }

    #[test]
    fn test_cook_dish_condition() {
        let mut condition = UnlockCondition::cook_dish("mapo_tofu", 3);
        let mut context = UnlockContext::default();

        context.cooked_dishes.insert("mapo_tofu".to_string(), 2);
        assert!(!condition.check(&context));

        context.cooked_dishes.insert("mapo_tofu".to_string(), 3);
        assert!(condition.check(&context));
    }
}
