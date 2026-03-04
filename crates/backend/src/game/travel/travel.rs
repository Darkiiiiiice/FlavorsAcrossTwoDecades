//! 旅行定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::destination::Destination;
use super::reward::{TravelPhoto, TravelReward};

/// 旅行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelStatus {
    /// 准备中
    Preparing,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

impl TravelStatus {
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            TravelStatus::Preparing => "准备中",
            TravelStatus::InProgress => "进行中",
            TravelStatus::Completed => "已完成",
            TravelStatus::Cancelled => "已取消",
        }
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        matches!(self, TravelStatus::Completed | TravelStatus::Cancelled)
    }
}

/// 旅行条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelCondition {
    /// 最低信任度
    pub min_trust: u32,
    /// 最低小馆稳定度
    pub min_shop_stability: f32,
    /// 最低能量
    pub min_energy: u32,
    /// 冷却时间（小时）
    pub cooldown_hours: u32,
}

impl Default for TravelCondition {
    fn default() -> Self {
        Self {
            min_trust: 20,
            min_shop_stability: 0.5,
            min_energy: 50,
            cooldown_hours: 24,
        }
    }
}

/// 旅行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Travel {
    /// 唯一 ID
    pub id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 目的地 ID
    pub destination_id: String,
    /// 目的地名称
    pub destination_name: String,
    /// 状态
    pub status: TravelStatus,
    /// 预计旅行时间（小时）
    pub travel_hours: u32,
    /// 实际旅行时间（小时）
    pub actual_hours: Option<u32>,
    /// 能量消耗（每小时）
    pub energy_cost_per_hour: u32,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 预计完成时间
    pub estimated_completion: DateTime<Utc>,
    /// 实际完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 获得的奖励
    pub rewards: Vec<TravelReward>,
    /// 拍摄的照片
    pub photos: Vec<TravelPhoto>,
    /// 旅行日记
    pub journal: String,
    /// 能量等级（开始时）
    pub start_energy: u32,
}

impl Travel {
    /// 创建新旅行
    pub fn new(destination: Destination, start_energy: u32) -> Self {
        let now = Utc::now();
        let travel_hours = destination.get_travel_time(1); // 默认移动模块等级 1

        Self {
            id: Uuid::new_v4(),
            save_id: Uuid::nil(),
            destination_id: destination.id.clone(),
            destination_name: destination.name.clone(),
            status: TravelStatus::Preparing,
            travel_hours,
            actual_hours: None,
            energy_cost_per_hour: 2,
            started_at: now,
            estimated_completion: now + chrono::Duration::hours(travel_hours as i64),
            completed_at: None,
            rewards: Vec::new(),
            photos: Vec::new(),
            journal: String::new(),
            start_energy,
        }
    }

    /// 设置存档 ID
    pub fn with_save_id(mut self, save_id: Uuid) -> Self {
        self.save_id = save_id;
        self
    }

    /// 设置移动模块等级（影响旅行时间）
    pub fn with_mobility_level(mut self, level: u32, destination: &Destination) -> Self {
        self.travel_hours = destination.get_travel_time(level);
        self.estimated_completion =
            self.started_at + chrono::Duration::hours(self.travel_hours as i64);
        self
    }

