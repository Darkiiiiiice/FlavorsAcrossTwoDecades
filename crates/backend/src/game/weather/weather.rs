//! 天气定义

use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};

/// 天气类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    /// 晴天
    Sunny,
    /// 阴天
    Cloudy,
    /// 雨天
    Rainy,
    /// 雪天
    Snowy,
}

impl WeatherType {
    /// 获取天气名称
    pub fn name(&self) -> &str {
        match self {
            WeatherType::Sunny => "晴天",
            WeatherType::Cloudy => "阴天",
            WeatherType::Rainy => "雨天",
            WeatherType::Snowy => "雪天",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            WeatherType::Sunny => "☀️",
            WeatherType::Cloudy => "☁️",
            WeatherType::Rainy => "🌧️",
            WeatherType::Snowy => "❄️",
        }
    }

    /// 获取基础概率（按季节可调整）
    pub fn base_probability(&self) -> f32 {
        match self {
            WeatherType::Sunny => 0.5,
            WeatherType::Cloudy => 0.25,
            WeatherType::Rainy => 0.2,
            WeatherType::Snowy => 0.05,
        }
    }
}

/// 天气效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEffect {
    /// 客流修正（1.0 = 正常）
    pub customer_flow_modifier: f32,
    /// 效果描述
    pub description: String,
    /// 能源消耗修正
    pub energy_modifier: f32,
    /// 种植效果
    pub garden_effect: Option<String>,
}

impl WeatherEffect {
    /// 晴天效果
    pub fn sunny() -> Self {
        Self {
            customer_flow_modifier: 1.0,
            description: "天气晴朗，适合出行".to_string(),
            energy_modifier: 1.0,
            garden_effect: None,
        }
    }

    /// 阴天效果
    pub fn cloudy() -> Self {
        Self {
            customer_flow_modifier: 0.9,
            description: "天气阴沉，客流略减".to_string(),
            energy_modifier: 1.0,
            garden_effect: None,
        }
    }

    /// 雨天效果
    pub fn rainy() -> Self {
        Self {
            customer_flow_modifier: 0.8,
            description: "下雨了，客流减少".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("种植需注意防涝".to_string()),
        }
    }

    /// 雪天效果
    pub fn snowy() -> Self {
        Self {
            customer_flow_modifier: 0.7,
            description: "下雪了，出行不便".to_string(),
            energy_modifier: 1.2, // 供暖需求增加
            garden_effect: Some("作物可能受冻".to_string()),
        }
    }
}

/// 天气
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    /// 天气类型
    pub weather_type: WeatherType,
    /// 天气效果
    pub effect: WeatherEffect,
    /// 持续时间（游戏小时）
    pub duration_hours: u32,
    /// 开始时间
    pub started_at: DateTime<Utc>,
}

impl Weather {
    /// 创建新天气
    pub fn new(weather_type: WeatherType) -> Self {
        let effect = weather_type.effect();
        Self {
            weather_type,
            effect,
            duration_hours: 24,
            started_at: Utc::now(),
        }
    }

    /// 获取天气效果
    pub fn effect(&self) -> &WeatherEffect {
        &self.effect
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.started_at);
        elapsed.num_hours() >= self.duration_hours as i64
    }
}

impl WeatherType {
    /// 获取天气效果
    pub fn effect(&self) -> WeatherEffect {
        match self {
            WeatherType::Sunny => WeatherEffect::sunny(),
            WeatherType::Cloudy => WeatherEffect::cloudy(),
            WeatherType::Rainy => WeatherEffect::rainy(),
            WeatherType::Snowy => WeatherEffect::snowy(),
        }
    }
}

/// 天气管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherManager {
    /// 当前天气
    pub current_weather: Weather,
    /// 历史天气记录
    pub history: Vec<Weather>,
}

impl WeatherManager {
    /// 创建新的天气管理器
    pub fn new() -> Self {
        let current_weather = Weather::new(WeatherType::Sunny);
        Self {
            current_weather,
            history: Vec::new(),
        }
    }

    /// 更新天气（每天调用）
    pub fn update_weather(&mut self) {
        // 保存当前天气到历史
        self.history.push(self.current_weather.clone());

        // 保留最近30天的历史
        if self.history.len() > 30 {
            self.history.remove(0);
        }

        // 生成新天气
        let new_weather_type = Self::generate_weather();
        self.current_weather = Weather::new(new_weather_type);
    }

