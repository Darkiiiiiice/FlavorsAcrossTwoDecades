//! 时间工具函数

use chrono::{Datelike, TimeZone, Timelike, Utc};

/// 日期时间组件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeComponents {
    /// 年
    pub year: i32,
    /// 月 (1-12)
    pub month: u32,
    /// 日 (1-31)
    pub day: u32,
    /// 时 (0-23)
    pub hour: u32,
    /// 分 (0-59)
    pub minute: u32,
    /// 秒 (0-59)
    pub second: u32,
}

impl DateTimeComponents {
    /// 从时间戳创建日期时间组件
    pub fn from_timestamp(timestamp: i64) -> Self {
        let date = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .unwrap_or_else(Utc::now);

        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
            hour: date.hour(),
            minute: date.minute(),
            second: date.second(),
        }
    }

    /// 获取日期字符串 (YYYY-MM-DD)
    pub fn date_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// 获取时间字符串 (HH:MM:SS)
    pub fn time_string(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    /// 获取完整日期时间字符串 (YYYY-MM-DD HH:MM:SS)
    pub fn datetime_string(&self) -> String {
        format!("{} {}", self.date_string(), self.time_string())
    }
}

/// 根据时间戳获取年月日、时分秒
///
/// # Arguments
/// * `timestamp` - Unix 时间戳（秒）
///
/// # Returns
/// 返回 `DateTimeComponents` 结构体，包含年、月、日、时、分、秒
///
/// # Example
/// ```
/// use flavors_backend::utils::get_datetime_from_timestamp;
///
/// let components = get_datetime_from_timestamp(1709510400);
/// println!("年: {}, 月: {}, 日: {}", components.year, components.month, components.day);
/// println!("时: {}, 分: {}, 秒: {}", components.hour, components.minute, components.second);
/// ```
pub fn get_datetime_from_timestamp(timestamp: i64) -> DateTimeComponents {
    DateTimeComponents::from_timestamp(timestamp)
}

/// 根据时间戳获取年份
pub fn get_year(timestamp: i64) -> i32 {
    get_datetime_from_timestamp(timestamp).year
}

/// 根据时间戳获取月份 (1-12)
pub fn get_month(timestamp: i64) -> u32 {
    get_datetime_from_timestamp(timestamp).month
}

/// 根据时间戳获取日期 (1-31)
pub fn get_day(timestamp: i64) -> u32 {
    get_datetime_from_timestamp(timestamp).day
}

/// 根据时间戳获取小时 (0-23)
pub fn get_hour(timestamp: i64) -> u32 {
    get_datetime_from_timestamp(timestamp).hour
}

/// 根据时间戳获取分钟 (0-59)
pub fn get_minute(timestamp: i64) -> u32 {
    get_datetime_from_timestamp(timestamp).minute
}

/// 根据时间戳获取秒 (0-59)
pub fn get_second(timestamp: i64) -> u32 {
    get_datetime_from_timestamp(timestamp).second
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_datetime_from_timestamp() {
        // 2024-03-04 00:00:00 UTC
        let timestamp = 1709510400_i64;
        let components = get_datetime_from_timestamp(timestamp);

        assert_eq!(components.year, 2024);
        assert_eq!(components.month, 3);
        assert_eq!(components.day, 4);
        assert_eq!(components.hour, 0);
        assert_eq!(components.minute, 0);
        assert_eq!(components.second, 0);
    }

    #[test]
    fn test_datetime_string() {
        let components = DateTimeComponents {
            year: 2024,
            month: 3,
            day: 4,
            hour: 12,
            minute: 30,
            second: 45,
        };

        assert_eq!(components.date_string(), "2024-03-04");
        assert_eq!(components.time_string(), "12:30:45");
        assert_eq!(components.datetime_string(), "2024-03-04 12:30:45");
    }
}
