//! 餐厅管理器
//!
//! 管理餐厅的状态更新和周期性任务

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Timelike, Utc};

use super::Restaurant;
use crate::db::repositories::customer::CustomerRepository;
use crate::{db::DbPool, game::PandaStatus};

/// 客流生成最小间隔（秒）
const MIN_CUSTOMER_INTERVAL: i64 = 30;
/// 客流生成最大间隔（秒）
const MAX_CUSTOMER_INTERVAL: i64 = 180;

/// 餐厅管理器
#[derive(Debug)]
pub struct RestaurantManager {
    /// 餐厅实例
    pub restaurant: Restaurant,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// 下次顾客到达时间戳
    next_customer_arrival: Option<i64>,
}

impl RestaurantManager {
    /// 创建新的餐厅管理器
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        let restaurant = Restaurant::new();
        Self {
            restaurant,
            db_pool,
            next_customer_arrival: None,
        }
    }

    /// 使用现有餐厅创建管理器
    pub fn with_restaurant(restaurant: Restaurant, db_pool: Arc<DbPool>) -> Self {
        Self {
            restaurant,
            db_pool,
            next_customer_arrival: None,
        }
    }

    /// 更新餐厅状态（每个 tick 调用）
    pub async fn tick(&mut self, _panda_status: PandaStatus, current_timestamp: i64) {
        tracing::debug!(
            "Restaurant update: status={:?}, customers={}/{}, next_arrival={:?}",
            self.restaurant.status,
            self.restaurant.current_customers.len(),
            self.restaurant.max_capacity,
            self.next_customer_arrival
        );
        self.restaurant.tick();

        // 只有营业中才处理客流
        if !self.restaurant.is_open() {
            return;
        }

        // 检查是否需要安排顾客到达
        if self.next_customer_arrival.is_none() {
            self.schedule_next_customer(current_timestamp);
        }

        // 检查是否到了顾客到达时间
        if let Some(arrival_time) = self.next_customer_arrival {
            if current_timestamp >= arrival_time {
                // 尝试让顾客进入
                if let Err(e) = self.try_admit_customer().await {
                    tracing::error!("Failed to admit customer: {}", e);
                }
                // 安排下一个顾客
                self.schedule_next_customer(current_timestamp);
            }
        }
    }

    /// 根据时间段计算客流密度（返回 0.0-2.0 的倍率）
    ///
    /// 模拟真实餐厅客流：
    /// - 早餐高峰：7:00-9:00
    /// - 午餐高峰：11:00-13:00
    /// - 晚餐高峰：17:00-20:00
    /// - 深夜低谷：22:00-6:00
    fn get_traffic_multiplier(timestamp: i64) -> f32 {
        let datetime: DateTime<Utc> = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .unwrap_or_else(Utc::now);
        let hour = datetime.hour();

        match hour {
            // 深夜低谷（几乎没客人）
            0..=5 => 0.0,
            // 早餐准备期
            6 => 0.2,
            // 早餐高峰
            7..=8 => 1.1,
            // 早餐结束
            9..=10 => 0.5,
            // 午餐高峰
            11..=13 => 1.8,
            // 下午茶时间
            14..=16 => 0.3,
            // 晚餐准备期
            17 => 0.7,
            // 晚餐高峰
            18..=20 => 2.0,
            // 夜宵时间
            21 => 1.0,
            _ => 0.0,
        }
    }

    /// 计算下一个顾客到达的间隔时间
    fn calculate_arrival_interval(timestamp: i64) -> i64 {
        let multiplier = Self::get_traffic_multiplier(timestamp);
        let base_interval = MIN_CUSTOMER_INTERVAL
            + (rand::random::<i64>() % (MAX_CUSTOMER_INTERVAL - MIN_CUSTOMER_INTERVAL));

        // 客流密度越高，间隔越短
        let adjusted_interval = (base_interval as f32 / multiplier) as i64;
        // 确保间隔不会太短
        adjusted_interval.max(10)
    }

    /// 安排下一个顾客到达
    fn schedule_next_customer(&mut self, current_timestamp: i64) {
        let interval = Self::calculate_arrival_interval(current_timestamp);
        self.next_customer_arrival = Some(current_timestamp + interval);
        tracing::debug!(
            "Next customer scheduled in {} seconds (traffic multiplier: {:.1})",
            interval,
            Self::get_traffic_multiplier(current_timestamp)
        );
    }

    /// 尝试让一位顾客进入餐厅
    async fn try_admit_customer(&mut self) -> crate::error::GameResult<()> {
        // 检查餐厅是否已满
        if self.restaurant.current_customers.len() >= self.restaurant.max_capacity as usize {
            tracing::debug!("Restaurant is full, cannot admit more customers");
            return Ok(());
        }

        // 从数据库随机获取一位顾客
        let repo = CustomerRepository::new(self.db_pool.pool().clone());
        let customer_record = repo.find_random().await?;

        match customer_record {
            Some(record) => {
                let customer = record.to_customer();
                tracing::info!(
                    "Customer {} entered the restaurant (capacity: {}/{})",
                    customer.name,
                    self.restaurant.current_customers.len() + 1,
                    self.restaurant.max_capacity
                );
                self.restaurant.current_customers.push(customer);
                Ok(())
            }
            None => {
                tracing::debug!("No customers in database to admit");
                Ok(())
            }
        }
    }

    /// 顾客离开餐厅
    pub fn customer_leave(&mut self, customer_id: i64) -> bool {
        let initial_len = self.restaurant.current_customers.len();
        self.restaurant
            .current_customers
            .retain(|c| c.id != customer_id);
        let removed = self.restaurant.current_customers.len() < initial_len;
        if removed {
            tracing::info!("Customer {} left the restaurant", customer_id);
        }
        removed
    }

    /// 获取当前顾客数量
    pub fn current_customer_count(&self) -> usize {
        self.restaurant.current_customers.len()
    }

    /// 餐厅是否已满
    pub fn is_full(&self) -> bool {
        self.restaurant.current_customers.len() >= self.restaurant.max_capacity as usize
    }

    /// 获取剩余座位数
    pub fn available_seats(&self) -> i32 {
        self.restaurant.max_capacity - self.restaurant.current_customers.len() as i32
    }
}
