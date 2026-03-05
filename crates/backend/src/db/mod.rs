//! 数据库模块

mod migrations;
mod pool;
pub mod seed;

// Phase 4: 数据模型和仓储层
pub mod models;
pub mod repositories;

pub use migrations::MIGRATIONS;
pub use pool::DbPool;
pub use seed::{get_config, initialize_seed_data, set_config};
