//! 天气数据模型

use serde::{Deserialize, Serialize};

use crate::game::WeatherType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub id: i64,
    pub r#type: WeatherType,
    pub temperature: f64,
    pub duration: i64,
    pub created_at: i64,
}

impl Weather {
    pub fn new(
        id: i64,
        r#type: WeatherType,
        temperature: f64,
        duration: i64,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            r#type,
            temperature,
            duration,
            created_at,
        }
    }

    pub fn is_expired(&self, current_time: i64) -> bool {
        self.created_at + self.duration < current_time
    }
}

impl From<crate::game::Weather> for Weather {
    fn from(value: crate::game::Weather) -> Self {
        Self {
            id: value.id,
            r#type: value.weather_type,
            temperature: value.temperature,
            duration: value.duration_hours,
            created_at: value.create_at.timestamp(),
        }
    }
}
