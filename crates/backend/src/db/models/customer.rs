//! 顾客数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::game::customer::{CustomerType, Customer};
use crate::game::customer::preference::{Preference, FlavorPreference, DietaryRestriction};

/// 顾客记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRecord {
    /// 顾客ID
    pub id: i64,
    /// 顾客姓名
    pub name: String,
    /// 年龄
    pub age: u32,
    /// 职业
    pub occupation: String,
    /// 顾客类型
    pub customer_type: CustomerType,
    /// 好感度 (0-1000)
    pub affinity: u32,
    /// 访问次数
    pub visit_count: u32,
    /// 故事背景
    pub story_background: String,
    /// 偏好
    pub preference: PreferenceRecord,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl CustomerRecord {
    /// 从游戏 Customer 创建记录
    pub fn from_customer(customer: &Customer, preference: PreferenceRecord) -> Self {
        Self {
            id: customer.id,
            name: customer.name.clone(),
            age: customer.age,
            occupation: customer.occupation.clone(),
            customer_type: customer.customer_type,
            affinity: customer.affinity,
            visit_count: customer.visit_count,
            story_background: customer.story_background.clone(),
            preference,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 转换为游戏 Customer
    pub fn to_customer(&self) -> Customer {
        Customer {
            id: self.id,
            name: self.name.clone(),
            age: self.age,
            occupation: self.occupation.clone(),
            customer_type: self.customer_type,
            preference: self.preference.to_preference(),
            affinity: self.affinity,
            visit_count: self.visit_count,
            story_background: self.story_background.clone(),
        }
    }
}

/// 偏好记录（数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceRecord {
    /// 偏好ID
    pub id: i64,
    /// 关联的顾客ID
    pub customer_id: i64,
    /// 口味偏好
    pub flavor: FlavorPreference,
    /// 饮食限制
    pub dietary: DietaryRestriction,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: u32,
    /// 耐心值 (0-100)
    pub patience: u32,
    /// 喜欢的菜品类型 (JSON数组)
    pub favorite_categories: String,
}

impl PreferenceRecord {
    /// 从游戏 Preference 创建记录
    pub fn from_preference(preference: &Preference, customer_id: i64) -> Self {
        Self {
            id: 0,
            customer_id,
            flavor: preference.flavor,
            dietary: preference.dietary,
            price_sensitivity: preference.price_sensitivity,
            patience: preference.patience,
            favorite_categories: serde_json::to_string(&preference.favorite_categories)
                .unwrap_or_else(|_| "[]".to_string()),
        }
    }

    /// 转换为游戏 Preference
    pub fn to_preference(&self) -> Preference {
        let favorite_categories = serde_json::from_str(&self.favorite_categories)
            .unwrap_or_else(|_| Vec::new());

        Preference {
            flavor: self.flavor,
            dietary: self.dietary,
            price_sensitivity: self.price_sensitivity,
            patience: self.patience,
            favorite_categories,
        }
    }
}

// ========== 枚举转换实现 ==========

impl From<CustomerType> for i32 {
    fn from(value: CustomerType) -> Self {
        match value {
            CustomerType::Normal => 0,
            CustomerType::Foodie => 1,
            CustomerType::Critic => 2,
        }
    }
}

impl TryFrom<i32> for CustomerType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CustomerType::Normal),
            1 => Ok(CustomerType::Foodie),
            2 => Ok(CustomerType::Critic),
            _ => Err(format!("Invalid CustomerType value: {}", value)),
        }
    }
}

impl From<FlavorPreference> for i32 {
    fn from(value: FlavorPreference) -> Self {
        match value {
            FlavorPreference::Light => 0,
            FlavorPreference::Medium => 1,
            FlavorPreference::Heavy => 2,
            FlavorPreference::Spicy => 3,
            FlavorPreference::SweetSour => 4,
        }
    }
}

impl TryFrom<i32> for FlavorPreference {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FlavorPreference::Light),
            1 => Ok(FlavorPreference::Medium),
            2 => Ok(FlavorPreference::Heavy),
            3 => Ok(FlavorPreference::Spicy),
            4 => Ok(FlavorPreference::SweetSour),
            _ => Err(format!("Invalid FlavorPreference value: {}", value)),
        }
    }
}

impl From<DietaryRestriction> for i32 {
    fn from(value: DietaryRestriction) -> Self {
        match value {
            DietaryRestriction::None => 0,
            DietaryRestriction::Vegetarian => 1,
            DietaryRestriction::Halal => 2,
            DietaryRestriction::GlutenFree => 3,
            DietaryRestriction::LowSugar => 4,
        }
    }
}

impl TryFrom<i32> for DietaryRestriction {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DietaryRestriction::None),
            1 => Ok(DietaryRestriction::Vegetarian),
            2 => Ok(DietaryRestriction::Halal),
            3 => Ok(DietaryRestriction::GlutenFree),
            4 => Ok(DietaryRestriction::LowSugar),
            _ => Err(format!("Invalid DietaryRestriction value: {}", value)),
        }
    }
}
