//! 节假日定义

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 节假日类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolidayType {
    /// 春节
    SpringFestival,
    /// 元宵节
    LanternFestival,
    /// 清明节
    QingmingFestival,
    /// 劳动节
    LaborDay,
    /// 端午节
    DragonBoatFestival,
    /// 中秋节
    MidAutumnFestival,
    /// 国庆节
    NationalDay,
    /// 元旦
    NewYear,
}

impl HolidayType {
    /// 获取节日名称
    pub fn name(&self) -> &str {
        match self {
            HolidayType::SpringFestival => "春节",
            HolidayType::LanternFestival => "元宵节",
            HolidayType::QingmingFestival => "清明节",
            HolidayType::LaborDay => "劳动节",
            HolidayType::DragonBoatFestival => "端午节",
            HolidayType::MidAutumnFestival => "中秋节",
            HolidayType::NationalDay => "国庆节",
            HolidayType::NewYear => "元旦",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            HolidayType::SpringFestival => "🧧",
            HolidayType::LanternFestival => "🏮",
            HolidayType::QingmingFestival => "🌸",
            HolidayType::LaborDay => "👷",
            HolidayType::DragonBoatFestival => "🐲",
            HolidayType::MidAutumnFestival => "🌕",
            HolidayType::NationalDay => "🇨🇳",
            HolidayType::NewYear => "🎆",
        }
    }

    /// 获取持续时间（天）
    pub fn duration_days(&self) -> u32 {
        match self {
            HolidayType::SpringFestival => 7,
            HolidayType::NationalDay => 7,
            HolidayType::LaborDay => 5,
            HolidayType::DragonBoatFestival => 3,
            HolidayType::MidAutumnFestival => 3,
            HolidayType::NewYear => 3,
            HolidayType::LanternFestival => 1,
            HolidayType::QingmingFestival => 3,
        }
    }
}

/// 节假日效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayEffect {
    /// 客流修正（叠加在天气之上）
    pub customer_flow_modifier: f32,
    /// 效果描述
    pub description: Option<String>,
    /// 特殊菜品需求
    pub special_dishes: Vec<String>,
    /// 特殊效果
    pub special_effects: Vec<String>,
}

impl HolidayEffect {
    /// 春节效果
    pub fn spring_festival() -> Self {
        Self {
            customer_flow_modifier: 0.5,
            description: Some("新春佳节，团圆聚餐需求旺盛".to_string()),
            special_dishes: vec!["饺子".to_string(), "年糕".to_string(), "鱼".to_string()],
            special_effects: vec!["红包活动".to_string(), "团圆宴".to_string()],
        }
    }

    /// 中秋节效果
    pub fn mid_autumn_festival() -> Self {
        Self {
            customer_flow_modifier: 0.3,
            description: Some("中秋团圆，赏月聚餐".to_string()),
            special_dishes: vec!["月饼".to_string(), "螃蟹".to_string()],
            special_effects: vec!["赏月活动".to_string()],
        }
    }

    /// 端午节效果
    pub fn dragon_boat_festival() -> Self {
        Self {
            customer_flow_modifier: 0.2,
            description: Some("端午节，粽香四溢".to_string()),
            special_dishes: vec!["粽子".to_string()],
            special_effects: vec!["龙舟活动".to_string()],
        }
    }

    /// 劳动节效果
    pub fn labor_day() -> Self {
        Self {
            customer_flow_modifier: 0.3,
            description: Some("劳动节假期，外出就餐增多".to_string()),
            special_dishes: vec![],
            special_effects: vec![],
        }
    }

    /// 国庆节效果
    pub fn national_day() -> Self {
        Self {
            customer_flow_modifier: 0.4,
            description: Some("国庆长假，客流大增".to_string()),
            special_dishes: vec![],
            special_effects: vec!["节日促销".to_string()],
        }
    }

    /// 元旦效果
    pub fn new_year() -> Self {
        Self {
            customer_flow_modifier: 0.2,
            description: Some("新年伊始，庆祝聚餐".to_string()),
            special_dishes: vec!["年糕".to_string()],
            special_effects: vec![],
        }
    }

    /// 元宵节效果
    pub fn lantern_festival() -> Self {
        Self {
            customer_flow_modifier: 0.15,
            description: Some("元宵佳节，赏灯吃汤圆".to_string()),
            special_dishes: vec!["汤圆".to_string()],
            special_effects: vec!["猜灯谜".to_string()],
        }
    }

    /// 清明节效果
    pub fn qingming_festival() -> Self {
        Self {
            customer_flow_modifier: 0.0,
            description: Some("清明节，祭祖踏青".to_string()),
            special_dishes: vec!["青团".to_string()],
            special_effects: vec![],
        }
    }

