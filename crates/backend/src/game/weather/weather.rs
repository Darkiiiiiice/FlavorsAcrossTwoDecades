//! 天气定义

use std::sync::Arc;

use chrono::{DateTime, Utc};
use rand::RngExt;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    db::{DbPool, models::Weather as DbWeather, repositories::weather::WeatherRepository},
    game::{LlmManager, WeatherEffect},
    utils::get_month,
};

const MAX_HISTORY: usize = 120;

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

/// 后台天气生成任务的结果
type WeatherGenerationResult = (i64, Weather);

/// 异步生成天气（独立函数，用于后台任务）
async fn generate_weather_async(
    llm_manager: Option<&Arc<LlmManager>>,
    timestamp: i64,
    history: Vec<Weather>,
    season: WeatherSeason,
) -> (WeatherType, f32) {
    // 如果有 LLM 管理器，尝试使用 LLM 生成
    if let Some(llm) = llm_manager {
        match generate_weather_with_llm_standalone(llm, timestamp, &history, season).await {
            Ok(result) => {
                tracing::info!(
                    "LLM generated weather: {:?}, temperature: {}°C for season {:?}",
                    result.0,
                    result.1,
                    season
                );
                return result;
            }
            Err(e) => {
                tracing::warn!("LLM weather generation failed: {}, using fallback", e);
            }
        }
    }

    // Fallback: 使用概率生成
    generate_weather_by_probability_standalone(season)
}

/// 使用概率生成天气（独立函数）
fn generate_weather_by_probability_standalone(season: WeatherSeason) -> (WeatherType, f32) {
    let probabilities = SeasonWeatherProbabilities::for_season(season);
    let weather_type = probabilities.random_weather();

    // 根据季节生成温度
    let (min_temp, max_temp) = get_temperature_range_standalone(season);
    let mut rng = rand::rng();
    let temperature = rng.random_range(min_temp..=max_temp);

    tracing::info!(
        "Generated weather by probability: {:?}, temperature: {}°C for season {:?}",
        weather_type,
        temperature,
        season
    );
    (weather_type, temperature)
}

/// 获取季节的温度范围（独立函数）
fn get_temperature_range_standalone(season: WeatherSeason) -> (f32, f32) {
    super::prompt::get_season_temperature_range(season)
}

