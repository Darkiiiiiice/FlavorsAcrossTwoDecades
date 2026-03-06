use serde::{Deserialize, Serialize};

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
    /// 晴朗效果
    pub fn sunny() -> Self {
        Self {
            customer_flow_modifier: 1.1,
            description: "天气晴朗，适合出行".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("光照充足，作物生长良好".to_string()),
        }
    }

    /// 多云效果
    pub fn cloudy() -> Self {
        Self {
            customer_flow_modifier: 1.0,
            description: "多云天气，温度适宜".to_string(),
            energy_modifier: 1.0,
            garden_effect: None,
        }
    }

    /// 阴天效果
    pub fn overcast() -> Self {
        Self {
            customer_flow_modifier: 0.9,
            description: "天气阴沉，客流略减".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("光照不足，作物生长缓慢".to_string()),
        }
    }

    /// 小雨效果
    pub fn light_rain() -> Self {
        Self {
            customer_flow_modifier: 0.85,
            description: "下小雨了，部分客人取消行程".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("适量降雨，有利于作物".to_string()),
        }
    }

    /// 大雨效果
    pub fn heavy_rain() -> Self {
        Self {
            customer_flow_modifier: 0.7,
            description: "大雨倾盆，客流明显减少".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("注意排水，防止涝害".to_string()),
        }
    }

    /// 雷暴效果
    pub fn thunderstorm() -> Self {
        Self {
            customer_flow_modifier: 0.5,
            description: "雷暴天气，建议减少外出".to_string(),
            energy_modifier: 1.1,
            garden_effect: Some("雷暴可能损坏作物".to_string()),
        }
    }

    /// 小雪效果
    pub fn light_snow() -> Self {
        Self {
            customer_flow_modifier: 0.8,
            description: "开始下雪，出行需小心".to_string(),
            energy_modifier: 1.1,
            garden_effect: Some("注意保暖，防止冻害".to_string()),
        }
    }

    /// 大雪效果
    pub fn heavy_snow() -> Self {
        Self {
            customer_flow_modifier: 0.6,
            description: "大雪纷飞，出行困难".to_string(),
            energy_modifier: 1.3,
            garden_effect: Some("作物可能受冻，需要防护".to_string()),
        }
    }

    /// 雾效果
    pub fn fog() -> Self {
        Self {
            customer_flow_modifier: 0.75,
            description: "大雾弥漫，能见度低".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("湿度较高，注意通风".to_string()),
        }
    }

    /// 风效果
    pub fn windy() -> Self {
        Self {
            customer_flow_modifier: 0.9,
            description: "风力较大，注意安全".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("大风可能影响作物".to_string()),
        }
    }

    /// 沙尘效果
    pub fn sandstorm() -> Self {
        Self {
            customer_flow_modifier: 0.5,
            description: "沙尘暴来袭，空气质量差".to_string(),
            energy_modifier: 1.1,
            garden_effect: Some("沙尘覆盖，需要清理".to_string()),
        }
    }

    /// 彩虹效果
    pub fn rainbow() -> Self {
        Self {
            customer_flow_modifier: 1.2,
            description: "雨后天晴，彩虹挂空，美景难得".to_string(),
            energy_modifier: 1.0,
            garden_effect: Some("雨后阳光，作物焕发生机".to_string()),
        }
    }
}
