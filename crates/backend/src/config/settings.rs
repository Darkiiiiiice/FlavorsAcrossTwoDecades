//! 配置设置定义

use serde::Deserialize;

/// 应用程序配置
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub game: GameConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub llm: LlmConfig,
}

impl Settings {
    /// 从配置文件加载配置
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_dir = crate::config::config_dir();
        let config = config::Config::builder()
            // 加载默认配置
            .add_source(config::File::from(config_dir.join("default.toml")))
            // 加载环境特定配置（如果存在）
            .add_source(config::File::from(config_dir.join("production.toml")).required(false))
            // 添加环境变量覆盖
            .add_source(config::Environment::with_prefix("FLAVORS").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    /// 从指定路径加载配置
    pub fn load_from(path: &std::path::Path) -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(config::Environment::with_prefix("FLAVORS").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
}

impl ServerConfig {
    /// 获取服务器地址
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    pub url: String,
}

/// LLM 配置
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LlmConfig {
    /// LLM 提供者
    #[serde(default = "default_provider")]
    pub provider: String,
    /// 模型名称
    #[serde(default = "default_model")]
    pub model: String,
    /// 基础 URL
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// 超时时间（秒)
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "qwen3:4".to_string()
}

fn default_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_timeout_seconds() -> u64 {
    60
}

/// 游戏配置
#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    /// 最小通信延迟（秒）
    pub min_communication_delay: u32,
    /// 最大通信延迟（秒）
    pub max_communication_delay: u32,
    /// 自动存档间隔（秒）
    pub auto_save_interval: u64,
    /// 存档版本
    pub save_version: u32,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
}
