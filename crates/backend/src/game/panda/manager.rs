//! Panda 管理器
//!
//! 管理 Panda 的状态更新和周期性任务

use std::sync::Arc;

use super::Panda;
use crate::db::DbPool;

/// Panda 管理器
#[derive(Debug)]
pub struct PandaManager {
    /// Panda 实例
    pub panda: Panda,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
}

impl PandaManager {
    /// 创建新的 Panda 管理器
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        let panda = Panda::new();
        Self { panda, db_pool }
    }

    /// 使用现有 Panda 创建管理器
    pub fn with_panda(panda: Panda, db_pool: Arc<DbPool>) -> Self {
        Self { panda, db_pool }
    }

    /// 更新 Panda 状态（每个 tick 调用）
    pub async fn update(&mut self) {
        self.panda.tick().await;
    }
}
