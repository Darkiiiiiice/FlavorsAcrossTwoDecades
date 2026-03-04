//! 菜园种植系统模块

mod crop;
mod plot;

pub use crop::{Crop, CropType, GrowthStage, Rarity, Season};
pub use plot::{GardenPlot, PestType, PlotState};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 菜园系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Garden {
    /// 存档 ID
    pub save_id: Uuid,
    /// 菜地列表
    pub plots: Vec<GardenPlot>,
    /// 菜园等级 (1-5)
    pub level: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl Garden {
    /// 创建新的菜园
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            plots: vec![GardenPlot::new(1)],
            level: 1,
            updated_at: Utc::now(),
        }
    }

    /// 升级菜园
    pub fn upgrade(&mut self) -> Result<u32, String> {
        if self.level >= 5 {
            return Err("菜园已达到最高等级".to_string());
        }

        self.level += 1;

        // 添加新的菜地
        if self.plots.len() < self.level as usize {
            self.plots.push(GardenPlot::new(self.plots.len() as u32 + 1));
        }

        self.updated_at = Utc::now();
        Ok(self.level)
    }

    /// 获取指定菜地
    pub fn get_plot(&self, plot_id: u32) -> Option<&GardenPlot> {
        self.plots.iter().find(|p| p.id == plot_id)
    }

    /// 获取指定菜地（可变）
    pub fn get_plot_mut(&mut self, plot_id: u32) -> Option<&mut GardenPlot> {
        self.plots.iter_mut().find(|p| p.id == plot_id)
    }

    /// 在指定菜地种植
    pub fn plant(&mut self, plot_id: u32, crop: Crop) -> Result<(), String> {
        let plot = self
            .get_plot_mut(plot_id)
            .ok_or("菜地不存在".to_string())?;

        plot.plant(crop)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 浇水
    pub fn water(&mut self, plot_id: u32) -> Result<(), String> {
        let plot = self
            .get_plot_mut(plot_id)
            .ok_or("菜地不存在".to_string())?;

        plot.water();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 施肥
    pub fn fertilize(&mut self, plot_id: u32, amount: u32) -> Result<(), String> {
        let plot = self
            .get_plot_mut(plot_id)
            .ok_or("菜地不存在".to_string())?;

        plot.fertilize(amount);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 收获
    pub fn harvest(&mut self, plot_id: u32) -> Result<(String, u32), String> {
        let plot = self
            .get_plot_mut(plot_id)
            .ok_or("菜地不存在".to_string())?;

        let result = plot.harvest()?;
        self.updated_at = Utc::now();
        Ok(result)
    }

    /// 更新所有菜地的生长状态
    pub fn update_growth(&mut self) {
        for plot in &mut self.plots {
            plot.update_growth();
        }
        self.updated_at = Utc::now();
    }

    /// 获取菜园总产量
    pub fn total_yield(&self) -> u32 {
        self.plots.iter().map(|p| p.get_yield()).sum()
    }

    /// 获取升级成本
    pub fn get_upgrade_cost(&self) -> u64 {
        400 * self.level as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garden_creation() {
        let save_id = Uuid::new_v4();
        let garden = Garden::new(save_id);

        assert_eq!(garden.level, 1);
        assert_eq!(garden.plots.len(), 1);
    }

    #[test]
    fn test_garden_upgrade() {
        let save_id = Uuid::new_v4();
        let mut garden = Garden::new(save_id);

        let result = garden.upgrade();
        assert!(result.is_ok());
        assert_eq!(garden.level, 2);
        assert_eq!(garden.plots.len(), 2);
    }

    #[test]
    fn test_garden_max_level() {
        let save_id = Uuid::new_v4();
        let mut garden = Garden::new(save_id);

        for _ in 1..5 {
            garden.upgrade().unwrap();
        }

        let result = garden.upgrade();
        assert!(result.is_err());
        assert_eq!(garden.level, 5);
    }

    #[test]
    fn test_plant_crop() {
        let save_id = Uuid::new_v4();
        let mut garden = Garden::new(save_id);

        let crop = Crop::tomato();
        let result = garden.plant(1, crop);

        assert!(result.is_ok());
        assert!(garden.get_plot(1).unwrap().has_crop());
    }
}
