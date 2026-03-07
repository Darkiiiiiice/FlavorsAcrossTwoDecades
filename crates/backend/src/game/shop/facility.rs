//! 设施区域系统

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 设施区域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilityZone {
    /// 餐厅
    Restaurant,
    /// 厨房
    Kitchen,
    /// 后院
    Backyard,
    /// 工坊
    Workshop,
}

impl FacilityZone {
    /// 获取区域名称
    pub fn name(&self) -> &str {
        match self {
            FacilityZone::Restaurant => "餐厅",
            FacilityZone::Kitchen => "厨房",
            FacilityZone::Backyard => "后院",
            FacilityZone::Workshop => "工坊",
        }
    }

    /// 获取区域描述
    pub fn description(&self) -> &str {
        match self {
            FacilityZone::Restaurant => "顾客用餐的主要场所",
            FacilityZone::Kitchen => "烹饪美食的工作区域",
            FacilityZone::Backyard => "种植食材的菜园",
            FacilityZone::Workshop => "制作和维修的工具间",
        }
    }

    /// 获取升级成本
    pub fn upgrade_cost(&self, current_level: u32) -> u64 {
        let base_cost = match self {
            FacilityZone::Restaurant => 500,
            FacilityZone::Kitchen => 600,
            FacilityZone::Backyard => 400,
            FacilityZone::Workshop => 450,
        };
        base_cost * current_level as u64
    }

    /// 获取最大等级
    pub fn max_level(&self) -> u32 {
        64
    }
}

/// 区域等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneLevel {
    /// 区域类型
    pub zone: FacilityZone,
    /// 等级 (1-64)
    pub level: u32,
    /// 口碑上限
    pub reputation_cap: u32,
    /// 已解锁的功能
    pub unlocked_features: Vec<String>,
}

impl ZoneLevel {
    /// 创建新的区域
    pub fn new(zone: FacilityZone) -> Self {
        let unlocked_features = Self::get_initial_features(&zone);
        Self {
            zone,
            level: 1,
            reputation_cap: Self::calculate_reputation_cap(&zone, 1),
            unlocked_features,
        }
    }

    /// 升级区域
    pub fn upgrade(&mut self) -> Result<u32, String> {
        if self.level >= self.zone.max_level() {
            return Err("已达到最高等级".to_string());
        }
        self.level += 1;
        self.reputation_cap = Self::calculate_reputation_cap(&self.zone, self.level);
        self.update_unlocked_features();
        Ok(self.level)
    }

    /// 计算口碑上限
    fn calculate_reputation_cap(zone: &FacilityZone, level: u32) -> u32 {
        let base_cap = match zone {
            FacilityZone::Restaurant => 30,
            FacilityZone::Kitchen => 25,
            FacilityZone::Backyard => 20,
            FacilityZone::Workshop => 15,
        };
        base_cap + (level - 1) * 5
    }

    /// 获取初始功能
    fn get_initial_features(zone: &FacilityZone) -> Vec<String> {
        match zone {
            FacilityZone::Restaurant => vec!["基础服务".to_string()],
            FacilityZone::Kitchen => vec!["基础烹饪".to_string()],
            FacilityZone::Backyard => vec!["1块菜地".to_string()],
            FacilityZone::Workshop => vec!["简单维修".to_string()],
        }
    }

    /// 更新已解锁功能
    fn update_unlocked_features(&mut self) {
        self.unlocked_features = match self.zone {
            FacilityZone::Restaurant => {
                let mut features = vec!["基础服务".to_string()];
                if self.level >= 3 {
                    features.push("新菜品槽".to_string());
                }
                if self.level >= 5 {
                    features.push("VIP服务".to_string());
                }
                if self.level >= 10 {
                    features.push("包间".to_string());
                }
                if self.level >= 20 {
                    features.push("主题装饰".to_string());
                }
                if self.level >= 40 {
                    features.push("米其林标准".to_string());
                }
                features
            }
            FacilityZone::Kitchen => {
                let mut features = vec!["基础烹饪".to_string()];
                if self.level >= 3 {
                    features.push("高级设备".to_string());
                }
                if self.level >= 5 {
                    features.push("专业厨房".to_string());
                }
                if self.level >= 10 {
                    features.push("分子料理".to_string());
                }
                if self.level >= 20 {
                    features.push("智能厨具".to_string());
                }
                if self.level >= 40 {
                    features.push("星际厨房".to_string());
                }
                features
            }
            FacilityZone::Backyard => {
                let mut features = vec![];
                features.push(format!("{}块菜地", self.level));
                if self.level >= 5 {
                    features.push("温室框架".to_string());
                }
                if self.level >= 10 {
                    features.push("自动浇水".to_string());
                }
                if self.level >= 20 {
                    features.push("完整温室".to_string());
                }
                if self.level >= 40 {
                    features.push("异星植物区".to_string());
                }
                features
            }
            FacilityZone::Workshop => {
                let mut features = vec!["简单维修".to_string()];
                if self.level >= 3 {
                    features.push("中等制作".to_string());
                }
                if self.level >= 5 {
                    features.push("高级制作".to_string());
                }
                if self.level >= 10 {
                    features.push("创意工坊".to_string());
                }
                if self.level >= 20 {
                    features.push("智能工坊".to_string());
                }
                if self.level >= 40 {
                    features.push("星夜工坊".to_string());
                }
                features
            }
        };
    }

