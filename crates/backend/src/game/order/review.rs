//! 评价系统

use chrono::{DateTime, Utc};

/// 评价
#[derive(Debug, Clone)]
pub struct Review {
    /// 评价 ID
    pub id: i64,
    /// 顾客 ID
    pub customer_id: i64,
    /// 订单 ID
    pub order_id: i64,
    /// 评分 (1-5)
    pub rating: u32,
    /// 评价内容
    pub content: String,
    /// 菜品评分
    pub dish_rating: u32,
    /// 服务评分
    pub service_rating: u32,
    /// 环境评分
    pub environment_rating: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Review {
    /// 创建新评价
    pub fn new(customer_id: i64, order_id: i64, rating: u32, content: String) -> Self {
        // 随机生成各维度评分（与总评分相关）
        let base_rating = rating as i32;
        let dish_rating = (base_rating + (rand::random::<i32>() % 2 - 1)).clamp(1, 5) as u32;
        let service_rating = (base_rating + (rand::random::<i32>() % 2 - 1)).clamp(1, 5) as u32;
        let environment_rating = (base_rating + (rand::random::<i32>() % 2 - 1)).clamp(1, 5) as u32;

        Self {
            id: 0,
            customer_id,
            order_id,
            rating,
            content,
            dish_rating,
            service_rating,
            environment_rating,
            created_at: Utc::now(),
        }
    }

    /// 计算综合评分
    pub fn calculate_overall_score(&self) -> f32 {
        (self.dish_rating + self.service_rating + self.environment_rating) as f32 / 3.0
    }
}
