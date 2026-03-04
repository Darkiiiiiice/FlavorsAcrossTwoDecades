//! 菜谱定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 菜谱状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeStatus {
    /// 损坏（需修复）
    Damaged,
    /// 模糊（需实验确定用量）
    Fuzzy,
    /// 精确（可直接制作）
    Precise,
    /// 掌握（有品质加成）
    Mastered,
}

impl RecipeStatus {
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            RecipeStatus::Damaged => "损坏",
            RecipeStatus::Fuzzy => "模糊",
            RecipeStatus::Precise => "精确",
            RecipeStatus::Mastered => "掌握",
        }
    }

    /// 是否可以制作
    pub fn can_cook(&self) -> bool {
        matches!(self, RecipeStatus::Precise | RecipeStatus::Mastered)
    }

    /// 是否需要实验
    pub fn needs_experiment(&self) -> bool {
        matches!(self, RecipeStatus::Fuzzy)
    }
}

/// 菜谱来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeSource {
    /// 传承（祖父留下的老菜谱）
    Inherited,
    /// 旅行（盼盼旅行带回）
    Travel,
    /// 创新（盼盼实验研发）
    Innovation,
}

impl RecipeSource {
    /// 获取来源名称
    pub fn name(&self) -> &str {
        match self {
            RecipeSource::Inherited => "传承",
            RecipeSource::Travel => "旅行",
            RecipeSource::Innovation => "创新",
        }
    }
}

/// 菜谱分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeCategory {
    /// 川菜
    Sichuan,
    /// 粤菜
    Cantonese,
    /// 湘菜
    Hunan,
    /// 鲁菜
    Shandong,
    /// 苏菜
    Jiangsu,
    /// 浙菜
    Zhejiang,
    /// 闽菜
    Fujian,
    /// 徽菜
    Anhui,
    /// 家常菜
    HomeStyle,
    /// 创意菜
    Creative,
    /// 异国料理
    Foreign,
}

impl RecipeCategory {
    /// 获取分类名称
    pub fn name(&self) -> &str {
        match self {
            RecipeCategory::Sichuan => "川菜",
            RecipeCategory::Cantonese => "粤菜",
            RecipeCategory::Hunan => "湘菜",
            RecipeCategory::Shandong => "鲁菜",
            RecipeCategory::Jiangsu => "苏菜",
            RecipeCategory::Zhejiang => "浙菜",
            RecipeCategory::Fujian => "闽菜",
            RecipeCategory::Anhui => "徽菜",
            RecipeCategory::HomeStyle => "家常菜",
            RecipeCategory::Creative => "创意菜",
            RecipeCategory::Foreign => "异国料理",
        }
    }
}

/// 食材用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientAmount {
    /// 食材 ID
    pub ingredient_id: String,
    /// 食材名称
    pub ingredient_name: String,
    /// 最小用量（模糊菜谱时使用）
    pub min_amount: Option<f32>,
    /// 最大用量（模糊菜谱时使用）
    pub max_amount: Option<f32>,
    /// 精确用量
    pub exact_amount: Option<f32>,
    /// 单位
    pub unit: String,
}

impl IngredientAmount {
    /// 创建精确用量
    pub fn exact(ingredient_id: String, ingredient_name: String, amount: f32, unit: String) -> Self {
        Self {
            ingredient_id,
            ingredient_name,
            min_amount: None,
            max_amount: None,
            exact_amount: Some(amount),
            unit,
        }
    }

    /// 创建模糊用量
    pub fn fuzzy(
        ingredient_id: String,
        ingredient_name: String,
        min: f32,
        max: f32,
        unit: String,
    ) -> Self {
        Self {
            ingredient_id,
            ingredient_name,
            min_amount: Some(min),
            max_amount: Some(max),
            exact_amount: None,
            unit,
        }
    }

    /// 获取用量范围
    pub fn get_amount_range(&self) -> (f32, f32) {
        if let Some(exact) = self.exact_amount {
            (exact, exact)
        } else if let (Some(min), Some(max)) = (self.min_amount, self.max_amount) {
            (min, max)
        } else {
            (0.0, 0.0)
        }
    }
}

