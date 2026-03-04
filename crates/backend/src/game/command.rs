//! 指令队列系统

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use super::time::CommunicationDelay;

/// 指令状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandStatus {
    /// 等待发送
    Pending,
    /// 传输中
    InTransit,
    /// 已到达地球
    Arrived,
    /// 盼盼处理中
    Processing,
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
}

/// 玩家指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// 指令 ID
    pub id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 指令内容
    pub content: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 预计到达时间
    pub arrival_time: DateTime<Utc>,
    /// 当前状态
    pub status: CommandStatus,
    /// 执行结果
    pub result: Option<String>,
}

impl Command {
    /// 创建新指令
    pub fn new(save_id: Uuid, content: String, delay_seconds: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            save_id,
            content,
            created_at: now,
            arrival_time: now + Duration::seconds(delay_seconds as i64),
            status: CommandStatus::Pending,
            result: None,
        }
    }
}

/// 指令队列
pub struct CommandQueue {
    /// 待处理指令
    pending_commands: VecDeque<Command>,
    /// 通信延迟计算器
    delay_calculator: CommunicationDelay,
}

impl CommandQueue {
    /// 创建新的指令队列
    pub fn new(delay_calculator: CommunicationDelay) -> Self {
        Self {
            pending_commands: VecDeque::new(),
            delay_calculator,
        }
    }

    /// 添加新指令
    pub fn add_command(&mut self, save_id: Uuid, content: String) -> Command {
        let delay = self.delay_calculator.total_delay();
        let command = Command::new(save_id, content, delay);
        self.pending_commands.push_back(command.clone());
        command
    }

    /// 处理已到达的指令
    pub fn process_arrived(&mut self) -> Vec<Command> {
        let now = Utc::now();
        let mut arrived = Vec::new();

        while let Some(cmd) = self.pending_commands.front() {
            if cmd.arrival_time <= now && cmd.status == CommandStatus::Pending {
                let mut cmd = self.pending_commands.pop_front().unwrap();
                cmd.status = CommandStatus::Arrived;
                arrived.push(cmd);
            } else {
                break;
            }
        }

        arrived
    }

    /// 获取指定存档的所有指令
    pub fn get_commands_for_save(&self, save_id: Uuid) -> Vec<&Command> {
        self.pending_commands
            .iter()
            .filter(|cmd| cmd.save_id == save_id)
            .collect()
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.pending_commands.len()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.pending_commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let save_id = Uuid::new_v4();
        let delay = CommunicationDelay::new(10);
        let mut queue = CommandQueue::new(delay);

        let cmd = queue.add_command(save_id, "测试指令".to_string());

        assert_eq!(cmd.save_id, save_id);
        assert_eq!(cmd.content, "测试指令");
        assert_eq!(cmd.status, CommandStatus::Pending);
        assert_eq!(queue.len(), 1);
    }
}
