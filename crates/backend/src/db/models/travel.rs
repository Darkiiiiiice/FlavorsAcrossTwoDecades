//! 旅行数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::travel::TravelStatus;

/// 旅行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Travel {
    /// 旅行ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 目的地
    pub destination: String,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 预计返回时间
    pub expected_return: DateTime<Utc>,
    /// 状态
    pub status: TravelStatus,
    /// 奖励（JSON）
    pub rewards: Option<String>,
}

impl Travel {
    /// 创建新旅行
    pub fn new(save_id: Uuid, destination: String, duration_days: u32) -> Self {
        let now = Utc::now();
        let expected_return = now + chrono::Duration::days(duration_days as i64);

        Self {
            id: Uuid::new_v4(),
            save_id,
            destination,
            started_at: now,
            expected_return,
            status: TravelStatus::InProgress,
            rewards: None,
        }
    }
}
