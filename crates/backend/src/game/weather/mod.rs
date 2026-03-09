//! 天气与节假日系统模块
//!
//! 管理游戏中的天气变化和节假日效果

mod effect;
mod holiday;
mod prompt;
mod weather;

use std::sync::Arc;

pub use effect::WeatherEffect;
pub use holiday::{Holiday, HolidayManager, HolidayType};
pub use weather::{Weather, WeatherManager, WeatherType};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{db::DbPool, game::LlmManager};

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
#[derive(Debug)]
pub struct EnvironmentManager {
    /// 当前环境状态
    pub state: EnvironmentState,
    /// 天气管理器
    pub weather_manager: WeatherManager,
    /// 节假日管理器
    pub holiday_manager: HolidayManager,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
}

impl EnvironmentManager {
    /// 创建新的环境管理器
    pub fn new(db_pool: Arc<DbPool>, llm_manager: Arc<LlmManager>) -> Self {
        let weather_manager = WeatherManager::new(db_pool.clone()).with_llm(llm_manager);
        let weather = weather_manager.current_weather.clone();
        let state = EnvironmentState::new(weather);

        Self {
            state,
            weather_manager,
            holiday_manager: HolidayManager::new(),
            db_pool,
        }
    }

    /// 更新环境（每天调用）
    pub async fn tick(&mut self, timestamp: i64) {
        self.weather_manager.tick(timestamp).await;
    }
}
