//! 顾客主体

use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::order::Order;
use super::preference::Preference;
use super::review::Review;
use super::vip::VIPStatus;

/// 顾客类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerType {
    /// 普通顾客
    Normal,
    /// 美食家
    Foodie,
    /// 评论家
    Critic,
    /// VIP
    VIP,
    /// 邻居
    Neighbor,
}

impl CustomerType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            CustomerType::Normal => "普通顾客",
            CustomerType::Foodie => "美食家",
            CustomerType::Critic => "评论家",
            CustomerType::VIP => "VIP顾客",
            CustomerType::Neighbor => "邻居",
        }
    }

    /// 获取小费倍率
    pub fn tip_multiplier(&self) -> f32 {
        match self {
            CustomerType::Normal => 1.0,
            CustomerType::Foodie => 1.2,
            CustomerType::Critic => 0.8,
            CustomerType::VIP => 1.5,
            CustomerType::Neighbor => 1.1,
        }
    }

    /// 获取耐心加成
    pub fn patience_bonus(&self) -> i32 {
        match self {
            CustomerType::Normal => 0,
            CustomerType::Foodie => 10,
            CustomerType::Critic => -10,
            CustomerType::VIP => 20,
            CustomerType::Neighbor => 15,
        }
    }

    /// 随机生成
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..20) {
            0 => CustomerType::Foodie,
            1 => CustomerType::Critic,
            2..=4 => CustomerType::VIP,
            5..=7 => CustomerType::Neighbor,
            _ => CustomerType::Normal,
        }
    }
}

/// 顾客
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    /// 顾客 ID
    pub id: Uuid,
    /// 顾客编号
    pub id_num: u32,
    /// 顾客姓名
    pub name: String,
    /// 顾客类型
    pub customer_type: CustomerType,
    /// 偏好
    pub preference: Preference,
    /// VIP 状态
    pub vip_status: VIPStatus,
    /// 好感度 (0-1000)
    pub affinity: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 当前订单
    pub current_order: Option<Order>,
    /// 评价历史
    pub reviews: Vec<Review>,
    /// 到达时间
    pub arrived_at: DateTime<Utc>,
    /// 是否已完成用餐
    pub finished: bool,
}

impl Customer {
    /// 创建新顾客
    pub fn new(id_num: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            id_num,
            name: format!("顾客#{}", id_num),
            customer_type: CustomerType::random(),
            preference: Preference::new(),
            vip_status: VIPStatus::new(),
            affinity: 0,
            visit_count: 0,
            current_order: None,
            reviews: Vec::new(),
            arrived_at: Utc::now(),
            finished: false,
        }
    }

    /// 生成随机顾客
    pub fn random(id_num: u32) -> Self {
        let mut customer = Self::new(id_num);
        customer.name = generate_random_name();
        customer
    }

    /// 更新好感度
    pub fn update_affinity(&mut self, delta: i32) {
        let new_affinity = (self.affinity as i32 + delta).clamp(0, 1000);
        self.affinity = new_affinity as u32;

        // 更新 VIP 等级
        self.vip_status.update_level(self.affinity);
    }

    /// 创建订单
    pub fn create_order(&mut self, save_id: Uuid) -> &mut Order {
        let order = Order::new(self.id, save_id);
        self.current_order = Some(order);
        self.visit_count += 1;
        self.current_order.as_mut().unwrap()
    }

    /// 完成订单
    pub fn complete_order(&mut self, satisfaction: f32) -> Option<Review> {
        let order = self.current_order.as_mut()?;
        order.complete();

        // 记录消费
        let actual_amount = order.actual_amount;
        let save_id = order.save_id;
        let order_id = order.id;

        self.vip_status.record_purchase(actual_amount);

        // 根据满意度调整好感度
        let affinity_delta = ((satisfaction - 50.0) / 10.0) as i32;
        self.update_affinity(affinity_delta);

        // 生成评价
        let rating = (satisfaction / 20.0).clamp(1.0, 5.0) as u32;
        let content = generate_review_content(rating);
        let review = Review::new(self.id, save_id, order_id, rating, content);

        self.reviews.push(review.clone());
        self.finished = true;

        Some(review)
    }

    /// 取消订单
    pub fn cancel_order(&mut self) {
        if let Some(order) = &mut self.current_order {
            order.cancel();
            self.update_affinity(-10); // 取消订单降低好感度
        }
        self.finished = true;
    }

    /// 是否有订单
    pub fn has_order(&self) -> bool {
        self.current_order.is_some()
    }

    /// 是否已完成用餐
    pub fn has_finished(&self) -> bool {
        self.finished
    }

    /// 获取等待时间（秒）
    pub fn wait_time(&self) -> i64 {
        (Utc::now() - self.arrived_at).num_seconds()
    }

    /// 获取平均评分
    pub fn average_rating(&self) -> f32 {
        if self.reviews.is_empty() {
            return 0.0;
        }

        let total: u32 = self.reviews.iter().map(|r| r.rating).sum();
        total as f32 / self.reviews.len() as f32
    }

    /// 是否为 VIP
    pub fn is_vip(&self) -> bool {
        self.vip_status.level != super::vip::VIPLevel::None
    }

    /// 是否为回头客
    pub fn is_returning(&self) -> bool {
        self.visit_count > 1
    }
}

