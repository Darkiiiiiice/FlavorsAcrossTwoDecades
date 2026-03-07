//! Panda 模块系统

use serde::{Deserialize, Serialize};

/// Panda 模块类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    /// 通信模块 - 影响通信延迟
    Communication,
    /// 记忆模块 - 影响记忆碎片解锁和容量
    Memory,
    /// 传感器模块 - 影响实验精度
    Sensor,
    /// 移动模块 - 影响旅行速度和维修能力
    Mobility,
    /// 电池模块 - 影响续航能力
    Battery,
    /// 厨房模块 - 影响烹饪成功率和菜品品质
    Kitchen,
    /// 社交模块 - 影响顾客互动和邻里关系
    Social,
}

impl ModuleType {
    /// 获取模块名称
    pub fn name(&self) -> &str {
        match self {
            ModuleType::Communication => "通信模块",
            ModuleType::Memory => "记忆模块",
            ModuleType::Sensor => "传感器模块",
            ModuleType::Mobility => "移动模块",
            ModuleType::Battery => "电池模块",
            ModuleType::Kitchen => "厨房模块",
            ModuleType::Social => "社交模块",
        }
    }

    /// 获取模块描述
    pub fn description(&self) -> &str {
        match self {
            ModuleType::Communication => "负责星际通信，降低通信延迟",
            ModuleType::Memory => "存储和管理记忆碎片",
            ModuleType::Sensor => "提高实验精度和检测能力",
            ModuleType::Mobility => "提升移动速度和维修能力",
            ModuleType::Battery => "提供能量，延长续航时间",
            ModuleType::Kitchen => "增强烹饪能力和菜品品质",
            ModuleType::Social => "改善与顾客和邻居的关系",
        }
    }
}

/// 模块状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// 模块类型
    pub module_type: ModuleType,
    /// 等级 (1-10)
    pub level: u32,
    /// 完好度 (0-100)
    pub condition: u32,
    /// 经验值
    pub experience: u32,
    /// 是否可用
    pub is_functional: bool,
}

impl Module {
    /// 创建新模块
    pub fn new(module_type: ModuleType) -> Self {
        Self {
            module_type,
            level: 1,
            condition: 100,
            experience: 0,
            is_functional: true,
        }
    }

    /// 创建默认模块列表（7个模块）
    pub fn default_modules() -> Vec<Self> {
        vec![
            Self::new(ModuleType::Communication),
            Self::new(ModuleType::Memory),
            Self::new(ModuleType::Sensor),
            Self::new(ModuleType::Mobility),
            Self::new(ModuleType::Battery),
            Self::new(ModuleType::Kitchen),
            Self::new(ModuleType::Social),
        ]
    }

    /// 升级所需经验
    pub fn experience_for_next_level(&self) -> u32 {
        if self.level >= 10 {
            return u32::MAX;
        }
        self.level * 100
    }

    /// 添加经验并尝试升级
    pub fn add_experience(&mut self, amount: u32) -> bool {
        if self.level >= 10 {
            return false;
        }

        self.experience += amount;
        let required = self.experience_for_next_level();

        if self.experience >= required {
            self.experience -= required;
            self.level += 1;
            return true;
        }

        false
    }

    /// 降低完好度
    pub fn damage(&mut self, amount: u32) {
        self.condition = self.condition.saturating_sub(amount);
        if self.condition < 20 {
            self.is_functional = false;
        }
    }

    /// 修复模块
    pub fn repair(&mut self, amount: u32) {
        self.condition = (self.condition + amount).min(100);
        if self.condition >= 20 {
            self.is_functional = true;
        }
    }

    /// 计算模块效果值（0.0-1.0）
    pub fn effect_multiplier(&self) -> f32 {
        if !self.is_functional {
            return 0.0;
        }

        let level_factor = (self.level as f32 - 1.0) / 9.0; // 0-1
        let condition_factor = self.condition as f32 / 100.0; // 0-1

        level_factor * 0.7 + condition_factor * 0.3
    }

    /// 计算通信延迟附加（仅通信模块）
    pub fn communication_delay(&self) -> u32 {
        if self.module_type != ModuleType::Communication {
            return 0;
        }

        match self.level {
            1 => 45,
            2 => 40,
            3 => 35,
            4 => 30,
            5 => 25,
            6 => 20,
            7 => 15,
            8 => 10,
            9 => 5,
            10 => 1,
            _ => 45,
        }
    }

    /// 计算电池续航时间（小时，仅电池模块）
    pub fn battery_duration(&self) -> u32 {
        if self.module_type != ModuleType::Battery {
            return 0;
        }

        match self.level {
            1 => 4,
            2 => 8,
            3 => 12,
            4 => 16,
            5 => 20,
            6 => 24,
            7 => 30,
            8 => 36,
            9 => 42,
            10 => 48,
            _ => 4,
        }
    }

    /// 计算实验误差（仅传感器模块）
    pub fn experiment_error_rate(&self) -> f32 {
        if self.module_type != ModuleType::Sensor {
            return 0.0;
        }

        match self.level {
            1 | 2 => 0.30,
            3 | 4 => 0.20,
            5 | 6 => 0.10,
            7 | 8 => 0.05,
            9 | 10 => 0.02,
            _ => 0.30,
        }
    }

    /// 计算烹饪成功率（仅厨房模块）
    pub fn cooking_success_rate(&self) -> f32 {
        if self.module_type != ModuleType::Kitchen {
            return 0.0;
        }

        let base_rate = match self.level {
            1 => 0.50,
            2 => 0.55,
            3 => 0.60,
            4 => 0.65,
            5 => 0.70,
            6 => 0.75,
            7 => 0.80,
            8 => 0.85,
            9 => 0.90,
            10 => 0.95,
            _ => 0.50,
        };

        base_rate * self.effect_multiplier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = Module::new(ModuleType::Communication);
        assert_eq!(module.level, 1);
        assert_eq!(module.condition, 100);
        assert!(module.is_functional);
    }

    #[test]
    fn test_module_level_up() {
        let mut module = Module::new(ModuleType::Communication);
        assert!(!module.add_experience(50)); // 需要100经验
        assert!(module.add_experience(50)); // 现在共100，升级
        assert_eq!(module.level, 2);
        assert_eq!(module.experience, 0);
    }

    #[test]
    fn test_module_damage() {
        let mut module = Module::new(ModuleType::Communication);
        module.damage(90);
        assert_eq!(module.condition, 10);
        assert!(!module.is_functional);
    }

    #[test]
    fn test_communication_delay() {
        let mut module = Module::new(ModuleType::Communication);
        assert_eq!(module.communication_delay(), 45);

        module.level = 5;
        assert_eq!(module.communication_delay(), 25);

        module.level = 10;
        assert_eq!(module.communication_delay(), 1);
    }
}
