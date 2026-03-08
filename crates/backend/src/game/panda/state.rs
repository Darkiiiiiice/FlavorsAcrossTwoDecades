//! Panda 状态系统

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum PandaLocation {
    Restaurant(RestaurantState),    // 餐厅
    Kitchen(KitchenState),          // 厨房
    Backyard(BackyardState),        // 后院
    Workshop(WorkshopState),        // 工作室
    ChargingStation(ChargingState), // 充电站
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChargingState {
    Charging,
    NotCharging,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkshopState {
    // 工作
    Working,
    // 实验
    Experimenting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestaurantState {
    Chatting, // 聊天
    Serving,  // 服务
    Cleaning, // 清洁
    Cooking,  // 烹饪
}

#[derive(Debug, Clone, PartialEq)]
pub enum KitchenState {
    // 洗菜
    WashingVegetables,
    // 切菜
    Cutting,
    // 炒菜
    Cooking,
    // 蒸煮
    Steaming,
    // 洗碗
    WashingDishes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackyardState {
    // 耕地
    Plowing,
    // 种地
    Planting,
    // 浇水
    Irrigating,
    // 除草
    Weeding,
    // 收获
    Harvesting,
}

/// 性格参数（0-100）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// 经营风格：理性(0) <-> 感性(100)，初始50
    pub business_style: u32,
    /// 创新倾向：保守(0) <-> 创新(100)，初始50
    pub innovation: u32,
    /// 独立倾向：服从(0) <-> 自主(100)，初始50
    pub independence: u32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            business_style: 50,
            innovation: 50,
            independence: 50,
        }
    }
}

impl Personality {
    /// 创建新的性格
    pub fn new() -> Self {
        Self::default()
    }

    /// 调整性格参数（限制在 0-100）
    pub fn adjust(&mut self, business_style: i32, innovation: i32, independence: i32) {
        self.business_style = (self.business_style as i32 + business_style).clamp(0, 100) as u32;
        self.innovation = (self.innovation as i32 + innovation).clamp(0, 100) as u32;
        self.independence = (self.independence as i32 + independence).clamp(0, 100) as u32;
    }

    /// 计算性格倾向描述
    pub fn describe(&self) -> String {
        let business = if self.business_style < 33 {
            "理性"
        } else if self.business_style > 66 {
            "感性"
        } else {
            "平衡"
        };

        let innov = if self.innovation < 33 {
            "保守"
        } else if self.innovation > 66 {
            "创新"
        } else {
            "稳健"
        };

        let indep = if self.independence < 33 {
            "服从"
        } else if self.independence > 66 {
            "自主"
        } else {
            "灵活"
        };

        format!("{}-{}-{}", business, innov, indep)
    }
}

/// 情绪状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Emotion {
    /// 开心 - 工作速度×1.1，错误率×0.9
    Happy,
    /// 平静 - 正常状态
    #[default]
    Calm,
    /// 疲惫 - 工作速度×0.9，错误率×1.2
    Tired,
    /// 困惑 - 错误率×1.1，更倾向请示
    Confused,
    /// 担忧 - 主动提醒问题
    Worried,
    /// 孤独 - 想引起注意
    Lonely,
    /// 兴奋 - 旅行时间×0.9
    Excited,
}

impl Emotion {
    /// 获取情绪名称
    pub fn name(&self) -> &str {
        match self {
            Emotion::Happy => "开心",
            Emotion::Calm => "平静",
            Emotion::Tired => "疲惫",
            Emotion::Confused => "困惑",
            Emotion::Worried => "担忧",
            Emotion::Lonely => "孤独",
            Emotion::Excited => "兴奋",
        }
    }

    /// 获取情绪图标
    pub fn icon(&self) -> &str {
        match self {
            Emotion::Happy => "😊",
            Emotion::Calm => "😐",
            Emotion::Tired => "😴",
            Emotion::Confused => "😕",
            Emotion::Worried => "😟",
            Emotion::Lonely => "😢",
            Emotion::Excited => "🤩",
        }
    }

    /// 根据条件更新情绪
    pub fn update_based_on_conditions(
        &mut self,
        battery: u32,
        trust_level: u32,
        recent_failures: u32,
    ) {
        // 低电量导致疲惫
        if battery < 20 {
            *self = Emotion::Tired;
            return;
        }

        // 低信任度导致孤独
        if trust_level < 30 {
            *self = Emotion::Lonely;
            return;
        }

        // 连续失败导致困惑或担忧
        if recent_failures >= 3 {
            *self = Emotion::Worried;
            return;
        }

        // 高信任度和满电量导致开心
        if trust_level > 80 && battery > 80 {
            *self = Emotion::Happy;
            return;
        }

        // 默认平静
        *self = Emotion::Calm;
    }
}

/// Panda 完整状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PandaFullState {
    /// 当前心情描述
    pub mood: String,
    /// 性格参数
    pub personality: Personality,
    /// 当前情绪
    pub emotion: Emotion,
    /// 最近失败次数（影响情绪）
    pub recent_failures: u32,
}

impl Default for PandaFullState {
    fn default() -> Self {
        Self {
            mood: "平静".to_string(),
            personality: Personality::default(),
            emotion: Emotion::default(),
            recent_failures: 0,
        }
    }
}

impl PandaFullState {
    /// 创建新的状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新心情
    pub fn update_mood(&mut self, mood: String) {
        self.mood = mood;
    }

    /// 记录失败
    pub fn record_failure(&mut self) {
        self.recent_failures += 1;
    }

    /// 重置失败计数
    pub fn reset_failures(&mut self) {
        self.recent_failures = 0;
    }

    /// 更新情绪（基于当前条件）
    pub fn update_emotion(&mut self, battery: u32, trust_level: u32) {
        self.emotion
            .update_based_on_conditions(battery, trust_level, self.recent_failures);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_creation() {
        let personality = Personality::new();
        assert_eq!(personality.business_style, 50);
        assert_eq!(personality.innovation, 50);
        assert_eq!(personality.independence, 50);
    }

    #[test]
    fn test_personality_adjustment() {
        let mut personality = Personality::new();
        personality.adjust(20, -30, 10);

        assert_eq!(personality.business_style, 70);
        assert_eq!(personality.innovation, 20);
        assert_eq!(personality.independence, 60);
    }

    #[test]
    fn test_emotion_update() {
        let mut emotion = Emotion::Calm;

        // 低电量导致疲惫
        emotion.update_based_on_conditions(10, 50, 0);
        assert_eq!(emotion, Emotion::Tired);

        // 低信任度导致孤独
        emotion.update_based_on_conditions(80, 20, 0);
        assert_eq!(emotion, Emotion::Lonely);

        // 连续失败导致担忧
        emotion.update_based_on_conditions(80, 50, 3);
        assert_eq!(emotion, Emotion::Worried);

        // 高信任度和满电量导致开心
        emotion.update_based_on_conditions(90, 90, 0);
        assert_eq!(emotion, Emotion::Happy);
    }

    #[test]
    fn test_personality_describe() {
        let mut personality = Personality::new();

        assert_eq!(personality.describe(), "平衡-稳健-灵活");

        personality.business_style = 20;
        personality.innovation = 80;
        personality.independence = 30;
        assert_eq!(personality.describe(), "理性-创新-服从");
    }
}
