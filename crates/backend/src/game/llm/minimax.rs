//! MiniMax LLM 提供者实现

use async_trait::async_trait;
use futures::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::LlmConfig;
use crate::error::{GameError, Result};

use super::provider::{Delta, LlmProvider, LlmRequest, LlmResponse, StreamResult};

/// MiniMax API 请求消息
#[derive(Debug, Serialize)]
struct MinimaxMessage {
    role: String,
    content: String,
}

/// MiniMax API 请求体
#[derive(Debug, Serialize)]
struct MinimaxRequest {
    model: String,
    messages: Vec<MinimaxMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// MiniMax API 响应体
#[derive(Debug, Deserialize)]
struct MinimaxResponse {
    choices: Vec<MinimaxChoice>,
    usage: Option<MinimaxUsage>,
    #[serde(default)]
    base_resp: Option<MinimaxBaseResp>,
}

#[derive(Debug, Deserialize)]
struct MinimaxChoice {
    message: MinimaxResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MinimaxResponseMessage {
    content: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct MinimaxUsage {
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct MinimaxBaseResp {
    status_code: i32,
    status_msg: String,
}

/// MiniMax 提供者
pub struct MinimaxProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl MinimaxProvider {
    /// 创建新的 MiniMax 提供者
    pub fn new(config: LlmConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(GameError::LlmError(
                "MiniMax API key is required".to_string(),
            ));
        }

        Ok(Self {
            client: Client::new(),
            api_key: config.api_key,
            base_url: config.base_url,
            model: config.model,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
}

#[async_trait]
impl LlmProvider for MinimaxProvider {
    /// 生成响应
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        tracing::debug!(
            "Calling MiniMax with model: {}, prompt length: {}",
            self.model,
            request.user_message.len()
        );

        let minimax_request = MinimaxRequest {
            model: self.model.clone(),
            messages: vec![
                MinimaxMessage {
                    role: "system".to_string(),
                    content: request.system_prompt,
                },
                MinimaxMessage {
                    role: "user".to_string(),
                    content: request.user_message,
                },
            ],
            temperature: Some(self.temperature),
            max_completion_tokens: Some(self.max_tokens),
            stream: None,
        };

        let url = format!("{}/v1/text/chatcompletion_v2", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&minimax_request)
            .send()
            .await
            .map_err(|e| GameError::LlmError(format!("MiniMax request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| GameError::LlmError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            tracing::error!("MiniMax API error: status={}, body={}", status, response_text);
            return Err(GameError::LlmError(format!(
                "MiniMax API error: {} - {}",
                status, response_text
            )));
        }

        let minimax_response: MinimaxResponse = serde_json::from_str(&response_text).map_err(
            |e| {
                GameError::LlmError(format!(
                    "Failed to parse MiniMax response: {} - {}",
                    e, response_text
                ))
            },
        )?;

        // 检查 base_resp 中的错误
        if let Some(base_resp) = &minimax_response.base_resp {
            if base_resp.status_code != 0 {
                return Err(GameError::LlmError(format!(
                    "MiniMax API error: {} - {}",
                    base_resp.status_code, base_resp.status_msg
                )));
            }
        }

        let choice = minimax_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| GameError::LlmError("No choices in MiniMax response".to_string()))?;

        tracing::info!(
            "MiniMax response: content length={}, finish_reason={:?}",
            choice.message.content.len(),
            choice.finish_reason
        );

        Ok(LlmResponse {
            content: choice.message.content,
            total_tokens: minimax_response.usage.map(|u| u.total_tokens).unwrap_or(0),
            finish_reason: choice.finish_reason,
        })
    }

    /// 流式生成响应（暂时使用非流式实现）
    async fn generate_stream(&self, request: LlmRequest) -> Result<StreamResult> {
        tracing::debug!("Starting MiniMax stream generation (using non-stream fallback)");

        // 暂时使用非流式生成作为 fallback
        let response = self.generate(request).await?;

        // 创建一个单元素的流
        let delta = Delta {
            content: response.content,
            done: true,
        };

        Ok(Box::pin(stream::once(async move { Ok(delta) })))
    }
}
