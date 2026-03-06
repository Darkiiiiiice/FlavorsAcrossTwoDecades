//! 时间系统与通信延迟

use chrono::{DateTime, FixedOffset, Utc};

/// 时间系统
pub struct TimeSystem {
    /// 地球时区（东八区）
    earth_timezone: FixedOffset,
    /// 测试模式加速倍率（正常为1，测试时可设为10）
    time_scale: i64,
    /// 是否启用加速模式
    accelerated_mode: bool,
    /// 游戏开始时间
    start_time: i64,
    /// 游戏内地球时间
    timestamp: i64,
}

impl TimeSystem {
    /// 创建新的时间系统
    pub fn new() -> Self {
        let now = Utc::now().timestamp();
        Self {
            earth_timezone: FixedOffset::east_opt(8 * 3600).unwrap(), // UTC+8
            time_scale: 1,
            accelerated_mode: false,
            start_time: now,
            timestamp: now,
        }
    }

    /// 获取当前地球时间（东八区）
    pub fn earth_time(&self) -> DateTime<FixedOffset> {
        Utc::now().with_timezone(&self.earth_timezone)
    }

    /// 切换加速模式
    pub fn toggle_accelerated_mode(&mut self, enabled: bool) {
        self.accelerated_mode = enabled;
        self.time_scale = if enabled { 10 } else { 1 };
    }

    /// 计算游戏内经过的时间（考虑加速）
    pub fn elapsed_game_time(&self, real_seconds: u64) -> u64 {
        real_seconds * self.time_scale as u64
    }

    /// 获取时间系统的起始时间戳
    pub fn start_time(&self) -> i64 {
        self.start_time
    }

    /// 获取当前时间戳
    pub fn current_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// 每帧更新
    pub fn tick(&mut self) {
        // 时间系统的tick可以用于更新内部状态
        // 目前暂时为空，未来可以添加更多逻辑
        if self.accelerated_mode {
            self.timestamp += self.time_scale;
        } else {
            self.timestamp += 1;
        }
    }
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 火星-地球通信延迟计算
pub struct CommunicationDelay {
    /// 基础物理延迟（秒）
    base_delay_seconds: u32,
    /// 模块附加延迟（秒）
    module_delay_seconds: u32,
}

impl CommunicationDelay {
    /// 创建新的通信延迟计算器
    pub fn new(base_delay: u32) -> Self {
        Self {
            base_delay_seconds: base_delay,
            module_delay_seconds: 45, // 默认模块延迟
        }
    }

    /// 计算总延迟
    pub fn total_delay(&self) -> u32 {
        self.base_delay_seconds + self.module_delay_seconds
    }

    /// 根据模块等级更新延迟
    pub fn update_from_module(&mut self, communication_level: u32) {
        self.module_delay_seconds = match communication_level {
            1 => 45,
            2 => 40,
            3 => 35,
            4 => 30,
            5 => 25,
            6 => 20,
            7 => 15,
            8 => 10,
            9 => 5,
            10 => 1,
            _ => 45,
        };
    }
}

impl Default for CommunicationDelay {
    fn default() -> Self {
        Self::new(10) // 默认基础延迟 10秒
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_system() {
        let time_system = TimeSystem::new();
        let earth_time = time_system.earth_time();
        assert_eq!(earth_time.timezone().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn test_communication_delay() {
        let mut delay = CommunicationDelay::new(10);
        assert_eq!(delay.total_delay(), 55); // 10 + 45

        delay.update_from_module(5);
        assert_eq!(delay.total_delay(), 35); // 10 + 25

        delay.update_from_module(10);
        assert_eq!(delay.total_delay(), 11); // 10 + 1
    }
}
