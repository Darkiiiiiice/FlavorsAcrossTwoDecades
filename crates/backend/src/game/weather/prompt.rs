//! 天气生成提示词模块
//!
//! 统一管理天气生成相关的 LLM 提示词

use super::weather::{WeatherSeason, WeatherType};
use crate::utils::get_datetime_from_timestamp;

/// 天气生成提示词构建器
pub struct WeatherPromptBuilder {
    /// 当前时间戳
    timestamp: i64,
    /// 季节
    season: WeatherSeason,
    /// 天气概率列表
    weather_probs: Vec<(WeatherType, u32)>,
    /// 温度范围
    temp_range: (f32, f32),
    /// 历史天气记录
    history: Vec<String>,
}

impl WeatherPromptBuilder {
    /// 创建新的提示词构建器
    pub fn new(timestamp: i64, season: WeatherSeason) -> Self {
        Self {
            timestamp,
            season,
            weather_probs: Vec::new(),
            temp_range: (0.0, 0.0),
            history: Vec::new(),
        }
    }

    /// 设置天气概率
    pub fn with_weather_probs(mut self, probs: Vec<(WeatherType, u32)>) -> Self {
        self.weather_probs = probs;
        self
    }

    /// 设置温度范围
    pub fn with_temp_range(mut self, min: f32, max: f32) -> Self {
        self.temp_range = (min, max);
        self
    }

    /// 设置历史天气记录
    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }

    /// 构建系统提示词
    pub fn build_system_prompt(&self) -> &'static str {
        r#"你是一个游戏天气模拟系统。请根据当前时间、季节、天气概率分布和最近一段时间内的天气历史，为当前时刻选择一个天气类型和温度。

输出格式要求（必须严格遵守）：
Weather: <天气名称>
Temperature: <温度数值>

规则：
1. 参考最近天气历史，让天气变化有连续性（如连续一段时间晴天后可能转阴或下雨）
2. 严格按照给定的概率分布选择天气，概率越高的天气越可能被选中
3. 温度必须在给定的温度范围内，保留一位小数
4. 温度变化应该平缓，参考历史温度，避免剧烈波动
5. 只输出两行，不要有任何其他文字、标点或解释、不要有单位
6. 不要使用 markdown 格式"#
    }

    /// 构建用户消息
    pub fn build_user_message(&self) -> String {
        let (season_name, season_desc) = self.season_info();
        let current_time = self.format_current_time();
        let weather_probs_str = self.format_weather_probs();
        let temp_range_str = self.format_temp_range();
        let history_str = self.format_history();

        format!(
            r#"当前时间：{}

当前季节：{}（{}）

可选天气类型及其出现概率：
{}

温度范围：{}°C

最近天气记录：
{}

请根据以上信息，为当前时刻生成天气和温度。"#,
            current_time, season_name, season_desc, weather_probs_str, temp_range_str, history_str
        )
    }

    /// 格式化当前时间
    fn format_current_time(&self) -> String {
        let dt = get_datetime_from_timestamp(self.timestamp);
        dt.datetime_string()
    }

    /// 获取季节信息
    fn season_info(&self) -> (&'static str, &'static str) {
        match self.season {
            WeatherSeason::Spring => ("春季", "3-5月，温暖多雨，万物复苏"),
            WeatherSeason::Summer => ("夏季", "6-8月，炎热，多雷雨"),
            WeatherSeason::Autumn => ("秋季", "9-11月，凉爽，多雾"),
            WeatherSeason::Winter => ("冬季", "12-2月，寒冷，可能降雪"),
        }
    }

    /// 格式化天气概率
    fn format_weather_probs(&self) -> String {
        let total: u32 = self.weather_probs.iter().map(|(_, w)| w).sum();
        self.weather_probs
            .iter()
            .map(|(weather_type, weight)| {
                let percent = if total > 0 {
                    (*weight as f32 / total as f32 * 100.0) as u32
                } else {
                    0
                };
                format!("- {}: {}%", weather_type.name(), percent)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 格式化温度范围
    fn format_temp_range(&self) -> String {
        format!("{:.0}-{:.0}", self.temp_range.0, self.temp_range.1)
    }

    /// 格式化历史记录
    fn format_history(&self) -> String {
        if self.history.is_empty() {
            "暂无历史记录".to_string()
        } else {
            self.history.join("\n")
        }
    }
}

/// 季节温度范围
pub fn get_season_temperature_range(season: WeatherSeason) -> (f32, f32) {
    match season {
        WeatherSeason::Spring => (10.0, 25.0),
        WeatherSeason::Summer => (25.0, 35.0),
        WeatherSeason::Autumn => (10.0, 20.0),
        WeatherSeason::Winter => (-5.0, 10.0),
    }
}

/// 季节描述
pub fn get_season_description(season: WeatherSeason) -> (&'static str, &'static str) {
    match season {
        WeatherSeason::Spring => ("春季", "3-5月，温暖多雨，万物复苏"),
        WeatherSeason::Summer => ("夏季", "6-8月，炎热，多雷雨"),
        WeatherSeason::Autumn => ("秋季", "9-11月，凉爽，多雾"),
        WeatherSeason::Winter => ("冬季", "12-2月，寒冷，可能降雪"),
    }
}
