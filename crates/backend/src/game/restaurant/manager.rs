//! 餐厅管理器
//!
//! 管理餐厅的状态更新和周期性任务

use std::sync::Arc;

use super::Restaurant;
use crate::game::order::Order;
use crate::{db::DbPool, game::PandaStatus};

/// 餐厅管理器
#[derive(Debug)]
pub struct RestaurantManager {
    /// 餐厅实例
    pub restaurant: Restaurant,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
}

impl RestaurantManager {
    /// 创建新的餐厅管理器
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        let restaurant = Restaurant::new();
        Self {
            restaurant,
            db_pool,
        }
    }

    /// 使用现有餐厅创建管理器
    pub fn with_restaurant(restaurant: Restaurant, db_pool: Arc<DbPool>) -> Self {
        Self {
            restaurant,
            db_pool,
        }
    }

    /// 更新餐厅状态（每个 tick 调用）
    pub async fn tick(&mut self, _panda_status: PandaStatus) {
        tracing::debug!(
            "Restaurant update: status={:?}, customers={}, pending_orders={}",
            self.restaurant.status,
            0,
            0
        );
    }
}
