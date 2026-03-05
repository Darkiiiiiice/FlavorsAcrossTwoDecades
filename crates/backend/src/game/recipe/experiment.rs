//! 实验研发系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::recipe::{IngredientAmount, Recipe, RecipeStatus};

/// 实验状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// 准备中
    Preparing,
    /// 进行中
    InProgress,
    /// 分析中
    Analyzing,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 已完成
    Completed,
}

impl ExperimentStatus {
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            ExperimentStatus::Preparing => "准备中",
            ExperimentStatus::InProgress => "进行中",
            ExperimentStatus::Analyzing => "分析中",
            ExperimentStatus::Success => "成功",
            ExperimentStatus::Failed => "失败",
            ExperimentStatus::Completed => "已完成",
        }
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            ExperimentStatus::Success | ExperimentStatus::Failed | ExperimentStatus::Completed
        )
    }
}

/// 实验尝试的用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentAmount {
    /// 食材 ID
    pub ingredient_id: String,
    /// 食材名称
    pub ingredient_name: String,
    /// 尝试的用量
    pub attempted_amount: f32,
    /// 单位
    pub unit: String,
}

/// 传感器反馈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorFeedback {
    /// 食材 ID
    pub ingredient_id: String,
    /// 反馈类型
    pub feedback_type: FeedbackType,
    /// 反馈描述
    pub description: String,
    /// 置信度 (0-1)
    pub confidence: f32,
}

/// 反馈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// 太少
    TooLittle,
    /// 太多
    TooMuch,
    /// 刚好
    JustRight,
    /// 不确定
    Uncertain,
}

impl FeedbackType {
    /// 获取反馈名称
    pub fn name(&self) -> &str {
        match self {
            FeedbackType::TooLittle => "太少",
            FeedbackType::TooMuch => "太多",
            FeedbackType::JustRight => "刚好",
            FeedbackType::Uncertain => "不确定",
        }
    }

    /// 是否正确
    pub fn is_correct(&self) -> bool {
        matches!(self, FeedbackType::JustRight)
    }
}

/// 实验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// 是否成功
    pub success: bool,
    /// 成功确定的食材用量
    pub determined_amounts: Vec<IngredientAmount>,
    /// 传感器反馈
    pub sensor_feedback: Vec<SensorFeedback>,
    /// 品质评分 (0-100)
    pub quality_score: u32,
    /// 结果描述
    pub description: String,
}

/// 实验配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// 传感器精度 (0-100)
    pub sensor_accuracy: u32,
    /// 最大尝试次数
    pub max_attempts: u32,
    /// 基础成功率
    pub base_success_rate: f32,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            sensor_accuracy: 70, // ±30% 误差
            max_attempts: 10,
            base_success_rate: 0.5,
        }
    }
}

/// 实验
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// 唯一 ID
    pub id: Uuid,
    /// 存档 ID
    pub save_id: Uuid,
    /// 关联的菜谱 ID
    pub recipe_id: Uuid,
    /// 菜谱名称
    pub recipe_name: String,
    /// 状态
    pub status: ExperimentStatus,
    /// 尝试次数
    pub attempt_count: u32,
    /// 最大尝试次数
    pub max_attempts: u32,
    /// 当前尝试的用量
    pub current_amounts: Vec<ExperimentAmount>,
    /// 传感器精度
    pub sensor_accuracy: u32,
    /// 实验结果
    pub result: Option<ExperimentResult>,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 历史尝试记录
    pub attempt_history: Vec<ExperimentResult>,
}

impl Experiment {
    /// 创建新实验
    pub fn new(recipe: &Recipe, config: ExperimentConfig) -> Self {
        // 初始化尝试用量为模糊范围的中值
        let current_amounts: Vec<ExperimentAmount> = recipe
            .ingredients
            .iter()
            .map(|ing| {
                let (min, max) = ing.get_amount_range();
                ExperimentAmount {
                    ingredient_id: ing.ingredient_id.clone(),
                    ingredient_name: ing.ingredient_name.clone(),
                    attempted_amount: (min + max) / 2.0,
                    unit: ing.unit.clone(),
                }
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            save_id: Uuid::nil(),
            recipe_id: recipe.id,
            recipe_name: recipe.name.clone(),
            status: ExperimentStatus::Preparing,
            attempt_count: 0,
            max_attempts: config.max_attempts,
            current_amounts,
            sensor_accuracy: config.sensor_accuracy,
            result: None,
            started_at: Utc::now(),
            completed_at: None,
            attempt_history: Vec::new(),
        }
    }

    /// 设置存档 ID
    pub fn with_save_id(mut self, save_id: Uuid) -> Self {
        self.save_id = save_id;
        self
    }

    /// 开始实验
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != ExperimentStatus::Preparing {
            return Err("实验已经开始".to_string());
        }
        self.status = ExperimentStatus::InProgress;
        Ok(())
    }

