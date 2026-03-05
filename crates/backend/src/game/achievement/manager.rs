//! 成就管理器

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Achievement, AchievementCategory, AchievementConditionType, AchievementStatus};

/// 成就管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementManager {
    /// 所有成就
    pub achievements: Vec<Achievement>,
    /// 成就点数总计
    pub total_points: u32,
    /// 已领取的奖励
    pub claimed_rewards: Vec<super::AchievementReward>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl AchievementManager {
    /// 创建新的成就管理器
    pub fn new() -> Self {
        Self {
            achievements: Self::create_default_achievements(),
            total_points: 0,
            claimed_rewards: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// 创建默认成就
    fn create_default_achievements() -> Vec<Achievement> {
        vec![
            // 经营成就
            Achievement::first_day_open(),
            Achievement::gaining_fame(),
            Achievement::good_reputation(),
            Achievement::famous(),
            // 探索成就
            Achievement::first_travel(),
            Achievement::travel_master(),
            // 社交成就
            Achievement::hospitality(),
            Achievement::neighborhood_harmony(),
            // 烹饪成就
            Achievement::first_dish(),
            Achievement::recipe_collector(),
            // 收集成就
            Achievement::memory_collector(),
            // 故事成就
            Achievement::grandfathers_legacy(),
        ]
    }

    /// 获取成就
    pub fn get_achievement(&self, id: &str) -> Option<&Achievement> {
        self.achievements.iter().find(|a| a.id == id)
    }

    /// 获取成就（可变）
    pub fn get_achievement_mut(&mut self, id: &str) -> Option<&mut Achievement> {
        self.achievements.iter_mut().find(|a| a.id == id)
    }

    /// 更新成就进度
    pub fn update_progress(&mut self, condition_type: AchievementConditionType, value: u32) {
        let mut completed_ids = Vec::new();

        for achievement in &mut self.achievements {
            let was_not_completed = achievement.status != AchievementStatus::Completed
                && achievement.status != AchievementStatus::Claimed;

            achievement.update_progress(condition_type, value);

            if was_not_completed && achievement.status == AchievementStatus::Completed {
                completed_ids.push(achievement.id.clone());
                self.total_points += achievement.get_points();
            }
        }

        if !completed_ids.is_empty() {
            self.updated_at = Utc::now();
        }
    }

    /// 领取成就奖励
    pub fn claim_reward(&mut self, achievement_id: &str) -> Option<Vec<super::AchievementReward>> {
        if let Some(achievement) = self.get_achievement_mut(achievement_id) {
            let rewards = achievement.claim_rewards();
            if let Some(ref rewards_list) = rewards {
                self.claimed_rewards.extend(rewards_list.clone());
                self.updated_at = Utc::now();
            }
            return rewards;
        }
        None
    }

    /// 获取已完成的成就
    pub fn get_completed_achievements(&self) -> Vec<&Achievement> {
        self.achievements
            .iter()
            .filter(|a| {
                a.status == AchievementStatus::Completed || a.status == AchievementStatus::Claimed
            })
            .collect()
    }

    /// 获取进行中的成就
    pub fn get_in_progress_achievements(&self) -> Vec<&Achievement> {
        self.achievements
            .iter()
            .filter(|a| a.status == AchievementStatus::InProgress)
            .collect()
    }

    /// 获取未解锁的成就
    pub fn get_locked_achievements(&self) -> Vec<&Achievement> {
        self.achievements
            .iter()
            .filter(|a| a.status == AchievementStatus::Locked)
            .collect()
    }

    /// 按类别获取成就
    pub fn get_achievements_by_category(&self, category: AchievementCategory) -> Vec<&Achievement> {
        self.achievements
            .iter()
            .filter(|a| a.category == category)
            .collect()
    }

    /// 获取待领取的成就
    pub fn get_claimable_achievements(&self) -> Vec<&Achievement> {
        self.achievements
            .iter()
            .filter(|a| a.status == AchievementStatus::Completed)
            .collect()
    }

    /// 获取进度统计
    pub fn get_progress_stats(&self) -> AchievementProgressStats {
        let mut stats = AchievementProgressStats::default();

        for achievement in &self.achievements {
            match achievement.status {
                AchievementStatus::Locked => stats.locked += 1,
                AchievementStatus::InProgress => stats.in_progress += 1,
                AchievementStatus::Completed => stats.completed += 1,
                AchievementStatus::Claimed => stats.claimed += 1,
            }
        }

        stats.total = self.achievements.len() as u32;
        stats.total_points = self.total_points;
        stats
    }

    /// 获取类别统计
    pub fn get_category_stats(&self) -> HashMap<AchievementCategory, CategoryStats> {
        let mut stats = HashMap::new();

        for category in [
            AchievementCategory::Business,
            AchievementCategory::Exploration,
            AchievementCategory::Social,
            AchievementCategory::Cooking,
            AchievementCategory::Collection,
            AchievementCategory::Story,
            AchievementCategory::Hidden,
        ] {
            let achievements = self.get_achievements_by_category(category);
            let completed = achievements
                .iter()
                .filter(|a| {
                    a.status == AchievementStatus::Completed
                        || a.status == AchievementStatus::Claimed
                })
                .count();

            stats.insert(
                category,
                CategoryStats {
                    total: achievements.len() as u32,
                    completed: completed as u32,
                },
            );
        }

        stats
    }
}

impl Default for AchievementManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 成就进度统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AchievementProgressStats {
    /// 总数
    pub total: u32,
    /// 未解锁
    pub locked: u32,
    /// 进行中
    pub in_progress: u32,
    /// 已完成
    pub completed: u32,
    /// 已领取
    pub claimed: u32,
    /// 总点数
    pub total_points: u32,
}

impl AchievementProgressStats {
    /// 获取完成百分比
    pub fn completion_percentage(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        ((self.completed + self.claimed) as f32 / self.total as f32) * 100.0
    }
}

/// 类别统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    /// 总数
    pub total: u32,
    /// 已完成
    pub completed: u32,
}

impl CategoryStats {
    /// 获取完成百分比
    pub fn completion_percentage(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f32 / self.total as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_manager_creation() {
        let manager = AchievementManager::new();
        assert!(!manager.achievements.is_empty());
        assert_eq!(manager.total_points, 0);
    }

    #[test]
    fn test_update_progress() {
        let mut manager = AchievementManager::new();

        // 更新游戏天数
        manager.update_progress(AchievementConditionType::GameDays, 1);

        // 检查"初次营业"成就是否完成
        let achievement = manager.get_achievement("first_day_open");
        assert!(achievement.is_some());
        assert_eq!(achievement.unwrap().status, AchievementStatus::Completed);
        assert!(manager.total_points > 0);
    }

    #[test]
    fn test_claim_reward() {
        let mut manager = AchievementManager::new();
        manager.update_progress(AchievementConditionType::GameDays, 1);

        let rewards = manager.claim_reward("first_day_open");
        assert!(rewards.is_some());

        let achievement = manager.get_achievement("first_day_open").unwrap();
        assert_eq!(achievement.status, AchievementStatus::Claimed);
    }

    #[test]
    fn test_get_achievements_by_category() {
        let manager = AchievementManager::new();

        let business = manager.get_achievements_by_category(AchievementCategory::Business);
        assert!(!business.is_empty());

        let cooking = manager.get_achievements_by_category(AchievementCategory::Cooking);
        assert!(!cooking.is_empty());
    }

    #[test]
    fn test_progress_stats() {
        let mut manager = AchievementManager::new();
        manager.update_progress(AchievementConditionType::GameDays, 1);

        let stats = manager.get_progress_stats();
        assert!(stats.completed > 0 || stats.claimed > 0);
        assert!(stats.completion_percentage() > 0.0);
    }

    #[test]
    fn test_category_stats() {
        let mut manager = AchievementManager::new();
        manager.update_progress(AchievementConditionType::GameDays, 1);
        manager.update_progress(AchievementConditionType::Reputation, 60);

        let stats = manager.get_category_stats();
        let business_stats = stats.get(&AchievementCategory::Business).unwrap();
        assert!(business_stats.completed > 0);
    }
}
