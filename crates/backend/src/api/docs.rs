//! OpenAPI 文档配置

use utoipa::OpenApi;

use crate::api::health::*;

/// OpenAPI 文档定义
#[derive(OpenApi)]
#[openapi(
    info(
        title = "味延廿载 API",
        version = "0.1.0",
        description = "跨星际餐厅经营游戏后端 API",
        license(name = "MIT"),
    ),
    paths(
        health_check,
        readiness_check,
        liveness_check,
    ),
    components(schemas(
        HealthStatus,
        ComponentHealth,
        ComponentsHealth,
        HealthResponse,
    )),
    tags(
        (name = "health", description = "健康检查 API"),
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// 获取 OpenAPI JSON 文档
    pub fn openapi_json() -> String {
        serde_json::to_string_pretty(&Self::openapi()).unwrap_or_default()
    }
}