/// 使用 LLM 生成天气（独立函数）
async fn generate_weather_with_llm_standalone(
    llm_manager: &Arc<LlmManager>,
    timestamp: i64,
    history: &[Weather],
    season: WeatherSeason,
) -> crate::error::Result<(WeatherType, f32)> {
    use super::prompt::{WeatherPromptBuilder, get_season_temperature_range};

    // 获取当前季节的天气概率配置
    let probabilities = SeasonWeatherProbabilities::for_season(season);
    let (min_temp, max_temp) = get_season_temperature_range(season);

    // 构建历史天气记录
    let history_vec: Vec<String> = history
        .iter()
        .rev()
        .take(MAX_HISTORY)
        .enumerate()
        .map(|(i, w)| {
            format!(
                "{}分钟前: {}，{:.1}°C",
                i,
                w.weather_type.name(),
                w.temperature
            )
        })
        .collect();

    // 使用提示词构建器
    let builder = WeatherPromptBuilder::new(timestamp, season)
        .with_weather_probs(probabilities.weights.clone())
        .with_temp_range(min_temp, max_temp)
        .with_history(history_vec);

    let system_prompt = builder.build_system_prompt();
    let user_message = builder.build_user_message();

    tracing::info!("generating weather system prompt: \n{}", system_prompt);
    tracing::info!("generating weather user prompt: \n{}", user_message);

    let response = llm_manager
        .generate_text(system_prompt.to_string(), user_message)
        .await?;

    tracing::info!("LLM returned response for weather: \n{}", response);

    // 使用正则表达式解析 LLM 返回的天气类型和温度
    // 正则可以匹配 LLM 返回的多余内容，只提取关键信息
    let weather_type = Regex::new(r"Weather[：:]\s*(\S+)")
        .ok()
        .and_then(|re| re.captures(&response))
        .and_then(|caps| caps.get(1))
        .and_then(|m| parse_weather_type_standalone(m.as_str()));

    let temperature = Regex::new(r"Temperature[：:]\s*(-?\d+\.?\d*)")
        .ok()
        .and_then(|re| re.captures(&response))
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<f32>().ok());

    let weather_type = weather_type.ok_or_else(|| {
        crate::error::GameError::LlmError(format!(
            "Missing or invalid weather type in response: {}",
            response
        ))
    })?;
    let temperature = temperature.ok_or_else(|| {
        crate::error::GameError::LlmError(format!(
            "Missing or invalid temperature in response: {}",
            response
        ))
    })?;

    // 验证温度是否在合理范围内
    let (min_temp, max_temp) = get_temperature_range_standalone(season);
    let temperature = temperature.clamp(min_temp, max_temp);

    tracing::info!(
        "LLM generated weather: {:?}, temperature: {}°C for season {:?}",
        weather_type,
        temperature,
        season
    );

    // 验证天气类型是否适合当前季节
    if is_weather_appropriate_for_season_standalone(weather_type, season) {
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

/// 解析天气类型字符串（独立函数）
fn parse_weather_type_standalone(name: &str) -> Option<WeatherType> {
    match name.trim() {
        "晴朗" | "晴" | "晴天" => Some(WeatherType::Sunny),
        "多云" => Some(WeatherType::Cloudy),
        "阴天" | "阴" => Some(WeatherType::Overcast),
        "小雨" => Some(WeatherType::LightRain),
        "大雨" => Some(WeatherType::HeavyRain),
        "雷暴" | "雷雨" => Some(WeatherType::Thunderstorm),
        "小雪" => Some(WeatherType::LightSnow),
        "大雪" => Some(WeatherType::HeavySnow),
        "雾" => Some(WeatherType::Fog),
        "风" | "大风" => Some(WeatherType::Windy),
        "沙尘" | "沙尘暴" => Some(WeatherType::Sandstorm),
        "彩虹" => Some(WeatherType::Rainbow),
        _ => None,
    }
}

/// 检查天气类型是否适合当前季节（独立函数）
fn is_weather_appropriate_for_season_standalone(
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

/// 天气管理器
#[derive(Debug)]
pub struct WeatherManager {
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// 当前天气
    pub current_weather: Weather,
    /// 历史天气记录
    pub history: Vec<Weather>,
    /// LLM 管理器（可选）
    llm_manager: Option<Arc<LlmManager>>,
    /// 后台生成任务句柄
    generation_task: Option<tokio::task::JoinHandle<WeatherGenerationResult>>,
    /// 待生成的时间戳
    pending_timestamp: Option<i64>,
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
            generation_task: None,
            pending_timestamp: None,
        }
    }

    /// 设置 LLM 管理器
    pub fn with_llm(mut self, llm_manager: Arc<LlmManager>) -> Self {
        self.llm_manager = Some(llm_manager);
        self
    }

    /// 更新天气（每秒调用）
    pub async fn update_weather(&mut self, timestamp: i64) {
        let weather_repo = WeatherRepository::new(self.db_pool.pool().clone());
        if self.history.len() == 0 {
            tracing::info!("Updating weather history from database...");
            let latest_weather = weather_repo.find_newest(MAX_HISTORY as i32).await;
            if let Ok(mut latest_weathers) = latest_weather {
                latest_weathers.reverse();
                for weather in latest_weathers {
                    tracing::info!("Adding weather: {:?}", weather);
                    self.history.push(weather.into());
                }
            }
        }

        // 1. 检查后台生成任务是否完成
        if let Some(task) = self.generation_task.take() {
            if task.is_finished() {
                match task.await {
                    Ok((ts, new_weather)) => {
                        tracing::info!(
                            "Background weather generation completed: {:?}, {}°C, exec: {}",
                            new_weather.weather_type,
                            new_weather.temperature,
                            ts,
                        );

                        // 保存到数据库
                        if let Err(e) = weather_repo.create(&new_weather.clone().into()).await {
                            tracing::error!("Failed to save new weather: {:?}", e);
                        } else {
                            self.current_weather = new_weather;
                            // 保存当前天气到历史
                            self.history.push(self.current_weather.clone());

                            // 保留最近30天的历史
                            if self.history.len() > MAX_HISTORY {
                                self.history.remove(0);
                            }
                        }
                        self.pending_timestamp = None;
                    }
                    Err(e) => {
                        tracing::error!("Background weather generation failed: {:?}", e);
                        // 使用 fallback 生成
                        if let Some(ts) = self.pending_timestamp.take() {
                            let weather = self
                                .generate_weather_by_probability(WeatherSeason::current_season(ts));
                            self.current_weather = weather;
                        }
                    }
                }
            } else {
                // 任务仍在运行，放回句柄
                self.generation_task = Some(task);
            }
        }

        // 2. 检查数据库中的最新天气
        if self.generation_task.is_none() {
            let weather_repo = WeatherRepository::new(self.db_pool.pool().clone());
            let latest_weather = weather_repo.find_latest().await;

            let mut expired = false;

            match latest_weather {
                Ok(Some(latest_weather)) => {
                    tracing::debug!("latest weather: {:?}", latest_weather);
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

            // 3. 如果天气过期且没有正在进行的生成任务，启动后台生成
            if expired {
                tracing::info!("Starting background weather generation");
                self.start_background_generation(timestamp);
            }
        }
    }

    /// 启动后台天气生成任务
    fn start_background_generation(&mut self, timestamp: i64) {
        let llm_manager = self.llm_manager.clone();
        let history = self.history.clone();

        self.pending_timestamp = Some(timestamp);

        let task = tokio::spawn(async move {
            let season = WeatherSeason::current_season(timestamp);
            let weather =
                generate_weather_async(llm_manager.as_ref(), timestamp, history, season).await;
            let mut weather = Weather::new(weather.0, weather.1);
            weather.id = timestamp;
            (timestamp, weather)
        });

        self.generation_task = Some(task);
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
