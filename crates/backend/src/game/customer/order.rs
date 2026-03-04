//! 订单系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// 待确认
    Pending,
    /// 已确认
    Confirmed,
    /// 制作中
    Cooking,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

impl OrderStatus {
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            OrderStatus::Pending => "待确认",
            OrderStatus::Confirmed => "已确认",
            OrderStatus::Cooking => "制作中",
            OrderStatus::Completed => "已完成",
            OrderStatus::Cancelled => "已取消",
        }
    }
}

/// 订单项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    /// 菜品 ID
    pub dish_id: String,
    /// 菜品名称
    pub dish_name: String,
    /// 数量
    pub quantity: u32,
    /// 单价
    pub unit_price: u64,
    /// 总价
    pub total_price: u64,
}

impl OrderItem {
    /// 创建新的订单项
    pub fn new(dish_id: String, dish_name: String, quantity: u32, unit_price: u64) -> Self {
        Self {
            dish_id,
            dish_name,
            quantity,
            unit_price,
            total_price: unit_price * quantity as u64,
        }
    }
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单 ID
    pub id: Uuid,
    /// 顾客 ID
    pub customer_id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 订单项列表
    pub items: Vec<OrderItem>,
    /// 订单状态
    pub status: OrderStatus,
    /// 订单总价
    pub total_amount: u64,
    /// 折扣金额
    pub discount_amount: u64,
    /// 实际支付
    pub actual_amount: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
}

impl Order {
    /// 创建新订单
    pub fn new(customer_id: Uuid, save_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            save_id,
            items: Vec::new(),
            status: OrderStatus::Pending,
            total_amount: 0,
            discount_amount: 0,
            actual_amount: 0,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// 添加订单项
    pub fn add_item(&mut self, dish_id: String, dish_name: String, quantity: u32, unit_price: u64) {
        let item = OrderItem::new(dish_id, dish_name, quantity, unit_price);
        self.total_amount += item.total_price;
        self.items.push(item);
    }

    /// 应用折扣
    pub fn apply_discount(&mut self, discount_rate: u32) {
        self.discount_amount = (self.total_amount as f64 * discount_rate as f64 / 100.0) as u64;
        self.actual_amount = self.total_amount - self.discount_amount;
    }

    /// 确认订单
    pub fn confirm(&mut self) {
        self.status = OrderStatus::Confirmed;
    }

    /// 开始制作
    pub fn start_cooking(&mut self) {
        self.status = OrderStatus::Cooking;
    }

    /// 完成订单
    pub fn complete(&mut self) {
        self.status = OrderStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// 取消订单
    pub fn cancel(&mut self) {
        self.status = OrderStatus::Cancelled;
    }

    /// 获取订单项数量
    pub fn item_count(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    /// 计算订单等待时间（秒）
    pub fn wait_time(&self) -> Option<i64> {
        self.completed_at
            .map(|completed| (completed - self.created_at).num_seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let order = Order::new(customer_id, save_id);

        assert_eq!(order.customer_id, customer_id);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.items.len(), 0);
    }

    #[test]
    fn test_order_add_item() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let mut order = Order::new(customer_id, save_id);

        order.add_item("dish_1".to_string(), "番茄炒蛋".to_string(), 2, 25);
        assert_eq!(order.items.len(), 1);
        assert_eq!(order.total_amount, 50);
        assert_eq!(order.item_count(), 2);
    }

    #[test]
    fn test_order_discount() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let mut order = Order::new(customer_id, save_id);

        order.add_item("dish_1".to_string(), "菜品".to_string(), 1, 100);
        order.apply_discount(10); // 10% 折扣

        assert_eq!(order.discount_amount, 10);
        assert_eq!(order.actual_amount, 90);
    }

    #[test]
    fn test_order_status_flow() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let mut order = Order::new(customer_id, save_id);

        assert_eq!(order.status, OrderStatus::Pending);

        order.confirm();
        assert_eq!(order.status, OrderStatus::Confirmed);

        order.start_cooking();
        assert_eq!(order.status, OrderStatus::Cooking);

        order.complete();
        assert_eq!(order.status, OrderStatus::Completed);
        assert!(order.completed_at.is_some());
    }

    #[test]
    fn test_order_wait_time() {
        let customer_id = Uuid::new_v4();
        let save_id = Uuid::new_v4();
        let mut order = Order::new(customer_id, save_id);

        // 未完成时没有等待时间
        assert!(order.wait_time().is_none());

        order.complete();
        assert!(order.wait_time().is_some());
    }
}
