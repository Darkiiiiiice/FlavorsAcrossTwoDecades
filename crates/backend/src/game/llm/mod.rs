//! LLM 集成模块
//!
//! 提供盼盼 AI 决策系统的核心功能

mod context;
mod decision;
mod ollama;
mod prompt;
mod provider;

pub use context::{MemoryFragment, PanpanContext, PanpanState, ShopSnapshot};
pub use decision::{Decision, DecisionType};
pub use ollama::OllamaProvider;
pub use prompt::PromptTemplates;
pub use provider::{LlmManager, LlmProvider};

use crate::config::LlmConfig;
use crate::error::Result;
use std::sync::Arc;

/// 创建 LLM 管理器
pub fn create_llm_manager(config: LlmConfig) -> Result<Arc<LlmManager>> {
    let provider = OllamaProvider::new(config.clone())?;
    Ok(Arc::new(LlmManager::new(Arc::new(provider), config)))
}
