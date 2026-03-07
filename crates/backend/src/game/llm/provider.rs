//! LLM 服务提供者抽象层

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;

use crate::config::LlmConfig;
use crate::error::Result;

use super::context::PandaContext;
use super::decision::DecisionType;

/// LLM 请求
#[derive(Debug, Clone, Serialize)]
pub struct LlmRequest {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户消息
    pub user_message: String,
    /// 温度参数 (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// 最大 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// LLM 响应
#[derive(Debug, Clone, Deserialize)]
pub struct LlmResponse {
    /// 生成的文本内容
    pub content: String,
    /// 使用的 token 数
    #[serde(default)]
    pub total_tokens: u32,
    /// 完成原因
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// LLM 流式响应的增量
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    /// 增量文本
    pub content: String,
    /// 是否完成
    pub done: bool,
}

/// 流式响应的类型别名（支持 trait object）
pub type StreamResult = Pin<Box<dyn Stream<Item = Result<Delta>> + Send>>;

/// LLM 服务提供者 trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 生成响应
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// 流式生成响应
    async fn generate_stream(&self, request: LlmRequest) -> Result<StreamResult>;
}

/// LLM 服务管理器
pub struct LlmManager {
    provider: Arc<dyn LlmProvider>,
    config: LlmConfig,
}

impl fmt::Debug for LlmManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LlmManager")
            .field("provider", &"Arc<dyn LlmProvider>")
            .field("config", &self.config)
            .finish()
    }
}

impl LlmManager {
    /// 创建新的 LLM 管理器
    pub fn new(provider: Arc<dyn LlmProvider>, config: LlmConfig) -> Self {
        Self { provider, config }
    }

    /// 生成盼盼的决策
    pub async fn generate_decision(
        &self,
        context: &PandaContext,
        decision_type: &DecisionType,
    ) -> Result<super::decision::Decision> {
        // 构建请求
        let request = LlmRequest {
            system_prompt: context.build_system_prompt(),
            user_message: context.build_decision_prompt(decision_type),
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
        };

        // 调用 LLM
        match self.provider.generate(request).await {
            Ok(response) => {
                // 解析决策
                super::decision::Decision::parse_from_response(
                    &response.content,
                    decision_type.clone(),
                )
            }
            Err(e) => {
                tracing::warn!("LLM call failed: {}, using fallback", e);
                // 降级策略：使用基于规则的简单决策
                self.fallback_decision(decision_type)
            }
        }
    }

    /// 生成文本（通用方法）
    pub async fn generate_text(
        &self,
        system_prompt: String,
        user_message: String,
    ) -> Result<String> {
        use std::time::Instant;

        let request = LlmRequest {
            system_prompt,
            user_message,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
        };

        let start = Instant::now();
        let result = self.provider.generate(request).await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                tracing::info!(
                    "LLM call completed in {:.2?}ms (model: {})",
                    elapsed.as_secs_f64() * 1000.0,
                    self.config.model
                );
                Ok(response.content)
            }
            Err(e) => {
                tracing::error!(
                    "LLM call failed after {:.2?}ms (model: {}): {}",
                    elapsed.as_secs_f64() * 1000.0,
                    self.config.model,
                    e
                );
                Err(e)
            }
        }
    }

    /// 降级决策：当 LLM 不可用时使用预设规则
    fn fallback_decision(&self, decision_type: &DecisionType) -> Result<super::decision::Decision> {
        use super::decision::Decision;

        tracing::info!("Using fallback decision for {:?}", decision_type);

        // 简单的规则引擎
        let decision = match decision_type {
            DecisionType::Command(cmd) => Decision {
                understood: true,
                interpretation: format!("执行指令: {}", cmd),
                will_execute: true,
                execution_plan: "按照指令执行".to_string(),
                modification: None,
                response_to_player: "收到指令，我会执行的。".to_string(),
                personality_changes: None,
            },
            DecisionType::AutonomousAction => Decision {
                understood: true,
                interpretation: "检查小馆状态".to_string(),
                will_execute: true,
                execution_plan: "巡视小馆，确保一切正常".to_string(),
                modification: None,
                response_to_player: "小馆一切正常。".to_string(),
                personality_changes: None,
            },
            _ => Decision {
                understood: true,
                interpretation: "默认决策".to_string(),
                will_execute: true,
                execution_plan: "执行默认操作".to_string(),
                modification: None,
                response_to_player: "好的。".to_string(),
                personality_changes: None,
            },
        };

        Ok(decision)
    }
}
