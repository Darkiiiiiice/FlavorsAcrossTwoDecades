//! 成就系统模块
//!
//! 管理游戏中的成就、里程碑和解锁奖励

mod achievement;
mod manager;

pub use achievement::{
    Achievement, AchievementCategory, AchievementCondition, AchievementConditionType,
    AchievementRarity, AchievementStatus,
};
pub use manager::AchievementManager;

use serde::{Deserialize, Serialize};

/// 成就解锁奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementReward {
    /// 奖励类型
    pub reward_type: AchievementRewardType,
    /// 奖励数量
    pub amount: u32,
    /// 奖励描述
    pub description: String,
}

/// 成就奖励类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRewardType {
    /// 金钱
    Money,
    /// 经验值
    Experience,
    /// 记忆点数
    MemoryPoints,
    /// 特殊物品
    SpecialItem,
    /// 解锁内容
    UnlockContent,
    /// 称号
    Title,
}

impl AchievementRewardType {
    /// 获取名称
    pub fn name(&self) -> &str {
        match self {
            AchievementRewardType::Money => "金钱",
            AchievementRewardType::Experience => "经验值",
            AchievementRewardType::MemoryPoints => "记忆点数",
            AchievementRewardType::SpecialItem => "特殊物品",
            AchievementRewardType::UnlockContent => "解锁内容",
            AchievementRewardType::Title => "称号",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_type() {
        assert_eq!(AchievementRewardType::Money.name(), "金钱");
        assert_eq!(AchievementRewardType::Title.name(), "称号");
    }
}
