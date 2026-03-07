//! Panda 系统模块

mod module;
mod state;

pub use module::{Module, ModuleType};
pub use state::{Emotion, PandaFullState, Personality};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Panda 完整状态（包含所有子系统）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panda {
    /// 存档 ID
    pub save_id: Uuid,
    /// 当前位置
    pub location: String,
    /// 完整状态
    pub state: PandaFullState,
    /// 模块列表
    pub modules: Vec<Module>,
    /// 信任度 (0-100)
    pub trust_level: u32,
    /// 性格参数
    pub personality: Personality,
    /// 当前情绪
    pub emotion: Emotion,
    /// 电池电量 (0-100)
    pub battery: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl Panda {
    /// 创建新的 Panda 实例
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            location: "星夜小馆".to_string(),
            state: PandaFullState::default(),
            modules: Module::default_modules(),
            trust_level: 50,
            personality: Personality::default(),
            emotion: Emotion::Calm,
            battery: 100,
            updated_at: Utc::now(),
        }
    }

    /// 获取指定类型的模块
    pub fn get_module(&self, module_type: ModuleType) -> Option<&Module> {
        self.modules.iter().find(|m| m.module_type == module_type)
    }

    /// 获取指定类型的可变模块
    pub fn get_module_mut(&mut self, module_type: ModuleType) -> Option<&mut Module> {
        self.modules
            .iter_mut()
            .find(|m| m.module_type == module_type)
    }

    /// 消耗能量
    pub fn consume_energy(&mut self, amount: u32) -> bool {
        if self.battery >= amount {
            self.battery -= amount;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// 充电
    pub fn charge(&mut self, amount: u32) {
        self.battery = (self.battery + amount).min(100);
        self.updated_at = Utc::now();
    }

    /// 更新信任度
    pub fn update_trust(&mut self, delta: i32) {
        let new_trust = self.trust_level as i32 + delta;
        self.trust_level = new_trust.clamp(0, 100) as u32;
        self.updated_at = Utc::now();
    }

    /// 更新情绪
    pub fn update_emotion(&mut self, emotion: Emotion) {
        self.emotion = emotion;
        self.updated_at = Utc::now();
    }

    /// 计算记忆恢复倍率（基于信任度）
    pub fn memory_recovery_rate(&self) -> f32 {
        match self.trust_level {
            0..=20 => 0.3,
            21..=40 => 0.6,
            41..=60 => 1.0,
            61..=80 => 1.5,
            81..=100 => 2.0,
            _ => 1.0,
        }
    }

    /// 计算主动提议概率（基于信任度）
    pub fn initiative_probability(&self) -> f32 {
        match self.trust_level {
            0..=20 => 0.0,
            21..=40 => 0.1,
            41..=60 => 0.3,
            61..=80 => 0.6,
            81..=100 => 0.9,
            _ => 0.3,
        }
    }

    /// 计算工作效率（基于情绪）
    pub fn work_efficiency(&self) -> f32 {
        match self.emotion {
            Emotion::Happy => 1.1,
            Emotion::Calm => 1.0,
            Emotion::Tired => 0.9,
            Emotion::Confused => 0.9,
            Emotion::Worried => 1.0,
            Emotion::Lonely => 0.95,
            Emotion::Excited => 1.0,
        }
    }

    /// 计算错误率（基于情绪）
    pub fn error_rate(&self) -> f32 {
        match self.emotion {
            Emotion::Happy => 0.9,
            Emotion::Calm => 1.0,
            Emotion::Tired => 1.2,
            Emotion::Confused => 1.1,
            Emotion::Worried => 1.0,
            Emotion::Lonely => 1.0,
            Emotion::Excited => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panda_creation() {
        let save_id = Uuid::new_v4();
        let panda = Panda::new(save_id);

        assert_eq!(panda.save_id, save_id);
        assert_eq!(panda.location, "星夜小馆");
        assert_eq!(panda.trust_level, 50);
        assert_eq!(panda.battery, 100);
        assert_eq!(panda.modules.len(), 7);
    }

    #[test]
    fn test_energy_consumption() {
        let save_id = Uuid::new_v4();
        let mut panda = Panda::new(save_id);

        assert!(panda.consume_energy(10));
        assert_eq!(panda.battery, 90);

        assert!(!panda.consume_energy(100));
        assert_eq!(panda.battery, 90);
    }

    #[test]
    fn test_trust_level_effects() {
        let save_id = Uuid::new_v4();
        let mut panda = Panda::new(save_id);

        panda.trust_level = 10;
        assert_eq!(panda.memory_recovery_rate(), 0.3);
        assert_eq!(panda.initiative_probability(), 0.0);

        panda.trust_level = 90;
        assert_eq!(panda.memory_recovery_rate(), 2.0);
        assert_eq!(panda.initiative_probability(), 0.9);
    }
}
