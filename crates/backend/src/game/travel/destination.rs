//! 旅行目的地定义

use serde::{Deserialize, Serialize};

/// 目的地分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestinationCategory {
    /// 本地
    Local,
    /// 国内城市
    Domestic,
    /// 国际城市
    International,
    /// 特殊地点
    Special,
}

impl DestinationCategory {
    /// 获取分类名称
    pub fn name(&self) -> &str {
        match self {
            DestinationCategory::Local => "本地",
            DestinationCategory::Domestic => "国内",
            DestinationCategory::International => "国际",
            DestinationCategory::Special => "特殊",
        }
    }

    /// 获取基础旅行时间（小时）
    pub fn base_travel_hours(&self) -> u32 {
        match self {
            DestinationCategory::Local => 2,
            DestinationCategory::Domestic => 24,
            DestinationCategory::International => 72,
            DestinationCategory::Special => 48,
        }
    }
}

/// 目的地特色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationFeature {
    /// 特色名称
    pub name: String,
    /// 特色描述
    pub description: String,
    /// 可获得的菜谱类型
    pub recipe_types: Vec<String>,
}

/// 旅行目的地
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    /// 目的地 ID
    pub id: String,
    /// 目的地名称
    pub name: String,
    /// 分类
    pub category: DestinationCategory,
    /// 描述
    pub description: String,
    /// 地区
    pub region: String,
    /// 特色菜系
    pub cuisine_style: String,
    /// 特色列表
    pub features: Vec<DestinationFeature>,
    /// 基础旅行时间（小时）
    pub base_travel_hours: u32,
    /// 所需信任度
    pub required_trust: u32,
    /// 解锁条件描述
    pub unlock_condition: String,
    /// 可获得的菜谱 ID 列表
    pub available_recipes: Vec<String>,
    /// 可获得的特殊食材
    pub special_ingredients: Vec<String>,
    /// 风景描述
    pub scenery_description: String,
}

impl Destination {
    /// 创建新目的地
    pub fn new(
        id: String,
        name: String,
        category: DestinationCategory,
        region: String,
        cuisine_style: String,
    ) -> Self {
        Self {
            base_travel_hours: category.base_travel_hours(),
            id,
            name,
            category,
            description: String::new(),
            region,
            cuisine_style,
            features: Vec::new(),
            required_trust: 0,
            unlock_condition: String::new(),
            available_recipes: Vec::new(),
            special_ingredients: Vec::new(),
            scenery_description: String::new(),
        }
    }

    /// 添加特色
    pub fn add_feature(&mut self, feature: DestinationFeature) {
        self.features.push(feature);
    }

    /// 获取旅行时间（考虑移动模块等级）
    pub fn get_travel_time(&self, mobility_level: u32) -> u32 {
        // 移动模块等级越高，旅行时间越短
        // 等级 1: +50%, 等级 10: -40%
        let modifier = match mobility_level {
            1 => 1.5,
            2 => 1.4,
            3 => 1.3,
            4 => 1.2,
            5 => 1.1,
            6 => 1.0,
            7 => 0.9,
            8 => 0.8,
            9 => 0.7,
            10 => 0.6,
            _ => 1.0,
        };
        (self.base_travel_hours as f32 * modifier) as u32
    }

    /// 检查是否满足解锁条件
    pub fn can_unlock(&self, trust_level: u32) -> bool {
        trust_level >= self.required_trust
    }
}

/// 目的地管理器
pub struct DestinationManager;

impl DestinationManager {
    /// 获取默认目的地列表
    pub fn default_destinations() -> Vec<Destination> {
        vec![
            // 本地
            Self::create_local_destination(),
            // 国内城市
            Self::create_chengdu_destination(),
            Self::create_xian_destination(),
            Self::create_guangzhou_destination(),
            Self::create_hangzhou_destination(),
            // 国际城市
            Self::create_paris_destination(),
            Self::create_kyoto_destination(),
            Self::create_bangkok_destination(),
        ]
    }

    fn create_local_destination() -> Destination {
        let mut dest = Destination::new(
            "local".to_string(),
            "老街周边".to_string(),
            DestinationCategory::Local,
            "本地".to_string(),
            "家常菜".to_string(),
        );
        dest.description = "探索小馆周边的老街区，寻找传统味道。".to_string();
        dest.scenery_description = "熟悉的街道，古老的建筑，充满生活气息的市井。".to_string();
        dest.unlock_condition = "默认解锁".to_string();
        dest.available_recipes = vec!["home_tomato_egg".to_string(), "home_braised_pork".to_string()];
        dest
    }

