//! 天气定义

use std::sync::Arc;

use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{
    db::{DbPool, models::Weather as DbWeather, repositories::weather::WeatherRepository},
    game::{LlmManager, WeatherEffect},
    utils::get_month,
};

/// 天气季节倾向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherSeason {
    /// 春季
    Spring,
    /// 夏季
    Summer,
    /// 冬季
    Winter,
    /// 秋冬
    Autumn,
}

impl WeatherSeason {
    /// 根据时间戳获取季节
    /// 春季: 3-5月, 夏季: 6-8月, 秋季: 9-11月, 冬季: 12-2月
    pub fn current_season(timestamp: i64) -> Self {
        match get_month(timestamp) {
            3..=5 => WeatherSeason::Spring,
            6..=8 => WeatherSeason::Summer,
            9..=11 => WeatherSeason::Autumn,
            _ => WeatherSeason::Winter, // 12, 1, 2
        }
    }
}

impl WeatherSeason {
    pub fn name(&self) -> &'static str {
        match self {
            WeatherSeason::Spring => "春季",
            WeatherSeason::Summer => "夏季",
            WeatherSeason::Winter => "冬季",
            WeatherSeason::Autumn => "秋冬",
        }
    }
}

/// 天气类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    /// 晴朗
    Sunny = 0,
    /// 多云
    Cloudy = 1,
    /// 阴天
    Overcast = 2,
    /// 小雨
    LightRain = 3,
    /// 大雨
    HeavyRain = 4,
    /// 雷暴
    Thunderstorm = 5,
    /// 小雪
    LightSnow = 6,
    /// 大雪
    HeavySnow = 7,
    /// 雾
    Fog = 8,
    /// 风
    Windy = 9,
    /// 沙尘
    Sandstorm = 10,
    /// 彩虹
    Rainbow = 11,
}

impl WeatherType {
    /// 获取天气名称
    pub fn name(&self) -> &str {
        match self {
            WeatherType::Sunny => "晴朗",
            WeatherType::Cloudy => "多云",
            WeatherType::Overcast => "阴天",
            WeatherType::LightRain => "小雨",
            WeatherType::HeavyRain => "大雨",
            WeatherType::Thunderstorm => "雷暴",
            WeatherType::LightSnow => "小雪",
            WeatherType::HeavySnow => "大雪",
            WeatherType::Fog => "雾",
            WeatherType::Windy => "风",
            WeatherType::Sandstorm => "沙尘",
            WeatherType::Rainbow => "彩虹",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            WeatherType::Sunny => "☀️",
            WeatherType::Cloudy => "⛅",
            WeatherType::Overcast => "☁️",
            WeatherType::LightRain => "🌧️",
            WeatherType::HeavyRain => "🌧️",
            WeatherType::Thunderstorm => "⛈️",
            WeatherType::LightSnow => "🌨️",
            WeatherType::HeavySnow => "❄️",
            WeatherType::Fog => "🌫️",
            WeatherType::Windy => "💨",
            WeatherType::Sandstorm => "🌪️",
            WeatherType::Rainbow => "🌈",
        }
    }

    /// 获取温度系数
    pub fn temperature_coefficient(&self) -> f32 {
        match self {
            WeatherType::Sunny => 0.2,
            WeatherType::Cloudy => 0.0,
            WeatherType::Overcast => -0.1,
            WeatherType::LightRain => -0.1,
            WeatherType::HeavyRain => -0.2,
            WeatherType::Thunderstorm => -0.1,
            WeatherType::LightSnow => -0.3,
            WeatherType::HeavySnow => -0.5,
            WeatherType::Fog => 0.0,
            WeatherType::Windy => -0.1,
            WeatherType::Sandstorm => -0.1,
            WeatherType::Rainbow => 0.1,
        }
    }

    /// 获取湿度系数
    pub fn humidity_coefficient(&self) -> f32 {
        match self {
            WeatherType::Sunny => -0.2,
            WeatherType::Cloudy => 0.0,
            WeatherType::Overcast => 0.1,
            WeatherType::LightRain => 0.3,
            WeatherType::HeavyRain => 0.5,
            WeatherType::Thunderstorm => 0.6,
            WeatherType::LightSnow => 0.0,
            WeatherType::HeavySnow => 0.1,
            WeatherType::Fog => 0.3,
            WeatherType::Windy => -0.2,
            WeatherType::Sandstorm => -0.5,
            WeatherType::Rainbow => -0.1,
        }
    }

    /// 获取光照系数
    pub fn light_coefficient(&self) -> f32 {
        match self {
            WeatherType::Sunny => 0.3,
            WeatherType::Cloudy => -0.1,
            WeatherType::Overcast => -0.3,
            WeatherType::LightRain => -0.2,
            WeatherType::HeavyRain => -0.4,
            WeatherType::Thunderstorm => -0.5,
            WeatherType::LightSnow => -0.2,
            WeatherType::HeavySnow => -0.4,
            WeatherType::Fog => -0.4,
            WeatherType::Windy => 0.0,
            WeatherType::Sandstorm => -0.3,
            WeatherType::Rainbow => 0.2,
        }
    }

