//! 邻里系统模块
//!
//! 管理邻居角色、互动和好感度

mod neighbor;
mod interaction;

pub use neighbor::{AffinityLevel, Neighbor, NeighborAbility, NeighborManager, NeighborRelation};
pub use interaction::{
    Interaction, InteractionType, InteractionResult, InteractionManager,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 邻居帮助请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpRequest {
    /// 请求 ID
    pub id: String,
    /// 请求类型
    pub request_type: HelpRequestType,
    /// 请求描述
    pub description: String,
    /// 所需好感度
    pub required_affinity: u32,
    /// 冷却时间（小时）
    pub cooldown_hours: u32,
    /// 上次使用时间
    pub last_used: Option<DateTime<Utc>>,
}

/// 帮助请求类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpRequestType {
    /// 种植建议
    GardeningAdvice,
    /// 维修帮助
    RepairHelp,
    /// 食材采购
    IngredientPurchase,
    /// 快递代收
    PackagePickup,
    /// 借用工具
    BorrowTool,
    /// 赠送礼物
    SendGift,
}

impl HelpRequestType {
    /// 获取请求名称
    pub fn name(&self) -> &str {
        match self {
            HelpRequestType::GardeningAdvice => "种植建议",
            HelpRequestType::RepairHelp => "维修帮助",
            HelpRequestType::IngredientPurchase => "食材采购",
            HelpRequestType::PackagePickup => "快递代收",
            HelpRequestType::BorrowTool => "借用工具",
            HelpRequestType::SendGift => "赠送礼物",
        }
    }
}

/// 邻居帮助结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpResult {
    /// 是否成功
    pub success: bool,
    /// 结果描述
    pub description: String,
    /// 获得的物品/效果
    pub rewards: Vec<String>,
    /// 好感度变化
    pub affinity_change: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_request_type_names() {
        assert_eq!(HelpRequestType::GardeningAdvice.name(), "种植建议");
        assert_eq!(HelpRequestType::RepairHelp.name(), "维修帮助");
        assert_eq!(HelpRequestType::IngredientPurchase.name(), "食材采购");
    }
}