/// 生成随机姓名
fn generate_random_name() -> String {
    let surnames = vec!["张", "李", "王", "刘", "陈", "杨", "黄", "赵", "吴", "周"];
    let names = [
        "小明", "小红", "小华", "小丽", "小强", "小芳", "小军", "小燕",
    ];

    let mut rng = rand::rng();
    let surname = surnames[rng.random_range(0..surnames.len())];
    let name = names[rng.random_range(0..names.len())];

    format!("{}{}", surname, name)
}

/// 根据评分生成评价内容
fn generate_review_content(rating: u32) -> String {
    match rating {
        1 => "很差，不会再来了。".to_string(),
        2 => "不太好，需要改进。".to_string(),
        3 => "还可以，中规中矩。".to_string(),
        4 => "很不错，值得推荐。".to_string(),
        5 => "非常棒，下次还会来！".to_string(),
        _ => "一般般。".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_creation() {
        let customer = Customer::new(1);

        assert_eq!(customer.id_num, 1);
        assert!(customer.name.starts_with("顾客#"));
        assert_eq!(customer.visit_count, 0);
    }

    #[test]
    fn test_customer_random() {
        let customer = Customer::random(2);

        assert_eq!(customer.id_num, 2);
        assert!(!customer.name.contains("顾客#"));
    }

    #[test]
    fn test_customer_affinity() {
        let mut customer = Customer::new(1);

        customer.update_affinity(50);
        assert_eq!(customer.affinity, 50);

        customer.update_affinity(-20);
        assert_eq!(customer.affinity, 30);

        // 不能低于 0
        customer.update_affinity(-100);
        assert_eq!(customer.affinity, 0);

        // 不能高于 1000
        customer.update_affinity(2000);
        assert_eq!(customer.affinity, 1000);
    }

    #[test]
    fn test_customer_order() {
        let save_id = Uuid::new_v4();
        let mut customer = Customer::new(1);

        assert!(!customer.has_order());

        customer.create_order(save_id);
        assert!(customer.has_order());
        assert_eq!(customer.visit_count, 1);
    }

    #[test]
    fn test_customer_complete_order() {
        let save_id = Uuid::new_v4();
        let mut customer = Customer::new(1);

        customer.create_order(save_id);
        let review = customer.complete_order(80.0);

        assert!(review.is_some());
        assert!(customer.has_finished());
        assert_eq!(customer.reviews.len(), 1);
        assert!(customer.affinity > 0);
    }

    #[test]
    fn test_customer_type() {
        let normal = CustomerType::Normal;
        assert_eq!(normal.tip_multiplier(), 1.0);

        let vip = CustomerType::VIP;
        assert_eq!(vip.tip_multiplier(), 1.5);
    }

    #[test]
    fn test_average_rating() {
        let save_id = Uuid::new_v4();
        let mut customer = Customer::new(1);

        // 没有评价时平均分为 0
        assert_eq!(customer.average_rating(), 0.0);

        // 创建订单并完成（自动生成评价）
        customer.create_order(save_id);
        customer.complete_order(80.0);

        assert!(customer.average_rating() > 0.0);
    }
}
