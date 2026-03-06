//! 盼盼状态数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::game::panpan::{Emotion, Module, Personality};

/// 盼盼状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanpanState {
    /// 名称
    pub name: String,
    /// 型号
    pub model: String,
    /// 制造日期
    pub manufacture_date: DateTime<Utc>,
    /// 性格
    pub personality: Personality,
    /// 信任等级
    pub trust_level: u32,
    /// 情绪
    pub emotion: Emotion,
    /// 当前能量
    pub energy_current: u32,
    /// 最大能量
    pub energy_max: u32,
    /// 位置
    pub location: String,
    /// 当前状态
    pub current_state: String,
    /// 当前任务（JSON）
    pub current_task: Option<String>,
}

impl PanpanState {
    /// 创建新的盼盼状态
    pub fn new() -> Self {
        Self {
            name: "盼盼".to_string(),
            model: "PP-X1".to_string(),
            manufacture_date: Utc::now(),
            personality: Personality::default(),
            trust_level: 50,
            emotion: Emotion::Calm,
            energy_current: 100,
            energy_max: 100,
            location: "小馆".to_string(),
            current_state: "空闲".to_string(),
            current_task: None,
        }
    }
}

impl Default for PanpanState {
    fn default() -> Self {
        Self::new()
    }
}

/// 模块状态（用于数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRecord {
    /// 模块ID
    pub id: String,
    /// 模块类型
    pub module_type: String,
    /// 等级
    pub level: u32,
    /// 状态值
    pub condition: u32,
    /// 经验值
    pub experience: u32,
    /// 是否可用
    pub is_functional: bool,
}

impl ModuleRecord {
    /// 从 Module 创建记录
    pub fn from_module(module: &Module) -> Self {
        use crate::game::panpan::ModuleType;

        let module_type = match module.module_type {
            ModuleType::Communication => "communication",
            ModuleType::Memory => "memory",
            ModuleType::Sensor => "sensor",
            ModuleType::Mobility => "mobility",
            ModuleType::Battery => "battery",
            ModuleType::Kitchen => "kitchen",
            ModuleType::Social => "social",
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            module_type: module_type.to_string(),
            level: module.level,
            condition: module.condition,
            experience: module.experience,
            is_functional: module.is_functional,
        }
    }
}
