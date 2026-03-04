//! 旅行收集系统模块

mod destination;
mod travel;
mod reward;

pub use destination::{Destination, DestinationCategory, DestinationManager};
pub use reward::{TravelPhoto, TravelReward, TravelRewardType};
pub use travel::{Travel, TravelCondition, TravelStatus};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 旅行管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelManager {
    /// 存档 ID
    pub save_id: Uuid,
    /// 可用的目的地
    pub destinations: Vec<Destination>,
    /// 已解锁的目的地 ID
    pub unlocked_destinations: Vec<String>,
    /// 进行中的旅行
    pub active_travels: Vec<Travel>,
    /// 旅行历史
    pub travel_history: Vec<Travel>,
    /// 收集的照片
    pub photos: Vec<TravelPhoto>,
    /// 下一个旅行 ID
    pub next_travel_id: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl TravelManager {
    /// 创建新的旅行管理器
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            destinations: DestinationManager::default_destinations(),
            unlocked_destinations: vec!["local".to_string()], // 默认解锁本地
            active_travels: Vec::new(),
            travel_history: Vec::new(),
            photos: Vec::new(),
            next_travel_id: 1,
            updated_at: Utc::now(),
        }
    }

    /// 获取可用的目的地
    pub fn get_available_destinations(&self) -> Vec<&Destination> {
        self.destinations
            .iter()
            .filter(|d| self.unlocked_destinations.contains(&d.id))
            .collect()
    }

    /// 解锁目的地
    pub fn unlock_destination(&mut self, destination_id: &str) -> Result<(), String> {
        if self.unlocked_destinations.contains(&destination_id.to_string()) {
            return Err("目的地已解锁".to_string());
        }

        if !self.destinations.iter().any(|d| d.id == destination_id) {
            return Err("目的地不存在".to_string());
        }

        self.unlocked_destinations.push(destination_id.to_string());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 开始旅行
    pub fn start_travel(&mut self, travel: Travel) -> Result<(), String> {
        // 检查目的地是否解锁
        if !self.unlocked_destinations.contains(&travel.destination_id) {
            return Err("目的地未解锁".to_string());
        }

        // 检查是否已有进行中的旅行
        if !self.active_travels.is_empty() {
            return Err("已有进行中的旅行".to_string());
        }

        self.active_travels.push(travel);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 完成旅行
    pub fn complete_travel(&mut self, travel_id: Uuid) -> Option<Travel> {
        if let Some(pos) = self.active_travels.iter().position(|t| t.id == travel_id) {
            let mut travel = self.active_travels.remove(pos);
            travel.status = TravelStatus::Completed;
            travel.completed_at = Some(Utc::now());

            // 添加照片
            self.photos.extend(travel.photos.clone());

            // 记录到历史
            self.travel_history.push(travel.clone());
            self.updated_at = Utc::now();

            Some(travel)
        } else {
            None
        }
    }

    /// 取消旅行
    pub fn cancel_travel(&mut self, travel_id: Uuid) -> Option<Travel> {
        if let Some(pos) = self.active_travels.iter().position(|t| t.id == travel_id) {
            let mut travel = self.active_travels.remove(pos);
            travel.status = TravelStatus::Cancelled;
            self.updated_at = Utc::now();
            Some(travel)
        } else {
            None
        }
    }

    /// 获取进行中的旅行
    pub fn get_active_travel(&self) -> Option<&Travel> {
        self.active_travels.first()
    }

    /// 检查是否可以开始旅行
    pub fn can_start_travel(&self, condition: &TravelCondition) -> Result<(), String> {
        if !self.active_travels.is_empty() {
            return Err("已有进行中的旅行".to_string());
        }

        if let Some(last_travel) = self.travel_history.last() {
            if let Some(completed_at) = last_travel.completed_at {
                let elapsed = (Utc::now() - completed_at).num_hours() as u32;
                if elapsed < condition.cooldown_hours {
                    return Err(format!(
                        "冷却中，还需等待 {} 小时",
                        condition.cooldown_hours - elapsed
                    ));
                }
            }
        }

        Ok(())
    }

    /// 获取已完成旅行次数
    pub fn completed_travel_count(&self) -> usize {
        self.travel_history
            .iter()
            .filter(|t| t.status == TravelStatus::Completed)
            .count()
    }

    /// 获取收集的照片数量
    pub fn photo_count(&self) -> usize {
        self.photos.len()
    }

    /// 按目的地获取照片
    pub fn get_photos_by_destination(&self, destination_id: &str) -> Vec<&TravelPhoto> {
        self.photos
            .iter()
            .filter(|p| p.destination_id == destination_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_travel_manager_creation() {
        let save_id = Uuid::new_v4();
        let manager = TravelManager::new(save_id);

        assert!(!manager.destinations.is_empty());
        assert!(manager.unlocked_destinations.contains(&"local".to_string()));
    }

    #[test]
    fn test_unlock_destination() {
        let save_id = Uuid::new_v4();
        let mut manager = TravelManager::new(save_id);

        let result = manager.unlock_destination("chengdu");
        assert!(result.is_ok());
        assert!(manager.unlocked_destinations.contains(&"chengdu".to_string()));
    }

    #[test]
    fn test_unlock_already_unlocked() {
        let save_id = Uuid::new_v4();
        let mut manager = TravelManager::new(save_id);

        let result = manager.unlock_destination("local");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_available_destinations() {
        let save_id = Uuid::new_v4();
        let manager = TravelManager::new(save_id);

        let available = manager.get_available_destinations();
        assert_eq!(available.len(), 1); // 只有 local
    }

    #[test]
    fn test_start_travel() {
        let save_id = Uuid::new_v4();
        let mut manager = TravelManager::new(save_id);

        let destination = DestinationManager::default_destinations()
            .into_iter()
            .find(|d| d.id == "local")
            .unwrap();

        let travel = Travel::new(destination, 10);
        let result = manager.start_travel(travel);

        assert!(result.is_ok());
        assert_eq!(manager.active_travels.len(), 1);
    }

    #[test]
    fn test_complete_travel() {
        let save_id = Uuid::new_v4();
        let mut manager = TravelManager::new(save_id);

        let destination = DestinationManager::default_destinations()
            .into_iter()
            .find(|d| d.id == "local")
            .unwrap();

        let travel = Travel::new(destination, 10);
        let travel_id = travel.id;

        manager.start_travel(travel).unwrap();
        let completed = manager.complete_travel(travel_id);

        assert!(completed.is_some());
        assert_eq!(manager.active_travels.len(), 0);
        assert_eq!(manager.travel_history.len(), 1);
    }
}
