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

    /// 获取 Panda 的当前电量
    pub fn battery(&self) -> u64 {
        self.panda.battery
    }

    /// 获取 Panda 的当前信任度
    pub fn trust_level(&self) -> u32 {
        self.panda.trust_level
    }

    /// 获取 Panda 的当前情绪
    pub fn emotion(&self) -> super::Emotion {
        self.panda.emotion.clone()
    }

    /// 获取 Panda 的当前位置
    pub fn location(&self) -> &super::PandaLocation {
        &self.panda.location
    }

    /// 为 Panda 充电
    pub fn charge(&mut self, amount: u64) {
        self.panda.charge(amount);
    }

    /// 消耗 Panda 的能量
    pub fn consume_energy(&mut self, amount: u64) -> bool {
        self.panda.consume_energy(amount)
    }

    /// 更新 Panda 的信任度
    pub fn update_trust(&mut self, delta: i32) {
        self.panda.update_trust(delta);
    }

    /// 更新 Panda 的情绪
    pub fn update_emotion(&mut self, emotion: super::Emotion) {
        self.panda.update_emotion(emotion);
    }

    /// 获取 Panda 的工作效率
    pub fn work_efficiency(&self) -> f32 {
        self.panda.work_efficiency()
    }

    /// 获取 Panda 的错误率
    pub fn error_rate(&self) -> f32 {
        self.panda.error_rate()
    }
}
