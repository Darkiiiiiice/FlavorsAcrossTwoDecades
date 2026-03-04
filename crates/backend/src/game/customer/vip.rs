//! VIP 系统

use serde::{Deserialize, Serialize};

/// VIP 等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VIPLevel {
    /// 普通顾客
    None,
    /// 铜牌会员
    Bronze,
    /// 银牌会员
    Silver,
    /// 金牌会员
    Gold,
    /// 钻石会员
    Diamond,
}

impl VIPLevel {
    /// 获取等级名称
    pub fn name(&self) -> &str {
        match self {
            VIPLevel::None => "普通",
            VIPLevel::Bronze => "铜牌",
            VIPLevel::Silver => "银牌",
            VIPLevel::Gold => "金牌",
            VIPLevel::Diamond => "钻石",
        }
    }

    /// 获取所需好感度
    pub fn required_affinity(&self) -> u32 {
        match self {
            VIPLevel::None => 0,
            VIPLevel::Bronze => 100,
            VIPLevel::Silver => 300,
            VIPLevel::Gold => 600,
            VIPLevel::Diamond => 1000,
        }
    }

    /// 获取折扣率（0-100）
    pub fn discount_rate(&self) -> u32 {
        match self {
            VIPLevel::None => 0,
            VIPLevel::Bronze => 5,
            VIPLevel::Silver => 10,
            VIPLevel::Gold => 15,
            VIPLevel::Diamond => 20,
        }
    }

    /// 获取积分倍率
    pub fn points_multiplier(&self) -> f32 {
        match self {
            VIPLevel::None => 1.0,
            VIPLevel::Bronze => 1.2,
            VIPLevel::Silver => 1.5,
            VIPLevel::Gold => 2.0,
            VIPLevel::Diamond => 3.0,
        }
    }

    /// 根据好感度确定等级
    pub fn from_affinity(affinity: u32) -> Self {
        if affinity >= 1000 {
            VIPLevel::Diamond
        } else if affinity >= 600 {
            VIPLevel::Gold
        } else if affinity >= 300 {
            VIPLevel::Silver
        } else if affinity >= 100 {
            VIPLevel::Bronze
        } else {
            VIPLevel::None
        }
    }

    /// 获取下一等级
    pub fn next_level(&self) -> Option<Self> {
        match self {
            VIPLevel::None => Some(VIPLevel::Bronze),
            VIPLevel::Bronze => Some(VIPLevel::Silver),
            VIPLevel::Silver => Some(VIPLevel::Gold),
            VIPLevel::Gold => Some(VIPLevel::Diamond),
            VIPLevel::Diamond => None,
        }
    }

    /// 获取升级所需好感度
    pub fn affinity_to_next_level(&self) -> Option<u32> {
        self.next_level().map(|next| next.required_affinity())
    }
}

/// VIP 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VIPStatus {
    /// VIP 等级
    pub level: VIPLevel,
    /// 累计积分
    pub total_points: u32,
    /// 当前积分
    pub current_points: u32,
    /// 累计消费
    pub total_spent: u64,
    /// 访问次数
    pub visit_count: u32,
}

impl VIPStatus {
    /// 创建新的 VIP 状态
    pub fn new() -> Self {
        Self {
            level: VIPLevel::None,
            total_points: 0,
            current_points: 0,
            total_spent: 0,
            visit_count: 0,
        }
    }

    /// 添加积分
    pub fn add_points(&mut self, points: u32) {
        let actual_points = (points as f32 * self.level.points_multiplier()) as u32;
        self.total_points += actual_points;
        self.current_points += actual_points;
    }

    /// 消费积分
    pub fn spend_points(&mut self, points: u32) -> Result<(), String> {
        if self.current_points < points {
            return Err("积分不足".to_string());
        }
        self.current_points -= points;
        Ok(())
    }

    /// 记录消费
    pub fn record_purchase(&mut self, amount: u64) {
        self.total_spent += amount;
        self.visit_count += 1;

        // 消费获得积分（1元=1积分）
        self.add_points(amount as u32);
    }

    /// 更新 VIP 等级
    pub fn update_level(&mut self, affinity: u32) {
        self.level = VIPLevel::from_affinity(affinity);
    }

    /// 获取折扣金额
    pub fn calculate_discount(&self, original_price: u64) -> u64 {
        let discount = self.level.discount_rate() as f64 / 100.0;
        (original_price as f64 * discount) as u64
    }

    /// 获取折扣后价格
    pub fn apply_discount(&self, original_price: u64) -> u64 {
        original_price - self.calculate_discount(original_price)
    }

    /// 检查是否可以升级
    pub fn can_upgrade(&self, affinity: u32) -> bool {
        if let Some(next) = self.level.next_level() {
            affinity >= next.required_affinity()
        } else {
            false
        }
    }
}

impl Default for VIPStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vip_level_from_affinity() {
        assert_eq!(VIPLevel::from_affinity(0), VIPLevel::None);
        assert_eq!(VIPLevel::from_affinity(100), VIPLevel::Bronze);
        assert_eq!(VIPLevel::from_affinity(300), VIPLevel::Silver);
        assert_eq!(VIPLevel::from_affinity(600), VIPLevel::Gold);
        assert_eq!(VIPLevel::from_affinity(1000), VIPLevel::Diamond);
    }

    #[test]
    fn test_vip_discount() {
        assert_eq!(VIPLevel::None.discount_rate(), 0);
        assert_eq!(VIPLevel::Bronze.discount_rate(), 5);
        assert_eq!(VIPLevel::Silver.discount_rate(), 10);
        assert_eq!(VIPLevel::Gold.discount_rate(), 15);
        assert_eq!(VIPLevel::Diamond.discount_rate(), 20);
    }

    #[test]
    fn test_vip_points_multiplier() {
        assert_eq!(VIPLevel::None.points_multiplier(), 1.0);
        assert_eq!(VIPLevel::Diamond.points_multiplier(), 3.0);
    }

    #[test]
    fn test_vip_status() {
        let mut status = VIPStatus::new();

        status.record_purchase(100);
        assert_eq!(status.total_points, 100);
        assert_eq!(status.visit_count, 1);
    }

    #[test]
    fn test_vip_discount_calculation() {
        let mut status = VIPStatus::new();
        status.level = VIPLevel::Gold;

        let original_price = 100;
        let discounted = status.apply_discount(original_price);

        assert_eq!(discounted, 85); // 100 - 15% = 85
    }
}