    fn create_chengdu_destination() -> Destination {
        let mut dest = Destination::new(
            "chengdu".to_string(),
            "成都".to_string(),
            DestinationCategory::Domestic,
            "西南".to_string(),
            "川菜".to_string(),
        );
        dest.description = "天府之国，美食之都。麻辣鲜香的川菜令人回味无穷。".to_string();
        dest.scenery_description = "宽窄巷子的青石板路，锦里古街的红灯笼，还有那弥漫在空气中的麻辣香气。".to_string();
        dest.required_trust = 30;
        dest.unlock_condition = "信任度达到 30".to_string();
        dest.features.push(DestinationFeature {
            name: "宽窄巷子".to_string(),
            description: "成都最具代表性的历史文化街区".to_string(),
            recipe_types: vec!["川菜".to_string(), "小吃".to_string()],
        });
        dest.features.push(DestinationFeature {
            name: "锦里古街".to_string(),
            description: "三国文化与成都民俗的交融之地".to_string(),
            recipe_types: vec!["传统川菜".to_string()],
        });
        dest.available_recipes = vec![
            "mapo_tofu".to_string(),
            "twice_cooked_pork".to_string(),
            "kung_pao_chicken".to_string(),
        ];
        dest.special_ingredients = vec![
            "sichuan_peppercorn".to_string(),
            "pixian_bean_paste".to_string(),
        ];
        dest
    }

    fn create_xian_destination() -> Destination {
        let mut dest = Destination::new(
            "xian".to_string(),
            "西安".to_string(),
            DestinationCategory::Domestic,
            "西北".to_string(),
            "陕菜".to_string(),
        );
        dest.description = "古都长安，丝绸之路的起点。厚重的面食文化与浓郁的西北风味。".to_string();
        dest.scenery_description = "古城墙的厚重，回民街的热闹，还有那飘香四溢的羊肉泡馍。".to_string();
        dest.required_trust = 40;
        dest.unlock_condition = "信任度达到 40".to_string();
        dest.features.push(DestinationFeature {
            name: "回民街".to_string(),
            description: "千年古街，西安美食的聚集地".to_string(),
            recipe_types: vec!["清真菜".to_string(), "面食".to_string()],
        });
        dest.available_recipes = vec![
            "yangrou_paomo".to_string(),
            "roujiamo".to_string(),
            "biangbiang_noodles".to_string(),
        ];
        dest.special_ingredients = vec!["cumin".to_string(), "lamb".to_string()];
        dest
    }

    fn create_guangzhou_destination() -> Destination {
        let mut dest = Destination::new(
            "guangzhou".to_string(),
            "广州".to_string(),
            DestinationCategory::Domestic,
            "华南".to_string(),
            "粤菜".to_string(),
        );
        dest.description = "食在广州，粤菜之乡。清淡鲜美，讲究原汁原味。".to_string();
        dest.scenery_description = "早茶楼的茶香，沙面岛的异国情调，珠江两岸的璀璨夜景。".to_string();
        dest.required_trust = 50;
        dest.unlock_condition = "信任度达到 50".to_string();
        dest.features.push(DestinationFeature {
            name: "上下九步行街".to_string(),
            description: "广州传统商业街区，老字号美食云集".to_string(),
            recipe_types: vec!["粤菜".to_string(), "点心".to_string()],
        });
        dest.available_recipes = vec![
            "dim_sum".to_string(),
            "white_cut_chicken".to_string(),
            "roast_goose".to_string(),
        ];
        dest.special_ingredients = vec!["cantonese_sausage".to_string(), "oyster_sauce".to_string()];
        dest
    }

    fn create_hangzhou_destination() -> Destination {
        let mut dest = Destination::new(
            "hangzhou".to_string(),
            "杭州".to_string(),
            DestinationCategory::Domestic,
            "华东".to_string(),
            "杭帮菜".to_string(),
        );
        dest.description = "人间天堂，西湖美景。精致的江南菜肴，诗意的生活美学。".to_string();
        dest.scenery_description = "西湖的烟雨，龙井的茶香，河坊街的古韵。".to_string();
        dest.required_trust = 45;
        dest.unlock_condition = "信任度达到 45".to_string();
        dest.features.push(DestinationFeature {
            name: "河坊街".to_string(),
            description: "杭州历史文化街区，传统美食汇聚".to_string(),
            recipe_types: vec!["杭帮菜".to_string(), "江南小吃".to_string()],
        });
        dest.available_recipes = vec![
            "dongpo_pork".to_string(),
            "west_lake_fish".to_string(),
            "longjing_shrimp".to_string(),
        ];
        dest.special_ingredients = vec!["longjing_tea".to_string(), "lotus_root".to_string()];
        dest
    }

