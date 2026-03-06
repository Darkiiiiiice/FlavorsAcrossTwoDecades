//! 天气仓储

use chrono::Utc;
use sqlx::SqlitePool;

use crate::{
    db::models::Weather,
    error::{DatabaseError, GameError, GameResult},
    game::WeatherType,
};

/// 天气仓储
pub struct WeatherRepository {
    pool: SqlitePool,
}

impl WeatherRepository {
    /// 创建新的天气仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建天气记录
    pub async fn create(&self, weather: &Weather) -> GameResult<()> {
        let now = Utc::now().timestamp();
        let weather_type: i64 = weather.r#type.into();

        sqlx::query(
            r#"INSERT INTO weather (id, type, temperature, duration, created_at)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(weather.id)
        .bind(weather_type)
        .bind(weather.temperature)
        .bind(weather.duration)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找天气
    pub async fn find_by_id(&self, id: i64) -> GameResult<Option<Weather>> {
        let row = sqlx::query_as::<_, WeatherRow>(
            r#"SELECT id, type, temperature, duration, created_at
               FROM weather WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_weather()?)),
            None => Ok(None),
        }
    }

    pub async fn find_all(&self) -> GameResult<Vec<Weather>> {
        let rows = sqlx::query_as::<_, WeatherRow>(
            r#"SELECT id, type, temperature, duration, created_at
               FROM weather ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|r| r.into_weather()).collect()
    }

    /// 获取存档的最新天气
    pub async fn find_latest(&self) -> GameResult<Option<Weather>> {
        let row = sqlx::query_as::<_, WeatherRow>(
            r#"SELECT id, type, temperature, duration, created_at
               FROM weather ORDER BY created_at DESC LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_weather()?)),
            None => Ok(None),
        }
    }

    /// 更新天气
    pub async fn update(&self, weather: &Weather) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE weather SET type = ?, temperature = ?, duration = ?
               WHERE id = ?"#,
        )
        .bind(weather.r#type as i64)
        .bind(weather.temperature)
        .bind(weather.duration)
        .bind(weather.id)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除天气
    pub async fn delete(&self, id: i64) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM weather WHERE id = ?"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    pub async fn delete_all(&self) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM weather"#)
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

/// 天气数据库行
#[derive(sqlx::FromRow)]
struct WeatherRow {
    id: i64,
    r#type: i64,
    temperature: f64,
    duration: i64,
    created_at: i64,
}

impl WeatherRow {
    /// 转换为领域模型
    fn into_weather(self) -> GameResult<Weather> {
        let weather_type = WeatherType::from(self.r#type);
        Ok(Weather {
            id: self.id,
            r#type: weather_type,
            temperature: self.temperature,
            duration: self.duration,
            created_at: self.created_at,
        })
    }
}
