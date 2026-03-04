//! 评价系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 评价情感
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSentiment {
    /// 差评 (1-2星)
    Negative,
    /// 中评 (3星)
    Neutral,
    /// 好评 (4-5星)
    Positive,
}

impl ReviewSentiment {
    /// 从评分获取情感
    pub fn from_rating(rating: u32) -> Self {
        match rating {
            1..=2 => ReviewSentiment::Negative,
            3 => ReviewSentiment::Neutral,
            4..=5 => ReviewSentiment::Positive,
            _ => ReviewSentiment::Neutral,
        }
    }

    /// 获取情感名称
    pub fn name(&self) -> &str {
        match self {
            ReviewSentiment::Negative => "差评",
            ReviewSentiment::Neutral => "中评",
            ReviewSentiment::Positive => "好评",
        }
    }
}

/// 评价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// 评价 ID
    pub id: Uuid,
    /// 顾客 ID
    pub customer_id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 订单 ID
    pub order_id: Uuid,
    /// 评分 (1-5)
    pub rating: u32,
    /// 评价内容
    pub content: String,
    /// 情感
    pub sentiment: ReviewSentiment,
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
    pub fn new(
        customer_id: Uuid,
        save_id: Uuid,
        order_id: Uuid,
        rating: u32,
        content: String,
    ) -> Self {
        let sentiment = ReviewSentiment::from_rating(rating);

        // 随机生成各维度评分（与总评分相关）
        let base_rating = rating as i32;
        let dish_rating = (base_rating + (rand::random::<i32>() % 2 - 1))
            .clamp(1, 5) as u32;
        let service_rating = (base_rating + (rand::random::<i32>() % 2 - 1))
            .clamp(1, 5) as u32;
        let environment_rating = (base_rating + (rand::random::<i32>() % 2 - 1))
            .clamp(1, 5) as u32;

        Self {
            id: Uuid::new_v4(),
            customer_id,
            save_id,
            order_id,
            rating,
            content,
            sentiment,
            dish_rating,
            service_rating,
            environment_rating,
            created_at: Utc::now(),
        }
    }

    /// 生成随机评价
    pub fn random(customer_id: Uuid, save_id: Uuid, order_id: Uuid) -> Self {
        let rating = 1 + rand::random::<u32>() % 5;
        let content = match rating {
            1..=2 => "这次体验不太好。".to_string(),
            3 => "还可以，中规中矩。".to_string(),
            4..=5 => "非常满意，下次还会来！".to_string(),
            _ => "一般般。".to_string(),
        };

        Self::new(customer_id, save_id, order_id, rating, content)
    }

    /// 计算综合评分
    pub fn calculate_overall_score(&self) -> f32 {
        (self.dish_rating + self.service_rating + self.environment_rating) as f32 / 3.0
    }

    /// 是否为好评
    pub fn is_positive(&self) -> bool {
        self.sentiment == ReviewSentiment::Positive
    }

    /// 是否为差评
    pub fn is_negative(&self) -> bool {
        self.sentiment == ReviewSentiment::Negative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_creation() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        let review = Review::new(
            customer_id,
            save_id,
            order_id,
            5,
            "非常好吃！".to_string(),
        );

        assert_eq!(review.rating, 5);
        assert_eq!(review.sentiment, ReviewSentiment::Positive);
        assert!(review.is_positive());
    }

    #[test]
    fn test_review_sentiment() {
        assert_eq!(
            ReviewSentiment::from_rating(1),
            ReviewSentiment::Negative
        );
        assert_eq!(
            ReviewSentiment::from_rating(3),
            ReviewSentiment::Neutral
        );
        assert_eq!(
            ReviewSentiment::from_rating(5),
            ReviewSentiment::Positive
        );
    }

    #[test]
    fn test_review_random() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        let review = Review::random(customer_id, save_id, order_id);

        assert!(review.rating >= 1 && review.rating <= 5);
        assert!(!review.content.is_empty());
    }

    #[test]
    fn test_review_overall_score() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        let mut review = Review::new(
            customer_id,
            save_id,
            order_id,
            5,
            "测试".to_string(),
        );

        review.dish_rating = 5;
        review.service_rating = 4;
        review.environment_rating = 4;

        let score = review.calculate_overall_score();
        assert!(score > 4.0 && score < 5.0);
    }
}
