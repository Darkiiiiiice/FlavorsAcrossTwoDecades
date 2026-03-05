//! 顾客偏好系统

use rand::RngExt;
use serde::{Deserialize, Serialize};

/// 口味偏好
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlavorPreference {
    /// 清淡
    Light,
    /// 适中
    Medium,
    /// 重口味
    Heavy,
    /// 麻辣
    Spicy,
    /// 酸甜
    SweetSour,
}

impl FlavorPreference {
    /// 获取偏好名称
    pub fn name(&self) -> &str {
        match self {
            FlavorPreference::Light => "清淡",
            FlavorPreference::Medium => "适中",
            FlavorPreference::Heavy => "重口味",
            FlavorPreference::Spicy => "麻辣",
            FlavorPreference::SweetSour => "酸甜",
        }
    }

    /// 随机生成
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..5) {
            0 => FlavorPreference::Light,
            1 => FlavorPreference::Medium,
            2 => FlavorPreference::Heavy,
            3 => FlavorPreference::Spicy,
            _ => FlavorPreference::SweetSour,
        }
    }
}

/// 饮食限制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DietaryRestriction {
    /// 无限制
    None,
    /// 素食
    Vegetarian,
    /// 清真
    Halal,
    /// 无麸质
    GlutenFree,
    /// 低糖
    LowSugar,
}

impl DietaryRestriction {
    /// 获取限制名称
    pub fn name(&self) -> &str {
        match self {
            DietaryRestriction::None => "无限制",
            DietaryRestriction::Vegetarian => "素食",
            DietaryRestriction::Halal => "清真",
            DietaryRestriction::GlutenFree => "无麸质",
            DietaryRestriction::LowSugar => "低糖",
        }
    }

    /// 随机生成
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..10) {
            0 => DietaryRestriction::Vegetarian,
            1 => DietaryRestriction::Halal,
            2 => DietaryRestriction::GlutenFree,
            3 => DietaryRestriction::LowSugar,
            _ => DietaryRestriction::None,
        }
    }
}

/// 顾客偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    /// 口味偏好
    pub flavor: FlavorPreference,
    /// 饮食限制
    pub dietary: DietaryRestriction,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: u32,
    /// 耐心值 (0-100)
    pub patience: u32,
    /// 喜欢的菜品类型
    pub favorite_categories: Vec<String>,
}

impl Preference {
    /// 创建新的偏好
    pub fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            flavor: FlavorPreference::random(),
            dietary: DietaryRestriction::random(),
            price_sensitivity: rng.random_range(0..100),
            patience: 50 + rng.random_range(0..50),
            favorite_categories: Vec::new(),
        }
    }

    /// 随机生成偏好
    pub fn random() -> Self {
        let mut pref = Self::new();
        let mut rng = rand::rng();

        // 随机添加喜欢的菜品类型
        let categories = ["川菜", "粤菜", "湘菜", "鲁菜", "西餐", "日料"];
        let count = 1 + rng.random_range(0..3);

        for _ in 0..count {
            let idx = rng.random_range(0..categories.len());
            if let Some(cat) = categories.get(idx)
                && !pref.favorite_categories.contains(&cat.to_string())
            {
                pref.favorite_categories.push(cat.to_string());
            }
        }

        pref
    }

    /// 检查菜品是否符合偏好
    pub fn matches_dish(&self, category: &str, _spiciness: u32, price: u64) -> bool {
        // 检查饮食限制
        if self.dietary != DietaryRestriction::None {
            // 这里应该检查菜品是否符合饮食限制
            // 简化版本，总是返回 true
        }

        // 检查价格敏感度
        let max_price = (100 - self.price_sensitivity) as u64 * 10;
        if price > max_price {
            return false;
        }

        // 检查菜品类型
        if !self.favorite_categories.is_empty()
            && !self.favorite_categories.contains(&category.to_string())
        {
            return false;
        }

        true
    }

    /// 计算满意度
    pub fn calculate_satisfaction(&self, dish_quality: f32, wait_time: u32, price: u64) -> f32 {
        let quality_score = dish_quality / 100.0 * 50.0;

        // 等待时间影响（基础耐心为 50-100）
        let wait_penalty = if wait_time > self.patience {
            (wait_time - self.patience) as f32 * 0.5
        } else {
            0.0
        };

        // 价格影响
        let price_score = if price > (100 - self.price_sensitivity) as u64 * 10 {
            -10.0
        } else {
            10.0
        };

        let total = quality_score + price_score - wait_penalty;
        total.clamp(0.0, 100.0)
    }
}

impl Default for Preference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_creation() {
        let pref = Preference::new();
        assert!(pref.price_sensitivity <= 100);
        assert!(pref.patience >= 50);
    }

    #[test]
    fn test_flavor_preference() {
        let flavor = FlavorPreference::random();
        assert!(!flavor.name().is_empty());
    }

    #[test]
    fn test_dietary_restriction() {
        let dietary = DietaryRestriction::random();
        assert!(!dietary.name().is_empty());
    }

    #[test]
    fn test_satisfaction_calculation() {
        let pref = Preference::new();

        let satisfaction = pref.calculate_satisfaction(80.0, 30, 50);
        assert!(satisfaction > 0.0);
        assert!(satisfaction <= 100.0);
    }
}
