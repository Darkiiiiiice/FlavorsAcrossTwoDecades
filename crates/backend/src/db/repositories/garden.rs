//! 菜地状态仓储

use sqlx::SqlitePool;

use crate::db::models::garden::GardenPlot;
use crate::error::{DatabaseError, GameError, GameResult};

/// 菜地状态仓储
pub struct GardenRepository {
    pool: SqlitePool,
}

impl GardenRepository {
    /// 创建新的菜地仓储
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建菜地
    pub async fn create(&self, plot: &GardenPlot) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO garden_plots (id, plot_number, is_unlocked, current_crop, fertility, moisture)
               VALUES (?, ?, ?, ?, ?, ?)"#
        )
        .bind(&plot.id)
        .bind(plot.plot_number as i64)
        .bind(plot.is_unlocked as i64)
        .bind(&plot.current_crop)
        .bind(plot.fertility as i64)
        .bind(plot.moisture as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找菜地
    pub async fn find_by_id(&self, id: &str) -> GameResult<Option<GardenPlot>> {
        let row = sqlx::query_as::<_, GardenPlotRow>(
            r#"SELECT id, plot_number, is_unlocked, current_crop, fertility, moisture
               FROM garden_plots WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_plot())),
            None => Ok(None),
        }
    }

    /// 获取所有菜地
    pub async fn find_all(&self) -> GameResult<Vec<GardenPlot>> {
        let rows = sqlx::query_as::<_, GardenPlotRow>(
            r#"SELECT id, plot_number, is_unlocked, current_crop, fertility, moisture
               FROM garden_plots ORDER BY plot_number ASC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        Ok(rows.into_iter().map(|row| row.into_plot()).collect())
    }

    /// 更新菜地状态
    pub async fn update(&self, plot: &GardenPlot) -> GameResult<()> {
        sqlx::query(
            r#"UPDATE garden_plots SET is_unlocked = ?, current_crop = ?, fertility = ?, moisture = ?
               WHERE id = ?"#
        )
        .bind(plot.is_unlocked as i64)
        .bind(&plot.current_crop)
        .bind(plot.fertility as i64)
        .bind(plot.moisture as i64)
        .bind(&plot.id)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 删除菜地
    pub async fn delete(&self, id: &str) -> GameResult<()> {
        sqlx::query(r#"DELETE FROM garden_plots WHERE id = ?"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 初始化菜地（创建6块地）
    pub async fn initialize_plots(&self) -> GameResult<Vec<GardenPlot>> {
        let mut plots = Vec::new();
        for i in 1..=6 {
            let plot = GardenPlot::new(i);
            self.create(&plot).await?;
            plots.push(plot);
        }
        Ok(plots)
    }
}

/// 菜地数据库行
#[derive(sqlx::FromRow)]
struct GardenPlotRow {
    id: String,
    plot_number: i64,
    is_unlocked: i64,
    current_crop: Option<String>,
    fertility: i64,
    moisture: i64,
}

impl GardenPlotRow {
    fn into_plot(self) -> GardenPlot {
        GardenPlot {
            id: self.id,
            plot_number: self.plot_number as u32,
            is_unlocked: self.is_unlocked != 0,
            current_crop: self.current_crop,
            fertility: self.fertility as u32,
            moisture: self.moisture as u32,
        }
    }
}