    /// 获取天气效果
    pub fn effect(&self) -> WeatherEffect {
        match self {
            WeatherType::Sunny => WeatherEffect::sunny(),
            WeatherType::Cloudy => WeatherEffect::cloudy(),
            WeatherType::Overcast => WeatherEffect::overcast(),
            WeatherType::LightRain => WeatherEffect::light_rain(),
            WeatherType::HeavyRain => WeatherEffect::heavy_rain(),
            WeatherType::Thunderstorm => WeatherEffect::thunderstorm(),
            WeatherType::LightSnow => WeatherEffect::light_snow(),
            WeatherType::HeavySnow => WeatherEffect::heavy_snow(),
            WeatherType::Fog => WeatherEffect::fog(),
            WeatherType::Windy => WeatherEffect::windy(),
            WeatherType::Sandstorm => WeatherEffect::sandstorm(),
            WeatherType::Rainbow => WeatherEffect::rainbow(),
        }
    }
}
impl From<WeatherType> for i64 {
    fn from(value: WeatherType) -> Self {
        match value {
            WeatherType::Sunny => 0,
            WeatherType::Cloudy => 1,
            WeatherType::Overcast => 2,
            WeatherType::LightRain => 3,
            WeatherType::HeavyRain => 4,
            WeatherType::Thunderstorm => 5,
            WeatherType::LightSnow => 6,
            WeatherType::HeavySnow => 7,
            WeatherType::Fog => 8,
            WeatherType::Windy => 9,
            WeatherType::Sandstorm => 10,
            WeatherType::Rainbow => 11,
            _ => 0,
        }
    }
}
/// 从 i64 转换为 WeatherType
impl From<i64> for WeatherType {
    fn from(value: i64) -> Self {
        match value {
            0 => WeatherType::Sunny,
            1 => WeatherType::Cloudy,
            2 => WeatherType::Overcast,
            3 => WeatherType::LightRain,
            4 => WeatherType::HeavyRain,
            5 => WeatherType::Thunderstorm,
            6 => WeatherType::LightSnow,
            7 => WeatherType::HeavySnow,
            8 => WeatherType::Fog,
            9 => WeatherType::Windy,
            10 => WeatherType::Sandstorm,
            11 => WeatherType::Rainbow,
            _ => WeatherType::Sunny,
        }
    }
}

/// 天气
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub id: i64,
    /// 天气类型
    pub weather_type: WeatherType,
    /// 天气效果
    pub effect: WeatherEffect,
    /// 温度
    pub temperature: f64,
    /// 持续时间（游戏秒）
    pub duration_hours: i64,
    /// 开始时间
    pub create_at: DateTime<Utc>,
}

impl Weather {
    /// 创建新天气
    pub fn new(weather_type: WeatherType, temperature: f32) -> Self {
        let effect = weather_type.effect();
        Self {
            id: 0,
            weather_type,
            effect,
            temperature: temperature as f64,
            duration_hours: 60,
            create_at: Utc::now(),
        }
    }

    /// 获取天气效果
    pub fn effect(&self) -> &WeatherEffect {
        &self.effect
    }
}

impl From<DbWeather> for Weather {
    fn from(value: DbWeather) -> Self {
        use chrono::TimeZone;
        Weather {
            id: value.id,
            weather_type: value.r#type,
            effect: value.r#type.effect(),
            temperature: value.temperature,
            duration_hours: value.duration,
            create_at: Utc
                .timestamp_opt(value.created_at, 0)
                .single()
                .unwrap_or_else(Utc::now),
        }
    }
}

/// 季节天气概率配置
/// 每个季节定义各天气类型出现的概率权重
struct SeasonWeatherProbabilities {
    /// 天气类型及其概率权重
    weights: Vec<(WeatherType, u32)>,
}

impl SeasonWeatherProbabilities {
    /// 春季天气概率（3-5月）：多雨、温暖
    fn spring() -> Self {
        Self {
            weights: vec![
                (WeatherType::Sunny, 25),       // 25% 晴朗
                (WeatherType::Cloudy, 20),      // 20% 多云
                (WeatherType::Overcast, 15),    // 15% 阴天
                (WeatherType::LightRain, 20),   // 20% 小雨
                (WeatherType::HeavyRain, 8),    // 8% 大雨
                (WeatherType::Thunderstorm, 5), // 5% 雷暴
                (WeatherType::Fog, 5),          // 5% 雾
                (WeatherType::Windy, 2),        // 2% 风
                (WeatherType::Rainbow, 0),      // 0% 彩虹（雨后特殊出现）
                (WeatherType::LightSnow, 0),    // 0% 小雪
                (WeatherType::HeavySnow, 0),    // 0% 大雪
                (WeatherType::Sandstorm, 0),    // 0% 沙尘
            ],
        }
    }

    /// 夏季天气概率（6-8月）：炎热、雷雨
    fn summer() -> Self {
        Self {
            weights: vec![
                (WeatherType::Sunny, 35),        // 35% 晴朗
                (WeatherType::Cloudy, 15),       // 15% 多云
                (WeatherType::Overcast, 10),     // 10% 阴天
                (WeatherType::LightRain, 10),    // 10% 小雨
                (WeatherType::HeavyRain, 10),    // 10% 大雨
                (WeatherType::Thunderstorm, 12), // 12% 雷暴
                (WeatherType::Fog, 2),           // 2% 雾
                (WeatherType::Windy, 3),         // 3% 风
                (WeatherType::Sandstorm, 2),     // 2% 沙尘
                (WeatherType::Rainbow, 1),       // 1% 彩虹
                (WeatherType::LightSnow, 0),     // 0% 小雪
                (WeatherType::HeavySnow, 0),     // 0% 大雪
            ],
        }
    }

