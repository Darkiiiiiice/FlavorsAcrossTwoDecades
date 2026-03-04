//! 小馆经营系统模块

mod facility;
mod finance;
mod inventory;
mod reputation;

pub use facility::{FacilityZone, ZoneLevel};
pub use finance::Finance;
pub use inventory::{Inventory, InventoryItem};
pub use reputation::Reputation;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 小馆完整状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    /// 存档 ID
    pub save_id: Uuid,
    /// 小馆名称
    pub name: String,
    /// 小馆位置
    pub location: String,
    /// 资金系统
    pub finance: Finance,
    /// 口碑系统
    pub reputation: Reputation,
    /// 区域等级
    pub zones: Vec<ZoneLevel>,
    /// 库存
    pub inventory: Inventory,
    /// 开业时间
    pub opened_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl Shop {
    /// 创建新的小馆
    pub fn new(save_id: Uuid) -> Self {
        let zones = vec![
            ZoneLevel::new(FacilityZone::Restaurant),
            ZoneLevel::new(FacilityZone::Kitchen),
            ZoneLevel::new(FacilityZone::Backyard),
            ZoneLevel::new(FacilityZone::Workshop),
        ];

        Self {
            save_id,
            name: "星夜小馆".to_string(),
            location: "地球·老街".to_string(),
            finance: Finance::new(),
            reputation: Reputation::new(),
            zones,
            inventory: Inventory::new(),
            opened_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 获取指定区域
    pub fn get_zone(&self, zone: FacilityZone) -> Option<&ZoneLevel> {
        self.zones.iter().find(|z| z.zone == zone)
    }

    /// 获取指定区域（可变）
    pub fn get_zone_mut(&mut self, zone: FacilityZone) -> Option<&mut ZoneLevel> {
        self.zones.iter_mut().find(|z| z.zone == zone)
    }

    /// 升级区域
    pub fn upgrade_zone(&mut self, zone: FacilityZone) -> Result<u32, String> {
        // 先计算升级成本
        let cost = if let Some(zone_level) = self.get_zone(zone) {
            zone_level.get_upgrade_cost()
        } else {
            return Err("区域不存在".to_string());
        };

        // 检查资金是否足够
        if !self.can_afford(cost) {
            return Err("资金不足".to_string());
        }

        // 执行升级
        let new_level = {
            if let Some(zone_level) = self.get_zone_mut(zone) {
                zone_level.upgrade()?;
                zone_level.level
            } else {
                return Err("区域不存在".to_string());
            }
        };

        // 扣除费用
        self.pay(cost)?;

        self.updated_at = Utc::now();
        Ok(new_level)
    }

    /// 计算每日支出
    pub fn calculate_daily_expenses(&self) -> u64 {
        // 基础支出：每个区域等级 × 100
        let zone_cost: u64 = self.zones.iter().map(|z| z.level as u64 * 100).sum();

        // 库存维护成本
        let inventory_cost = self.inventory.maintenance_cost();

        zone_cost + inventory_cost
    }

    /// 结算每日收支
    pub fn settle_daily_accounts(&mut self) {
        let expenses = self.calculate_daily_expenses();
        self.finance.daily_expenses = expenses;
        self.finance.cash = self.finance.cash.saturating_sub(expenses);
        self.finance.daily_revenue = 0; // 重置每日收入
        self.updated_at = Utc::now();
    }

    /// 添加收入
    pub fn add_revenue(&mut self, amount: u64) {
        self.finance.add_revenue(amount);
        self.updated_at = Utc::now();
    }

    /// 更新口碑
    pub fn update_reputation(&mut self) {
        self.reputation.update_score(&self.zones);
        self.updated_at = Utc::now();
    }

    /// 检查是否可以支付
    pub fn can_afford(&self, amount: u64) -> bool {
        self.finance.cash >= amount
    }

    /// 支付费用
    pub fn pay(&mut self, amount: u64) -> Result<(), String> {
        if !self.can_afford(amount) {
            return Err("资金不足".to_string());
        }
        self.finance.cash -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shop_creation() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        assert_eq!(shop.name, "星夜小馆");
        assert_eq!(shop.location, "地球·老街");
        assert_eq!(shop.zones.len(), 4);
        assert_eq!(shop.finance.cash, 10000);
    }

    #[test]
    fn test_zone_upgrade() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        // 升级餐厅
        let result = shop.upgrade_zone(FacilityZone::Restaurant);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        // 检查升级成本
        assert!(shop.finance.cash < 10000);
    }

    #[test]
    fn test_revenue_and_expenses() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        // 添加收入
        shop.add_revenue(500);
        assert_eq!(shop.finance.daily_revenue, 500);
        assert_eq!(shop.finance.total_revenue, 500);

        // 结算
        shop.settle_daily_accounts();
        assert_eq!(shop.finance.daily_revenue, 0);
        assert!(shop.finance.daily_expenses > 0);
    }

    #[test]
    fn test_payment() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        // 可以支付
        assert!(shop.can_afford(5000));
        assert!(shop.pay(5000).is_ok());
        assert_eq!(shop.finance.cash, 5000);

        // 不能支付
        assert!(!shop.can_afford(10000));
        assert!(shop.pay(10000).is_err());
    }
}