    /// 获取升级成本
    pub fn get_upgrade_cost(&self) -> u64 {
        self.zone.upgrade_cost(self.level)
    }
}

// ========== 设施子系统 ==========

/// 金额类型
pub type Money = u64;

/// 设施类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilityType {
    // 餐厅设施
    DiningTables,
    Lighting,
    Signboard,
    ClimateControl,
    CashierSystem,
    Decoration,
    // 厨房设施
    Stove,
    OvenSteamer,
    Refrigerator,
    Cookware,
    Ventilation,
    Sink,
    StorageCabinet,
    // 后院设施
    VegetablePatch,
    Irrigation,
    ToolShed,
    Greenhouse,
    CompostArea,
    // 工坊设施
    Workbench,
    MaterialRack,
    RepairToolkit,
    PowerLighting,
}

impl FacilityType {
    pub fn name(&self) -> &'static str {
        match self {
            FacilityType::DiningTables => "餐桌椅",
            FacilityType::Lighting => "照明系统",
            FacilityType::Signboard => "招牌",
            FacilityType::ClimateControl => "空调/暖气",
            FacilityType::CashierSystem => "收银系统",
            FacilityType::Decoration => "装饰风格",
            FacilityType::Stove => "灶台",
            FacilityType::OvenSteamer => "烤箱/蒸箱",
            FacilityType::Refrigerator => "冰箱/冷柜",
            FacilityType::Cookware => "厨具",
            FacilityType::Ventilation => "通风系统",
            FacilityType::Sink => "水槽",
            FacilityType::StorageCabinet => "储物柜/货架",
            FacilityType::VegetablePatch => "菜地",
            FacilityType::Irrigation => "灌溉系统",
            FacilityType::ToolShed => "工具房",
            FacilityType::Greenhouse => "温室",
            FacilityType::CompostArea => "堆肥区",
            FacilityType::Workbench => "工作台",
            FacilityType::MaterialRack => "材料架",
            FacilityType::RepairToolkit => "维修工具箱",
            FacilityType::PowerLighting => "电源与照明",
        }
    }

    pub fn zone(&self) -> FacilityZone {
        match self {
            FacilityType::DiningTables
            | FacilityType::Lighting
            | FacilityType::Signboard
            | FacilityType::ClimateControl
            | FacilityType::CashierSystem
            | FacilityType::Decoration => FacilityZone::Restaurant,
            FacilityType::Stove
            | FacilityType::OvenSteamer
            | FacilityType::Refrigerator
            | FacilityType::Cookware
            | FacilityType::Ventilation
            | FacilityType::Sink
            | FacilityType::StorageCabinet => FacilityZone::Kitchen,
            FacilityType::VegetablePatch
            | FacilityType::Irrigation
            | FacilityType::ToolShed
            | FacilityType::Greenhouse
            | FacilityType::CompostArea => FacilityZone::Backyard,
            FacilityType::Workbench
            | FacilityType::MaterialRack
            | FacilityType::RepairToolkit
            | FacilityType::PowerLighting => FacilityZone::Workshop,
        }
    }

    pub fn max_level(&self) -> u32 {
        match self {
            FacilityType::DiningTables
            | FacilityType::Lighting
            | FacilityType::Decoration
            | FacilityType::Stove
            | FacilityType::Cookware
            | FacilityType::Workbench => 5,
            FacilityType::OvenSteamer
            | FacilityType::Refrigerator
            | FacilityType::VegetablePatch => 4,
            FacilityType::Signboard
            | FacilityType::ClimateControl
            | FacilityType::CashierSystem
            | FacilityType::Ventilation
            | FacilityType::Sink
            | FacilityType::StorageCabinet
            | FacilityType::Irrigation
            | FacilityType::Greenhouse
            | FacilityType::CompostArea
            | FacilityType::MaterialRack
            | FacilityType::RepairToolkit
            | FacilityType::PowerLighting => 3,
            FacilityType::ToolShed => 2,
        }
    }

    pub fn initial_condition(&self) -> u32 {
        match self {
            FacilityType::Stove => 20,
            FacilityType::OvenSteamer => 25,
            FacilityType::Ventilation => 25,
            FacilityType::Signboard => 30,
            FacilityType::ClimateControl => 0,
            FacilityType::Lighting => 40,
            FacilityType::Refrigerator => 45,
            FacilityType::Cookware | FacilityType::DiningTables => 50,
            FacilityType::Decoration => 60,
            FacilityType::CashierSystem => 70,
            FacilityType::Sink | FacilityType::StorageCabinet => 55,
            FacilityType::VegetablePatch => 80,
            FacilityType::Irrigation => 30,
            FacilityType::ToolShed => 40,
            FacilityType::Greenhouse | FacilityType::CompostArea => 20,
            FacilityType::Workbench => 40,
            FacilityType::MaterialRack => 50,
            FacilityType::RepairToolkit => 30,
            FacilityType::PowerLighting => 60,
        }
    }
}