    /// 秋季天气概率（9-11月）：凉爽、多雾
    fn autumn() -> Self {
        Self {
            weights: vec![
                (WeatherType::Sunny, 30),       // 30% 晴朗
                (WeatherType::Cloudy, 20),      // 20% 多云
                (WeatherType::Overcast, 15),    // 15% 阴天
                (WeatherType::LightRain, 15),   // 15% 小雨
                (WeatherType::HeavyRain, 5),    // 5% 大雨
                (WeatherType::Thunderstorm, 2), // 2% 雷暴
                (WeatherType::Fog, 8),          // 8% 雾
                (WeatherType::Windy, 5),        // 5% 风
                (WeatherType::LightSnow, 0),    // 0% 小雪
                (WeatherType::HeavySnow, 0),    // 0% 大雪
                (WeatherType::Sandstorm, 0),    // 0% 沙尘
                (WeatherType::Rainbow, 0),      // 0% 彩虹
            ],
        }
    }

    /// 冬季天气概率（12-2月）：寒冷、降雪
    fn winter() -> Self {
        Self {
            weights: vec![
                (WeatherType::Sunny, 20),       // 20% 晴朗
                (WeatherType::Cloudy, 20),      // 20% 多云
                (WeatherType::Overcast, 20),    // 20% 阴天
                (WeatherType::LightSnow, 15),   // 15% 小雪
                (WeatherType::HeavySnow, 10),   // 10% 大雪
                (WeatherType::Fog, 10),         // 10% 雾
                (WeatherType::Windy, 5),        // 5% 风
                (WeatherType::LightRain, 0),    // 0% 小雨
                (WeatherType::HeavyRain, 0),    // 0% 大雨
                (WeatherType::Thunderstorm, 0), // 0% 雷暴
                (WeatherType::Sandstorm, 0),    // 0% 沙尘
                (WeatherType::Rainbow, 0),      // 0% 彩虹
            ],
        }
    }

    /// 根据季节获取概率配置
    fn for_season(season: WeatherSeason) -> Self {
        match season {
            WeatherSeason::Spring => Self::spring(),
            WeatherSeason::Summer => Self::summer(),
            WeatherSeason::Autumn => Self::autumn(),
            WeatherSeason::Winter => Self::winter(),
        }
    }

    /// 根据概率权重随机选择天气类型
    fn random_weather(&self) -> WeatherType {
        let total: u32 = self.weights.iter().map(|(_, w)| w).sum();
        let mut rng = rand::rng();
        let mut roll = rng.random_range(0..total);

        for (weather_type, weight) in &self.weights {
            if roll < *weight {
                return *weather_type;
            }
            roll -= weight;
        }

        // 默认返回晴天
        WeatherType::Sunny
    }
}

/// 天气管理器
#[derive(Debug, Clone)]
pub struct WeatherManager {
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// 当前天气
    pub current_weather: Weather,
    /// 历史天气记录
    pub history: Vec<Weather>,
    /// LLM 管理器（可选）
    llm_manager: Option<Arc<LlmManager>>,
    /// 是否正在生成天气
    generating: bool,
}

