//! 记忆碎片数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 记忆碎片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// 记忆ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 记忆类型
    pub fragment_type: String,
    /// 标题
    pub title: String,
    /// 内容
    pub content: String,
    /// 是否解锁
    pub is_unlocked: bool,
    /// 解锁时间
    pub unlocked_at: Option<DateTime<Utc>>,
    /// 触发条件
    pub trigger_condition: String,
}

impl MemoryFragment {
    /// 创建新记忆碎片
    pub fn new(
        save_id: Uuid,
        fragment_type: String,
        title: String,
        content: String,
        trigger_condition: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            fragment_type,
            title,
            content,
            is_unlocked: false,
            unlocked_at: None,
            trigger_condition,
        }
    }

    /// 解锁记忆
    pub fn unlock(&mut self) {
        if !self.is_unlocked {
            self.is_unlocked = true;
            self.unlocked_at = Some(Utc::now());
        }
    }
}
