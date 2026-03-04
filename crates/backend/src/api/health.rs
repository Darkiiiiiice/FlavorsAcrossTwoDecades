//! 健康检查 API
use axum::{Json, extract::State};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use utoipa::ToSchema;

use crate::config::LlmConfig;
use crate::db::DbPool;
use crate::error::GameResult;
use crate::game::AppState;

/// 活跃状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级
    Degraded,
    /// 不健康
    Unhealthy,
}

/// 组件健康状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    /// 状态
    pub status: HealthStatus,
    /// 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 延迟（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl Default for ComponentHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }
}

/// 组件健康状态集合
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComponentsHealth {
    /// 数据库状态
    pub database: ComponentHealth,
    /// 配置状态
    pub config: ComponentHealth,
    /// LLM 服务状态
    pub llm: ComponentHealth,
}

impl Default for ComponentsHealth {
    fn default() -> Self {
        Self {
            database: ComponentHealth::default(),
            config: ComponentHealth::default(),
            llm: ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("LLM check not implemented".to_string()),
                latency_ms: None,
            },
        }
    }
}

/// 健康检查响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// 服务状态
    pub status: HealthStatus,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 各组件状态
    pub components: ComponentsHealth,
    /// 时间戳
    pub timestamp: String,
}

impl HealthResponse {
    pub fn new(status: HealthStatus, components: ComponentsHealth) -> Self {
        Self {
            status,
            uptime_seconds: 0,
            components,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 健康检查器
pub struct HealthChecker {
    start_time: Instant,
    db_pool: Arc<DbPool>,
    llm_config: LlmConfig,
    http_client: Client,
}

impl HealthChecker {
    pub fn new(db_pool: Arc<DbPool>, llm_config: LlmConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            db_pool,
            llm_config,
            http_client,
            start_time: Instant::now(),
        }
    }

    /// 执行健康检查
    pub async fn check(&self) -> HealthResponse {
        let db_health = self.check_database().await;
        let config_health = self.check_config();
        let llm_health = self.check_llm().await;

        // 综合状态判断
        let overall_status = if db_health.status == HealthStatus::Unhealthy
            || llm_health.status == HealthStatus::Unhealthy
        {
            HealthStatus::Unhealthy
        } else if config_health.status == HealthStatus::Degraded
            || llm_health.status == HealthStatus::Degraded
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthResponse {
            status: overall_status,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components: ComponentsHealth {
                database: db_health,
                config: config_health,
                llm: llm_health,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 检查数据库健康状态
    async fn check_database(&self) -> ComponentHealth {
        let start = Instant::now();

        // 执行简单查询
        match sqlx::query("SELECT 1").fetch_one(self.db_pool.pool()).await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                latency_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Database error: {}", e)),
                latency_ms: None,
            },
        }
    }

    /// 检查配置健康状态
    fn check_config(&self) -> ComponentHealth {
        ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }

    /// 检查 LLM 服务健康状态
    async fn check_llm(&self) -> ComponentHealth {
        let start = Instant::now();

        // 构建健康检查 URL
        let health_url = format!(
            "{}/api/tags",
            self.llm_config.base_url.trim_end_matches('/')
        );

        // 发送请求检查 Ollama 服务
        match self.http_client.get(&health_url).send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;

                if response.status().is_success() {
                    // 尝试解析响应，验证模型是否可用
                    match response.json::<OllamaTagsResponse>().await {
                        Ok(tags) => {
                            // 检查配置的模型是否在可用模型列表中
                            if self.llm_config.model.is_empty() {
                                return ComponentHealth {
                                    status: HealthStatus::Healthy,
                                    message: Some("Ollama service is running".to_string()),
                                    latency_ms: Some(latency_ms),
                                };
                            }

                            // 检查模型是否在列表中
                            let model_exists = tags.models.iter().any(|m| {
                                m.name == self.llm_config.model
                                    || m.name.starts_with(&self.llm_config.model)
                            });

                            if model_exists {
                                ComponentHealth {
                                    status: HealthStatus::Healthy,
                                    message: Some(format!(
                                        "Model '{}' is available",
                                        self.llm_config.model
                                    )),
                                    latency_ms: Some(latency_ms),
                                }
                            } else {
                                ComponentHealth {
                                    status: HealthStatus::Degraded,
                                    message: Some(format!(
                                        "Model '{}' not found in Ollama, may need to pull",
                                        self.llm_config.model
                                    )),
                                    latency_ms: Some(latency_ms),
                                }
                            }
                        }
                        Err(e) => ComponentHealth {
                            status: HealthStatus::Degraded,
                            message: Some(format!("Failed to parse Ollama response: {}", e)),
                            latency_ms: Some(latency_ms),
                        },
                    }
                } else {
                    ComponentHealth {
                        status: HealthStatus::Degraded,
                        message: Some(format!("Ollama returned status: {}", response.status())),
                        latency_ms: Some(latency_ms),
                    }
                }
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Failed to connect to Ollama: {}", e)),
                    latency_ms: Some(latency_ms),
                }
            }
        }
    }
}

/// Ollama 标签响应
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

/// 健康检查端点
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "服务健康状态", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> GameResult<Json<HealthResponse>> {
    let checker = &state.health_checker;
    Ok(Json(checker.check().await))
}

/// 就绪检查端点
#[utoipa::path(
    get,
    path = "/api/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "服务就绪状态", body = HealthResponse)
    )
)]
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> GameResult<Json<HealthResponse>> {
    // 只检查服务是否运行，不检查数据库
    health_check(State(state)).await
}

/// 存活检查端点
#[utoipa::path(
    get,
    path = "/api/health/live",
    tag = "health",
    responses(
        (status = 200, description = "服务存活状态", body = HealthResponse)
    )
)]
pub async fn liveness_check(
    State(state): State<Arc<AppState>>,
) -> GameResult<Json<HealthResponse>> {
    // 活检查只检查服务是否运行，不检查数据库
    health_check(State(state)).await
}