/// 效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    MaxCustomers,
    CookingSpeed,
    StorageCapacity,
    PlantingSlots,
    CraftingAbility,
    AtmosphereScore,
    CustomerDiscovery,
    TurnoverRate,
    SeasonModifier,
    ExperimentSuccess,
    FreshnessTime,
    RepairAbility,
}

/// 设施效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityEffect {
    pub effect_type: EffectType,
    pub base_value: f32,
    pub description: String,
}

/// 子设施
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFacility {
    pub id: String,
    pub name: String,
    pub zone: FacilityZone,
    pub facility_type: FacilityType,
    pub level: u32,
    pub max_level: u32,
    pub condition: u32,
    pub is_functional: bool,
    pub effect: FacilityEffect,
}

impl SubFacility {
    pub fn new(facility_type: FacilityType) -> Self {
        let max_level = facility_type.max_level();
        let condition = facility_type.initial_condition();
        let effect = calculate_effect(facility_type, 1, condition);

        Self {
            id: Uuid::new_v4().to_string(),
            name: facility_type.name().to_string(),
            zone: facility_type.zone(),
            facility_type,
            level: 1,
            max_level,
            condition,
            is_functional: condition >= 20,
            effect,
        }
    }

    pub fn upgrade(&mut self) -> Result<u32, String> {
        if self.level >= self.max_level {
            return Err(format!("{} 已达到最高等级 {}", self.name, self.max_level));
        }
        self.level += 1;
        self.effect = calculate_effect(self.facility_type, self.level, self.condition);
        Ok(self.level)
    }

    pub fn repair(&mut self, amount: u32) {
        self.condition = (self.condition + amount).min(100);
        self.is_functional = self.condition >= 20;
        self.effect = calculate_effect(self.facility_type, self.level, self.condition);
    }
}

