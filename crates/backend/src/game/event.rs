//! 事件系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 游戏事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEventType {
    // 时间事件
    /// 每日简报
    DailyReport,
    /// 作物成熟
    CropMature,
    /// 旅行归来
    TravelReturn,

    // 触发事件
    /// 顾客到访
    CustomerVisit,
    /// 邻居互动
    NeighborInteraction,
    /// 设备故障
    EquipmentFailure,

    // 特殊事件
    /// 节日
    Festival,
    /// 记忆解锁
    MemoryUnlock,
    /// 成就达成
    Achievement,
}

/// 游戏事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// 事件 ID
    pub id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 事件类型
    pub event_type: GameEventType,
    /// 触发时间
    pub trigger_time: DateTime<Utc>,
    /// 事件数据
    pub data: serde_json::Value,
    /// 是否已处理
    pub processed: bool,
}

impl GameEvent {
    /// 创建新事件
    pub fn new(
        save_id: Uuid,
        event_type: GameEventType,
        trigger_time: DateTime<Utc>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            event_type,
            trigger_time,
            data,
            processed: false,
        }
    }

    /// 创建立即触发的事件
    pub fn immediate(save_id: Uuid, event_type: GameEventType, data: serde_json::Value) -> Self {
        Self::new(save_id, event_type, Utc::now(), data)
    }
}

/// 事件分发器
pub struct EventDispatcher {
    /// 待处理事件
    pending_events: Vec<GameEvent>,
}

impl EventDispatcher {
    /// 创建新的事件分发器
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
        }
    }

    /// 添加事件
    pub fn schedule(&mut self, event: GameEvent) {
        self.pending_events.push(event);
    }

    /// 处理到期事件
    pub fn process_due_events(&mut self) -> Vec<GameEvent> {
        let now = Utc::now();
        let mut due_events = Vec::new();

        self.pending_events.retain(|event| {
            if event.trigger_time <= now && !event.processed {
                due_events.push(event.clone());
                false // 移除已处理事件
            } else {
                true
            }
        });

        due_events
    }

    /// 获取指定存档的所有事件
    pub fn get_events_for_save(&self, save_id: Uuid) -> Vec<&GameEvent> {
        self.pending_events
            .iter()
            .filter(|event| event.save_id == save_id)
            .collect()
    }

    /// 获取待处理事件数量
    pub fn len(&self) -> usize {
        self.pending_events.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.pending_events.is_empty()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let save_id = Uuid::new_v4();
        let event = GameEvent::immediate(
            save_id,
            GameEventType::DailyReport,
            serde_json::json!({ "test": "data" }),
        );

        assert_eq!(event.save_id, save_id);
        assert!(!event.processed);
    }

    #[test]
    fn test_event_dispatcher() {
        let mut dispatcher = EventDispatcher::new();
        let save_id = Uuid::new_v4();

        let event =
            GameEvent::immediate(save_id, GameEventType::DailyReport, serde_json::json!({}));

        dispatcher.schedule(event);
        assert_eq!(dispatcher.len(), 1);

        let due = dispatcher.process_due_events();
        assert_eq!(due.len(), 1);
        assert!(dispatcher.is_empty());
    }
}
