//! 餐厅订单系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OrderStatus {
    /// 待确认
    #[default]
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

/// 餐厅订单
#[derive(Debug, Clone)]
pub struct Order {
    /// 订单 ID
    pub id: i64,
    /// 餐厅 ID
    pub restaurant_id: i64,
    /// 顾客 ID
    pub customer_id: i64,
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
    pub fn new(restaurant_id: i64, customer_id: i64) -> Self {
        Self {
            id: 0,
            restaurant_id,
            customer_id,
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

    /// 是否待处理
    pub fn is_pending(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::Confirmed | OrderStatus::Cooking)
    }
}

/// 订单管理器
#[derive(Debug, Clone, Default)]
pub struct OrderManager {
    /// 待处理订单
    pub pending_orders: Vec<Order>,
    /// 已完成订单
    pub completed_orders: Vec<Order>,
    /// 下一个订单 ID
    pub next_order_id: i64,
}

impl OrderManager {
    /// 创建新的订单管理器
    pub fn new() -> Self {
        Self {
            pending_orders: Vec::new(),
            completed_orders: Vec::new(),
            next_order_id: 1,
        }
    }

    /// 创建新订单
    pub fn create_order(&mut self, restaurant_id: i64, customer_id: i64) -> Order {
        let order = Order {
            id: self.next_order_id,
            restaurant_id,
            customer_id,
            items: Vec::new(),
            status: OrderStatus::Pending,
            total_amount: 0,
            discount_amount: 0,
            actual_amount: 0,
            created_at: Utc::now(),
            completed_at: None,
        };
        self.next_order_id += 1;
        order
    }

    /// 添加订单到待处理队列
    pub fn add_order(&mut self, order: Order) {
        self.pending_orders.push(order);
    }

    /// 获取订单（通过 ID）
    pub fn get_order(&self, order_id: i64) -> Option<&Order> {
        self.pending_orders
            .iter()
            .find(|o| o.id == order_id)
            .or_else(|| self.completed_orders.iter().find(|o| o.id == order_id))
    }

    /// 获取订单（可变，通过 ID）
    pub fn get_order_mut(&mut self, order_id: i64) -> Option<&mut Order> {
        self.pending_orders
            .iter_mut()
            .find(|o| o.id == order_id)
            .or_else(|| self.completed_orders.iter_mut().find(|o| o.id == order_id))
    }

    /// 获取顾客的待处理订单
    pub fn get_customer_orders(&self, customer_id: i64) -> Vec<&Order> {
        self.pending_orders
            .iter()
            .filter(|o| o.customer_id == customer_id)
            .collect()
    }

    /// 完成订单（从待处理移到已完成）
    pub fn complete_order(&mut self, order_id: i64) -> Option<Order> {
        if let Some(pos) = self.pending_orders.iter().position(|o| o.id == order_id) {
            let mut order = self.pending_orders.remove(pos);
            order.complete();
            self.completed_orders.push(order.clone());
            return Some(order);
        }
        None
    }

    /// 取消订单
    pub fn cancel_order(&mut self, order_id: i64) -> Option<Order> {
        if let Some(pos) = self.pending_orders.iter().position(|o| o.id == order_id) {
            let mut order = self.pending_orders.remove(pos);
            order.cancel();
            self.completed_orders.push(order.clone());
            return Some(order);
        }
        None
    }

    /// 获取待处理订单数量
    pub fn pending_count(&self) -> usize {
        self.pending_orders.len()
    }

    /// 获取已完成订单数量
    pub fn completed_count(&self) -> usize {
        self.completed_orders.len()
    }

    /// 更新订单状态（每个 tick 调用）
    pub fn update(&mut self) {
        tracing::debug!(
            "OrderManager update: pending={}, completed={}",
            self.pending_count(),
            self.completed_count()
        );
    }
}
