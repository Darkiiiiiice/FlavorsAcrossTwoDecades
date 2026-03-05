//! 存档数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 存档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    /// 存档ID
    pub id: Uuid,
    /// 存档名称
    pub name: String,
    /// 玩家名称
    pub player_name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后游玩时间
    pub last_played: DateTime<Utc>,
    /// 游玩时长（秒）
    pub play_time_seconds: u64,
    /// 当前章节
    pub chapter: u32,
}

impl Save {
    /// 创建新存档
    pub fn new(name: String, player_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            player_name,
            created_at: now,
            last_played: now,
            play_time_seconds: 0,
            chapter: 1,
        }
    }
}

/// 创建存档请求
#[derive(Debug, Deserialize)]
pub struct CreateSaveRequest {
    /// 存档名称
    pub name: String,
    /// 玩家名称
    pub player_name: String,
}

/// 更新存档请求
#[derive(Debug, Deserialize)]
pub struct UpdateSaveRequest {
    /// 存档名称
    pub name: Option<String>,
    /// 玩家名称
    pub player_name: Option<String>,
}
