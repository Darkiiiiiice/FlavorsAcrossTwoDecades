//! 指令数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::command::CommandStatus;

/// 指令记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// 指令ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 指令内容
    pub content: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 到达时间
    pub arrival_time: DateTime<Utc>,
    /// 状态
    pub status: CommandStatus,
    /// 执行结果
    pub result: Option<String>,
}

impl Command {
    /// 创建新指令
    pub fn new(save_id: Uuid, content: String, delay_seconds: i64) -> Self {
        let now = Utc::now();
        let arrival_time = now + chrono::Duration::seconds(delay_seconds);

        Self {
            id: Uuid::new_v4(),
            save_id,
            content,
            created_at: now,
            arrival_time,
            status: CommandStatus::Pending,
            result: None,
        }
    }

    /// 检查指令是否已到达
    pub fn has_arrived(&self) -> bool {
        Utc::now() >= self.arrival_time
    }

    /// 标记为已到达
    pub fn mark_arrived(&mut self) {
        self.status = CommandStatus::Arrived;
    }

    /// 标记为已完成
    pub fn mark_completed(&mut self, result: String) {
        self.status = CommandStatus::Completed;
        self.result = Some(result);
    }
}
