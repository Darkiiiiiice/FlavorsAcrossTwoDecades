//! 游戏核心模块

pub mod command;
pub mod engine;
pub mod event;
pub mod llm;
pub mod time;

use std::sync::Arc;
use std::time::Instant;

use crate::api::HealthChecker;
use crate::config::LlmConfig;
use crate::db::DbPool;

// 重新导出常用类型
pub use command::{Command, CommandQueue, CommandStatus};
pub use engine::GameEngine;
pub use event::{EventDispatcher, GameEvent, GameEventType};
pub use llm::{create_llm_manager, LlmManager};
pub use time::{CommunicationDelay, TimeSystem};

/// API 状态
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub start_time: Instant,
    pub health_checker: Arc<HealthChecker>,
    pub llm_config: LlmConfig,
}

impl AppState {
    pub fn new(db_pool: Arc<DbPool>, llm_config: LlmConfig) -> Self {
        let health_checker = Arc::new(HealthChecker::new(Arc::clone(&db_pool), llm_config.clone()));
        Self {
            db_pool,
            start_time: Instant::now(),
            health_checker,
            llm_config,
        }
    }
}
