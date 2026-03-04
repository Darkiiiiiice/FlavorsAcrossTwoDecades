//! 菜地系统

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::crop::{Crop, GrowthStage};

/// 菜地状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotState {
    /// 空闲
    Empty,
    /// 种植中
    Planted,
    /// 生长中
    Growing,
    /// 成熟（可收获）
    Mature,
    /// 枯萎
    Withered,
}

/// 病虫害类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PestType {
    /// 蚜虫
    Aphids,
    /// 霉菌
    Mold,
    /// 杂草
    Weeds,
    /// 鸟雀
    Birds,
}

impl PestType {
    /// 获取病虫害名称
    pub fn name(&self) -> &str {
        match self {
            PestType::Aphids => "蚜虫",
            PestType::Mold => "霉菌",
            PestType::Weeds => "杂草",
            PestType::Birds => "鸟雀",
        }
    }

    /// 获取处理方法
    pub fn treatment(&self) -> &str {
        match self {
            PestType::Aphids => "喷洒药水",
            PestType::Mold => "杀菌剂+通风",
            PestType::Weeds => "手工拔除",
            PestType::Birds => "稻草人",
        }
    }

    /// 获取严重程度影响（0-1）
    pub fn severity_impact(&self) -> f32 {
        match self {
            PestType::Aphids => 0.3,
            PestType::Mold => 0.4,
            PestType::Weeds => 0.2,
            PestType::Birds => 0.5,
        }
    }
}

impl PlotState {
    /// 获取状态名称
    pub fn name(&self) -> &str {
        match self {
            PlotState::Empty => "空闲",
            PlotState::Planted => "种植中",
            PlotState::Growing => "生长中",
            PlotState::Mature => "成熟",
            PlotState::Withered => "枯萎",
        }
    }
}

/// 菜地
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GardenPlot {
    /// 菜地 ID
    pub id: u32,
    /// 菜地状态
    pub state: PlotState,
    /// 种植的作物
    pub crop: Option<Crop>,
    /// 病虫害列表
    pub pests: Vec<PestType>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 肥力 (0-100)
    pub fertility: u32,
}

