//! 菜地数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 菜地状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenPlot {
    /// 菜地ID
    pub id: Uuid,
    /// 关联的存档ID
    pub save_id: Uuid,
    /// 地块编号
    pub plot_number: u32,
    /// 是否解锁
    pub is_unlocked: bool,
    /// 当前作物（JSON）
    pub current_crop: Option<String>,
    /// 肥力
    pub fertility: u32,
    /// 湿度
    pub moisture: u32,
}

impl GardenPlot {
    /// 创建新菜地
    pub fn new(save_id: Uuid, plot_number: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            save_id,
            plot_number,
            is_unlocked: plot_number == 1, // 第一块地默认解锁
            current_crop: None,
            fertility: 100,
            moisture: 50,
        }
    }
}