/// 烹饪步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookingStep {
    /// 步骤编号
    pub step_number: u32,
    /// 步骤描述
    pub description: String,
    /// 预计时间（秒）
    pub duration_seconds: u32,
    /// 关键技巧
    pub tips: Option<String>,
}

/// 菜谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// 唯一 ID
    pub id: Uuid,
    /// 编号 ID
    pub id_num: u32,
    /// 菜品名称
    pub name: String,
    /// 菜谱分类
    pub category: RecipeCategory,
    /// 来源
    pub source: RecipeSource,
    /// 状态
    pub status: RecipeStatus,
    /// 食材列表
    pub ingredients: Vec<IngredientAmount>,
    /// 烹饪步骤
    pub steps: Vec<CookingStep>,
    /// 基础价格
    pub base_price: u64,
    /// 难度等级 (1-10)
    pub difficulty: u32,
    /// 制作时间（分钟）
    pub cooking_time_minutes: u32,
    /// 口味标签
    pub flavor_tags: Vec<String>,
    /// 描述
    pub description: String,
    /// 关联的记忆碎片 ID
    pub memory_fragment_id: Option<Uuid>,
    /// 解锁时间
    pub unlocked_at: DateTime<Utc>,
    /// 成功制作次数
    pub cook_count: u32,
    /// 品质加成（掌握状态时）
    pub quality_bonus: f32,
}

impl Recipe {
    /// 创建新菜谱
    pub fn new(name: String, category: RecipeCategory, source: RecipeSource) -> Self {
        Self {
            id: Uuid::new_v4(),
            id_num: 0,
            name,
            category,
            source,
            status: RecipeStatus::Fuzzy,
            ingredients: Vec::new(),
            steps: Vec::new(),
            base_price: 50,
            difficulty: 1,
            cooking_time_minutes: 15,
            flavor_tags: Vec::new(),
            description: String::new(),
            memory_fragment_id: None,
            unlocked_at: Utc::now(),
            cook_count: 0,
            quality_bonus: 0.0,
        }
    }

    /// 创建传承菜谱（初始为损坏状态）
    pub fn inherited(name: String, category: RecipeCategory) -> Self {
        let mut recipe = Self::new(name, category, RecipeSource::Inherited);
        recipe.status = RecipeStatus::Damaged;
        recipe
    }

    /// 创建旅行获得的菜谱（初始为模糊状态）
    pub fn from_travel(name: String, category: RecipeCategory) -> Self {
        let mut recipe = Self::new(name, category, RecipeSource::Travel);
        recipe.status = RecipeStatus::Fuzzy;
        recipe
    }

    /// 添加食材
    pub fn add_ingredient(&mut self, ingredient: IngredientAmount) {
        self.ingredients.push(ingredient);
    }

    /// 添加烹饪步骤
    pub fn add_step(&mut self, description: String, duration_seconds: u32, tips: Option<String>) {
        let step = CookingStep {
            step_number: self.steps.len() as u32 + 1,
            description,
            duration_seconds,
            tips,
        };
        self.steps.push(step);
    }

    /// 修复菜谱（损坏 -> 模糊）
    pub fn repair(&mut self) -> Result<(), String> {
        if self.status != RecipeStatus::Damaged {
            return Err("菜谱不需要修复".to_string());
        }
        self.status = RecipeStatus::Fuzzy;
        Ok(())
    }

    /// 精确化菜谱（模糊 -> 精确）
    pub fn make_precise(&mut self) -> Result<(), String> {
        if self.status != RecipeStatus::Fuzzy {
            return Err("菜谱不需要精确化".to_string());
        }
        // 将模糊用量转为精确用量
        for ingredient in &mut self.ingredients {
            if ingredient.exact_amount.is_none() {
                if let (Some(min), Some(max)) = (ingredient.min_amount, ingredient.max_amount) {
                    ingredient.exact_amount = Some((min + max) / 2.0);
                    ingredient.min_amount = None;
                    ingredient.max_amount = None;
                }
            }
        }
        self.status = RecipeStatus::Precise;
        Ok(())
    }

