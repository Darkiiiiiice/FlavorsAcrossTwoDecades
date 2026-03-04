//! 库存系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 库存项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    /// 物品 ID
    pub item_id: String,
    /// 物品名称
    pub name: String,
    /// 数量
    pub quantity: u32,
    /// 品质 (0-1)
    pub quality: f32,
    /// 新鲜度 (0-1)
    pub freshness: f32,
    /// 过期时间
    pub expiry_date: Option<DateTime<Utc>>,
    /// 单价
    pub unit_price: u64,
}

impl InventoryItem {
    /// 创建新的库存项
    pub fn new(item_id: String, name: String, quantity: u32, unit_price: u64) -> Self {
        Self {
            item_id,
            name,
            quantity,
            quality: 1.0,
            freshness: 1.0,
            expiry_date: None,
            unit_price,
        }
    }

    /// 添加数量
    pub fn add(&mut self, amount: u32) {
        self.quantity += amount;
    }

    /// 减少数量
    pub fn reduce(&mut self, amount: u32) -> Result<u32, String> {
        if self.quantity < amount {
            return Err("数量不足".to_string());
        }
        self.quantity -= amount;
        Ok(self.quantity)
    }

    /// 更新鲜度
    pub fn update_freshness(&mut self, decay_rate: f32) {
        self.freshness = (self.freshness - decay_rate).max(0.0);
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry_date {
            return Utc::now() > expiry;
        }
        self.freshness <= 0.0
    }

    /// 计算总价值
    pub fn total_value(&self) -> u64 {
        (self.quantity as u64) * self.unit_price
    }
}

/// 库存管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// 物品列表
    items: HashMap<String, InventoryItem>,
    /// 最大容量
    pub max_capacity: u32,
    /// 当前容量
    pub current_capacity: u32,
}

impl Inventory {
    /// 创建新的库存
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            max_capacity: 100,
            current_capacity: 0,
        }
    }

    /// 添加物品
    pub fn add_item(&mut self, item: InventoryItem) -> Result<(), String> {
        let total = self.current_capacity + item.quantity;
        if total > self.max_capacity {
            return Err("库存容量不足".to_string());
        }

        let item_id = item.item_id.clone();
        if let Some(existing) = self.items.get_mut(&item_id) {
            existing.add(item.quantity);
        } else {
            self.items.insert(item_id, item);
        }

        self.current_capacity = total;
        Ok(())
    }

    /// 移除物品
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> Result<(), String> {
        if let Some(item) = self.items.get_mut(item_id) {
            item.reduce(quantity)?;
            self.current_capacity -= quantity;

            if item.quantity == 0 {
                self.items.remove(item_id);
            }
            Ok(())
        } else {
            Err("物品不存在".to_string())
        }
    }

    /// 获取物品
    pub fn get_item(&self, item_id: &str) -> Option<&InventoryItem> {
        self.items.get(item_id)
    }

    /// 获取物品（可变）
    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut InventoryItem> {
        self.items.get_mut(item_id)
    }

    /// 检查是否有足够的物品
    pub fn has_item(&self, item_id: &str, quantity: u32) -> bool {
        if let Some(item) = self.items.get(item_id) {
            item.quantity >= quantity
        } else {
            false
        }
    }

    /// 更新所有物品的新鲜度
    pub fn update_freshness(&mut self, decay_rate: f32) {
        for item in self.items.values_mut() {
            item.update_freshness(decay_rate);
        }
    }

    /// 移除过期物品
    pub fn remove_expired(&mut self) -> Vec<InventoryItem> {
        let expired_ids: Vec<String> = self
            .items
            .iter()
            .filter(|(_, item)| item.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        let expired_items: Vec<InventoryItem> = expired_ids
            .iter()
            .filter_map(|id| {
                let item = self.items.remove(id)?;
                self.current_capacity -= item.quantity;
                Some(item)
            })
            .collect();

        expired_items
    }

    /// 计算库存维护成本
    pub fn maintenance_cost(&self) -> u64 {
        let total_value: u64 = self.items.values().map(|item| item.total_value()).sum();
        (total_value as f64 * 0.01) as u64 // 1% 的维护成本
    }

    /// 获取所有物品
    pub fn all_items(&self) -> Vec<&InventoryItem> {
        self.items.values().collect()
    }

    /// 清空库存
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_capacity = 0;
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_item() {
        let mut item = InventoryItem::new(
            "tomato".to_string(),
            "番茄".to_string(),
            10,
            5,
        );

        assert_eq!(item.quantity, 10);
        item.add(5);
        assert_eq!(item.quantity, 15);
        item.reduce(3).unwrap();
        assert_eq!(item.quantity, 12);
    }

    #[test]
    fn test_inventory_add() {
        let mut inventory = Inventory::new();

        let item = InventoryItem::new(
            "tomato".to_string(),
            "番茄".to_string(),
            10,
            5,
        );

        assert!(inventory.add_item(item).is_ok());
        assert_eq!(inventory.current_capacity, 10);
    }

    #[test]
    fn test_inventory_remove() {
        let mut inventory = Inventory::new();

        let item = InventoryItem::new(
            "tomato".to_string(),
            "番茄".to_string(),
            10,
            5,
        );
        inventory.add_item(item).unwrap();

        assert!(inventory.remove_item("tomato", 5).is_ok());
        assert_eq!(inventory.current_capacity, 5);
        assert_eq!(inventory.get_item("tomato").unwrap().quantity, 5);
    }

    #[test]
    fn test_inventory_capacity() {
        let mut inventory = Inventory::new();
        inventory.max_capacity = 20;

        let item1 = InventoryItem::new(
            "tomato".to_string(),
            "番茄".to_string(),
            15,
            5,
        );
        assert!(inventory.add_item(item1).is_ok());

        let item2 = InventoryItem::new(
            "potato".to_string(),
            "土豆".to_string(),
            10,
            3,
        );
        assert!(inventory.add_item(item2).is_err()); // 超过容量
    }

    #[test]
    fn test_expired_items() {
        let mut inventory = Inventory::new();

        let mut expired_item = InventoryItem::new(
            "tomato".to_string(),
            "番茄".to_string(),
            10,
            5,
        );
        expired_item.freshness = 0.0;

        inventory.add_item(expired_item).unwrap();

        let expired = inventory.remove_expired();
        assert_eq!(expired.len(), 1);
        assert_eq!(inventory.current_capacity, 0);
    }
}
