//! 菜谱仓储

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::models::recipe::Recipe;
use crate::error::{DatabaseError, GameError, GameResult};
use crate::game::recipe::{RecipeCategory, RecipeSource, RecipeStatus};

/// 菜谱仓储
pub struct RecipeRepository {
    pool: SqlitePool,
}

impl RecipeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 创建菜谱
    pub async fn create(&self, recipe: &Recipe) -> GameResult<()> {
        sqlx::query(
            r#"INSERT INTO recipes (id, save_id, name, category, status, ingredients, source, unlock_condition)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(recipe.id.to_string())
        .bind(recipe.save_id.to_string())
        .bind(&recipe.name)
        .bind(category_to_string(&recipe.category))
        .bind(status_to_string(&recipe.status))
        .bind(&recipe.ingredients)
        .bind(source_to_string(&recipe.source))
        .bind(&recipe.unlock_condition)
        .execute(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }

    /// 根据ID查找菜谱
    pub async fn find_by_id(&self, id: Uuid) -> GameResult<Option<Recipe>> {
        let row = sqlx::query_as::<_, RecipeRow>(
            r#"SELECT id, save_id, name, category, status, ingredients, source, unlock_condition
               FROM recipes WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        match row {
            Some(r) => Ok(Some(r.into_recipe()?)),
            None => Ok(None),
        }
    }

    /// 获取存档的所有菜谱
    pub async fn find_by_save_id(&self, save_id: Uuid) -> GameResult<Vec<Recipe>> {
        let rows = sqlx::query_as::<_, RecipeRow>(
            r#"SELECT id, save_id, name, category, status, ingredients, source, unlock_condition
               FROM recipes WHERE save_id = ?"#,
        )
        .bind(save_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GameError::Database(DatabaseError::QueryFailed(e.to_string())))?;

        rows.into_iter().map(|row| row.into_recipe()).collect()
    }

    /// 更新菜谱状态
    pub async fn update_status(&self, id: Uuid, status: RecipeStatus) -> GameResult<()> {
        sqlx::query(r#"UPDATE recipes SET status = ? WHERE id = ?"#)
            .bind(status_to_string(&status))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| GameError::Database(DatabaseError::WriteFailed(e.to_string())))?;

        Ok(())
    }
}

fn category_to_string(category: &RecipeCategory) -> String {
    match category {
        RecipeCategory::Sichuan => "sichuan".to_string(),
        RecipeCategory::Cantonese => "cantonese".to_string(),
        RecipeCategory::Hunan => "hunan".to_string(),
        RecipeCategory::Shandong => "shandong".to_string(),
        RecipeCategory::Jiangsu => "jiangsu".to_string(),
        RecipeCategory::Zhejiang => "zhejiang".to_string(),
        RecipeCategory::Anhui => "anhui".to_string(),
        RecipeCategory::Fujian => "fujian".to_string(),
        RecipeCategory::HomeStyle => "homestyle".to_string(),
        RecipeCategory::Creative => "creative".to_string(),
        RecipeCategory::Foreign => "foreign".to_string(),
    }
}

fn string_to_category(s: &str) -> GameResult<RecipeCategory> {
    match s {
        "sichuan" => Ok(RecipeCategory::Sichuan),
        "cantonese" => Ok(RecipeCategory::Cantonese),
        "hunan" => Ok(RecipeCategory::Hunan),
        "shandong" => Ok(RecipeCategory::Shandong),
        "jiangsu" => Ok(RecipeCategory::Jiangsu),
        "zhejiang" => Ok(RecipeCategory::Zhejiang),
        "anhui" => Ok(RecipeCategory::Anhui),
        "fujian" => Ok(RecipeCategory::Fujian),
        "homestyle" => Ok(RecipeCategory::HomeStyle),
        "creative" => Ok(RecipeCategory::Creative),
        "foreign" => Ok(RecipeCategory::Foreign),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe category: {}", s),
        }),
    }
}

fn status_to_string(status: &RecipeStatus) -> String {
    match status {
        RecipeStatus::Damaged => "damaged".to_string(),
        RecipeStatus::Fuzzy => "fuzzy".to_string(),
        RecipeStatus::Precise => "precise".to_string(),
        RecipeStatus::Mastered => "mastered".to_string(),
    }
}

fn string_to_status(s: &str) -> GameResult<RecipeStatus> {
    match s {
        "damaged" => Ok(RecipeStatus::Damaged),
        "fuzzy" => Ok(RecipeStatus::Fuzzy),
        "precise" => Ok(RecipeStatus::Precise),
        "mastered" => Ok(RecipeStatus::Mastered),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe status: {}", s),
        }),
    }
}

fn source_to_string(source: &RecipeSource) -> String {
    match source {
        RecipeSource::Inherited => "inherited".to_string(),
        RecipeSource::Travel => "travel".to_string(),
        RecipeSource::Innovation => "innovation".to_string(),
    }
}

fn string_to_source(s: &str) -> GameResult<RecipeSource> {
    match s {
        "inherited" => Ok(RecipeSource::Inherited),
        "travel" => Ok(RecipeSource::Travel),
        "innovation" => Ok(RecipeSource::Innovation),
        _ => Err(GameError::Validation {
            details: format!("Invalid recipe source: {}", s),
        }),
    }
}

#[derive(sqlx::FromRow)]
struct RecipeRow {
    id: String,
    save_id: String,
    name: String,
    category: String,
    status: String,
    ingredients: String,
    source: String,
    unlock_condition: Option<String>,
}

impl RecipeRow {
    fn into_recipe(self) -> GameResult<Recipe> {
        let id = Uuid::parse_str(&self.id).map_err(|e| GameError::Validation {
            details: format!("Invalid UUID: {}", e),
        })?;

        let save_id = Uuid::parse_str(&self.save_id).map_err(|e| GameError::Validation {
            details: format!("Invalid save_id UUID: {}", e),
        })?;

        let category = string_to_category(&self.category)?;
        let status = string_to_status(&self.status)?;
        let source = string_to_source(&self.source)?;

        Ok(Recipe {
            id,
            save_id,
            name: self.name,
            category,
            status,
            ingredients: self.ingredients,
            source,
            unlock_condition: self.unlock_condition,
        })
    }
}
