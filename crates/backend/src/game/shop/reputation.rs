//! 口碑系统

use serde::{Deserialize, Serialize};

use super::facility::ZoneLevel;

/// 口碑管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    /// 当前口碑分数 (0-100)
    pub score: f32,
    /// 菜品品质评分 (0-100)
    pub dish_quality: f32,
    /// 服务质量评分 (0-100)
    pub service_quality: f32,
    /// 环境评分 (0-100)
    pub environment_score: f32,
    /// 邻里关系评分 (0-100)
    pub neighbor_relation: f32,
    /// 常客加成 (0-100)
    pub regular_customer_bonus: f32,
}

impl Reputation {
    /// 创建新的口碑系统
    pub fn new() -> Self {
        Self {
            score: 30.0, // 初始口碑
            dish_quality: 50.0,
            service_quality: 50.0,
            environment_score: 50.0,
            neighbor_relation: 50.0,
            regular_customer_bonus: 0.0,
        }
    }

    /// 计算综合口碑分数
    pub fn update_score(&mut self, zones: &[ZoneLevel]) {
        // 计算口碑上限（所有区域的上限之和）
        let reputation_cap: u32 = zones.iter().map(|z| z.reputation_cap).sum();

        // 加权计算
        let dish_score = self.dish_quality * 0.40;
        let service_score = self.service_quality * 0.20;
        let environment_score = self.environment_score * 0.15;
        let neighbor_score = self.neighbor_relation * 0.15;
        let regular_score = self.regular_customer_bonus * 0.10;

        let raw_score =
            dish_score + service_score + environment_score + neighbor_score + regular_score;

        // 应用上限
        self.score = raw_score.min(reputation_cap as f32);
    }

    /// 更新菜品品质
    pub fn update_dish_quality(&mut self, quality: f32) {
        self.dish_quality = quality.clamp(0.0, 100.0);
    }

    /// 更新服务质量
    pub fn update_service_quality(&mut self, quality: f32) {
        self.service_quality = quality.clamp(0.0, 100.0);
    }

    /// 更新环境评分
    pub fn update_environment_score(&mut self, score: f32) {
        self.environment_score = score.clamp(0.0, 100.0);
    }

    /// 更新邻里关系
    pub fn update_neighbor_relation(&mut self, relation: f32) {
        self.neighbor_relation = relation.clamp(0.0, 100.0);
    }

    /// 更新常客加成
    pub fn update_regular_bonus(&mut self, bonus: f32) {
        self.regular_customer_bonus = bonus.clamp(0.0, 100.0);
    }

    /// 获取口碑等级
    pub fn get_level(&self) -> ReputationLevel {
        match self.score as u32 {
            0..=20 => ReputationLevel::Poor,
            21..=40 => ReputationLevel::Average,
            41..=60 => ReputationLevel::Good,
            61..=80 => ReputationLevel::Excellent,
            81..=100 => ReputationLevel::Outstanding,
            _ => ReputationLevel::Legendary,
        }
    }

    /// 获取口碑描述
    pub fn get_description(&self) -> String {
        format!(
            "口碑: {:.1} ({}) | 菜品: {:.1} | 服务: {:.1} | 环境: {:.1}",
            self.score,
            self.get_level().name(),
            self.dish_quality,
            self.service_quality,
            self.environment_score
        )
    }

    /// 计算顾客吸引力（基于口碑）
    pub fn customer_attraction(&self) -> f32 {
        self.score / 100.0 * 1.5 // 0-1.5倍
    }

    /// 计算价格加成（基于口碑）
    pub fn price_bonus(&self) -> f32 {
        (self.score - 50.0) / 100.0 // -0.5 到 +0.5
    }
}

impl Default for Reputation {
    fn default() -> Self {
        Self::new()
    }
}

/// 口碑等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationLevel {
    /// 差评
    Poor,
    /// 一般
    Average,
    /// 良好
    Good,
    /// 优秀
    Excellent,
    /// 杰出
    Outstanding,
    /// 传说
    Legendary,
}

impl ReputationLevel {
    /// 获取等级名称
    pub fn name(&self) -> &str {
        match self {
            ReputationLevel::Poor => "差评",
            ReputationLevel::Average => "一般",
            ReputationLevel::Good => "良好",
            ReputationLevel::Excellent => "优秀",
            ReputationLevel::Outstanding => "杰出",
            ReputationLevel::Legendary => "传说",
        }
    }

    /// 获取最小分数
    pub fn min_score(&self) -> u32 {
        match self {
            ReputationLevel::Poor => 0,
            ReputationLevel::Average => 21,
            ReputationLevel::Good => 41,
            ReputationLevel::Excellent => 61,
            ReputationLevel::Outstanding => 81,
            ReputationLevel::Legendary => 101,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::shop::facility::FacilityZone;

    #[test]
    fn test_reputation_creation() {
        let rep = Reputation::new();
        assert_eq!(rep.score, 30.0);
        assert_eq!(rep.dish_quality, 50.0);
    }

    #[test]
    fn test_reputation_update() {
        let mut rep = Reputation::new();
        let zones = vec![
            ZoneLevel::new(FacilityZone::Restaurant),
            ZoneLevel::new(FacilityZone::Kitchen),
        ];

        rep.update_dish_quality(80.0);
        rep.update_service_quality(70.0);
        rep.update_environment_score(60.0);
        rep.update_score(&zones);

        assert!(rep.score > 30.0);
    }

    #[test]
    fn test_reputation_level() {
        let mut rep = Reputation::new();

        rep.score = 15.0;
        assert_eq!(rep.get_level(), ReputationLevel::Poor);

        rep.score = 50.0;
        assert_eq!(rep.get_level(), ReputationLevel::Good);

        rep.score = 90.0;
        assert_eq!(rep.get_level(), ReputationLevel::Outstanding);
    }

    #[test]
    fn test_customer_attraction() {
        let mut rep = Reputation::new();

        rep.score = 0.0;
        assert_eq!(rep.customer_attraction(), 0.0);

        rep.score = 100.0;
        assert_eq!(rep.customer_attraction(), 1.5);
    }

    #[test]
    fn test_price_bonus() {
        let mut rep = Reputation::new();

        rep.score = 50.0;
        assert_eq!(rep.price_bonus(), 0.0);

        rep.score = 100.0;
        assert_eq!(rep.price_bonus(), 0.5);

        rep.score = 0.0;
        assert_eq!(rep.price_bonus(), -0.5);
    }
}
