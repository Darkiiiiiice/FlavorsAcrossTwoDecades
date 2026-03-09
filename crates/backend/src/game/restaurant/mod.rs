//! 餐厅系统模块
//!
//! 管理餐厅的整体运营状态，包括顾客服务、订单处理等

mod manager;

pub use manager::RestaurantManager;

use crate::game::Customer;

const MAX_CAPACITY: i32 = 20;

/// 餐厅状态
#[derive(Debug, Clone, Default)]
pub struct Restaurant {
    /// ID
    pub id: i64,
    /// 餐厅名称
    pub name: String,
    /// 当前营业状态
    pub status: RestaurantStatus,
    /// 最大顾客容量
    pub max_capacity: i32,
    /// 当前餐厅顾客
    pub current_customers: Vec<Customer>,
}

impl Restaurant {
    /// 创建新的餐厅实例
    pub fn new() -> Self {
        Self {
            id: 0,
            name: "星夜小馆".to_string(),
            status: RestaurantStatus::Open,
            max_capacity: MAX_CAPACITY,
            current_customers: Vec::new(),
        }
    }

    /// 餐厅是否营业中
    pub fn is_open(&self) -> bool {
        matches!(self.status, RestaurantStatus::Open)
    }

    pub fn tick(&mut self) {
        tracing::debug!(
            "Restaurant tick: status={:?}, customers={:?}",
            self.status,
            self.current_customers
        );
    }
}

/// 餐厅营业状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestaurantStatus {
    /// 关闭
    #[default]
    Closed,
    /// 营业中
    Open,
    /// 清洁中
    Cleaning,
}
