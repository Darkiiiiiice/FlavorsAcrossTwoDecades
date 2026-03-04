//! 顾客系统模块

mod customer;
mod order;
mod preference;
mod review;
mod vip;

pub use customer::{Customer, CustomerType};
pub use order::{Order, OrderItem, OrderStatus};
pub use preference::{DietaryRestriction, FlavorPreference, Preference};
pub use review::{Review, ReviewSentiment};
pub use vip::{VIPLevel, VIPStatus};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 顾客管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerManager {
    /// 存档 ID
    pub save_id: Uuid,
    /// 活跃顾客列表
    pub active_customers: Vec<Customer>,
    /// 历史顾客记录
    pub customer_history: Vec<Uuid>,
    /// VIP 顾客列表
    pub vip_customers: Vec<Uuid>,
    /// 下一个顾客 ID
    pub next_customer_id: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl CustomerManager {
    /// 创建新的顾客管理器
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            active_customers: Vec::new(),
            customer_history: Vec::new(),
            vip_customers: Vec::new(),
            next_customer_id: 1,
            updated_at: Utc::now(),
        }
    }

    /// 生成新顾客
    pub fn generate_customer(&mut self) -> Customer {
        let customer = Customer::random(self.next_customer_id);
        self.next_customer_id += 1;
        self.updated_at = Utc::now();
        customer
    }

    /// 添加顾客到小馆
    pub fn add_customer(&mut self, customer: Customer) {
        let customer_id = customer.id;
        self.active_customers.push(customer);
        self.updated_at = Utc::now();

        // 记录到历史
        if !self.customer_history.contains(&customer_id) {
            self.customer_history.push(customer_id);
        }
    }

    /// 移除顾客
    pub fn remove_customer(&mut self, customer_id: Uuid) -> Option<Customer> {
        if let Some(pos) = self.active_customers.iter().position(|c| c.id == customer_id) {
            let customer = self.active_customers.remove(pos);
            self.updated_at = Utc::now();
            return Some(customer);
        }
        None
    }

    /// 获取顾客
    pub fn get_customer(&self, customer_id: Uuid) -> Option<&Customer> {
        self.active_customers.iter().find(|c| c.id == customer_id)
    }

    /// 获取顾客（可变）
    pub fn get_customer_mut(&mut self, customer_id: Uuid) -> Option<&mut Customer> {
        self.active_customers.iter_mut().find(|c| c.id == customer_id)
    }

    /// 更新顾客好感度
    pub fn update_affinity(&mut self, customer_id: Uuid, delta: i32) -> Result<(), String> {
        if let Some(customer) = self.get_customer_mut(customer_id) {
            customer.update_affinity(delta);

            // 如果是 VIP 顾客，记录
            if customer.vip_status.level != VIPLevel::None {
                if !self.vip_customers.contains(&customer_id) {
                    self.vip_customers.push(customer_id);
                }
            }

            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("顾客不存在".to_string())
        }
    }

    /// 获取活跃顾客数量
    pub fn active_count(&self) -> usize {
        self.active_customers.len()
    }

    /// 获取 VIP 顾客数量
    pub fn vip_count(&self) -> usize {
        self.vip_customers.len()
    }

    /// 清理已完成用餐的顾客
    pub fn cleanup_finished(&mut self) -> Vec<Customer> {
        let finished: Vec<Customer> = self
            .active_customers
            .iter()
            .filter(|c| c.has_finished())
            .cloned()
            .collect();

        for customer in &finished {
            self.remove_customer(customer.id);
        }

        if !finished.is_empty() {
            self.updated_at = Utc::now();
        }

        finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_manager_creation() {
        let save_id = Uuid::new_v4();
        let manager = CustomerManager::new(save_id);

        assert_eq!(manager.active_count(), 0);
        assert_eq!(manager.next_customer_id, 1);
    }

    #[test]
    fn test_generate_customer() {
        let save_id = Uuid::new_v4();
        let mut manager = CustomerManager::new(save_id);

        let customer = manager.generate_customer();
        assert_eq!(customer.id_num, 1);
        assert_eq!(manager.next_customer_id, 2);
    }

    #[test]
    fn test_add_and_remove_customer() {
        let save_id = Uuid::new_v4();
        let mut manager = CustomerManager::new(save_id);

        let customer = manager.generate_customer();
        let customer_id = customer.id;

        manager.add_customer(customer);
        assert_eq!(manager.active_count(), 1);

        let removed = manager.remove_customer(customer_id);
        assert!(removed.is_some());
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_update_affinity() {
        let save_id = Uuid::new_v4();
        let mut manager = CustomerManager::new(save_id);

        let customer = manager.generate_customer();
        let customer_id = customer.id;
        let initial_affinity = customer.affinity;

        manager.add_customer(customer);
        manager.update_affinity(customer_id, 10).unwrap();

        let updated = manager.get_customer(customer_id).unwrap();
        assert_eq!(updated.affinity, initial_affinity + 10);
    }
}
