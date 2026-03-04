//! 设施区域系统

use serde::{Deserialize, Serialize};

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

        // 升级到2级
        let result = zone.upgrade();
        assert!(result.is_ok());
        assert_eq!(zone.level, 2);
        assert!(zone.reputation_cap > 30);

        // 升级到3级（解锁新功能）
        zone.upgrade().unwrap();
        assert_eq!(zone.level, 3);
        assert!(zone.unlocked_features.contains(&"新菜品槽".to_string()));
    }

    #[test]
    fn test_upgrade_cost() {
        let zone = ZoneLevel::new(FacilityZone::Restaurant);
        let cost = zone.get_upgrade_cost();
        assert_eq!(cost, 500); // 500 * 1

        let mut zone2 = zone.clone();
        zone2.upgrade().unwrap();
        let cost2 = zone2.get_upgrade_cost();
        assert_eq!(cost2, 1000); // 500 * 2
    }

    #[test]
    fn test_backyard_features() {
        let mut zone = ZoneLevel::new(FacilityZone::Backyard);

        // 初始：1块菜地
        assert!(zone.unlocked_features[0].contains("1块菜地"));

        // 升级到3级：3块菜地
        zone.upgrade().unwrap();
        zone.upgrade().unwrap();
        assert!(zone.unlocked_features[0].contains("3块菜地"));
    }
}
