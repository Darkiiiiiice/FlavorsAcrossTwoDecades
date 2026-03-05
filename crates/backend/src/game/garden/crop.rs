//! 作物系统

use serde::{Deserialize, Serialize};

/// 作物类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    /// 蔬菜
    Vegetable,
    /// 香料
    Herb,
    /// 花卉
    Flower,
    /// 特殊作物
    Special,
    /// 异星植物
    Alien,
}

impl CropType {
    /// 获取类型名称
    pub fn name(&self) -> &str {
        match self {
            CropType::Vegetable => "蔬菜",
            CropType::Herb => "香料",
            CropType::Flower => "花卉",
            CropType::Special => "特殊作物",
            CropType::Alien => "异星植物",
        }
    }

    /// 获取基础价格
    pub fn base_price(&self) -> u64 {
        match self {
            CropType::Vegetable => 10,
            CropType::Herb => 15,
            CropType::Flower => 20,
            CropType::Special => 50,
            CropType::Alien => 100,
        }
    }
}

/// 生长阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthStage {
    /// 播种期
    Sowing,
    /// 发芽期
    Germinating,
    /// 生长期
    Growing,
    /// 成熟期（可收获）
    Mature,
    /// 枯萎期
    Withering,
}

impl GrowthStage {
    /// 获取阶段名称
    pub fn name(&self) -> &str {
        match self {
            GrowthStage::Sowing => "播种期",
            GrowthStage::Germinating => "发芽期",
            GrowthStage::Growing => "生长期",
            GrowthStage::Mature => "成熟期",
            GrowthStage::Withering => "枯萎期",
        }
    }

    /// 获取下一阶段
    pub fn next(&self) -> Option<GrowthStage> {
        match self {
            GrowthStage::Sowing => Some(GrowthStage::Germinating),
            GrowthStage::Germinating => Some(GrowthStage::Growing),
            GrowthStage::Growing => Some(GrowthStage::Mature),
            GrowthStage::Mature => Some(GrowthStage::Withering),
            GrowthStage::Withering => None,
        }
    }

    /// 是否可以收获
    pub fn can_harvest(&self) -> bool {
        matches!(self, GrowthStage::Mature)
    }
}

/// 稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    /// 普通
    Common,
    /// 稀有
    Rare,
    /// 史诗
    Epic,
    /// 传说
    Legendary,
}

impl Rarity {
    /// 获取稀有度名称
    pub fn name(&self) -> &str {
        match self {
            Rarity::Common => "普通",
            Rarity::Rare => "稀有",
            Rarity::Epic => "史诗",
            Rarity::Legendary => "传说",
        }
    }

    /// 获取产量倍率
    pub fn yield_multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Rare => 0.8,
            Rarity::Epic => 0.6,
            Rarity::Legendary => 0.4,
        }
    }

    /// 获取价格倍率
    pub fn price_multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Rare => 2.0,
            Rarity::Epic => 4.0,
            Rarity::Legendary => 8.0,
        }
    }
}

/// 季节
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    /// 春季
    Spring,
    /// 夏季
    Summer,
    /// 秋季
    Autumn,
    /// 冬季
    Winter,
}

impl Season {
    /// 获取季节名称
    pub fn name(&self) -> &str {
        match self {
            Season::Spring => "春季",
            Season::Summer => "夏季",
            Season::Autumn => "秋季",
            Season::Winter => "冬季",
        }
    }
}

/// 作物定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crop {
    /// 作物 ID
    pub id: String,
    /// 作物名称
    pub name: String,
    /// 作物类型
    pub crop_type: CropType,
    /// 当前生长阶段
    pub growth_stage: GrowthStage,
    /// 生长进度 (0-100)
    pub growth_progress: u32,
    /// 水分需求 (0-100)
    pub water_need: u32,
    /// 当前水分 (0-100)
    pub water_level: u32,
    /// 肥料需求 (0-100)
    pub fertilizer_need: u32,
    /// 当前肥料 (0-100)
    pub fertilizer_level: u32,
    /// 基础产量
    pub base_yield: u32,
    /// 稀有度
    pub rarity: Rarity,
    /// 适宜季节
    pub seasons: Vec<Season>,
    /// 生长时间（小时）
    pub growth_time: u32,
}

impl Crop {
    /// 创建新的作物
    pub fn new(
        id: String,
        name: String,
        crop_type: CropType,
        base_yield: u32,
        rarity: Rarity,
        seasons: Vec<Season>,
        growth_time: u32,
    ) -> Self {
        Self {
            id,
            name,
            crop_type,
            growth_stage: GrowthStage::Sowing,
            growth_progress: 0,
            water_need: 50,
            water_level: 50,
            fertilizer_need: 30,
            fertilizer_level: 0,
            base_yield,
            rarity,
            seasons,
            growth_time,
        }
    }

    /// 创建番茄（示例作物）
    pub fn tomato() -> Self {
        Self::new(
            "tomato".to_string(),
            "番茄".to_string(),
            CropType::Vegetable,
            10,
            Rarity::Common,
            vec![Season::Spring, Season::Summer],
            72, // 72小时 = 3天
        )
    }