/// 计算设施效果
pub fn calculate_effect(facility_type: FacilityType, level: u32, condition: u32) -> FacilityEffect {
    let cf = condition as f32 / 100.0;
    let lf = 0.5 + level as f32 * 0.1;

    match facility_type {
        FacilityType::DiningTables => FacilityEffect {
            effect_type: EffectType::MaxCustomers,
            base_value: (4 + level * 2) as f32 * cf,
            description: format!("可同时接待 {} 位顾客", (4 + level * 2)),
        },
        FacilityType::Lighting => FacilityEffect {
            effect_type: EffectType::AtmosphereScore,
            base_value: lf * cf * 20.0,
            description: format!("照明氛围 +{}%", (lf * cf * 20.0 * 100.0) as i32),
        },
        FacilityType::Signboard => FacilityEffect {
            effect_type: EffectType::CustomerDiscovery,
            base_value: lf * cf * 5.0,
            description: format!("新顾客发现率 +{}%", (lf * cf * 5.0 * 100.0) as i32),
        },
        FacilityType::ClimateControl => FacilityEffect {
            effect_type: EffectType::SeasonModifier,
            base_value: lf * cf * 15.0,
            description: format!("季节客流修正 +{}%", (lf * cf * 15.0 * 100.0) as i32),
        },
        FacilityType::CashierSystem => FacilityEffect {
            effect_type: EffectType::TurnoverRate,
            base_value: lf * cf * 10.0,
            description: format!("翻台率 +{}%", (lf * cf * 10.0 * 100.0) as i32),
        },
        FacilityType::Decoration => FacilityEffect {
            effect_type: EffectType::AtmosphereScore,
            base_value: lf * cf * 10.0,
            description: format!("氛围评分 +{}", (lf * cf * 10.0) as i32),
        },
        FacilityType::Stove => FacilityEffect {
            effect_type: EffectType::CookingSpeed,
            base_value: lf * cf,
            description: format!("烹饪速度 +{}%", (lf * cf * 100.0) as i32),
        },
        FacilityType::OvenSteamer => FacilityEffect {
            effect_type: EffectType::CookingSpeed,
            base_value: lf * cf * 0.8,
            description: format!("烘焙/蒸制速度 +{}%", (lf * cf * 0.8 * 100.0) as i32),
        },
        FacilityType::Refrigerator => FacilityEffect {
            effect_type: EffectType::FreshnessTime,
            base_value: (1.0 + level as f32 * 0.5) * cf,
            description: format!(
                "食材保鲜时间 {} 天",
                ((1.0 + level as f32 * 0.5) * cf) as i32
            ),
        },
        FacilityType::Cookware => FacilityEffect {
            effect_type: EffectType::CookingSpeed,
            base_value: lf * cf * 0.5,
            description: format!("烹饪效率 +{}%", (lf * cf * 0.5 * 100.0) as i32),
        },
        FacilityType::Ventilation => FacilityEffect {
            effect_type: EffectType::AtmosphereScore,
            base_value: lf * cf * 8.0,
            description: format!("厨房环境 +{}", (lf * cf * 8.0) as i32),
        },
        FacilityType::Sink => FacilityEffect {
            effect_type: EffectType::CookingSpeed,
            base_value: lf * cf * 0.3,
            description: format!("清洗效率 +{}%", (lf * cf * 0.3 * 100.0) as i32),
        },
        FacilityType::StorageCabinet => FacilityEffect {
            effect_type: EffectType::StorageCapacity,
            base_value: (20 + level as i32 * 10) as f32 * cf,
            description: format!("存储容量 {} 单位", (20 + level as i32 * 10)),
        },
        FacilityType::VegetablePatch => FacilityEffect {
            effect_type: EffectType::PlantingSlots,
            base_value: (1 + level) as f32,
            description: format!("菜地槽位: {}", 1 + level),
        },
        FacilityType::Irrigation => FacilityEffect {
            effect_type: EffectType::PlantingSlots,
            base_value: level as f32 * 0.5,
            description: format!("灌溉效率 +{}%", level * 20),
        },
        FacilityType::ToolShed => FacilityEffect {
            effect_type: EffectType::StorageCapacity,
            base_value: (10 + level as i32 * 15) as f32 * cf,
            description: format!("工具存储 {} 单位", (10 + level as i32 * 15)),
        },
        FacilityType::Greenhouse => FacilityEffect {
            effect_type: EffectType::PlantingSlots,
            base_value: level as f32 * 2.0,
            description: format!("温室种植槽位 +{}", level * 2),
        },
        FacilityType::CompostArea => FacilityEffect {
            effect_type: EffectType::PlantingSlots,
            base_value: level as f32 * 0.3,
            description: format!("堆肥产出 +{}%", level * 15),
        },
        FacilityType::Workbench => FacilityEffect {
            effect_type: EffectType::CraftingAbility,
            base_value: lf * cf,
            description: format!("制作能力 +{}%", (lf * cf * 100.0) as i32),
        },
        FacilityType::MaterialRack => FacilityEffect {
            effect_type: EffectType::StorageCapacity,
            base_value: (30 + level as i32 * 20) as f32 * cf,
            description: format!("材料容量 {} 单位", (30 + level as i32 * 20)),
        },
        FacilityType::RepairToolkit => FacilityEffect {
            effect_type: EffectType::RepairAbility,
            base_value: lf * cf,
            description: format!("维修能力 +{}%", (lf * cf * 100.0) as i32),
        },
        FacilityType::PowerLighting => FacilityEffect {
            effect_type: EffectType::CraftingAbility,
            base_value: lf * cf * 0.8,
            description: format!("工坊电力 +{}%", (lf * cf * 0.8 * 100.0) as i32),
        },
    }
}

/// 材料类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    Wood,
    Fabric,
    LightBulb,
    OldPhoto,
    RetroTile,
    Metal,
    Plastic,
    Glass,
    Ceramic,
    Seed,
    Fertilizer,
}

/// 材料成本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCost {
    pub material_type: MaterialType,
    pub quantity: u32,
}

