//! 小馆状态数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 小馆状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopState {
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 小馆名称
    pub name: String,
    /// 资金
    pub funds: u64,
    /// 声望
    pub reputation: f32,
    /// 餐厅等级
    pub restaurant_level: u32,
    /// 厨房等级
    pub kitchen_level: u32,
    /// 后院等级
    pub backyard_level: u32,
    /// 工作间等级
    pub workshop_level: u32,
}

impl ShopState {
    /// 创建新的小馆状态
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            name: "星夜小馆".to_string(),
            funds: 10000,
            reputation: 0.0,
            restaurant_level: 1,
            kitchen_level: 1,
            backyard_level: 1,
            workshop_level: 1,
        }
    }
}

/// 设施状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityRecord {
    /// 设施ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 区域
    pub zone: String,
    /// 名称
    pub name: String,
    /// 等级
    pub level: u32,
    /// 状态值
    pub condition: u32,
    /// 升级进度（JSON）
    pub upgrade_progress: Option<String>,
}

impl FacilityRecord {
    /// 创建新设施
    pub fn new(save_id: Uuid, zone: String, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            zone,
            name,
            level: 1,
            condition: 100,
            upgrade_progress: None,
        }
    }
}