impl WeatherManager {
    /// 创建新的天气管理器
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        let current_weather = Weather::new(WeatherType::Sunny, 20.0);
        Self {
            db_pool,
            current_weather,
            history: Vec::new(),
            llm_manager: None,
            generating: false,
        }
    }

    /// 设置 LLM 管理器
    pub fn with_llm(mut self, llm_manager: Arc<LlmManager>) -> Self {
        self.llm_manager = Some(llm_manager);
        self
    }

    /// 更新天气（每天调用）
    pub async fn update_weather(&mut self, timestamp: i64) {
        let weather_repo = WeatherRepository::new(self.db_pool.pool().clone());
        let latest_weather = weather_repo.find_latest().await;

        // 天气是否过期
        let mut expired = false;

        match latest_weather {
            Ok(Some(latest_weather)) => {
                tracing::info!("latest weather: {:?}", latest_weather);
                if latest_weather.is_expired(timestamp) {
                    expired = true;
                } else {
                    self.current_weather = latest_weather.into();
                }
            }
            Ok(None) => {
                tracing::info!("no latest weather");
                expired = true;
            }
            Err(e) => {
                tracing::error!("failed to get latest weather: {:?}", e);
            }
        }

        // 重新生成weather
        if expired && !self.generating {
            tracing::info!("generating new weather");
            self.generating = true;
            let mut new_weather = self.generate_weather(timestamp).await;
            new_weather.id = timestamp;

            if weather_repo
                .create(&new_weather.clone().into())
                .await
                .is_ok()
            {
                self.current_weather = new_weather;
            };
            self.generating = false;
        }

        // 保存当前天气到历史
        self.history.push(self.current_weather.clone());

        // 保留最近30天的历史
        if self.history.len() > 30 {
            self.history.remove(0);
        }
    }

    /// 生成新天气（优先使用 LLM，失败时使用概率）
    async fn generate_weather(&self, timestamp: i64) -> Weather {
        let season = WeatherSeason::current_season(timestamp);

        // 尝试使用 LLM 生成天气
        if let Some(ref llm_manager) = self.llm_manager {
            match self.generate_weather_with_llm(llm_manager, season).await {
                Ok((weather_type, temperature)) => {
                    tracing::info!(
                        "LLM generated weather: {:?}, temperature: {}°C for season {:?}",
                        weather_type,
                        temperature,
                        season
                    );
                    return Weather::new(weather_type, temperature);
                }
                Err(e) => {
                    tracing::warn!("LLM weather generation failed: {}, using fallback", e);
                }
            }
        }

        // Fallback: 使用概率生成
        self.generate_weather_by_probability(season)
    }

    /// 使用 LLM 生成天气类型和温度
    async fn generate_weather_with_llm(
        &self,
        llm_manager: &Arc<LlmManager>,
        season: WeatherSeason,
    ) -> crate::error::Result<(WeatherType, f32)> {
        // 获取当前季节的天气概率配置
        let probabilities = SeasonWeatherProbabilities::for_season(season);

        // 计算总权重
        let total: u32 = probabilities.weights.iter().map(|(_, w)| w).sum();

        // 构建天气概率列表
        let weather_probs: Vec<String> = probabilities
            .weights
            .iter()
            .map(|(weather_type, weight)| {
                let percent = (*weight as f32 / total as f32 * 100.0) as u32;
                format!("- {}: {}%", weather_type.name(), percent)
            })
            .collect();

        // 构建季节描述和温度范围
        let (season_name, season_desc, temp_range) = match season {
            WeatherSeason::Spring => ("春季", "3-5月，温暖多雨，万物复苏", "10-25"),
            WeatherSeason::Summer => ("夏季", "6-8月，炎热，多雷雨", "25-35"),
            WeatherSeason::Autumn => ("秋季", "9-11月，凉爽，多雾", "10-20"),
            WeatherSeason::Winter => ("冬季", "12-2月，寒冷，可能降雪", "-5-10"),
        };

        // 构建最近天气历史（最近7天）
        let history_info = if self.history.is_empty() {
            "暂无历史记录".to_string()
        } else {
            let recent: Vec<String> = self
                .history
                .iter()
                .rev()
                .take(7)
                .enumerate()
                .map(|(i, w)| {
                    format!(
                        "{}秒前: {}，{:.1}°C",
                        i,
                        w.weather_type.name(),
                        w.temperature
                    )
                })
                .collect();
            recent.join("\n")
        };

        let system_prompt = r#"你是一个游戏天气模拟系统。请根据当前季节、天气概率分布和最近天气历史，为今天选择一个天气类型和温度。

输出格式要求（必须严格遵守）：
天气: <天气名称>
温度: <温度数值>

规则：
1. 参考最近天气历史，让天气变化有连续性（如连续几天晴天后可能转阴或下雨）
2. 严格按照给定的概率分布选择天气，概率越高的天气越可能被选中
3. 温度必须在给定的温度范围内，保留一位小数
4. 温度变化应该平缓，参考历史温度，避免剧烈波动
5. 只输出两行，不要有任何其他文字、标点或解释
6. 不要使用 markdown 格式"#;

        let user_message = format!(
            r#"当前季节：{}（{}）

可选天气类型及其出现概率：
{}

温度范围：{}°C

最近天气记录：
{}

请根据以上信息，为今天生成天气和温度。"#,
            season_name,
            season_desc,
            weather_probs.join("\n"),
            temp_range,
            history_info
        );

        tracing::info!("generating weather prompt: \n{}", user_message);

        let response = llm_manager
            .generate_text(system_prompt.to_string(), user_message)
            .await?;

        tracing::info!("LLM returned response for weather: \n{}", response);

        // 解析 LLM 返回的天气类型和温度
        let mut weather_type: Option<WeatherType> = None;
        let mut temperature: Option<f32> = None;

        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("天气:") || line.starts_with("天气：") {
                // 支持中英文冒号分割
                let name = line.split([':', '：']).last().unwrap_or("").trim();
                if let Ok(wt) = self.parse_weather_type(name) {
                    weather_type = Some(wt);
                }
            } else if line.starts_with("温度:") || line.starts_with("温度：") {
                // 支持中英文冒号分割
                let temp_str = line.split([':', '：']).last().unwrap_or("").trim();
                temperature = temp_str.parse::<f32>().ok();
            }
        }

        let weather_type = weather_type.ok_or_else(|| {
            crate::error::GameError::LlmError("Missing weather type in response".to_string())
        })?;
        let temperature = temperature.ok_or_else(|| {
            crate::error::GameError::LlmError("Missing temperature in response".to_string())
        })?;

        // 验证温度是否在合理范围内
        let (min_temp, max_temp) = Self::get_temperature_range(season);
        let temperature = temperature.clamp(min_temp, max_temp);

        tracing::info!(
            "LLM generated weather: {:?}, temperature: {}°C for season {:?}",
            weather_type,
            temperature,
            season
        );

        // 验证天气类型是否适合当前季节
        if self.is_weather_appropriate_for_season(weather_type, season) {
            Ok((weather_type, temperature))
        } else {
            tracing::warn!(
                "Weather {:?} not appropriate for season {:?}, using fallback",
                weather_type,
                season
            );
            Err(crate::error::GameError::LlmError(
                "Weather not appropriate for season".to_string(),
            ))
        }
    }

    /// 解析天气类型字符串
    fn parse_weather_type(&self, name: &str) -> Result<WeatherType, ()> {
        match name.trim() {
            "晴朗" | "晴" | "晴天" => Ok(WeatherType::Sunny),
            "多云" => Ok(WeatherType::Cloudy),
            "阴天" | "阴" => Ok(WeatherType::Overcast),
            "小雨" => Ok(WeatherType::LightRain),
            "大雨" => Ok(WeatherType::HeavyRain),
            "雷暴" | "雷雨" => Ok(WeatherType::Thunderstorm),
            "小雪" => Ok(WeatherType::LightSnow),
            "大雪" => Ok(WeatherType::HeavySnow),
            "雾" => Ok(WeatherType::Fog),
            "风" | "大风" => Ok(WeatherType::Windy),
            "沙尘" | "沙尘暴" => Ok(WeatherType::Sandstorm),
            "彩虹" => Ok(WeatherType::Rainbow),
            _ => Err(()),
        }
    }

    /// 获取季节的温度范围
    fn get_temperature_range(season: WeatherSeason) -> (f32, f32) {
        match season {
            WeatherSeason::Spring => (10.0, 25.0),
            WeatherSeason::Summer => (25.0, 35.0),
            WeatherSeason::Autumn => (10.0, 20.0),
            WeatherSeason::Winter => (-5.0, 10.0),
        }
    }

    /// 检查天气类型是否适合当前季节
    fn is_weather_appropriate_for_season(
        &self,
        weather_type: WeatherType,
        season: WeatherSeason,
    ) -> bool {
        match season {
            WeatherSeason::Spring => !matches!(
                weather_type,
                WeatherType::HeavySnow | WeatherType::LightSnow | WeatherType::Sandstorm
            ),
            WeatherSeason::Summer => !matches!(
                weather_type,
                WeatherType::HeavySnow | WeatherType::LightSnow
            ),
            WeatherSeason::Autumn => !matches!(
                weather_type,
                WeatherType::HeavySnow | WeatherType::LightSnow | WeatherType::Sandstorm
            ),
            WeatherSeason::Winter => !matches!(
                weather_type,
                WeatherType::Thunderstorm | WeatherType::Sandstorm | WeatherType::Rainbow
            ),
        }
    }

    /// 基于概率生成天气（Fallback）
    fn generate_weather_by_probability(&self, season: WeatherSeason) -> Weather {
        let probabilities = SeasonWeatherProbabilities::for_season(season);
        let weather_type = probabilities.random_weather();

        // 根据季节生成温度
        let (min_temp, max_temp) = Self::get_temperature_range(season);
        let mut rng = rand::rng();
        let temperature = rng.random_range(min_temp..=max_temp);

        tracing::info!(
            "Generated weather by probability: {:?}, temperature: {}°C for season {:?}",
            weather_type,
            temperature,
            season
        );
        Weather::new(weather_type, temperature)
    }

    /// 根据季节调整天气概率
    pub fn update_weather_with_season(&mut self, season: crate::game::garden::Season) {
        // 保存当前天气到历史
        self.history.push(self.current_weather.clone());

        if self.history.len() > 30 {
            self.history.remove(0);
        }

        let new_weather_type = Self::generate_weather_for_season(season);
        // 使用默认温度 20.0
        self.current_weather = Weather::new(new_weather_type, 20.0);
    }

    /// 根据季节生成天气
    fn generate_weather_for_season(_season: crate::game::garden::Season) -> WeatherType {
        WeatherType::Cloudy
    }

    /// 强制设置天气
    pub fn set_weather(&mut self, weather_type: WeatherType, temperature: f32) {
        self.history.push(self.current_weather.clone());
        self.current_weather = Weather::new(weather_type, temperature);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmConfig;
    use crate::game::llm::{Delta, LlmProvider, LlmRequest, LlmResponse};
    use async_trait::async_trait;
    use futures::Stream;
    use std::pin::Pin;

    /// Mock LLM Provider for testing
    struct MockLlmProvider {
        response: String,
    }

    impl MockLlmProvider {
        fn new(response: &str) -> Self {
            Self {
                response: response.to_string(),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn generate(&self, _request: LlmRequest) -> crate::error::Result<LlmResponse> {
            Ok(LlmResponse {
                content: self.response.clone(),
                total_tokens: 100,
                finish_reason: Some("stop".to_string()),
            })
        }

        async fn generate_stream(
            &self,
            _request: LlmRequest,
        ) -> crate::error::Result<Pin<Box<dyn Stream<Item = crate::error::Result<Delta>> + Send>>>
        {
            unimplemented!("Stream not used in tests")
        }
    }

    fn create_test_llm_manager(response: &str) -> Arc<LlmManager> {
        let mock_provider = Arc::new(MockLlmProvider::new(response));
        let config = LlmConfig {
            provider: "ollama".to_string(),
            model: "qwen3:1.7b".to_string(),
            base_url: "http://localhost".to_string(),
            port: 11434,
            timeout_seconds: 60,
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 1000,
        };
        Arc::new(LlmManager::new(mock_provider, config))
    }

    async fn create_test_weather_manager() -> WeatherManager {
        // 创建内存数据库用于测试
        let db_pool = Arc::new(
            crate::db::DbPool::new("sqlite::memory:")
                .await
                .expect("Failed to create test database"),
        );
        WeatherManager::new(db_pool)
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_success() {
        let manager = create_test_weather_manager().await;
        let llm_manager = create_test_llm_manager("天气: 晴朗\n温度: 28.5");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        assert!(result.is_ok());
        let (weather_type, temperature) = result.unwrap();
        assert_eq!(weather_type, WeatherType::Sunny);
        assert!((temperature - 28.5).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_chinese_colon() {
        let manager = create_test_weather_manager().await;
        let llm_manager = create_test_llm_manager("天气：多云\n温度：15.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Spring)
            .await;

        assert!(result.is_ok());
        let (weather_type, temperature) = result.unwrap();
        assert_eq!(weather_type, WeatherType::Cloudy);
        assert!((temperature - 15.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_missing_weather_type() {
        let manager = create_test_weather_manager().await;
        let llm_manager = create_test_llm_manager("温度: 20.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Spring)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::GameError::LlmError(_)));
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_missing_temperature() {
        let manager = create_test_weather_manager().await;
        let llm_manager = create_test_llm_manager("天气: 晴朗");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::GameError::LlmError(_)));
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_invalid_format() {
        let manager = create_test_weather_manager().await;
        let llm_manager = create_test_llm_manager("这是一个无效的响应格式");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_temperature_clamping_high() {
        let manager = create_test_weather_manager().await;
        // 夏季温度范围是 25-35，传入超出范围的温度 40
        let llm_manager = create_test_llm_manager("天气: 晴朗\n温度: 40.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        assert!(result.is_ok());
        let (_, temperature) = result.unwrap();
        // 温度应该被 clamp 到 35.0
        assert!((temperature - 35.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_temperature_clamping_low() {
        let manager = create_test_weather_manager().await;
        // 春季温度范围是 10-25，传入低于范围的温度 5
        let llm_manager = create_test_llm_manager("天气: 多云\n温度: 5.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Spring)
            .await;

        assert!(result.is_ok());
        let (_, temperature) = result.unwrap();
        // 温度应该被 clamp 到 10.0
        assert!((temperature - 10.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_inappropriate_weather_snow_in_summer() {
        let manager = create_test_weather_manager().await;
        // 夏季不应该有雪
        let llm_manager = create_test_llm_manager("天气: 大雪\n温度: -5.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::GameError::LlmError(_)));
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_inappropriate_weather_thunderstorm_in_winter() {
        let manager = create_test_weather_manager().await;
        // 冬季不应该有雷暴
        let llm_manager = create_test_llm_manager("天气: 雷暴\n温度: 5.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Winter)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_winter_snow_is_valid() {
        let manager = create_test_weather_manager().await;
        // 冬季可以有雪
        let llm_manager = create_test_llm_manager("天气: 小雪\n温度: -2.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Winter)
            .await;

        assert!(result.is_ok());
        let (weather_type, temperature) = result.unwrap();
        assert_eq!(weather_type, WeatherType::LightSnow);
        assert!((temperature - (-2.0_f32)).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_spring_rain_is_valid() {
        let manager = create_test_weather_manager().await;
        // 春季可以有雨
        let llm_manager = create_test_llm_manager("天气: 小雨\n温度: 15.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Spring)
            .await;

        assert!(result.is_ok());
        let (weather_type, _) = result.unwrap();
        assert_eq!(weather_type, WeatherType::LightRain);
    }

    #[tokio::test]
    async fn test_generate_weather_with_llm_autumn_fog_is_valid() {
        let manager = create_test_weather_manager().await;
        // 秋季可以有雾
        let llm_manager = create_test_llm_manager("天气: 雾\n温度: 12.0");

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Autumn)
            .await;

        assert!(result.is_ok());
        let (weather_type, _) = result.unwrap();
        assert_eq!(weather_type, WeatherType::Fog);
    }

    #[tokio::test]
    async fn test_parse_weather_type() {
        let manager = create_test_weather_manager().await;

        // 测试各种天气名称解析
        assert_eq!(
            manager.parse_weather_type("晴朗").unwrap(),
            WeatherType::Sunny
        );
        assert_eq!(
            manager.parse_weather_type("晴").unwrap(),
            WeatherType::Sunny
        );
        assert_eq!(
            manager.parse_weather_type("晴天").unwrap(),
            WeatherType::Sunny
        );
        assert_eq!(
            manager.parse_weather_type("多云").unwrap(),
            WeatherType::Cloudy
        );
        assert_eq!(
            manager.parse_weather_type("阴天").unwrap(),
            WeatherType::Overcast
        );
        assert_eq!(
            manager.parse_weather_type("阴").unwrap(),
            WeatherType::Overcast
        );
        assert_eq!(
            manager.parse_weather_type("小雨").unwrap(),
            WeatherType::LightRain
        );
        assert_eq!(
            manager.parse_weather_type("大雨").unwrap(),
            WeatherType::HeavyRain
        );
        assert_eq!(
            manager.parse_weather_type("雷暴").unwrap(),
            WeatherType::Thunderstorm
        );
        assert_eq!(
            manager.parse_weather_type("雷雨").unwrap(),
            WeatherType::Thunderstorm
        );
        assert_eq!(
            manager.parse_weather_type("小雪").unwrap(),
            WeatherType::LightSnow
        );
        assert_eq!(
            manager.parse_weather_type("大雪").unwrap(),
            WeatherType::HeavySnow
        );
        assert_eq!(manager.parse_weather_type("雾").unwrap(), WeatherType::Fog);
        assert_eq!(
            manager.parse_weather_type("风").unwrap(),
            WeatherType::Windy
        );
        assert_eq!(
            manager.parse_weather_type("大风").unwrap(),
            WeatherType::Windy
        );
        assert_eq!(
            manager.parse_weather_type("沙尘").unwrap(),
            WeatherType::Sandstorm
        );
        assert_eq!(
            manager.parse_weather_type("沙尘暴").unwrap(),
            WeatherType::Sandstorm
        );
        assert_eq!(
            manager.parse_weather_type("彩虹").unwrap(),
            WeatherType::Rainbow
        );

        // 测试无效输入
        assert!(manager.parse_weather_type("无效天气").is_err());
        assert!(manager.parse_weather_type("").is_err());
    }

    #[test]
    fn test_get_temperature_range() {
        assert_eq!(
            WeatherManager::get_temperature_range(WeatherSeason::Spring),
            (10.0, 25.0)
        );
        assert_eq!(
            WeatherManager::get_temperature_range(WeatherSeason::Summer),
            (25.0, 35.0)
        );
        assert_eq!(
            WeatherManager::get_temperature_range(WeatherSeason::Autumn),
            (10.0, 20.0)
        );
        assert_eq!(
            WeatherManager::get_temperature_range(WeatherSeason::Winter),
            (-5.0, 10.0)
        );
    }

    #[tokio::test]
    async fn test_is_weather_appropriate_for_season() {
        let manager = create_test_weather_manager().await;

        // 春季不应该有雪和沙尘
        assert!(
            manager
                .is_weather_appropriate_for_season(WeatherType::LightRain, WeatherSeason::Spring)
        );
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::LightSnow, WeatherSeason::Spring)
        );
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::HeavySnow, WeatherSeason::Spring)
        );
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::Sandstorm, WeatherSeason::Spring)
        );

        // 夏季不应该有雪
        assert!(
            manager.is_weather_appropriate_for_season(
                WeatherType::Thunderstorm,
                WeatherSeason::Summer
            )
        );
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::LightSnow, WeatherSeason::Summer)
        );

        // 秋季不应该有雪和沙尘
        assert!(manager.is_weather_appropriate_for_season(WeatherType::Fog, WeatherSeason::Autumn));
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::LightSnow, WeatherSeason::Autumn)
        );

        // 冬季不应该有雷暴、沙尘、彩虹
        assert!(
            manager
                .is_weather_appropriate_for_season(WeatherType::HeavySnow, WeatherSeason::Winter)
        );
        assert!(
            !manager.is_weather_appropriate_for_season(
                WeatherType::Thunderstorm,
                WeatherSeason::Winter
            )
        );
        assert!(
            !manager
                .is_weather_appropriate_for_season(WeatherType::Sandstorm, WeatherSeason::Winter)
        );
        assert!(
            !manager.is_weather_appropriate_for_season(WeatherType::Rainbow, WeatherSeason::Winter)
        );
    }

    #[test]
    fn test_weather_season_current_season() {
        // 春季: 3-5月
        // 2024-03-15 (春季)
        let spring_ts = 1710460800_i64;
        assert_eq!(
            WeatherSeason::current_season(spring_ts),
            WeatherSeason::Spring
        );

        // 2024-05-15 (春季)
        let spring_ts2 = 1715731200_i64;
        assert_eq!(
            WeatherSeason::current_season(spring_ts2),
            WeatherSeason::Spring
        );

        // 夏季: 6-8月
        // 2024-06-15 (夏季)
        let summer_ts = 1718409600_i64;
        assert_eq!(
            WeatherSeason::current_season(summer_ts),
            WeatherSeason::Summer
        );

        // 秋季: 9-11月
        // 2024-10-15 (秋季)
        let autumn_ts = 1728950400_i64;
        assert_eq!(
            WeatherSeason::current_season(autumn_ts),
            WeatherSeason::Autumn
        );

        // 冬季: 12-2月
        // 2024-12-15 (冬季)
        let winter_ts = 1734220800_i64;
        assert_eq!(
            WeatherSeason::current_season(winter_ts),
            WeatherSeason::Winter
        );

        // 2024-01-15 (冬季)
        let winter_ts2 = 1705276800_i64;
        assert_eq!(
            WeatherSeason::current_season(winter_ts2),
            WeatherSeason::Winter
        );
    }

    #[test]
    fn test_weather_type_name() {
        assert_eq!(WeatherType::Sunny.name(), "晴朗");
        assert_eq!(WeatherType::Cloudy.name(), "多云");
        assert_eq!(WeatherType::Overcast.name(), "阴天");
        assert_eq!(WeatherType::LightRain.name(), "小雨");
        assert_eq!(WeatherType::HeavyRain.name(), "大雨");
        assert_eq!(WeatherType::Thunderstorm.name(), "雷暴");
        assert_eq!(WeatherType::LightSnow.name(), "小雪");
        assert_eq!(WeatherType::HeavySnow.name(), "大雪");
        assert_eq!(WeatherType::Fog.name(), "雾");
        assert_eq!(WeatherType::Windy.name(), "风");
        assert_eq!(WeatherType::Sandstorm.name(), "沙尘");
        assert_eq!(WeatherType::Rainbow.name(), "彩虹");
    }

    #[test]
    fn test_weather_season_name() {
        assert_eq!(WeatherSeason::Spring.name(), "春季");
        assert_eq!(WeatherSeason::Summer.name(), "夏季");
        assert_eq!(WeatherSeason::Autumn.name(), "秋冬");
        assert_eq!(WeatherSeason::Winter.name(), "冬季");
    }

    // ==================== 集成测试：使用真实 Ollama ====================

    /// 创建连接本地 Ollama 的 LLM 管理器
    fn create_real_llm_manager() -> Option<Arc<LlmManager>> {
        let config = LlmConfig {
            provider: "ollama".to_string(),
            model: "qwen3:1.7b".to_string(),
            base_url: "http://localhost".to_string(),
            port: 11434,
            timeout_seconds: 60,
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 100,
        };
        crate::game::llm::create_llm_manager(config).ok()
    }

    /// 集成测试：使用真实 Ollama 生成春季天气
    #[tokio::test]
    async fn test_real_ollama_generate_spring_weather() {
        let Some(llm_manager) = create_real_llm_manager() else {
            println!("跳过测试: Ollama 服务不可用");
            return;
        };

        let manager = create_test_weather_manager().await;

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Spring)
            .await;

        match &result {
            Ok((weather_type, temperature)) => {
                println!(
                    "春季天气生成成功: {:?}, 温度: {}°C",
                    weather_type, temperature
                );
                // 验证温度在春季范围内
                assert!(*temperature >= 10.0 && *temperature <= 25.0);
                // 验证天气类型适合春季
                assert!(
                    manager.is_weather_appropriate_for_season(*weather_type, WeatherSeason::Spring)
                );
            }
            Err(e) => {
                println!("春季天气生成失败: {:?}", e);
            }
        }

        assert!(result.is_ok());
    }

    /// 集成测试：使用真实 Ollama 生成夏季天气
    #[tokio::test]
    async fn test_real_ollama_generate_summer_weather() {
        let Some(llm_manager) = create_real_llm_manager() else {
            println!("跳过测试: Ollama 服务不可用");
            return;
        };

        let manager = create_test_weather_manager().await;

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Summer)
            .await;

        match &result {
            Ok((weather_type, temperature)) => {
                println!(
                    "夏季天气生成成功: {:?}, 温度: {}°C",
                    weather_type, temperature
                );
                // 验证温度在夏季范围内
                assert!(*temperature >= 25.0 && *temperature <= 35.0);
                // 验证天气类型适合夏季
                assert!(
                    manager.is_weather_appropriate_for_season(*weather_type, WeatherSeason::Summer)
                );
            }
            Err(e) => {
                println!("夏季天气生成失败: {:?}", e);
            }
        }

        assert!(result.is_ok());
    }

    /// 集成测试：使用真实 Ollama 生成秋季天气
    #[tokio::test]
    async fn test_real_ollama_generate_autumn_weather() {
        let Some(llm_manager) = create_real_llm_manager() else {
            println!("跳过测试: Ollama 服务不可用");
            return;
        };

        let manager = create_test_weather_manager().await;

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Autumn)
            .await;

        match &result {
            Ok((weather_type, temperature)) => {
                println!(
                    "秋季天气生成成功: {:?}, 温度: {}°C",
                    weather_type, temperature
                );
                // 验证温度在秋季范围内
                assert!(*temperature >= 10.0 && *temperature <= 20.0);
                // 验证天气类型适合秋季
                assert!(
                    manager.is_weather_appropriate_for_season(*weather_type, WeatherSeason::Autumn)
                );
            }
            Err(e) => {
                println!("秋季天气生成失败: {:?}", e);
            }
        }

        assert!(result.is_ok());
    }

    /// 集成测试：使用真实 Ollama 生成冬季天气
    #[tokio::test]
    async fn test_real_ollama_generate_winter_weather() {
        let Some(llm_manager) = create_real_llm_manager() else {
            println!("跳过测试: Ollama 服务不可用");
            return;
        };

        let manager = create_test_weather_manager().await;

        let result = manager
            .generate_weather_with_llm(&llm_manager, WeatherSeason::Winter)
            .await;

        match &result {
            Ok((weather_type, temperature)) => {
                println!(
                    "冬季天气生成成功: {:?}, 温度: {}°C",
                    weather_type, temperature
                );
                // 验证温度在冬季范围内
                assert!(*temperature >= -5.0 && *temperature <= 10.0);
                // 验证天气类型适合冬季
                assert!(
                    manager.is_weather_appropriate_for_season(*weather_type, WeatherSeason::Winter)
                );
            }
            Err(e) => {
                println!("冬季天气生成失败: {:?}", e);
            }
        }

        assert!(result.is_ok());
    }
}
