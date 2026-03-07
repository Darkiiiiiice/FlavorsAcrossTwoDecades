//! LLM 集成模块
//!
//! 提供 Panda AI 决策系统的核心功能

mod context;
mod decision;
mod minimax;
mod ollama;
mod prompt;
mod provider;

pub use context::{MemoryFragment, PandaContext, PandaState, ShopSnapshot};
pub use decision::{Decision, DecisionType};
pub use minimax::MinimaxProvider;
pub use ollama::OllamaProvider;
pub use prompt::PromptTemplates;
pub use provider::{Delta, LlmManager, LlmProvider, LlmRequest, LlmResponse};

use crate::config::LlmConfig;
use crate::error::{GameError, Result};
use std::sync::Arc;

/// 创建 LLM 管理器
pub fn create_llm_manager(config: LlmConfig) -> Result<Arc<LlmManager>> {
    let provider: Arc<dyn LlmProvider> = match config.provider.as_str() {
        "ollama" => Arc::new(OllamaProvider::new(config.clone())?),
        "minimax" => Arc::new(MinimaxProvider::new(config.clone())?),
        other => {
            return Err(GameError::LlmError(format!(
                "Unknown LLM provider: {}",
                other
            )))
        }
    };
    Ok(Arc::new(LlmManager::new(provider, config)))
}
