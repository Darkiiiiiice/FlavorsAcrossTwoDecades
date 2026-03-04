//! 食材定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 食材分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngredientCategory {
    /// 蔬菜
    Vegetable,
    /// 肉类
    Meat,
    /// 海鲜
    Seafood,
    /// 香料
    Herb,
    /// 调味料
    Seasoning,
    /// 主食
    Staple,
    /// 豆制品
    SoyProduct,
    /// 蛋奶
    EggDairy,
    /// 水果
    Fruit,
    /// 特殊食材
    Special,
}

impl IngredientCategory {
    /// 获取分类名称
    pub fn name(&self) -> &str {
        match self {
            IngredientCategory::Vegetable => "蔬菜",
            IngredientCategory::Meat => "肉类",
            IngredientCategory::Seafood => "海鲜",
            IngredientCategory::Herb => "香料",
            IngredientCategory::Seasoning => "调味料",
            IngredientCategory::Staple => "主食",
            IngredientCategory::SoyProduct => "豆制品",
            IngredientCategory::EggDairy => "蛋奶",
            IngredientCategory::Fruit => "水果",
            IngredientCategory::Special => "特殊食材",
        }
    }
}

/// 食材品质
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IngredientQuality {
    /// 劣质 (0.3)
    Poor,
    /// 普通 (0.6)
    Normal,
    /// 优质 (0.8)
    Good,
    /// 极品 (1.0)
    Excellent,
}

impl IngredientQuality {
    /// 获取品质名称
    pub fn name(&self) -> &str {
        match self {
            IngredientQuality::Poor => "劣质",
            IngredientQuality::Normal => "普通",
            IngredientQuality::Good => "优质",
            IngredientQuality::Excellent => "极品",
        }
    }

    /// 获取品质系数
    pub fn multiplier(&self) -> f32 {
        match self {
            IngredientQuality::Poor => 0.3,
            IngredientQuality::Normal => 0.6,
            IngredientQuality::Good => 0.8,
            IngredientQuality::Excellent => 1.0,
        }
    }

    /// 从数值创建品质
    pub fn from_value(value: f32) -> Self {
        if value >= 0.9 {
            IngredientQuality::Excellent
        } else if value >= 0.7 {
            IngredientQuality::Good
        } else if value >= 0.4 {
            IngredientQuality::Normal
        } else {
            IngredientQuality::Poor
        }
    }
}

/// 食材
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    /// 唯一 ID
    pub id: Uuid,
    /// 食材 ID（模板 ID）
    pub ingredient_id: String,
    /// 食材名称
    pub name: String,
    /// 分类
    pub category: IngredientCategory,
    /// 数量
    pub quantity: u32,
    /// 单位
    pub unit: String,
    /// 品质 (0-1)
    pub quality: f32,
    /// 新鲜度 (0-1)
    pub freshness: f32,
    /// 获取时间
    pub obtained_at: DateTime<Utc>,
    /// 过期时间
    pub expiry_date: Option<DateTime<Utc>>,
    /// 来源描述
    pub source: String,
    /// 单价
    pub unit_price: u64,
}

impl Ingredient {
    /// 创建新食材
    pub fn new(
        ingredient_id: String,
        name: String,
        category: IngredientCategory,
        quantity: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            ingredient_id,
            name,
            category,
            quantity,
            unit: "份".to_string(),
            quality: 0.6,
            freshness: 1.0,
            obtained_at: Utc::now(),
            expiry_date: None,
            source: "购买".to_string(),
            unit_price: 10,
        }
    }

    /// 创建高品质食材
    pub fn high_quality(
        ingredient_id: String,
        name: String,
        category: IngredientCategory,
        quantity: u32,
    ) -> Self {
        let mut ingredient = Self::new(ingredient_id, name, category, quantity);
        ingredient.quality = 0.9;
        ingredient.source = "精选".to_string();
        ingredient
    }

    /// 创建特殊食材
    pub fn special(
        ingredient_id: String,
        name: String,
        category: IngredientCategory,
        quantity: u32,
        source: String,
    ) -> Self {
        let mut ingredient = Self::new(ingredient_id, name, category, quantity);
        ingredient.category = IngredientCategory::Special;
        ingredient.quality = 1.0;
        ingredient.source = source;
        ingredient
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, hours: u32) -> Self {
        self.expiry_date = Some(self.obtained_at + chrono::Duration::hours(hours as i64));
        self
    }

    /// 更新鲜度
    pub fn update_freshness(&mut self, now: DateTime<Utc>) {
        if let Some(expiry) = self.expiry_date {
            let total_duration = (expiry - self.obtained_at).num_seconds() as f32;
            let elapsed = (now - self.obtained_at).num_seconds() as f32;

            if elapsed >= total_duration {
                self.freshness = 0.0;
            } else {
                self.freshness = 1.0 - (elapsed / total_duration);
            }
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        self.freshness <= 0.0
    }

    /// 计算有效品质（品质 * 新鲜度）
    pub fn effective_quality(&self) -> f32 {
        self.quality * self.freshness
    }

    /// 获取品质等级
    pub fn quality_level(&self) -> IngredientQuality {
        IngredientQuality::from_value(self.effective_quality())
    }

    /// 分割食材
    pub fn split(&mut self, amount: u32) -> Option<Self> {
        if amount >= self.quantity {
            return None;
        }

        self.quantity -= amount;

        Some(Self {
            id: Uuid::new_v4(),
            ingredient_id: self.ingredient_id.clone(),
            name: self.name.clone(),
            category: self.category,
            quantity: amount,
            unit: self.unit.clone(),
            quality: self.quality,
            freshness: self.freshness,
            obtained_at: self.obtained_at,
            expiry_date: self.expiry_date,
            source: self.source.clone(),
            unit_price: self.unit_price,
        })
    }

    /// 计算总价值
    pub fn total_value(&self) -> u64 {
        (self.unit_price as f32 * self.quantity as f32 * self.effective_quality()) as u64
    }
}