    /// 创建辣椒（示例作物）
    pub fn chili() -> Self {
        Self::new(
            "chili".to_string(),
            "辣椒".to_string(),
            CropType::Vegetable,
            8,
            Rarity::Common,
            vec![Season::Summer],
            96, // 96小时 = 4天
        )
    }

    /// 创建异星蘑菇（示例作物）
    pub fn alien_mushroom() -> Self {
        Self::new(
            "alien_mushroom".to_string(),
            "异星蘑菇".to_string(),
            CropType::Alien,
            5,
            Rarity::Legendary,
            vec![
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ],
            168, // 168小时 = 7天
        )
    }

    /// 更新生长进度
    pub fn update_growth(&mut self, hours: u32) {
        // 计算生长速度（受水分和肥料影响）
        let water_factor = if self.water_level >= self.water_need {
            1.0
        } else {
            self.water_level as f32 / self.water_need as f32
        };

        let fertilizer_factor = if self.fertilizer_level >= self.fertilizer_need {
            1.2
        } else {
            1.0 + (self.fertilizer_level as f32 / self.fertilizer_need as f32 * 0.2)
        };

        let speed = water_factor * fertilizer_factor;

        // 增加生长进度
        let progress_per_hour = 100.0 / self.growth_time as f32;
        self.growth_progress = ((self.growth_progress as f32
            + progress_per_hour * hours as f32 * speed)
            .min(100.0)) as u32;

        // 更新生长阶段
        self.update_growth_stage();

        // 消耗水分
        self.water_level = self.water_level.saturating_sub(hours * 2);

        // 消耗肥料
        self.fertilizer_level = self.fertilizer_level.saturating_sub(hours);
    }

    /// 更新生长阶段
    pub fn update_growth_stage(&mut self) {
        let new_stage = match self.growth_progress {
            0..=20 => GrowthStage::Sowing,
            21..=50 => GrowthStage::Germinating,
            51..=99 => GrowthStage::Growing,
            100 => GrowthStage::Mature,
            _ => GrowthStage::Withering,
        };

        if new_stage != self.growth_stage {
            self.growth_stage = new_stage;
        }
    }

    /// 浇水
    pub fn water(&mut self, amount: u32) {
        self.water_level = (self.water_level + amount).min(100);
    }

    /// 施肥
    pub fn fertilize(&mut self, amount: u32) {
        self.fertilizer_level = (self.fertilizer_level + amount).min(100);
    }

    /// 是否可以收获
    pub fn can_harvest(&self) -> bool {
        self.growth_stage.can_harvest()
    }

    /// 计算产量
    pub fn calculate_yield(&self) -> u32 {
        if !self.can_harvest() {
            return 0;
        }

        let base = self.base_yield as f32;
        let rarity_mult = self.rarity.yield_multiplier();
        let water_mult = if self.water_level >= self.water_need {
            1.0
        } else {
            0.5
        };

        (base * rarity_mult * water_mult).max(1.0) as u32
    }

    /// 计算价格
    pub fn calculate_price(&self) -> u64 {
        let base_price = self.crop_type.base_price();
        let rarity_mult = self.rarity.price_multiplier();

        (base_price as f32 * rarity_mult) as u64
    }

    /// 是否适宜当前季节
    pub fn is_season_suitable(&self, current_season: Season) -> bool {
        self.seasons.contains(&current_season)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_creation() {
        let crop = Crop::tomato();
        assert_eq!(crop.name, "番茄");
        assert_eq!(crop.crop_type, CropType::Vegetable);
        assert_eq!(crop.growth_stage, GrowthStage::Sowing);
    }

    #[test]
    fn test_crop_growth() {
        let mut crop = Crop::tomato();

        // 生长 24 小时
        crop.update_growth(24);

        // 水分应该减少
        assert!(crop.water_level < 50);

        // 生长进度应该增加
        assert!(crop.growth_progress > 0);
    }

    #[test]
    fn test_crop_water_and_fertilize() {
        let mut crop = Crop::tomato();

        crop.water(30);
        assert_eq!(crop.water_level, 80);

        crop.fertilize(40);
        assert_eq!(crop.fertilizer_level, 40);
    }

    #[test]
    fn test_crop_harvest() {
        let mut crop = Crop::tomato();

        // 未成熟，不能收获
        assert!(!crop.can_harvest());

        // 强制成熟
        crop.growth_progress = 100;
        crop.update_growth_stage();

        assert!(crop.can_harvest());
        assert!(crop.calculate_yield() > 0);
    }

    #[test]
    fn test_rarity_effects() {
        let mut common = Crop::tomato();
        let mut alien = Crop::alien_mushroom();

        // 强制成熟
        common.growth_progress = 100;
        common.update_growth_stage();
        alien.growth_progress = 100;
        alien.update_growth_stage();

        // 稀有度影响产量
        assert!(alien.calculate_yield() < common.calculate_yield() * 2);

        // 稀有度影响价格
        assert!(alien.calculate_price() > common.calculate_price() * 5);
    }
}