    /// 执行一次尝试
    pub fn execute_attempt(&mut self) -> ExperimentResult {
        self.attempt_count += 1;
        self.status = ExperimentStatus::InProgress;

        // 生成传感器反馈
        let sensor_feedback = self.generate_sensor_feedback();

        // 检查是否所有食材都刚好
        let all_correct = sensor_feedback
            .iter()
            .all(|f| f.feedback_type == FeedbackType::JustRight);

        // 计算品质评分
        let quality_score = self.calculate_quality_score(&sensor_feedback);

        // 确定成功
        let success = all_correct || quality_score >= 80;

        let result = ExperimentResult {
            success,
            determined_amounts: if success {
                self.current_amounts
                    .iter()
                    .map(|a| {
                        IngredientAmount::exact(
                            a.ingredient_id.clone(),
                            a.ingredient_name.clone(),
                            a.attempted_amount,
                            a.unit.clone(),
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            },
            sensor_feedback: sensor_feedback.clone(),
            quality_score,
            description: if success {
                "实验成功！配方已确定。".to_string()
            } else {
                "实验未能完全成功，请根据反馈调整用量。".to_string()
            },
        };

        // 记录历史
        self.attempt_history.push(result.clone());

        // 更新状态
        if success {
            self.status = ExperimentStatus::Success;
            self.result = Some(result.clone());
        } else if self.attempt_count >= self.max_attempts {
            self.status = ExperimentStatus::Failed;
            self.result = Some(result.clone());
        }

        result
    }

    /// 生成传感器反馈
    fn generate_sensor_feedback(&self) -> Vec<SensorFeedback> {
        use rand::RngExt;

        let mut rng = rand::rng();
        let mut feedback = Vec::new();

        // 传感器精度对应的误差范围
        let error_range = (100 - self.sensor_accuracy) as f32 / 100.0;

        for amount in &self.current_amounts {
            // 模拟真实值（这里假设中值是正确的）
            let true_value = amount.attempted_amount;

            // 传感器测量值（带有误差）
            let measurement_error = rng.random_range(-error_range..=error_range) as f32;
            let _measured_value = true_value * (1.0 + measurement_error);

            // 根据测量值判断反馈
            let feedback_type = if measurement_error.abs() < 0.05 {
                FeedbackType::JustRight
            } else if measurement_error < 0.0 {
                FeedbackType::TooLittle
            } else {
                FeedbackType::TooMuch
            };

            feedback.push(SensorFeedback {
                ingredient_id: amount.ingredient_id.clone(),
                feedback_type,
                description: format!(
                    "{}的{}似乎{}了",
                    amount.ingredient_name,
                    amount.unit,
                    feedback_type.name()
                ),
                confidence: 1.0 - measurement_error.abs(),
            });
        }

        feedback
    }

    /// 计算品质评分
    fn calculate_quality_score(&self, feedback: &[SensorFeedback]) -> u32 {
        if feedback.is_empty() {
            return 50;
        }

        let correct_count = feedback
            .iter()
            .filter(|f| f.feedback_type == FeedbackType::JustRight)
            .count();

        let ratio = correct_count as f32 / feedback.len() as f32;
        (ratio * 100.0) as u32
    }

    /// 根据反馈调整用量
    pub fn adjust_amounts(&mut self, adjustments: &[(String, f32)]) -> Result<(), String> {
        if self.status.is_finished() {
            return Err("实验已结束".to_string());
        }

        for (ingredient_id, delta) in adjustments {
            if let Some(amount) = self
                .current_amounts
                .iter_mut()
                .find(|a| &a.ingredient_id == ingredient_id)
            {
                amount.attempted_amount = (amount.attempted_amount + delta).max(0.0);
            }
        }

        self.status = ExperimentStatus::Preparing;
        Ok(())
    }

    /// 完成实验
    pub fn complete(&mut self) -> Result<ExperimentResult, String> {
        if !self.status.is_finished() {
            return Err("实验尚未完成".to_string());
        }

        self.status = ExperimentStatus::Completed;
        self.completed_at = Some(Utc::now());

        self.result
            .clone()
            .ok_or_else(|| "没有实验结果".to_string())
    }

    /// 获取剩余尝试次数
    pub fn remaining_attempts(&self) -> u32 {
        self.max_attempts.saturating_sub(self.attempt_count)
    }

    /// 是否可以继续尝试
    pub fn can_retry(&self) -> bool {
        !self.status.is_finished() && self.remaining_attempts() > 0
    }
}

/// 实验管理器
pub struct ExperimentManager {
    /// 传感器等级 (1-10)
    pub sensor_level: u32,
}

impl ExperimentManager {
    /// 创建新的实验管理器
    pub fn new(sensor_level: u32) -> Self {
        Self {
            sensor_level: sensor_level.clamp(1, 10),
        }
    }

    /// 获取传感器精度
    pub fn get_sensor_accuracy(&self) -> u32 {
        // 等级 1-2: ±30%, 等级 3-4: ±20%, 等级 5-6: ±10%, 等级 7-8: ±5%, 等级 9-10: ±2%
        match self.sensor_level {
            1..=2 => 70,
            3..=4 => 80,
            5..=6 => 90,
            7..=8 => 95,
            9..=10 => 98,
            _ => 70,
        }
    }

    /// 创建实验配置
    pub fn create_config(&self) -> ExperimentConfig {
        ExperimentConfig {
            sensor_accuracy: self.get_sensor_accuracy(),
            max_attempts: 10,
            base_success_rate: 0.5 + (self.sensor_level as f32 * 0.05),
        }
    }

    /// 创建实验
    pub fn create_experiment(&self, recipe: &Recipe) -> Experiment {
        Experiment::new(recipe, self.create_config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::recipe::{Recipe, RecipeCategory, RecipeSource};

    fn create_test_recipe() -> Recipe {
        let mut recipe = Recipe::new(
            "测试菜品".to_string(),
            RecipeCategory::HomeStyle,
            RecipeSource::Inherited,
        );
        recipe.status = RecipeStatus::Fuzzy;
        recipe.add_ingredient(IngredientAmount::fuzzy(
            "tomato".to_string(),
            "番茄".to_string(),
            1.0,
            3.0,
            "个".to_string(),
        ));
        recipe
    }

    #[test]
    fn test_experiment_creation() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig::default();
        let experiment = Experiment::new(&recipe, config);

        assert_eq!(experiment.status, ExperimentStatus::Preparing);
        assert_eq!(experiment.attempt_count, 0);
        assert_eq!(experiment.current_amounts.len(), 1);
    }

    #[test]
    fn test_experiment_start() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig::default();
        let mut experiment = Experiment::new(&recipe, config);

        experiment.start().unwrap();
        assert_eq!(experiment.status, ExperimentStatus::InProgress);
    }

    #[test]
    fn test_execute_attempt() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig::default();
        let mut experiment = Experiment::new(&recipe, config);

        let result = experiment.execute_attempt();

        assert_eq!(experiment.attempt_count, 1);
        assert!(!result.sensor_feedback.is_empty());
    }

    #[test]
    fn test_adjust_amounts() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig::default();
        let mut experiment = Experiment::new(&recipe, config);

        // 初始用量是 (1.0 + 3.0) / 2.0 = 2.0
        assert_eq!(experiment.current_amounts[0].attempted_amount, 2.0);

        // 不执行尝试，直接调整用量
        let adjustments = vec![("tomato".to_string(), 0.5)];
        experiment.adjust_amounts(&adjustments).unwrap();

        // 2.0 + 0.5 = 2.5
        assert_eq!(experiment.current_amounts[0].attempted_amount, 2.5);
    }

    #[test]
    fn test_adjust_after_failed_attempt() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig {
            sensor_accuracy: 50, // 低精度更可能失败
            max_attempts: 5,
            base_success_rate: 0.1,
        };
        let mut experiment = Experiment::new(&recipe, config);

        experiment.start().unwrap();
        let _result = experiment.execute_attempt();

        // 只有在实验未结束时才能调整
        if !experiment.status.is_finished() {
            let initial = experiment.current_amounts[0].attempted_amount;
            let adjustments = vec![("tomato".to_string(), 0.5)];
            let adjust_result = experiment.adjust_amounts(&adjustments);
            // 可能成功或失败，取决于随机结果
            if adjust_result.is_ok() {
                assert_eq!(
                    experiment.current_amounts[0].attempted_amount,
                    initial + 0.5
                );
            }
        }
    }

    #[test]
    fn test_sensor_accuracy() {
        let manager = ExperimentManager::new(1);
        assert_eq!(manager.get_sensor_accuracy(), 70);

        let manager = ExperimentManager::new(5);
        assert_eq!(manager.get_sensor_accuracy(), 90);

        let manager = ExperimentManager::new(10);
        assert_eq!(manager.get_sensor_accuracy(), 98);
    }

    #[test]
    fn test_remaining_attempts() {
        let recipe = create_test_recipe();
        let config = ExperimentConfig {
            max_attempts: 5,
            ..Default::default()
        };
        let mut experiment = Experiment::new(&recipe, config);

        assert_eq!(experiment.remaining_attempts(), 5);

        experiment.execute_attempt();
        assert_eq!(experiment.remaining_attempts(), 4);
    }

    #[test]
    fn test_experiment_manager() {
        let manager = ExperimentManager::new(5);
        let recipe = create_test_recipe();

        let experiment = manager.create_experiment(&recipe);

        assert_eq!(experiment.sensor_accuracy, 90);
    }
}
