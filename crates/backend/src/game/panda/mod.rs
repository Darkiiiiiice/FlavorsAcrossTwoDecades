//! Panda 系统模块

mod manager;
mod module;
mod state;

pub use manager::PandaManager;
pub use module::{Module, ModuleType};
pub use state::{Emotion, PandaFullState, Personality};

use chrono::{DateTime, Utc};

use crate::game::panda::state::{ChargingState, PandaLocation};

/// Panda 完整状态（包含所有子系统）
#[derive(Debug, Clone)]
pub struct Panda {
    /// ID
    pub id: i64,
    /// 是否正在思考
    pub thinking: bool,
    /// Panda 当前状态
    pub state: String,
    /// 当前位置
    pub location: PandaLocation,
    /// 完整状态
    pub state1: PandaFullState,
    /// 模块列表
    pub modules: Vec<Module>,
    /// 信任度 (0-100)
    pub trust_level: u32,
    /// 性格参数
    pub personality: Personality,
    /// 当前情绪
    pub emotion: Emotion,
    /// 电池电量 (0-100000000)
    pub battery: u64,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl Panda {
    /// 创建新的 Panda 实例
    pub fn new() -> Self {
        let location = PandaLocation::ChargingStation(ChargingState::Charging);
        Self {
            id: 0,
            location: location,
            thinking: false,
            state: "".to_string(),
            state1: PandaFullState::default(),
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
    pub fn consume_energy(&mut self, amount: u64) -> bool {
        if self.battery >= amount {
            self.battery -= amount;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// 充电
    pub fn charge(&mut self, amount: u64) {
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

    pub async fn tick(&mut self) {
        tracing::info!("Panda tick!, battery = {}", self.battery);
        self.battery -= 1;
    }
}
