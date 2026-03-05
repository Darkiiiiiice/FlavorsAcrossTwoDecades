//! 小馆经营系统模块

mod facility;
mod finance;
mod inventory;
mod reputation;

pub use facility::{FacilityType, FacilityZone, SubFacility, UpgradePath, ZoneLevel};
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
    /// 设施集合
    pub facilities: Vec<SubFacility>,
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

        // 初始化所有设施
        let facilities = Self::create_initial_facilities();

        Self {
            save_id,
            name: "星夜小馆".to_string(),
            location: "地球·老街".to_string(),
            finance: Finance::new(),
            reputation: Reputation::new(),
            zones,
            facilities,
            inventory: Inventory::new(),
            opened_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_initial_facilities() -> Vec<SubFacility> {
        // 餐厅设施 (6个)
        vec![
            SubFacility::new(FacilityType::DiningTables),
            SubFacility::new(FacilityType::Lighting),
            SubFacility::new(FacilityType::Signboard),
            SubFacility::new(FacilityType::ClimateControl),
            SubFacility::new(FacilityType::CashierSystem),
            SubFacility::new(FacilityType::Decoration),
            // 厨房设施 (6个)
            SubFacility::new(FacilityType::Stove),
            SubFacility::new(FacilityType::OvenSteamer),
            SubFacility::new(FacilityType::Refrigerator),
            SubFacility::new(FacilityType::Cookware),
            SubFacility::new(FacilityType::Ventilation),
            SubFacility::new(FacilityType::Sink),
            // 后院设施 (5个)
            SubFacility::new(FacilityType::VegetablePatch),
            SubFacility::new(FacilityType::Irrigation),
            SubFacility::new(FacilityType::ToolShed),
            SubFacility::new(FacilityType::Greenhouse),
            SubFacility::new(FacilityType::CompostArea),
            // 工坊设施 (4个)
            SubFacility::new(FacilityType::Workbench),
            SubFacility::new(FacilityType::MaterialRack),
            SubFacility::new(FacilityType::RepairToolkit),
            SubFacility::new(FacilityType::PowerLighting),
        ]
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
        let cost = if let Some(zone_level) = self.get_zone(zone) {
            zone_level.get_upgrade_cost()
        } else {
            return Err("区域不存在".to_string());
        };

        if !self.can_afford(cost) {
            return Err("资金不足".to_string());
        }

        let new_level = {
            if let Some(zone_level) = self.get_zone_mut(zone) {
                zone_level.upgrade()?;
                zone_level.level
            } else {
                return Err("区域不存在".to_string());
            }
        };

        self.pay(cost)?;
        self.updated_at = Utc::now();
        Ok(new_level)
    }

    /// 计算每日支出
    pub fn calculate_daily_expenses(&self) -> u64 {
        let zone_cost: u64 = self.zones.iter().map(|z| z.level as u64 * 100).sum();
        let inventory_cost = self.inventory.maintenance_cost();
        zone_cost + inventory_cost
    }

    /// 结算每日收支
    pub fn settle_daily_accounts(&mut self) {
        let expenses = self.calculate_daily_expenses();
        self.finance.daily_expenses = expenses;
        self.finance.cash = self.finance.cash.saturating_sub(expenses);
        self.finance.daily_revenue = 0;
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

    /// 升级设施
    pub fn upgrade_facility(&mut self, facility_id: &str) -> Result<u32, String> {
        let (facility_type, current_level) = {
            let facility = self
                .facilities
                .iter()
                .find(|f| f.id == facility_id)
                .ok_or("设施不存在")?;
            (facility.facility_type, facility.level)
        };

        let upgrade_cost = UpgradePath::get_upgrade_path(facility_type, current_level)
            .map(|p| p.cost)
            .unwrap_or(500);

        if !self.can_afford(upgrade_cost) {
            return Err("资金不足".to_string());
        }

        self.pay(upgrade_cost)?;

        let new_level = {
            let facility = self
                .facilities
                .iter_mut()
                .find(|f| f.id == facility_id)
                .ok_or("设施不存在")?;
            facility.upgrade()?
        };

        self.updated_at = Utc::now();
        Ok(new_level)
    }

    /// 维修设施
    pub fn repair_facility(&mut self, facility_id: &str, repair_amount: u32) -> Result<(), String> {
        let facility_index = self
            .facilities
            .iter()
            .position(|f| f.id == facility_id)
            .ok_or("设施不存在")?;

        let repair_cost = repair_amount as u64 * 10;
        if !self.can_afford(repair_cost) {
            return Err("资金不足".to_string());
        }

        self.pay(repair_cost)?;
        self.facilities[facility_index].repair(repair_amount);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 获取所有功能正常的设施
    pub fn functional_facilities(&self) -> Vec<&SubFacility> {
        self.facilities.iter().filter(|f| f.is_functional).collect()
    }

    /// 获取指定区域的设施
    pub fn facilities_by_zone(&self, zone: FacilityZone) -> Vec<&SubFacility> {
        self.facilities.iter().filter(|f| f.zone == zone).collect()
    }

    /// 获取餐厅氛围评分
    pub fn atmosphere_score(&self) -> f32 {
        self.facilities
            .iter()
            .filter(|f| f.zone == FacilityZone::Restaurant && f.is_functional)
            .map(|f| f.effect.base_value)
            .sum()
    }

    /// 获取厨房烹饪加成
    pub fn kitchen_bonus(&self) -> f32 {
        self.facilities
            .iter()
            .filter(|f| f.zone == FacilityZone::Kitchen && f.is_functional)
            .map(|f| f.effect.base_value)
            .sum()
    }

    /// 获取最大顾客数
    pub fn max_customers(&self) -> i32 {
        self.facilities
            .iter()
            .filter(|f| f.facility_type == FacilityType::DiningTables && f.is_functional)
            .map(|f| f.effect.base_value as i32)
            .sum()
    }

    /// 获取食材保鲜时间加成
    pub fn freshness_bonus(&self) -> f32 {
        self.facilities
            .iter()
            .filter(|f| f.facility_type == FacilityType::Refrigerator && f.is_functional)
            .map(|f| f.effect.base_value)
            .sum()
    }

    /// 获取后院种植槽位数
    pub fn planting_slots(&self) -> i32 {
        self.facilities
            .iter()
            .filter(|f| f.zone == FacilityZone::Backyard && f.is_functional)
            .map(|f| f.effect.base_value as i32)
            .sum()
    }

    /// 获取工坊制作能力
    pub fn crafting_ability(&self) -> f32 {
        self.facilities
            .iter()
            .filter(|f| f.zone == FacilityZone::Workshop && f.is_functional)
            .map(|f| f.effect.base_value)
            .sum()
    }

    /// 获取存储容量
    pub fn storage_capacity(&self) -> i32 {
        self.facilities
            .iter()
            .filter(|f| {
                matches!(
                    f.facility_type,
                    FacilityType::StorageCabinet
                        | FacilityType::ToolShed
                        | FacilityType::MaterialRack
                ) && f.is_functional
            })
            .map(|f| f.effect.base_value as i32)
            .sum()
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
        assert_eq!(shop.facilities.len(), 21); // 6+6+5+4
        assert_eq!(shop.finance.cash, 10000);
    }

    #[test]
    fn test_zone_upgrade() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        let result = shop.upgrade_zone(FacilityZone::Restaurant);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert!(shop.finance.cash < 10000);
    }

    #[test]
    fn test_revenue_and_expenses() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        shop.add_revenue(500);
        assert_eq!(shop.finance.daily_revenue, 500);
        assert_eq!(shop.finance.total_revenue, 500);

        shop.settle_daily_accounts();
        assert_eq!(shop.finance.daily_revenue, 0);
        assert!(shop.finance.daily_expenses > 0);
    }

    #[test]
    fn test_payment() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        assert!(shop.can_afford(5000));
        assert!(shop.pay(5000).is_ok());
        assert_eq!(shop.finance.cash, 5000);

        assert!(!shop.can_afford(10000));
        assert!(shop.pay(10000).is_err());
    }

    #[test]
    fn test_facilities_initialization() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        // 检查餐厅设施
        let restaurant_facilities = shop.facilities_by_zone(FacilityZone::Restaurant);
        assert_eq!(restaurant_facilities.len(), 6);

        // 检查厨房设施
        let kitchen_facilities = shop.facilities_by_zone(FacilityZone::Kitchen);
        assert_eq!(kitchen_facilities.len(), 6);

        // 检查后院设施
        let backyard_facilities = shop.facilities_by_zone(FacilityZone::Backyard);
        assert_eq!(backyard_facilities.len(), 5);

        // 检查工坊设施
        let workshop_facilities = shop.facilities_by_zone(FacilityZone::Workshop);
        assert_eq!(workshop_facilities.len(), 4);
    }

    #[test]
    fn test_facility_upgrade() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        let facility_id = shop.facilities[0].id.clone();
        let result = shop.upgrade_facility(&facility_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_facility_repair() {
        let save_id = Uuid::new_v4();
        let mut shop = Shop::new(save_id);

        let facility_id = shop.facilities[0].id.clone();
        let initial_condition = shop.facilities[0].condition;

        let result = shop.repair_facility(&facility_id, 30);
        assert!(result.is_ok());
        assert_eq!(
            shop.facilities[0].condition,
            (initial_condition + 30).min(100)
        );
    }

    #[test]
    fn test_atmosphere_score() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        let score = shop.atmosphere_score();
        assert!(score > 0.0);
    }

    #[test]
    fn test_max_customers() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        let max = shop.max_customers();
        assert!(max > 0);
    }

    #[test]
    fn test_planting_slots() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        let slots = shop.planting_slots();
        assert!(slots > 0);
    }

    #[test]
    fn test_crafting_ability() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        let ability = shop.crafting_ability();
        assert!(ability > 0.0);
    }

    #[test]
    fn test_storage_capacity() {
        let save_id = Uuid::new_v4();
        let shop = Shop::new(save_id);

        let capacity = shop.storage_capacity();
        assert!(capacity > 0);
    }
}