    /// 生成新天气
    fn generate_weather() -> WeatherType {
        let mut rng = rand::rng();
        let roll = rng.random_range(0.0..1.0);

        // 基于概率选择天气
        let mut cumulative = 0.0;
        let types = [
            WeatherType::Sunny,
            WeatherType::Cloudy,
            WeatherType::Rainy,
            WeatherType::Snowy,
        ];

        for weather_type in types {
            cumulative += weather_type.base_probability();
            if roll < cumulative {
                return weather_type;
            }
        }

        WeatherType::Sunny
    }

    /// 根据季节调整天气概率
    pub fn update_weather_with_season(&mut self, season: crate::game::garden::Season) {
        // 保存当前天气到历史
        self.history.push(self.current_weather.clone());

        if self.history.len() > 30 {
            self.history.remove(0);
        }

        let new_weather_type = Self::generate_weather_for_season(season);
        self.current_weather = Weather::new(new_weather_type);
    }

    /// 根据季节生成天气
    fn generate_weather_for_season(season: crate::game::garden::Season) -> WeatherType {
        let mut rng = rand::rng();

        let (sunny_prob, cloudy_prob, rainy_prob, _snowy_prob) = match season {
            crate::game::garden::Season::Spring => (0.4, 0.3, 0.3, 0.0),
            crate::game::garden::Season::Summer => (0.6, 0.2, 0.2, 0.0),
            crate::game::garden::Season::Autumn => (0.5, 0.3, 0.2, 0.0),
            crate::game::garden::Season::Winter => (0.3, 0.3, 0.1, 0.3),
        };

        let roll = rng.random_range(0.0..1.0);

        if roll < sunny_prob {
            WeatherType::Sunny
        } else if roll < sunny_prob + cloudy_prob {
            WeatherType::Cloudy
        } else if roll < sunny_prob + cloudy_prob + rainy_prob {
            WeatherType::Rainy
        } else {
            WeatherType::Snowy
        }
    }

    /// 强制设置天气
    pub fn set_weather(&mut self, weather_type: WeatherType) {
        self.history.push(self.current_weather.clone());
        self.current_weather = Weather::new(weather_type);
    }

    /// 获取天气统计
    pub fn get_statistics(&self) -> WeatherStatistics {
        let mut stats = WeatherStatistics::default();

        for weather in &self.history {
            match weather.weather_type {
                WeatherType::Sunny => stats.sunny_days += 1,
                WeatherType::Cloudy => stats.cloudy_days += 1,
                WeatherType::Rainy => stats.rainy_days += 1,
                WeatherType::Snowy => stats.snowy_days += 1,
            }
        }

        stats.total_days = self.history.len() as u32;
        stats
    }
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 天气统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeatherStatistics {
    /// 总天数
    pub total_days: u32,
    /// 晴天天数
    pub sunny_days: u32,
    /// 阴天天数
    pub cloudy_days: u32,
    /// 雨天天数
    pub rainy_days: u32,
    /// 雪天天数
    pub snowy_days: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_creation() {
        let weather = Weather::new(WeatherType::Sunny);
        assert_eq!(weather.weather_type, WeatherType::Sunny);
        assert_eq!(weather.duration_hours, 24);
    }

    #[test]
    fn test_weather_effect() {
        let sunny_effect = WeatherType::Sunny.effect();
        assert_eq!(sunny_effect.customer_flow_modifier, 1.0);

        let rainy_effect = WeatherType::Rainy.effect();
        assert_eq!(rainy_effect.customer_flow_modifier, 0.8);
        assert!(rainy_effect.garden_effect.is_some());
    }

    #[test]
    fn test_weather_manager_creation() {
        let manager = WeatherManager::new();
        assert_eq!(manager.current_weather.weather_type, WeatherType::Sunny);
        assert!(manager.history.is_empty());
    }

    #[test]
    fn test_weather_manager_update() {
        let mut manager = WeatherManager::new();
        manager.update_weather();

        assert_eq!(manager.history.len(), 1);
    }

    #[test]
    fn test_weather_type_names() {
        assert_eq!(WeatherType::Sunny.name(), "晴天");
        assert_eq!(WeatherType::Rainy.name(), "雨天");
        assert_eq!(WeatherType::Snowy.name(), "雪天");
        assert_eq!(WeatherType::Cloudy.name(), "阴天");
    }
}
