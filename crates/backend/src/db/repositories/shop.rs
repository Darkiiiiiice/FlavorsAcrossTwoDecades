//! 小馆状态仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::shop::{FacilityRecord, ShopState};
use crate::error::{DatabaseError, GameError, GameResult};

/// 小馆状态仓储
pub struct ShopRepository {
    pool: SqlitePool,
}

impl ShopRepository {
    /// 创建新的小馆仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建小馆状态
    pub async fn create(&self, state: &ShopState) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO shop_states (save_id, name, funds, reputation, restaurant_level,
               kitchen_level, backyard_level, workshop_level)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(state.save_id.to_string())
        .bind(&state.name)
        .bind(state.funds as i64)
        .bind(state.reputation as f64)
        .bind(state.restaurant_level as i64)
        .bind(state.kitchen_level as i64)
        .bind(state.backyard_level as i64)
        .bind(state.workshop_level as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据存档ID查找小馆状态
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Option<ShopState>> {
        let row = sqlx::query_as::<_, ShopStateRow>(
            r#"SELECT save_id, name, funds, reputation, restaurant_level,
               kitchen_level, backyard_level, workshop_level
               FROM shop_states WHERE save_id = ?"#,
        )
        .bind(save_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_state()?)),
            None => Ok(None),
        }
    }

    /// 更新小馆状态
    pub async fn update(&self, state: &ShopState) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE shop_states SET name = ?, funds = ?, reputation = ?,
               restaurant_level = ?, kitchen_level = ?, backyard_level = ?, workshop_level = ?
               WHERE save_id = ?"#,
        )
        .bind(&state.name)
        .bind(state.funds as i64)
        .bind(state.reputation as f64)
        .bind(state.restaurant_level as i64)
        .bind(state.kitchen_level as i64)
        .bind(state.backyard_level as i64)
        .bind(state.workshop_level as i64)
        .bind(state.save_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 创建或更新小馆状态
    pub async fn upsert(&self, state: &ShopState) -> GameResult<()> {
        let existing = self.find_by_save_id(state.save_id).await?;
        if existing.is_some() {
            self.update(state).await
        } else {
            self.create(state).await
        }
    }

    // ========== 设施管理 ==========

    /// 创建设施记录
    pub async fn create_facility(&self, facility: &FacilityRecord) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO facilities (id, save_id, zone, name, level, condition, upgrade_progress)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(facility.id.to_string())
        .bind(facility.save_id.to_string())
        .bind(&facility.zone)
        .bind(&facility.name)
        .bind(facility.level as i64)
        .bind(facility.condition as i64)
        .bind(&facility.upgrade_progress)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 获取存档的所有设施
    pub async fn find_facilities(&self, save_id: Uuid) -> GameResult<Vec<FacilityRecord>> {
        let rows = sqlx::query_as::<_, FacilityRow>(
            r#"SELECT id, save_id, zone, name, level, condition, upgrade_progress
               FROM facilities WHERE save_id = ?"#,
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter()
            .map(|row| row.into_facility_record())
            .collect()
    }

    /// 更新设施状态
    pub async fn update_facility(&self, facility: &FacilityRecord) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE facilities SET level = ?, condition = ?, upgrade_progress = ?
               WHERE id = ?"#,
        )
        .bind(facility.level as i64)
        .bind(facility.condition as i64)
        .bind(&facility.upgrade_progress)
        .bind(facility.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

/// 小馆状态数据库行
#[derive(sqlx::FromRow)]
struct ShopStateRow {
    save_id: String,
    name: String,
    funds: i64,
    reputation: f64,
    restaurant_level: i64,
    kitchen_level: i64,
    backyard_level: i64,
    workshop_level: i64,
}

impl ShopStateRow {
    fn into_state(self) -> GameResult<ShopState> {
        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        Ok(ShopState {
            save_id,
            name: self.name,
            funds: self.funds as u64,
            reputation: self.reputation as f32,
            restaurant_level: self.restaurant_level as u32,
            kitchen_level: self.kitchen_level as u32,
            backyard_level: self.backyard_level as u32,
            workshop_level: self.workshop_level as u32,
        })
    }
}

/// 设施数据库行
#[derive(sqlx::FromRow)]
struct FacilityRow {
    id: String,
    save_id: String,
    zone: String,
    name: String,
    level: i64,
    condition: i64,
    upgrade_progress: Option<String>,
}

impl FacilityRow {
    fn into_facility_record(self) -> GameResult<FacilityRecord> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        Ok(FacilityRecord {
            id,
            save_id,
            zone: self.zone,
            name: self.name,
            level: self.level as u32,
            condition: self.condition as u32,
            upgrade_progress: self.upgrade_progress,
        })
    }
}
