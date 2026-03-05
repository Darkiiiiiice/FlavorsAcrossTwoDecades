#![allow(clippy::module_inception)]
//! 天气与节假日系统模块
//!
//! 管理游戏中的天气变化和节假日效果

mod holiday;
mod weather;

pub use holiday::{Holiday, HolidayManager, HolidayType};
pub use weather::{Weather, WeatherEffect, WeatherManager, WeatherType};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 环境状态（天气 + 节假日综合效果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// 当前天气
    pub weather: Weather,
    /// 当前节假日（如果有）
    pub holiday: Option<Holiday>,
    /// 综合客流修正
    pub customer_flow_modifier: f32,
    /// 特殊效果列表
    pub active_effects: Vec<String>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl EnvironmentState {
    /// 创建新的环境状态
    pub fn new(weather: Weather) -> Self {
        let customer_flow_modifier = weather.effect().customer_flow_modifier;
        let effect_desc = weather.effect().description.clone();
        Self {
            weather,
            holiday: None,
            customer_flow_modifier,
            active_effects: vec![effect_desc],
            updated_at: Utc::now(),
        }
    }

    /// 设置节假日
    pub fn set_holiday(&mut self, holiday: Holiday) {
        let weather_modifier = self.weather.effect().customer_flow_modifier;
        let holiday_modifier = holiday.effect().customer_flow_modifier;

        // 节假日效果叠加
        self.customer_flow_modifier = weather_modifier + holiday_modifier;

        if let Some(ref holiday_effect) = holiday.effect().description {
            self.active_effects.push(holiday_effect.clone());
        }
        self.holiday = Some(holiday);
        self.updated_at = Utc::now();
    }

    /// 清除节假日
    pub fn clear_holiday(&mut self) {
        self.holiday = None;
        self.customer_flow_modifier = self.weather.effect().customer_flow_modifier;
        self.active_effects = vec![self.weather.effect().description.clone()];
        self.updated_at = Utc::now();
    }

    /// 更新天气
    pub fn set_weather(&mut self, weather: Weather) {
        let weather_modifier = weather.effect().customer_flow_modifier;

        if let Some(ref holiday) = self.holiday {
            let holiday_modifier = holiday.effect().customer_flow_modifier;
            self.customer_flow_modifier = weather_modifier + holiday_modifier;
        } else {
            self.customer_flow_modifier = weather_modifier;
        }

        self.active_effects = vec![weather.effect().description.clone()];
        if let Some(ref holiday) = self.holiday
            && let Some(ref holiday_effect) = holiday.effect().description
        {
            self.active_effects.push(holiday_effect.clone());
        }
        self.weather = weather;
        self.updated_at = Utc::now();
    }
}

/// 环境管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentManager {
    /// 当前环境状态
    pub state: EnvironmentState,
    /// 天气管理器
    pub weather_manager: WeatherManager,
    /// 节假日管理器
    pub holiday_manager: HolidayManager,
}

impl EnvironmentManager {
    /// 创建新的环境管理器
    pub fn new() -> Self {
        let weather_manager = WeatherManager::new();
        let weather = weather_manager.current_weather.clone();
        let state = EnvironmentState::new(weather);

        Self {
            state,
            weather_manager,
            holiday_manager: HolidayManager::new(),
        }
    }

    /// 更新环境（每天调用）
    pub fn update(&mut self, date: chrono::NaiveDate) {
        // 更新天气
        self.weather_manager.update_weather();
        self.state
            .set_weather(self.weather_manager.current_weather.clone());

        // 检查节假日
        if let Some(holiday) = self.holiday_manager.get_holiday(date) {
            self.state.set_holiday(holiday);
        } else {
            self.state.clear_holiday();
        }
    }

    /// 获取当前客流修正
    pub fn get_customer_flow_modifier(&self) -> f32 {
        self.state.customer_flow_modifier
    }

    /// 检查是否是特殊天气
    pub fn is_special_weather(&self) -> bool {
        matches!(
            self.state.weather.weather_type,
            WeatherType::Rainy | WeatherType::Snowy
        )
    }

    /// 检查是否是节假日
    pub fn is_holiday(&self) -> bool {
        self.state.holiday.is_some()
    }
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_state_creation() {
        let weather = Weather::new(WeatherType::Sunny);
        let state = EnvironmentState::new(weather);

        assert!(state.holiday.is_none());
        assert_eq!(state.customer_flow_modifier, 1.0); // 晴天客流正常
    }

    #[test]
    fn test_environment_with_holiday() {
        let weather = Weather::new(WeatherType::Sunny);
        let mut state = EnvironmentState::new(weather);

        let holiday = Holiday::new(HolidayType::SpringFestival);
        let holiday_modifier = holiday.effect().customer_flow_modifier;
        state.set_holiday(holiday);

        assert!(state.holiday.is_some());
        // 晴天(1.0) + 春节(0.5)
        assert_eq!(state.customer_flow_modifier, 1.0 + holiday_modifier);
    }

    #[test]
    fn test_environment_manager_creation() {
        let manager = EnvironmentManager::new();

        assert!(manager.state.holiday.is_none());
        assert_eq!(manager.state.customer_flow_modifier, 1.0);
    }

    #[test]
    fn test_rainy_weather_effect() {
        let weather = Weather::new(WeatherType::Rainy);
        let state = EnvironmentState::new(weather);

        assert_eq!(state.customer_flow_modifier, 0.8); // 雨天客流-20%
    }

    #[test]
    fn test_snowy_weather_effect() {
        let weather = Weather::new(WeatherType::Snowy);
        let state = EnvironmentState::new(weather);

        assert_eq!(state.customer_flow_modifier, 0.7); // 雪天客流-30%
    }
}