impl GardenPlot {
    /// 创建新的菜地
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: PlotState::Empty,
            crop: None,
            pests: Vec::new(),
            updated_at: Utc::now(),
            fertility: 50,
        }
    }

    /// 种植作物
    pub fn plant(&mut self, crop: Crop) -> Result<(), String> {
        if self.state != PlotState::Empty {
            return Err("菜地不为空".to_string());
        }

        self.state = PlotState::Planted;
        self.crop = Some(crop);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 浇水
    pub fn water(&mut self) {
        if let Some(crop) = &mut self.crop {
            crop.water(20);
        }
        self.updated_at = Utc::now();
    }

    /// 施肥
    pub fn fertilize(&mut self, amount: u32) {
        self.fertility = (self.fertility + amount).min(100);
        if let Some(crop) = &mut self.crop {
            crop.fertilize(amount);
        }
        self.updated_at = Utc::now();
    }

    /// 更新生长状态
    pub fn update_growth(&mut self) {
        if let Some(crop) = &mut self.crop {
            // 更新作物生长（假设每小时更新一次）
            crop.update_growth(1);

            // 更新菜地状态
            self.state = match crop.growth_stage {
                GrowthStage::Sowing => PlotState::Planted,
                GrowthStage::Germinating | GrowthStage::Growing => PlotState::Growing,
                GrowthStage::Mature => PlotState::Mature,
                GrowthStage::Withering => PlotState::Withered,
            };

            // 随机产生病虫害（10%概率）
            if rand::random::<f32>() < 0.1 && self.pests.len() < 2 {
                self.add_random_pest();
            }
        }

        self.updated_at = Utc::now();
    }

    /// 收获
    pub fn harvest(&mut self) -> Result<(String, u32), String> {
        if self.state != PlotState::Mature {
            return Err("作物未成熟".to_string());
        }

        if let Some(crop) = &self.crop {
            if !crop.can_harvest() {
                return Err("作物不能收获".to_string());
            }

            let name = crop.name.clone();
            let yield_amount = crop.calculate_yield();

            // 清空菜地
            self.state = PlotState::Empty;
            self.crop = None;
            self.pests.clear();
            self.fertility = self.fertility.saturating_sub(10);
            self.updated_at = Utc::now();

            return Ok((name, yield_amount));
        }

        Err("没有作物".to_string())
    }

    /// 添加随机病虫害
    fn add_random_pest(&mut self) {
        let pest = match rand::random::<u32>() % 4 {
            0 => PestType::Aphids,
            1 => PestType::Mold,
            2 => PestType::Weeds,
            _ => PestType::Birds,
        };

        if !self.pests.contains(&pest) {
            self.pests.push(pest);
        }
    }

    /// 处理病虫害
    pub fn treat_pest(&mut self, pest_type: PestType) -> Result<(), String> {
        if let Some(index) = self.pests.iter().position(|&p| p == pest_type) {
            self.pests.remove(index);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("没有这种病虫害".to_string())
        }
    }

    /// 检查是否有作物
    pub fn has_crop(&self) -> bool {
        self.crop.is_some()
    }

    /// 获取产量
    pub fn get_yield(&self) -> u32 {
        if let Some(crop) = &self.crop {
            if crop.can_harvest() {
                return crop.calculate_yield();
            }
        }
        0
    }

    /// 计算病虫害影响
    pub fn pest_impact(&self) -> f32 {
        self.pests.iter().map(|p| p.severity_impact()).sum()
    }

    /// 是否需要处理
    pub fn needs_treatment(&self) -> bool {
        !self.pests.is_empty()
    }

    /// 清理枯萎作物
    pub fn clear_withered(&mut self) -> bool {
        if self.state == PlotState::Withered {
            self.state = PlotState::Empty;
            self.crop = None;
            self.pests.clear();
            self.fertility = self.fertility.saturating_sub(20);
            self.updated_at = Utc::now();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::garden::crop::CropType;

    #[test]
    fn test_plot_creation() {
        let plot = GardenPlot::new(1);
        assert_eq!(plot.id, 1);
        assert_eq!(plot.state, PlotState::Empty);
        assert!(!plot.has_crop());
    }

    #[test]
    fn test_plot_plant() {
        let mut plot = GardenPlot::new(1);
        let crop = Crop::tomato();

        let result = plot.plant(crop);
        assert!(result.is_ok());
        assert_eq!(plot.state, PlotState::Planted);
        assert!(plot.has_crop());
    }

    #[test]
    fn test_plot_plant_twice() {
        let mut plot = GardenPlot::new(1);

        plot.plant(Crop::tomato()).unwrap();
        let result = plot.plant(Crop::chili());

        assert!(result.is_err());
    }

    #[test]
    fn test_plot_water_and_fertilize() {
        let mut plot = GardenPlot::new(1);
        plot.plant(Crop::tomato()).unwrap();

        plot.water();
        assert_eq!(plot.crop.as_ref().unwrap().water_level, 70);

        plot.fertilize(30);
        assert_eq!(plot.crop.as_ref().unwrap().fertilizer_level, 30);
    }

    #[test]
    fn test_plot_harvest() {
        let mut plot = GardenPlot::new(1);
        let mut crop = Crop::tomato();
        crop.growth_progress = 100;
        crop.update_growth_stage();

        plot.plant(crop).unwrap();
        plot.state = PlotState::Mature;

        let result = plot.harvest();
        assert!(result.is_ok());

        let (name, yield_amount) = result.unwrap();
        assert_eq!(name, "番茄");
        assert!(yield_amount > 0);
        assert_eq!(plot.state, PlotState::Empty);
    }

    #[test]
    fn test_plot_harvest_immature() {
        let mut plot = GardenPlot::new(1);
        plot.plant(Crop::tomato()).unwrap();

        let result = plot.harvest();
        assert!(result.is_err());
    }

    #[test]
    fn test_pest_treatment() {
        let mut plot = GardenPlot::new(1);
        plot.pests.push(PestType::Aphids);

        assert!(plot.needs_treatment());

        let result = plot.treat_pest(PestType::Aphids);
        assert!(result.is_ok());
        assert!(!plot.needs_treatment());
    }
}
