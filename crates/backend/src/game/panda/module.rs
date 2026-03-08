//! Panda 模块系统

use serde::{Deserialize, Serialize};

const MAX_BATTERY_LEVELS: usize = 10;

const MAX_POWER: [f32; MAX_BATTERY_LEVELS] = [
    1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 7000.0, 8000.0, 9000.0, 10000.0,
];
const WORK_SPEED: [f32; MAX_BATTERY_LEVELS] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];
const CHARGE_RATE: [f32; MAX_BATTERY_LEVELS] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

const LEVEL_UP_XP: [u32; MAX_BATTERY_LEVELS] = [
    100000, 200000, 300000, 400000, 500000, 600000, 700000, 800000, 900000, 1000000,
];

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

#[derive(Debug, Clone)]
pub struct BatteryModule {
    /// 等级 (1-10)
    pub level: u32,
    /// 完好度 (0-100)
    pub condition: f32,
    /// 经验值
    pub experience: u32,
    /// 是否可用
    pub is_functional: bool,
    /// 电量
    pub power: f32,
    /// 充电速度
    pub charge_rate: f32,
    /// 充电中
    pub is_charging: bool,
}

impl BatteryModule {
    pub fn new() -> Self {
        Self {
            level: 1,
            condition: 100.0,
            experience: 0,
            is_functional: true,
            power: MAX_POWER[0] * 0.99,
            charge_rate: CHARGE_RATE[0],
            is_charging: false,
        }
    }

    pub fn power_is_full(&self) -> bool {
        self.power >= MAX_POWER[0]
    }

    pub fn is_charging(&self) -> bool {
        self.is_charging
    }

    pub fn set_charging(&mut self, is_charging: bool) {
        self.is_charging = is_charging;
    }

    pub fn tick(&mut self) {
        tracing::info!(
            "Battery Status: level = {}, condition: = {}, experience = {}, power = {}, charge_rate = {}, is_charging = {}",
            self.level,
            self.condition,
            self.experience,
            self.power,
            self.charge_rate,
            self.is_charging
        );
        if self.is_charging {
            self.charge();
        } else {
            self.work();
        }
        self.level_up();
    }

    fn level_up(&mut self) {
        if self.experience >= LEVEL_UP_XP[(self.level - 1) as usize] {
            self.level += 1;
            self.experience = 0;
        }
    }

    fn charge(&mut self) {
        let max_power = MAX_POWER[(self.level - 1) as usize];
        if self.power >= max_power {
            self.is_charging = false;
            return;
        }
        self.power = (self.power + self.charge_rate).min(max_power);
    }

    fn work(&mut self) {
        if self.power > 0.0 {
            self.power -= WORK_SPEED[(self.level - 1) as usize];
            self.condition -= 0.001;
            self.experience += 1;
        }
    }
}