    /// 开始旅行
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != TravelStatus::Preparing {
            return Err("旅行已经开始".to_string());
        }
        self.status = TravelStatus::InProgress;
        Ok(())
    }

    /// 完成旅行
    pub fn complete(&mut self) -> Result<Vec<TravelReward>, String> {
        if self.status != TravelStatus::InProgress {
            return Err("旅行未在进行中".to_string());
        }

        let now = Utc::now();
        self.status = TravelStatus::Completed;
        self.completed_at = Some(now);
        self.actual_hours =
            Some((now - self.started_at).num_hours().max(0) as u32);

        Ok(self.rewards.clone())
    }

    /// 取消旅行
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status.is_finished() {
            return Err("旅行已结束".to_string());
        }
        self.status = TravelStatus::Cancelled;
        Ok(())
    }

    /// 添加奖励
    pub fn add_reward(&mut self, reward: TravelReward) {
        self.rewards.push(reward);
    }

    /// 添加照片
    pub fn add_photo(&mut self, photo: TravelPhoto) {
        self.photos.push(photo);
    }

    /// 写旅行日记
    pub fn write_journal(&mut self, entry: &str) {
        if self.journal.is_empty() {
            self.journal = entry.to_string();
        } else {
            self.journal.push_str("\n\n");
            self.journal.push_str(entry);
        }
    }

    /// 计算总能量消耗
    pub fn total_energy_cost(&self) -> u32 {
        self.travel_hours * self.energy_cost_per_hour
    }

    /// 检查能量是否足够
    pub fn has_enough_energy(&self) -> bool {
        self.start_energy >= self.total_energy_cost()
    }

    /// 获取剩余时间（秒）
    pub fn remaining_seconds(&self) -> i64 {
        if self.status != TravelStatus::InProgress {
            return 0;
        }

        let remaining = self.estimated_completion - Utc::now();
        remaining.num_seconds().max(0)
    }

    /// 检查是否已完成（时间上）
    pub fn is_time_completed(&self) -> bool {
        self.status == TravelStatus::InProgress && Utc::now() >= self.estimated_completion
    }

    /// 获取进度百分比
    pub fn progress_percentage(&self) -> f32 {
        if self.status == TravelStatus::Preparing {
            return 0.0;
        }
        if self.status.is_finished() {
            return 100.0;
        }

        let total_duration = (self.estimated_completion - self.started_at).num_seconds() as f32;
        let elapsed = (Utc::now() - self.started_at).num_seconds() as f32;

        (elapsed / total_duration * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::travel::{DestinationCategory, DestinationManager};

    fn create_test_destination() -> Destination {
        DestinationManager::default_destinations()
            .into_iter()
            .find(|d| d.id == "local")
            .unwrap()
    }

    #[test]
    fn test_travel_creation() {
        let dest = create_test_destination();
        let travel = Travel::new(dest, 100);

        assert_eq!(travel.status, TravelStatus::Preparing);
        assert!(travel.rewards.is_empty());
        assert!(travel.photos.is_empty());
    }

    #[test]
    fn test_travel_start() {
        let dest = create_test_destination();
        let mut travel = Travel::new(dest, 100);

        travel.start().unwrap();
        assert_eq!(travel.status, TravelStatus::InProgress);
    }

    #[test]
    fn test_travel_cancel() {
        let dest = create_test_destination();
        let mut travel = Travel::new(dest, 100);

        travel.cancel().unwrap();
        assert_eq!(travel.status, TravelStatus::Cancelled);
    }

    #[test]
    fn test_energy_cost() {
        let dest = create_test_destination();
        let travel = Travel::new(dest, 100);

        // 本地旅行基础 2 小时，移动模块等级 1 时为 2 * 1.5 = 3 小时
        // 每小时消耗 2 能量，总计 3 * 2 = 6 能量
        assert_eq!(travel.total_energy_cost(), 6);
    }

    #[test]
    fn test_add_reward() {
        let dest = create_test_destination();
        let mut travel = Travel::new(dest, 100);

        travel.add_reward(TravelReward {
            reward_type: crate::game::travel::TravelRewardType::Recipe,
            name: "测试菜谱".to_string(),
            description: "从旅行中获得的菜谱".to_string(),
            value: "recipe_001".to_string(),
        });

        assert_eq!(travel.rewards.len(), 1);
    }

    #[test]
    fn test_write_journal() {
        let dest = create_test_destination();
        let mut travel = Travel::new(dest, 100);

        travel.write_journal("第一天：到达目的地");
        travel.write_journal("第二天：品尝当地美食");

        assert!(travel.journal.contains("第一天"));
        assert!(travel.journal.contains("第二天"));
    }

    #[test]
    fn test_progress_percentage() {
        let dest = create_test_destination();
        let travel = Travel::new(dest, 100);

        // 准备中时进度为 0
        assert_eq!(travel.progress_percentage(), 0.0);
    }
}
