#![allow(clippy::module_inception)]
//! 菜谱与实验系统模块

mod experiment;
mod ingredient;
mod recipe;

pub use experiment::{Experiment, ExperimentResult, ExperimentStatus};
pub use ingredient::{Ingredient, IngredientCategory, IngredientQuality};
pub use recipe::{Recipe, RecipeCategory, RecipeSource, RecipeStatus};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 菜谱管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeManager {
    /// 存档 ID
    pub save_id: Uuid,
    /// 已解锁的菜谱
    pub recipes: Vec<Recipe>,
    /// 可用食材库存
    pub ingredients: Vec<Ingredient>,
    /// 进行中的实验
    pub active_experiments: Vec<Experiment>,
    /// 已完成的实验
    pub experiment_history: Vec<Experiment>,
    /// 下一个菜谱 ID
    pub next_recipe_id: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl RecipeManager {
    /// 创建新的菜谱管理器
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            recipes: Vec::new(),
            ingredients: Vec::new(),
            active_experiments: Vec::new(),
            experiment_history: Vec::new(),
            next_recipe_id: 1,
            updated_at: Utc::now(),
        }
    }

    /// 添加菜谱
    pub fn add_recipe(&mut self, mut recipe: Recipe) {
        recipe.id_num = self.next_recipe_id;
        self.next_recipe_id += 1;
        self.recipes.push(recipe);
        self.updated_at = Utc::now();
    }

    /// 获取菜谱
    pub fn get_recipe(&self, recipe_id: Uuid) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == recipe_id)
    }

    /// 获取菜谱（可变）
    pub fn get_recipe_mut(&mut self, recipe_id: Uuid) -> Option<&mut Recipe> {
        self.recipes.iter_mut().find(|r| r.id == recipe_id)
    }

    /// 按状态获取菜谱
    pub fn get_recipes_by_status(&self, status: RecipeStatus) -> Vec<&Recipe> {
        self.recipes.iter().filter(|r| r.status == status).collect()
    }

    /// 按来源获取菜谱
    pub fn get_recipes_by_source(&self, source: RecipeSource) -> Vec<&Recipe> {
        self.recipes.iter().filter(|r| r.source == source).collect()
    }

    /// 添加食材
    pub fn add_ingredient(&mut self, ingredient: Ingredient) {
        // 如果已有相同食材，合并数量
        if let Some(existing) = self
            .ingredients
            .iter_mut()
            .find(|i| i.ingredient_id == ingredient.ingredient_id)
        {
            existing.quantity += ingredient.quantity;
            // 更新品质为较低的值
            existing.quality = existing.quality.min(ingredient.quality);
        } else {
            self.ingredients.push(ingredient);
        }
        self.updated_at = Utc::now();
    }

    /// 使用食材
    pub fn use_ingredient(&mut self, ingredient_id: &str, quantity: u32) -> Result<(), String> {
        if let Some(ingredient) = self
            .ingredients
            .iter_mut()
            .find(|i| i.ingredient_id == ingredient_id)
        {
            if ingredient.quantity < quantity {
                return Err("食材数量不足".to_string());
            }
            ingredient.quantity -= quantity;
            if ingredient.quantity == 0 {
                self.ingredients
                    .retain(|i| i.ingredient_id != ingredient_id);
            }
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("食材不存在".to_string())
        }
    }

    /// 获取食材
    pub fn get_ingredient(&self, ingredient_id: &str) -> Option<&Ingredient> {
        self.ingredients
            .iter()
            .find(|i| i.ingredient_id == ingredient_id)
    }

    /// 开始实验
    pub fn start_experiment(&mut self, experiment: Experiment) {
        self.active_experiments.push(experiment);
        self.updated_at = Utc::now();
    }

    /// 完成实验
    pub fn complete_experiment(&mut self, experiment_id: Uuid) -> Option<Experiment> {
        if let Some(pos) = self
            .active_experiments
            .iter()
            .position(|e| e.id == experiment_id)
        {
            let mut experiment = self.active_experiments.remove(pos);
            experiment.status = ExperimentStatus::Completed;
            experiment.completed_at = Some(Utc::now());
            self.experiment_history.push(experiment.clone());
            self.updated_at = Utc::now();
            Some(experiment)
        } else {
            None
        }
    }

    /// 获取活跃实验数量
    pub fn active_experiment_count(&self) -> usize {
        self.active_experiments.len()
    }

    /// 获取已掌握的菜谱数量
    pub fn mastered_recipe_count(&self) -> usize {
        self.recipes
            .iter()
            .filter(|r| r.status == RecipeStatus::Mastered)
            .count()
    }

    /// 获取可制作的菜谱（精确或已掌握状态）
    pub fn get_cookable_recipes(&self) -> Vec<&Recipe> {
        self.recipes
            .iter()
            .filter(|r| r.status == RecipeStatus::Precise || r.status == RecipeStatus::Mastered)
            .collect()
    }

    /// 更新食材新鲜度
    pub fn update_ingredient_freshness(&mut self) {
        let now = Utc::now();
        for ingredient in &mut self.ingredients {
            ingredient.update_freshness(now);
        }
        // 移除过期食材
        self.ingredients.retain(|i| i.freshness > 0.0);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_manager_creation() {
        let save_id = Uuid::new_v4();
        let manager = RecipeManager::new(save_id);

        assert_eq!(manager.recipes.len(), 0);
        assert_eq!(manager.ingredients.len(), 0);
        assert_eq!(manager.next_recipe_id, 1);
    }

    #[test]
    fn test_add_recipe() {
        let save_id = Uuid::new_v4();
        let mut manager = RecipeManager::new(save_id);

        let recipe = Recipe::new(
            "番茄炒蛋".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );

        manager.add_recipe(recipe);
        assert_eq!(manager.recipes.len(), 1);
        assert_eq!(manager.next_recipe_id, 2);
    }

    #[test]
    fn test_add_ingredient() {
        let save_id = Uuid::new_v4();
        let mut manager = RecipeManager::new(save_id);

        let ingredient = Ingredient::new(
            "tomato".to_string(),
            "番茄".to_string(),
            IngredientCategory::Vegetable,
            10,
        );

        manager.add_ingredient(ingredient);
        assert_eq!(manager.ingredients.len(), 1);
        assert_eq!(manager.ingredients[0].quantity, 10);
    }

    #[test]
    fn test_merge_ingredients() {
        let save_id = Uuid::new_v4();
        let mut manager = RecipeManager::new(save_id);

        let ing1 = Ingredient::new(
            "tomato".to_string(),
            "番茄".to_string(),
            IngredientCategory::Vegetable,
            10,
        );
        let ing2 = Ingredient::new(
            "tomato".to_string(),
            "番茄".to_string(),
            IngredientCategory::Vegetable,
            5,
        );

        manager.add_ingredient(ing1);
        manager.add_ingredient(ing2);

        assert_eq!(manager.ingredients.len(), 1);
        assert_eq!(manager.ingredients[0].quantity, 15);
    }

    #[test]
    fn test_use_ingredient() {
        let save_id = Uuid::new_v4();
        let mut manager = RecipeManager::new(save_id);

        let ingredient = Ingredient::new(
            "tomato".to_string(),
            "番茄".to_string(),
            IngredientCategory::Vegetable,
            10,
        );
        manager.add_ingredient(ingredient);

        let result = manager.use_ingredient("tomato", 3);
        assert!(result.is_ok());
        assert_eq!(manager.ingredients[0].quantity, 7);

        let result = manager.use_ingredient("tomato", 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cookable_recipes() {
        let save_id = Uuid::new_v4();
        let mut manager = RecipeManager::new(save_id);

        let mut recipe1 = Recipe::new(
            "菜品1".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );
        recipe1.status = RecipeStatus::Mastered;

        let mut recipe2 = Recipe::new(
            "菜品2".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );
        recipe2.status = RecipeStatus::Fuzzy;

        manager.add_recipe(recipe1);
        manager.add_recipe(recipe2);

        let cookable = manager.get_cookable_recipes();
        assert_eq!(cookable.len(), 1);
    }
}
