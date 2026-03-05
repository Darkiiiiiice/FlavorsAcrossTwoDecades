//! 顾客数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 顾客记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRecord {
    /// 顾客ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 顾客类型
    pub customer_type: String,
    /// 名称
    pub name: String,
    /// 好感度
    pub favorability: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 最后访问时间
    pub last_visit: DateTime<Utc>,
    /// 偏好列表（JSON）
    pub preferences: String,
}

impl CustomerRecord {
    /// 创建新顾客记录
    pub fn new(save_id: Uuid, customer_type: String, name: String, preferences: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            customer_type,
            name,
            favorability: 50,
            visit_count: 1,
            last_visit: Utc::now(),
            preferences,
        }
    }

    /// 更新访问记录
    pub fn record_visit(&mut self) {
        self.visit_count += 1;
        self.last_visit = Utc::now();
    }

    /// 调整好感度
    pub fn adjust_favorability(&mut self, delta: i32) {
        if delta > 0 {
            self.favorability = (self.favorability + delta as u32).min(100);
        } else {
            self.favorability = self.favorability.saturating_sub((-delta) as u32);
        }
    }
}
