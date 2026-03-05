//! 旅行状态仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::travel::Travel;
use crate::error::{DatabaseError, GameError, GameResult};
use crate::game::travel::TravelStatus;

/// 旅行状态仓储
pub struct TravelRepository {
    pool: SqlitePool,
}

impl TravelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建旅行记录
    pub async fn create(&self, travel: &Travel) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO travels (id, save_id, destination, started_at, expected_return, status, rewards)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(travel.id.to_string())
        .bind(travel.save_id.to_string())
        .bind(&travel.destination)
        .bind(travel.started_at.to_rfc3339())
        .bind(travel.expected_return.to_rfc3339())
        .bind(status_to_string(&travel.status))
        .bind(&travel.rewards)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找旅行
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<Travel>> {
        let row = sqlx::query_as::<_, TravelRow>(
            r#"SELECT id, save_id, destination, started_at, expected_return, status, rewards
               FROM travels WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_travel()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有旅行记录
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<Travel>> {
        let rows = sqlx::query_as::<_, TravelRow>(
            r#"SELECT id, save_id, destination, started_at, expected_return, status, rewards
               FROM travels WHERE save_id = ? ORDER BY started_at DESC"#,
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_travel()).collect()
    }

    /// 更新旅行状态
    pub async fn update(&self, travel: &Travel) -> GameResult<()> {
        sqlx::query(r#"UPDATE travels SET status = ?, rewards = ? WHERE id = ?"#)
            .bind(status_to_string(&travel.status))
            .bind(&travel.rewards)
            .bind(travel.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

fn status_to_string(status: &TravelStatus) -> String {
    match status {
        TravelStatus::Preparing => "preparing".to_string(),
        TravelStatus::InProgress => "in_progress".to_string(),
        TravelStatus::Completed => "completed".to_string(),
        TravelStatus::Cancelled => "cancelled".to_string(),
    }
}

fn string_to_status(s: &str) -> GameResult<TravelStatus> {
    match s {
        "preparing" => Ok(TravelStatus::Preparing),
        "in_progress" => Ok(TravelStatus::InProgress),
        "completed" => Ok(TravelStatus::Completed),
        "cancelled" => Ok(TravelStatus::Cancelled),
        _ => Err(GameError::Validation {
            details: format!("Invalid travel status: {}", s),
        }),
    }
}

#[derive(sqlx::FromRow)]
struct TravelRow {
    id: String,
    save_id: String,
    destination: String,
    started_at: String,
    expected_return: String,
    status: String,
    rewards: Option<String>,
}

impl TravelRow {
    fn into_travel(self) -> GameResult<Travel> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        let started_at = DateTime::parse_from_rfc3339(&self.started_at)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid started_at: {}", e),
            })?
            .with_timezone(&Utc);

        let expected_return = DateTime::parse_from_rfc3339(&self.expected_return)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid expected_return: {}", e),
            })?
            .with_timezone(&Utc);

        let status = string_to_status(&self.status)?;

        Ok(Travel {
            id,
            save_id,
            destination: self.destination,
            started_at,
            expected_return,
            status,
            rewards: self.rewards,
        })
    }
}