    fn create_paris_destination() -> Destination {
        let mut dest = Destination::new(
            "paris".to_string(),
            "巴黎".to_string(),
            DestinationCategory::International,
            "欧洲".to_string(),
            "法餐".to_string(),
        );
        dest.description = "浪漫之都，世界美食的殿堂。精致优雅的法式料理艺术。".to_string();
        dest.scenery_description = "塞纳河畔的咖啡香，香榭丽舍大道的优雅，埃菲尔铁塔的浪漫。".to_string();
        dest.required_trust = 70;
        dest.unlock_condition = "信任度达到 70".to_string();
        dest.features.push(DestinationFeature {
            name: "拉丁区".to_string(),
            description: "巴黎最古老的街区，充满艺术气息".to_string(),
            recipe_types: vec!["法式料理".to_string(), "甜点".to_string()],
        });
        dest.available_recipes = vec![
            "french_onion_soup".to_string(),
            "coq_au_vin".to_string(),
            "croissant".to_string(),
        ];
        dest.special_ingredients = vec!["french_butter".to_string(), "herbes_de_provence".to_string()];
        dest
    }

    fn create_kyoto_destination() -> Destination {
        let mut dest = Destination::new(
            "kyoto".to_string(),
            "京都".to_string(),
            DestinationCategory::International,
            "东亚".to_string(),
            "日料".to_string(),
        );
        dest.description = "千年古都，日本传统文化的精髓。禅意与美食的完美融合。".to_string();
        dest.scenery_description = "金阁寺的倒影，岚山的竹林，祇园的艺伎文化。".to_string();
        dest.required_trust = 60;
        dest.unlock_condition = "信任度达到 60".to_string();
        dest.features.push(DestinationFeature {
            name: "祇园".to_string(),
            description: "京都最著名的艺伎区，传统日料聚集地".to_string(),
            recipe_types: vec!["怀石料理".to_string(), "寿司".to_string()],
        });
        dest.available_recipes = vec![
            "kaiseki".to_string(),
            "sushi".to_string(),
            "tempura".to_string(),
        ];
        dest.special_ingredients = vec!["miso".to_string(), "wasabi".to_string()];
        dest
    }

    fn create_bangkok_destination() -> Destination {
        let mut dest = Destination::new(
            "bangkok".to_string(),
            "曼谷".to_string(),
            DestinationCategory::International,
            "东南亚".to_string(),
            "泰餐".to_string(),
        );
        dest.description = "天使之城，香料的天堂。酸辣甜咸的完美平衡。".to_string();
        dest.scenery_description = "大皇宫的金碧辉煌，湄南河的繁忙，街头小吃的诱人香气。".to_string();
        dest.required_trust = 55;
        dest.unlock_condition = "信任度达到 55".to_string();
        dest.features.push(DestinationFeature {
            name: "考山路".to_string(),
            description: "背包客天堂，街头美食的汇聚地".to_string(),
            recipe_types: vec!["泰式料理".to_string(), "街头小吃".to_string()],
        });
        dest.available_recipes = vec![
            "tom_yum_kung".to_string(),
            "pad_thai".to_string(),
            "green_curry".to_string(),
        ];
        dest.special_ingredients = vec!["lemongrass".to_string(), "coconut_milk".to_string()];
        dest
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination_creation() {
        let dest = Destination::new(
            "test".to_string(),
            "测试目的地".to_string(),
            DestinationCategory::Domestic,
            "测试地区".to_string(),
            "测试菜系".to_string(),
        );

        assert_eq!(dest.id, "test");
        assert_eq!(dest.category, DestinationCategory::Domestic);
        assert_eq!(dest.base_travel_hours, 24);
    }

    #[test]
    fn test_travel_time_modifier() {
        let dest = Destination::new(
            "test".to_string(),
            "测试".to_string(),
            DestinationCategory::Domestic,
            "测试".to_string(),
            "测试".to_string(),
        );

        // 等级 1: +50%
        assert_eq!(dest.get_travel_time(1), 36);
        // 等级 10: -40%
        assert_eq!(dest.get_travel_time(10), 14);
    }

    #[test]
    fn test_default_destinations() {
        let destinations = DestinationManager::default_destinations();

        assert!(!destinations.is_empty());
        assert!(destinations.iter().any(|d| d.id == "local"));
        assert!(destinations.iter().any(|d| d.id == "chengdu"));
    }

    #[test]
    fn test_can_unlock() {
        let dest = Destination {
            id: "test".to_string(),
            name: "测试".to_string(),
            category: DestinationCategory::Domestic,
            description: String::new(),
            region: String::new(),
            cuisine_style: String::new(),
            features: Vec::new(),
            base_travel_hours: 24,
            required_trust: 50,
            unlock_condition: String::new(),
            available_recipes: Vec::new(),
            special_ingredients: Vec::new(),
            scenery_description: String::new(),
        };

        assert!(!dest.can_unlock(30));
        assert!(dest.can_unlock(50));
        assert!(dest.can_unlock(70));
    }
}
