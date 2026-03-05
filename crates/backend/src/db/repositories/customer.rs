//! 顾客记录仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::customer::CustomerRecord;
use crate::error::{DatabaseError, GameError, GameResult};

/// 顾客记录仓储
pub struct CustomerRepository {
    pool: SqlitePool,
}

impl CustomerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建顾客记录
    pub async fn create(&self, customer: &CustomerRecord) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO customers (id, save_id, customer_type, name, favorability, visit_count, last_visit, preferences)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(customer.id.to_string())
        .bind(customer.save_id.to_string())
        .bind(&customer.customer_type)
        .bind(&customer.name)
        .bind(customer.favorability as i64)
        .bind(customer.visit_count as i64)
        .bind(customer.last_visit.to_rfc3339())
        .bind(&customer.preferences)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找顾客
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<CustomerRecord>> {
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, save_id, customer_type, name, favorability, visit_count, last_visit, preferences
               FROM customers WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_customer()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有顾客
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<CustomerRecord>> {
        let rows = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, save_id, customer_type, name, favorability, visit_count, last_visit, preferences
               FROM customers WHERE save_id = ? ORDER BY last_visit DESC"#
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_customer()).collect()
    }

    /// 更新顾客记录
    pub async fn update(&self, customer: &CustomerRecord) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE customers SET favorability = ?, visit_count = ?, last_visit = ?, preferences = ?
               WHERE id = ?"#
        )
        .bind(customer.favorability as i64)
        .bind(customer.visit_count as i64)
        .bind(customer.last_visit.to_rfc3339())
        .bind(&customer.preferences)
        .bind(customer.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除顾客
    pub async fn delete(&self, id: Uuid) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM customers WHERE id = ?"#)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CustomerRow {
    id: String,
    save_id: String,
    customer_type: String,
    name: String,
    favorability: i64,
    visit_count: i64,
    last_visit: String,
    preferences: String,
}

impl CustomerRow {
    fn into_customer(self) -> GameResult<CustomerRecord> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        let last_visit = DateTime::parse_from_rfc3339(&self.last_visit)
            .map_err(|e| GameError::Validation {
                details: format!("Invalid last_visit: {}", e),
            })?
            .with_timezone(&Utc);

        Ok(CustomerRecord {
            id,
            save_id,
            customer_type: self.customer_type,
            name: self.name,
            favorability: self.favorability as u32,
            visit_count: self.visit_count as u32,
            last_visit,
            preferences: self.preferences,
        })
    }
}
