//! Ollama LLM 提供者实现

use async_trait::async_trait;
use futures::stream;
use ollama_rs::Ollama;
use ollama_rs::generation::parameters::ThinkType;
use ollama_rs::models::ModelOptions;
use std::sync::Arc;

use crate::config::LlmConfig;
use crate::error::{GameError, Result};

use super::provider::{Delta, LlmProvider, LlmRequest, LlmResponse, StreamResult};

/// Ollama 提供者
pub struct OllamaProvider {
    client: Arc<Ollama>,
    model: String,
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供者
    pub fn new(config: LlmConfig) -> Result<Self> {
        let ollama = Ollama::new(config.base_url.clone(), config.port);
        Ok(Self {
            client: Arc::new(ollama),
            model: config.model,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    /// 生成响应
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        tracing::debug!(
            "Calling Ollama with model: {}, prompt length: {}",
            self.model,
            request.user_message.len()
        );

        let gen_request = ollama_rs::generation::completion::request::GenerationRequest::new(
            self.model.clone(),
            request.user_message.clone(),
        )
        // .think(ThinkType::High)
        .system(request.system_prompt.clone());

        tracing::info!("ollama request: {:?}", gen_request);

        let response = self
            .client
            .generate(gen_request)
            .await
            .map_err(|e| GameError::LlmError(format!("Ollama generation failed: {}", e)))?;

        tracing::info!("ollama response model: {:?}", response.model);
        tracing::info!("ollama response created_at: {:?}", response.created_at);
        tracing::info!("ollama response response: {:?}", response.response);
        tracing::info!("ollama response done: {:?}", response.done);
        tracing::info!(
            "ollama response total_duration: {:?}",
            response.total_duration
        );
        tracing::info!(
            "ollama response load_duration: {:?}",
            response.load_duration
        );
        tracing::info!(
            "ollama response prompt_eval_count: {:?}",
            response.prompt_eval_count
        );
        tracing::info!(
            "ollama response prompt_eval_duration: {:?}",
            response.prompt_eval_duration
        );
        tracing::info!("ollama response eval_count: {:?}", response.eval_count);
        tracing::info!(
            "ollama response eval_duration: {:?}",
            response.eval_duration
        );
        tracing::info!("ollama response think: {:?}", response.thinking);

        Ok(LlmResponse {
            content: response.response,
            total_tokens: response.eval_count.unwrap_or(0) as u32,
            finish_reason: Some("stop".to_string()),
        })
    }

    /// 流式生成响应（暂时使用非流式实现）
    async fn generate_stream(&self, request: LlmRequest) -> Result<StreamResult> {
        tracing::debug!("Starting Ollama stream generation (using non-stream fallback)");

        // 暂时使用非流式生成作为fallback
        let response = self.generate(request).await?;

        // 创建一个单元素的流
        let delta = Delta {
            content: response.content,
            done: true,
        };

        Ok(Box::pin(stream::once(async move { Ok(delta) })))
    }
}
