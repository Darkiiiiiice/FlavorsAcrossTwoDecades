//! 配置管理模块

mod settings;

use std::path::PathBuf;

/// 获取配置文件目录
pub fn config_dir() -> PathBuf {
    // 优先使用环境变量
    if let Ok(path) = std::env::var("CONFIG_DIR") {
        PathBuf::from(path)
    } else {
        // 默认从当前目录查找
        PathBuf::from("crates/backend/config")
    }
}

pub use settings::{DatabaseConfig, GameConfig, LlmConfig, LoggingConfig, ServerConfig, Settings};