/// 食材模板（用于定义食材类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientTemplate {
    /// 食材 ID
    pub ingredient_id: String,
    /// 名称
    pub name: String,
    /// 分类
    pub category: IngredientCategory,
    /// 默认单位
    pub default_unit: String,
    /// 默认单价
    pub default_price: u64,
    /// 默认保质期（小时）
    pub default_shelf_life: u32,
    /// 描述
    pub description: String,
    /// 适配的菜系
    pub compatible_cuisines: Vec<String>,
}

impl IngredientTemplate {
    /// 创建食材实例
    pub fn create_instance(&self, quantity: u32) -> Ingredient {
        let mut ingredient = Ingredient::new(
            self.ingredient_id.clone(),
            self.name.clone(),
            self.category,
            quantity,
        );
        ingredient.unit = self.default_unit.clone();
        ingredient.unit_price = self.default_price;

        if self.default_shelf_life > 0 {
            ingredient = ingredient.with_expiry(self.default_shelf_life);
        }

        ingredient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingredient_creation() {
        let ingredient = Ingredient::new(
            "tomato".to_string(),
            "番茄".to_string(),
            IngredientCategory::Vegetable,
            10,
        );

        assert_eq!(ingredient.name, "番茄");
        assert_eq!(ingredient.quantity, 10);
        assert_eq!(ingredient.category, IngredientCategory::Vegetable);
    }

    #[test]
    fn test_high_quality_ingredient() {
        let ingredient = Ingredient::high_quality(
            "wagyu".to_string(),
            "和牛".to_string(),
            IngredientCategory::Meat,
            5,
        );

        assert_eq!(ingredient.quality, 0.9);
        assert_eq!(ingredient.source, "精选");
    }

    #[test]
    fn test_ingredient_expiry() {
        let ingredient = Ingredient::new(
            "fish".to_string(),
            "鱼".to_string(),
            IngredientCategory::Seafood,
            1,
        )
        .with_expiry(24);

        assert!(ingredient.expiry_date.is_some());
    }

    #[test]
    fn test_effective_quality() {
        let mut ingredient = Ingredient::new(
            "test".to_string(),
            "测试".to_string(),
            IngredientCategory::Vegetable,
            1,
        );
        ingredient.quality = 0.8;
        ingredient.freshness = 0.5;

        let effective = ingredient.effective_quality();
        assert!((effective - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_split_ingredient() {
        let mut ingredient = Ingredient::new(
            "test".to_string(),
            "测试".to_string(),
            IngredientCategory::Vegetable,
            10,
        );

        let split = ingredient.split(3);
        assert!(split.is_some());
        assert_eq!(ingredient.quantity, 7);
        assert_eq!(split.unwrap().quantity, 3);
    }

    #[test]
    fn test_cannot_split_more_than_available() {
        let mut ingredient = Ingredient::new(
            "test".to_string(),
            "测试".to_string(),
            IngredientCategory::Vegetable,
            5,
        );

        let split = ingredient.split(10);
        assert!(split.is_none());
        assert_eq!(ingredient.quantity, 5);
    }

    #[test]
    fn test_total_value() {
        let ingredient = Ingredient::new(
            "test".to_string(),
            "测试".to_string(),
            IngredientCategory::Vegetable,
            10,
        );

        // 默认品质 0.6，新鲜度 1.0，单价 10，数量 10
        let value = ingredient.total_value();
        assert_eq!(value, 60);
    }

    #[test]
    fn test_ingredient_template() {
        let template = IngredientTemplate {
            ingredient_id: "tomato".to_string(),
            name: "番茄".to_string(),
            category: IngredientCategory::Vegetable,
            default_unit: "个".to_string(),
            default_price: 5,
            default_shelf_life: 72,
            description: "新鲜番茄".to_string(),
            compatible_cuisines: vec!["家常菜".to_string()],
        };

        let instance = template.create_instance(5);
        assert_eq!(instance.quantity, 5);
        assert_eq!(instance.unit, "个");
        assert_eq!(instance.unit_price, 5);
    }
}