/// 人员类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonnelType {
    PandaOnly,
    NeedElectrician,
    NeedCarpenter,
    NeedHelper,
    NeedNeighbor,
}

/// 升级路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePath {
    pub facility_type: FacilityType,
    pub from_level: u32,
    pub to_level: u32,
    pub cost: Money,
    pub materials: Vec<MaterialCost>,
    pub time_days: u32,
    pub required_personnel: PersonnelType,
    pub unlocks: Option<String>,
}

impl UpgradePath {
    pub fn get_upgrade_path(facility_type: FacilityType, from_level: u32) -> Option<Self> {
        let to_level = from_level + 1;

        let cost: Money = match facility_type {
            FacilityType::DiningTables => 2000,
            FacilityType::Greenhouse => 1800,
            FacilityType::Workbench => 1500,
            FacilityType::Stove => 1200,
            FacilityType::OvenSteamer | FacilityType::Refrigerator => 1000,
            FacilityType::Signboard => 800,
            FacilityType::ClimateControl => 1200,
            FacilityType::Decoration => 900,
            _ => 500,
        } * from_level as u64;

        let materials = match facility_type {
            FacilityType::DiningTables => vec![MaterialCost {
                material_type: MaterialType::Wood,
                quantity: from_level * 2,
            }],
            FacilityType::Lighting => vec![MaterialCost {
                material_type: MaterialType::LightBulb,
                quantity: from_level,
            }],
            FacilityType::Stove => vec![MaterialCost {
                material_type: MaterialType::Metal,
                quantity: from_level * 2,
            }],
            FacilityType::Workbench => vec![MaterialCost {
                material_type: MaterialType::Wood,
                quantity: from_level * 3,
            }],
            _ => vec![],
        };

        let (time_days, personnel) = match facility_type {
            FacilityType::Stove | FacilityType::Refrigerator => (2, PersonnelType::NeedElectrician),
            FacilityType::Lighting => (1, PersonnelType::NeedElectrician),
            _ => (1, PersonnelType::PandaOnly),
        };

        Some(UpgradePath {
            facility_type,
            from_level,
            to_level,
            cost,
            materials,
            time_days,
            required_personnel: personnel,
            unlocks: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_creation() {
        let zone = ZoneLevel::new(FacilityZone::Restaurant);
        assert_eq!(zone.level, 1);
        assert!(zone.unlocked_features.contains(&"基础服务".to_string()));
    }

    #[test]
    fn test_zone_upgrade() {
        let mut zone = ZoneLevel::new(FacilityZone::Restaurant);
        let result = zone.upgrade();
        assert!(result.is_ok());
        assert_eq!(zone.level, 2);
        assert!(zone.reputation_cap > 30);
        zone.upgrade().unwrap();
        assert_eq!(zone.level, 3);
        assert!(zone.unlocked_features.contains(&"新菜品槽".to_string()));
    }

    #[test]
    fn test_upgrade_cost() {
        let zone = ZoneLevel::new(FacilityZone::Restaurant);
        let cost = zone.get_upgrade_cost();
        assert_eq!(cost, 500);
        let mut zone2 = zone;
        zone2.upgrade().unwrap();
        let cost2 = zone2.get_upgrade_cost();
        assert_eq!(cost2, 1000);
    }

    #[test]
    fn test_backyard_features() {
        let mut zone = ZoneLevel::new(FacilityZone::Backyard);
        assert!(zone.unlocked_features[0].contains("1块菜地"));
        zone.upgrade().unwrap();
        zone.upgrade().unwrap();
        assert!(zone.unlocked_features[0].contains("3块菜地"));
    }

    #[test]
    fn test_subfacility_creation() {
        let facility = SubFacility::new(FacilityType::DiningTables);
        assert_eq!(facility.level, 1);
        assert_eq!(facility.condition, 50);
        assert!(facility.is_functional);
    }

    #[test]
    fn test_facility_upgrade() {
        let mut facility = SubFacility::new(FacilityType::Stove);
        assert_eq!(facility.level, 1);
        assert_eq!(facility.condition, 20);
        let result = facility.upgrade();
        assert!(result.is_ok());
        assert_eq!(facility.level, 2);
    }

    #[test]
    fn test_facility_repair() {
        let mut facility = SubFacility::new(FacilityType::ClimateControl);
        assert_eq!(facility.condition, 0);
        assert!(!facility.is_functional);
        facility.repair(50);
        assert_eq!(facility.condition, 50);
        assert!(facility.is_functional);
    }
}
