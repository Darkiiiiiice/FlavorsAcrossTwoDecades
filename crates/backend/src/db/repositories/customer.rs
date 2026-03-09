//! 顾客记录仓储

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::db::models::customer::{CustomerRecord, PreferenceRecord};
use crate::game::customer::Customer;
use crate::error::{DatabaseError, GameError, GameResult};

/// 顾客记录仓储
pub struct CustomerRepository {
    pool: SqlitePool,
}

impl CustomerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建顾客（包含偏好）
    pub async fn create(&self, customer: &Customer) -> GameResult<i64> {
        let now = Utc::now();
        let now_ts = now.timestamp();

        // 开启事务
        let mut tx = self.pool.begin().await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        // 插入顾客
        let result = sqlx::query(
            r#"INSERT INTO customers (name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&customer.name)
        .bind(customer.age as i64)
        .bind(&customer.occupation)
        .bind(i32::from(customer.customer_type))
        .bind(customer.affinity as i64)
        .bind(customer.visit_count as i64)
        .bind(&customer.story_background)
        .bind(now_ts)
        .bind(now_ts)
        .execute(&mut *tx)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        let customer_id = result.last_insert_rowid();

        // 插入偏好
        let pref_json = serde_json::to_string(&customer.preference.favorite_categories)
            .unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            r#"INSERT INTO preferences (customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(customer_id)
        .bind(i32::from(customer.preference.flavor))
        .bind(i32::from(customer.preference.dietary))
        .bind(customer.preference.price_sensitivity as i64)
        .bind(customer.preference.patience as i64)
        .bind(&pref_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        tx.commit().await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(customer_id)
    }

    /// 根据ID查找顾客（包含偏好）
    pub async fn find_by_id(&self, id: i64) -> GameResult<Option<CustomerRecord>> {
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at
               FROM customers WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(customer_row) => {
                // 查询偏好
                let pref_row = sqlx::query_as::<_, PreferenceRow>(
                    r#"SELECT id, customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories
                       FROM preferences WHERE customer_id = ?"#
                )
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

                Ok(Some(customer_row.into_record(pref_row)?))
            }
            None => Ok(None),
        }
    }

    /// 获取所有顾客
    pub async fn find_all(&self) -> GameResult<Vec<CustomerRecord>> {
        let customer_rows = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at
               FROM customers ORDER BY updated_at DESC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        let mut customers = Vec::with_capacity(customer_rows.len());
        for customer_row in customer_rows {
            let pref_row = sqlx::query_as::<_, PreferenceRow>(
                r#"SELECT id, customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories
                   FROM preferences WHERE customer_id = ?"#
            )
            .bind(customer_row.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

            customers.push(customer_row.into_record(pref_row)?);
        }

        Ok(customers)
    }

    /// 更新顾客（包含偏好）
    pub async fn update(&self, customer: &Customer) -> GameResult<()> {
        let now_ts = Utc::now().timestamp();

        let mut tx = self.pool.begin().await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        // 更新顾客
        sqlx::query(
            r#"UPDATE customers SET name = ?, age = ?, occupation = ?, customer_type = ?, affinity = ?, visit_count = ?, story_background = ?, updated_at = ?
               WHERE id = ?"#
        )
        .bind(&customer.name)
        .bind(customer.age as i64)
        .bind(&customer.occupation)
        .bind(i32::from(customer.customer_type))
        .bind(customer.affinity as i64)
        .bind(customer.visit_count as i64)
        .bind(&customer.story_background)
        .bind(now_ts)
        .bind(customer.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        // 更新偏好
        let pref_json = serde_json::to_string(&customer.preference.favorite_categories)
            .unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            r#"UPDATE preferences SET flavor = ?, dietary = ?, price_sensitivity = ?, patience = ?, favorite_categories = ?
               WHERE customer_id = ?"#
        )
        .bind(i32::from(customer.preference.flavor))
        .bind(i32::from(customer.preference.dietary))
        .bind(customer.preference.price_sensitivity as i64)
        .bind(customer.preference.patience as i64)
        .bind(&pref_json)
        .bind(customer.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        tx.commit().await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除顾客（偏好会级联删除）
    pub async fn delete(&self, id: i64) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM customers WHERE id = ?"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据名称搜索顾客
    pub async fn find_by_name(&self, name: &str) -> GameResult<Vec<CustomerRecord>> {
        let customer_rows = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at
               FROM customers WHERE name LIKE ? ORDER BY updated_at DESC"#
        )
        .bind(format!("%{}%", name))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        let mut customers = Vec::with_capacity(customer_rows.len());
        for customer_row in customer_rows {
            let pref_row = sqlx::query_as::<_, PreferenceRow>(
                r#"SELECT id, customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories
                   FROM preferences WHERE customer_id = ?"#
            )
            .bind(customer_row.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

            customers.push(customer_row.into_record(pref_row)?);
        }

        Ok(customers)
    }

    /// 获取最近创建的10位顾客（按创建时间倒序）
    pub async fn find_recent(&self) -> GameResult<Vec<CustomerRecord>> {
        let customer_rows = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at
               FROM customers ORDER BY created_at DESC LIMIT 10"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        let mut customers = Vec::with_capacity(customer_rows.len());
        for customer_row in customer_rows {
            let pref_row = sqlx::query_as::<_, PreferenceRow>(
                r#"SELECT id, customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories
                   FROM preferences WHERE customer_id = ?"#
            )
            .bind(customer_row.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

            customers.push(customer_row.into_record(pref_row)?);
        }

        Ok(customers)
    }

    /// 随机获取一位顾客
    pub async fn find_random(&self) -> GameResult<Option<CustomerRecord>> {
        // 使用 RANDOM() 随机选择一位顾客
        let row = sqlx::query_as::<_, CustomerRow>(
            r#"SELECT id, name, age, occupation, customer_type, affinity, visit_count, story_background, created_at, updated_at
               FROM customers ORDER BY RANDOM() LIMIT 1"#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(customer_row) => {
                let pref_row = sqlx::query_as::<_, PreferenceRow>(
                    r#"SELECT id, customer_id, flavor, dietary, price_sensitivity, patience, favorite_categories
                       FROM preferences WHERE customer_id = ?"#
                )
                .bind(customer_row.id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

                Ok(Some(customer_row.into_record(pref_row)?))
            }
            None => Ok(None),
        }
    }

    /// 统计顾客总数
    pub async fn count(&self) -> GameResult<i64> {
        let row: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM customers"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        Ok(row.0)
    }
}

// ========== 数据库行结构 ==========

#[derive(sqlx::FromRow)]
struct CustomerRow {
    id: i64,
    name: String,
    age: i64,
    occupation: String,
    customer_type: i32,
    affinity: i64,
    visit_count: i64,
    story_background: String,
    created_at: i64,
    updated_at: i64,
}

impl CustomerRow {
    fn into_record(self, pref_row: PreferenceRow) -> GameResult<CustomerRecord> {
        use crate::game::customer::CustomerType;

        let customer_type = CustomerType::try_from(self.customer_type)
            .map_err(|e| GameError::Validation { details: e })?;

        let created_at = DateTime::from_timestamp(self.created_at, 0)
            .unwrap_or_else(|| Utc::now());

        let updated_at = DateTime::from_timestamp(self.updated_at, 0)
            .unwrap_or_else(|| Utc::now());

        let preference = pref_row.into_record()?;

        Ok(CustomerRecord {
            id: self.id,
            name: self.name,
            age: self.age as u32,
            occupation: self.occupation,
            customer_type,
            affinity: self.affinity as u32,
            visit_count: self.visit_count as u32,
            story_background: self.story_background,
            preference,
            created_at,
            updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct PreferenceRow {
    id: i64,
    customer_id: i64,
    flavor: i32,
    dietary: i32,
    price_sensitivity: i64,
    patience: i64,
    favorite_categories: String,
}

impl PreferenceRow {
    fn into_record(self) -> GameResult<PreferenceRecord> {
        use crate::game::customer::preference::{FlavorPreference, DietaryRestriction};

        let flavor = FlavorPreference::try_from(self.flavor)
            .map_err(|e| GameError::Validation { details: e })?;

        let dietary = DietaryRestriction::try_from(self.dietary)
            .map_err(|e| GameError::Validation { details: e })?;

        Ok(PreferenceRecord {
            id: self.id,
            customer_id: self.customer_id,
            flavor,
            dietary,
            price_sensitivity: self.price_sensitivity as u32,
            patience: self.patience as u32,
            favorite_categories: self.favorite_categories,
        })
    }
}
