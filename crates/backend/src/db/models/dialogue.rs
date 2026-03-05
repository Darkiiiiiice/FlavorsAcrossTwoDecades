//! 对话消息数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueMessage {
    /// 消息ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 发送者
    pub sender: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 消息类型
    pub message_type: String,
    /// 状态
    pub status: String,
}

impl DialogueMessage {
    /// 创建新消息
    pub fn new(save_id: Uuid, sender: String, content: String, message_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            sender,
            content,
            timestamp: Utc::now(),
            message_type,
            status: "delivered".to_string(),
        }
    }

    /// 创建玩家消息
    pub fn player_message(save_id: Uuid, content: String) -> Self {
        Self::new(save_id, "player".to_string(), content, "command".to_string())
    }

    /// 创建盼盼消息
    pub fn panpan_message(save_id: Uuid, content: String) -> Self {
        Self::new(save_id, "panpan".to_string(), content, "response".to_string())
    }

    /// 创建系统消息
    pub fn system_message(save_id: Uuid, content: String) -> Self {
        Self::new(save_id, "system".to_string(), content, "notification".to_string())
    }
}
