//! 游戏引擎主循环

use std::sync::Arc;
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;

use super::command::CommandQueue;
use super::event::EventDispatcher;
use super::llm::LlmManager;
use super::time::{CommunicationDelay, TimeSystem};

/// 游戏引擎
pub struct GameEngine {
    /// 时间系统
    time_system: TimeSystem,
    /// 指令队列
    command_queue: CommandQueue,
    /// 事件分发器
    event_dispatcher: EventDispatcher,
    /// LLM 管理器
    llm_manager: Arc<LlmManager>,
    /// 取消令牌（用于优雅退出）
    cancel_token: CancellationToken,
}

impl GameEngine {
    /// 创建新的游戏引擎
    pub fn new(llm_manager: Arc<LlmManager>) -> Self {
        Self::with_cancel_token(llm_manager, CancellationToken::new())
    }

    /// 使用指定的取消令牌创建游戏引擎
    pub fn with_cancel_token(llm_manager: Arc<LlmManager>, cancel_token: CancellationToken) -> Self {
        let delay = CommunicationDelay::default();

        Self {
            time_system: TimeSystem::new(),
            command_queue: CommandQueue::new(delay),
            event_dispatcher: EventDispatcher::new(),
            llm_manager,
            cancel_token,
        }
    }

    /// 获取取消令牌的克隆（用于外部触发停止）
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 启动游戏引擎
    pub async fn start(&mut self) {
        self.run().await;
    }

    /// 停止游戏引擎
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }

    /// 主循环
    async fn run(&mut self) {
        let mut tick_interval = interval(Duration::from_secs(1));

        loop {
            // 使用 select! 来同时等待 tick 和取消信号
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("GameEngine received shutdown signal");
                    break;
                }
                _ = tick_interval.tick() => {
                    tracing::info!("Tick!!!");

                    // 1. 处理时间更新
                    self.time_system.tick();

                    // 2. 处理到达的指令
                    let arrived_commands = self.command_queue.process_arrived();
                    for cmd in arrived_commands {
                        if let Err(e) = self.process_command(cmd).await {
                            tracing::error!("Failed to process command: {}", e);
                        }
                    }

                    // 3. 处理到期事件
                    let due_events = self.event_dispatcher.process_due_events();
                    for event in due_events {
                        if let Err(e) = self.process_event(event).await {
                            tracing::error!("Failed to process event: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// 处理指令
    async fn process_command(
        &mut self,
        _command: super::command::Command,
    ) -> crate::error::Result<()> {
        // TODO: 实现指令处理逻辑
        // 1. 调用 LLM 获取盼盼决策
        // 2. 执行决策
        // 3. 更新状态
        // 4. 生成反馈
        tracing::debug!("Processing command");
        Ok(())
    }

    /// 处理事件
    async fn process_event(&mut self, event: super::event::GameEvent) -> crate::error::Result<()> {
        use super::event::GameEventType;

        match event.event_type {
            GameEventType::DailyReport => {
                // 生成每日简报
                tracing::info!("Generating daily report");
            }
            GameEventType::CropMature => {
                // 处理作物成熟
                tracing::info!("Crop matured");
            }
            GameEventType::CustomerVisit => {
                // 处理顾客到访
                tracing::info!("Customer visited");
            }
            _ => {
                tracing::debug!("Unhandled event type: {:?}", event.event_type);
            }
        }

        Ok(())
    }

    /// 获取时间系统
    pub fn time_system(&self) -> &TimeSystem {
        &self.time_system
    }

    /// 获取指令队列
    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    /// 获取事件分发器
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.event_dispatcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmConfig;

    #[tokio::test]
    async fn test_game_engine_creation() {
        let mut config = LlmConfig::default();
        // 修复 URL 格式
        config.base_url = "http://localhost".to_string();
        config.port = 11434;

        let provider = crate::game::llm::OllamaProvider::new(config.clone()).unwrap();
        let llm_manager = Arc::new(LlmManager::new(Arc::new(provider), config));

        let engine = GameEngine::new(llm_manager);
        // 验证取消令牌未被触发
        assert!(!engine.cancel_token().is_cancelled());
    }
}
