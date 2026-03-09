//! 顾客主体

use serde::{Deserialize, Serialize};

use super::preference::Preference;

/// 顾客类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerType {
    /// 普通顾客
    Normal,
    /// 美食家
    Foodie,
    /// 评论家
    Critic,
}

impl CustomerType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            CustomerType::Normal => "普通顾客",
            CustomerType::Foodie => "美食家",
            CustomerType::Critic => "评论家",
        }
    }

    /// 获取小费倍率
    pub fn tip_multiplier(&self) -> f32 {
        match self {
            CustomerType::Normal => 1.0,
            CustomerType::Foodie => 1.2,
            CustomerType::Critic => 0.8,
        }
    }

    /// 获取耐心加成
    pub fn patience_bonus(&self) -> i32 {
        match self {
            CustomerType::Normal => 0,
            CustomerType::Foodie => 10,
            CustomerType::Critic => -10,
        }
    }
}

/// 顾客
#[derive(Debug, Clone)]
pub struct Customer {
    /// 顾客 ID
    pub id: i64,
    /// 顾客姓名
    pub name: String,
    /// 年龄
    pub age: u32,
    /// 职业
    pub occupation: String,
    /// 顾客类型
    pub customer_type: CustomerType,
    /// 偏好
    pub preference: Preference,
    /// 好感度 (0-1000)
    pub affinity: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 故事背景
    pub story_background: String,
}

impl Customer {
    /// 创建新顾客
    pub fn new(id_num: u32) -> Self {
        Self {
            id: 0,
            name: format!("顾客#{}", id_num),
            age: 30,
            occupation: String::new(),
            customer_type: CustomerType::Normal,
            preference: Preference::new(),
            affinity: 0,
            visit_count: 0,
            story_background: String::new(),
        }
    }

    /// 更新好感度
    pub fn update_affinity(&mut self, delta: i32) {
        let new_affinity = (self.affinity as i32 + delta).clamp(0, 1000);
        self.affinity = new_affinity as u32;
    }

    /// 是否为回头客
    pub fn is_returning(&self) -> bool {
        self.visit_count > 1
    }
}