    /// 掌握菜谱（精确 -> 掌握）
    pub fn master(&mut self) -> Result<(), String> {
        if self.status != RecipeStatus::Precise {
            return Err("只有精确状态的菜谱可以掌握".to_string());
        }
        self.status = RecipeStatus::Mastered;
        self.quality_bonus = 0.2; // 20% 品质加成
        Ok(())
    }

    /// 记录制作
    pub fn record_cooking(&mut self, success: bool) {
        if success {
            self.cook_count += 1;
            // 制作 10 次后自动掌握
            if self.cook_count >= 10 && self.status == RecipeStatus::Precise {
                let _ = self.master();
            }
        }
    }

    /// 计算实际价格
    pub fn calculate_price(&self, quality_multiplier: f32) -> u64 {
        let base = self.base_price as f64;
        let bonus = if self.status == RecipeStatus::Mastered {
            1.2 // 掌握状态有 20% 价格加成
        } else {
            1.0
        };
        ((base * bonus * quality_multiplier as f64) as u64).max(1)
    }

    /// 获取总烹饪时间（秒）
    pub fn total_cooking_time(&self) -> u32 {
        self.steps.iter().map(|s| s.duration_seconds).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = Recipe::new(
            "番茄炒蛋".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );

        assert_eq!(recipe.name, "番茄炒蛋");
        assert_eq!(recipe.status, RecipeStatus::Fuzzy);
        assert_eq!(recipe.source, RecipeSource::Inherited);
    }

    #[test]
    fn test_inherited_recipe() {
        let recipe = Recipe::inherited("麻婆豆腐".to_string(), RecipeCategory::Sichuan);

        assert_eq!(recipe.status, RecipeStatus::Damaged);
        assert_eq!(recipe.source, RecipeSource::Inherited);
    }

    #[test]
    fn test_recipe_status_flow() {
        let mut recipe = Recipe::inherited("菜品".to_string(), RecipeCategory::HomeStyle);

        assert_eq!(recipe.status, RecipeStatus::Damaged);

        recipe.repair().unwrap();
        assert_eq!(recipe.status, RecipeStatus::Fuzzy);

        recipe.make_precise().unwrap();
        assert_eq!(recipe.status, RecipeStatus::Precise);

        recipe.master().unwrap();
        assert_eq!(recipe.status, RecipeStatus::Mastered);
    }

    #[test]
    fn test_add_ingredient() {
        let mut recipe = Recipe::new(
            "菜品".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );

        recipe.add_ingredient(IngredientAmount::exact(
            "tomato".to_string(),
            "番茄".to_string(),
            2.0,
            "个".to_string(),
        ));

        assert_eq!(recipe.ingredients.len(), 1);
    }

    #[test]
    fn test_add_step() {
        let mut recipe = Recipe::new(
            "菜品".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );

        recipe.add_step("切番茄".to_string(), 60, None);
        recipe.add_step("打蛋".to_string(), 30, Some("充分搅拌".to_string()));

        assert_eq!(recipe.steps.len(), 2);
        assert_eq!(recipe.total_cooking_time(), 90);
    }

    #[test]
    fn test_auto_master() {
        let mut recipe = Recipe::new(
            "菜品".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );
        recipe.status = RecipeStatus::Precise;

        for _ in 0..10 {
            recipe.record_cooking(true);
        }

        assert_eq!(recipe.status, RecipeStatus::Mastered);
    }

    #[test]
    fn test_calculate_price() {
        let mut recipe = Recipe::new(
            "菜品".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );
        recipe.base_price = 100;

        let price = recipe.calculate_price(1.0);
        assert_eq!(price, 100);

        recipe.status = RecipeStatus::Mastered;
        let mastered_price = recipe.calculate_price(1.0);
        assert_eq!(mastered_price, 120); // 20% 加成
    }
}