    /// 默认效果（无节日）
    pub fn none() -> Self {
        Self {
            customer_flow_modifier: 0.0,
            description: None,
            special_dishes: vec![],
            special_effects: vec![],
        }
    }
}

impl HolidayType {
    /// 获取节日效果
    pub fn effect(&self) -> HolidayEffect {
        match self {
            HolidayType::SpringFestival => HolidayEffect::spring_festival(),
            HolidayType::MidAutumnFestival => HolidayEffect::mid_autumn_festival(),
            HolidayType::DragonBoatFestival => HolidayEffect::dragon_boat_festival(),
            HolidayType::LaborDay => HolidayEffect::labor_day(),
            HolidayType::NationalDay => HolidayEffect::national_day(),
            HolidayType::NewYear => HolidayEffect::new_year(),
            HolidayType::LanternFestival => HolidayEffect::lantern_festival(),
            HolidayType::QingmingFestival => HolidayEffect::qingming_festival(),
        }
    }
}

/// 节假日
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    /// 节日类型
    pub holiday_type: HolidayType,
    /// 节日效果
    pub effect: HolidayEffect,
    /// 开始日期
    pub start_date: NaiveDate,
    /// 结束日期
    pub end_date: NaiveDate,
    /// 开始时间
    pub started_at: DateTime<Utc>,
}

impl Holiday {
    /// 创建新节假日
    pub fn new(holiday_type: HolidayType) -> Self {
        let effect = holiday_type.effect();
        let duration = holiday_type.duration_days();
        let today = Utc::now().date_naive();

        Self {
            holiday_type,
            effect,
            start_date: today,
            end_date: today + chrono::Duration::days(duration as i64),
            started_at: Utc::now(),
        }
    }

    /// 创建指定日期的节假日
    pub fn with_date(holiday_type: HolidayType, start_date: NaiveDate) -> Self {
        let effect = holiday_type.effect();
        let duration = holiday_type.duration_days();

        Self {
            holiday_type,
            effect,
            start_date,
            end_date: start_date + chrono::Duration::days(duration as i64),
            started_at: Utc::now(),
        }
    }

    /// 获取效果
    pub fn effect(&self) -> &HolidayEffect {
        &self.effect
    }

    /// 检查日期是否在节假日期间
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }
}

/// 节假日管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayManager {
    /// 当前节假日
    pub current_holiday: Option<Holiday>,
    /// 预定义的节假日日期（年份 -> 节假日列表）
    pub holidays: Vec<HolidayDefinition>,
}

/// 节假日定义（用于计算农历节日）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayDefinition {
    /// 节日类型
    pub holiday_type: HolidayType,
    /// 固定日期的月份（1-12）
    pub fixed_month: Option<u32>,
    /// 固定日期的日（1-31）
    pub fixed_day: Option<u32>,
    /// 是否是农历节日
    pub is_lunar: bool,
}

impl HolidayDefinition {
    /// 创建固定日期的节日
    pub fn fixed(holiday_type: HolidayType, month: u32, day: u32) -> Self {
        Self {
            holiday_type,
            fixed_month: Some(month),
            fixed_day: Some(day),
            is_lunar: false,
        }
    }

    /// 创建农历节日
    pub fn lunar(holiday_type: HolidayType, month: u32, day: u32) -> Self {
        Self {
            holiday_type,
            fixed_month: Some(month),
            fixed_day: Some(day),
            is_lunar: true,
        }
    }

    /// 获取指定年份的日期（简化版，不实际计算农历）
    pub fn get_date_for_year(&self, year: i32) -> Option<NaiveDate> {
        if self.is_lunar {
            // 农历节日需要复杂计算，这里使用近似日期
            // 实际项目中应使用农历库
            let (month, day) = self.get_approximate_gregorian_date(year);
            NaiveDate::from_ymd_opt(year, month, day)
        } else {
            NaiveDate::from_ymd_opt(year, self.fixed_month?, self.fixed_day?)
        }
    }

    /// 获取农历节日的近似公历日期
    fn get_approximate_gregorian_date(&self, year: i32) -> (u32, u32) {
        // 简化处理：使用固定的近似日期
        match self.holiday_type {
            HolidayType::SpringFestival => {
                // 春节通常在1月下旬或2月上旬
                let offset = (year - 2024) % 3;
                if offset == 0 {
                    (2, 10)
                } else if offset == 1 {
                    (1, 29)
                } else {
                    (2, 1)
                }
            }
            HolidayType::LanternFestival => (2, 24), // 元宵节通常在2月
            HolidayType::QingmingFestival => (4, 4), // 清明节固定
            HolidayType::DragonBoatFestival => (6, 10), // 端午节通常在6月
            HolidayType::MidAutumnFestival => (9, 17), // 中秋节通常在9月
            _ => (self.fixed_month.unwrap_or(1), self.fixed_day.unwrap_or(1)),
        }
    }
}

impl HolidayManager {
    /// 创建新的节假日管理器
    pub fn new() -> Self {
        Self {
            current_holiday: None,
            holidays: Self::create_default_holidays(),
        }
    }

    /// 创建默认节假日定义
    fn create_default_holidays() -> Vec<HolidayDefinition> {
        vec![
            // 公历节日
            HolidayDefinition::fixed(HolidayType::NewYear, 1, 1),
            HolidayDefinition::fixed(HolidayType::LaborDay, 5, 1),
            HolidayDefinition::fixed(HolidayType::NationalDay, 10, 1),
            // 农历节日（使用近似日期）
            HolidayDefinition::lunar(HolidayType::SpringFestival, 1, 1),
            HolidayDefinition::lunar(HolidayType::LanternFestival, 1, 15),
            HolidayDefinition::lunar(HolidayType::QingmingFestival, 4, 4),
            HolidayDefinition::lunar(HolidayType::DragonBoatFestival, 5, 5),
            HolidayDefinition::lunar(HolidayType::MidAutumnFestival, 8, 15),
        ]
    }

    /// 获取指定日期的节假日
    pub fn get_holiday(&self, date: NaiveDate) -> Option<Holiday> {
        let year = date.year();

        for definition in &self.holidays {
            if let Some(holiday_date) = definition.get_date_for_year(year) {
                let holiday = Holiday::with_date(definition.holiday_type.clone(), holiday_date);
                if holiday.is_active_on(date) {
                    return Some(holiday);
                }
            }
        }

        None
    }

    /// 检查指定日期是否是节假日
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.get_holiday(date).is_some()
    }

    /// 获取指定年份的所有节假日
    pub fn get_holidays_for_year(&self, year: i32) -> Vec<Holiday> {
        let mut holidays = Vec::new();

        for definition in &self.holidays {
            if let Some(date) = definition.get_date_for_year(year) {
                holidays.push(Holiday::with_date(definition.holiday_type.clone(), date));
            }
        }

        // 按日期排序
        holidays.sort_by_key(|h| h.start_date);
        holidays
    }
}

impl Default for HolidayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holiday_creation() {
        let holiday = Holiday::new(HolidayType::SpringFestival);
        assert_eq!(holiday.holiday_type, HolidayType::SpringFestival);
        assert!(holiday.effect.customer_flow_modifier > 0.0);
    }

    #[test]
    fn test_holiday_effect() {
        let spring_effect = HolidayType::SpringFestival.effect();
        assert_eq!(spring_effect.customer_flow_modifier, 0.5);
        assert!(!spring_effect.special_dishes.is_empty());

        let mid_autumn_effect = HolidayType::MidAutumnFestival.effect();
        assert_eq!(mid_autumn_effect.customer_flow_modifier, 0.3);
    }

    #[test]
    fn test_holiday_manager_creation() {
        let manager = HolidayManager::new();
        assert!(manager.current_holiday.is_none());
        assert!(!manager.holidays.is_empty());
    }

    #[test]
    fn test_is_holiday() {
        let manager = HolidayManager::new();

        // 元旦
        let new_year = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert!(manager.is_holiday(new_year));

        // 劳动节
        let labor_day = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        assert!(manager.is_holiday(labor_day));

        // 普通日子
        let normal_day = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert!(!manager.is_holiday(normal_day));
    }

    #[test]
    fn test_holiday_duration() {
        assert_eq!(HolidayType::SpringFestival.duration_days(), 7);
        assert_eq!(HolidayType::NationalDay.duration_days(), 7);
        assert_eq!(HolidayType::LaborDay.duration_days(), 5);
        assert_eq!(HolidayType::LanternFestival.duration_days(), 1);
    }

    #[test]
    fn test_holiday_active() {
        let holiday = Holiday::with_date(
            HolidayType::SpringFestival,
            NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        );

        assert!(holiday.is_active_on(NaiveDate::from_ymd_opt(2024, 2, 10).unwrap()));
        assert!(holiday.is_active_on(NaiveDate::from_ymd_opt(2024, 2, 12).unwrap()));
        assert!(!holiday.is_active_on(NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()));
    }
}
