//! 邻居角色定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 邻居能力类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeighborAbility {
    /// 种植指导
    Gardening,
    /// 维修技能
    Repair,
    /// 食材批发
    IngredientWholesale,
    /// 快递服务
    DeliveryService,
    /// 烹饪建议
    CookingAdvice,
    /// 故事讲述
    Storytelling,
}

impl NeighborAbility {
    /// 获取能力名称
    pub fn name(&self) -> &str {
        match self {
            NeighborAbility::Gardening => "种植指导",
            NeighborAbility::Repair => "维修技能",
            NeighborAbility::IngredientWholesale => "食材批发",
            NeighborAbility::DeliveryService => "快递服务",
            NeighborAbility::CookingAdvice => "烹饪建议",
            NeighborAbility::Storytelling => "故事讲述",
        }
    }

    /// 获取能力描述
    pub fn description(&self) -> &str {
        match self {
            NeighborAbility::Gardening => "可以提供种植技巧和花种",
            NeighborAbility::Repair => "可以帮忙维修设备和设施",
            NeighborAbility::IngredientWholesale => "可以批发价格采购食材",
            NeighborAbility::DeliveryService => "可以代收快递和包裹",
            NeighborAbility::CookingAdvice => "可以分享烹饪心得",
            NeighborAbility::Storytelling => "可以讲述老街的历史故事",
        }
    }
}

/// 邻居角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    /// 邻居 ID
    pub id: String,
    /// 姓名
    pub name: String,
    /// 头像
    pub avatar: String,
    /// 年龄
    pub age: u32,
    /// 职业/身份
    pub occupation: String,
    /// 性格描述
    pub personality: String,
    /// 能力列表
    pub abilities: Vec<NeighborAbility>,
    /// 背景故事
    pub background: String,
    /// 喜欢的礼物
    pub favorite_gifts: Vec<String>,
    /// 讨厌的东西
    pub disliked_items: Vec<String>,
    /// 对话风格
    pub dialogue_style: String,
}

impl Neighbor {
    /// 创建新邻居
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            avatar: String::new(),
            age: 50,
            occupation: String::new(),
            personality: String::new(),
            abilities: Vec::new(),
            background: String::new(),
            favorite_gifts: Vec::new(),
            disliked_items: Vec::new(),
            dialogue_style: String::new(),
        }
    }

    /// 添加能力
    pub fn with_ability(mut self, ability: NeighborAbility) -> Self {
        self.abilities.push(ability);
        self
    }

    /// 检查是否有某能力
    pub fn has_ability(&self, ability: NeighborAbility) -> bool {
        self.abilities.contains(&ability)
    }

    /// 创建王奶奶（种植专家）
    pub fn grandma_wang() -> Self {
        Self {
            id: "grandma_wang".to_string(),
            name: "王奶奶".to_string(),
            avatar: "👵".to_string(),
            age: 72,
            occupation: "退休花农".to_string(),
            personality: "和蔼可亲，热心肠，喜欢分享种植经验".to_string(),
            abilities: vec![NeighborAbility::Gardening, NeighborAbility::CookingAdvice],
            background: "王奶奶在老街住了一辈子，年轻时是远近闻名的花农。她的阳台上总是种满了各种花草，对植物有着独特的心得。".to_string(),
            favorite_gifts: vec!["花种".to_string(), "茶叶".to_string()],
            disliked_items: vec!["快餐".to_string()],
            dialogue_style: "慈祥温和，说话慢条斯理".to_string(),
        }
    }

    /// 创建李大爷（维修能手）
    pub fn uncle_li() -> Self {
        Self {
            id: "uncle_li".to_string(),
            name: "李大爷".to_string(),
            avatar: "👴".to_string(),
            age: 68,
            occupation: "退休工人".to_string(),
            personality: "豪爽直率，手巧心细，喜欢喝酒".to_string(),
            abilities: vec![NeighborAbility::Repair, NeighborAbility::Storytelling],
            background: "李大爷是祖父的老朋友，年轻时在工厂当技工。他什么都会修，是小馆的常客，总爱讲过去的故事。".to_string(),
            favorite_gifts: vec!["二锅头".to_string(), "花生米".to_string()],
            disliked_items: vec!["甜食".to_string()],
            dialogue_style: "豪爽直率，爱说俏皮话".to_string(),
        }
    }

    /// 创建老张（食材商贩）
    pub fn zhang_vendor() -> Self {
        Self {
            id: "zhang_vendor".to_string(),
            name: "老张".to_string(),
            avatar: "🧔".to_string(),
            age: 55,
            occupation: "食材批发商".to_string(),
            personality: "精明能干，为人实在，重信誉".to_string(),
            abilities: vec![NeighborAbility::IngredientWholesale],
            background:
                "老张经营食材批发生意二十多年，和祖父是老交情。他总能以批发价提供新鲜的食材。"
                    .to_string(),
            favorite_gifts: vec!["香烟".to_string(), "好酒".to_string()],
            disliked_items: vec!["讨价还价".to_string()],
            dialogue_style: "生意人风格，说话简洁".to_string(),
        }
    }

    /// 创建小刘（快递小哥）
    pub fn xiao_liu() -> Self {
        Self {
            id: "xiao_liu".to_string(),
            name: "小刘".to_string(),
            avatar: "👨".to_string(),
            age: 28,
            occupation: "快递员".to_string(),
            personality: "勤快热心，人缘好，消息灵通".to_string(),
            abilities: vec![NeighborAbility::DeliveryService],
            background: "小刘负责老街片区的快递配送，每天骑着电动车穿梭在街巷中。他消息灵通，知道街坊邻里的各种八卦。".to_string(),
            favorite_gifts: vec!["饮料".to_string(), "零食".to_string()],
            disliked_items: vec!["慢吞吞".to_string()],
            dialogue_style: "年轻人风格，说话快".to_string(),
        }
    }

    /// 创建老周（美食评论家）
    pub fn zhou_critic() -> Self {
        Self {
            id: "zhou_critic".to_string(),
            name: "老周".to_string(),
            avatar: "🧓".to_string(),
            age: 60,
            occupation: "自由撰稿人".to_string(),
            personality: "儒雅随和，见多识广，热爱美食".to_string(),
            abilities: vec![NeighborAbility::CookingAdvice, NeighborAbility::Storytelling],
            background: "老周是退休的记者，现在写美食评论。他尝遍各地美食，对烹饪有独到见解。祖父在世时，他们经常一起讨论美食。".to_string(),
            favorite_gifts: vec!["好书".to_string(), "好茶".to_string()],
            disliked_items: vec!["粗俗之物".to_string()],
            dialogue_style: "文人风格，喜欢引经据典".to_string(),
        }
    }
}

/// 邻居好感度关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborRelation {
    /// 邻居 ID
    pub neighbor_id: String,
    /// 好感度 (0-100)
    pub affinity: u32,
    /// 互动次数
    pub interaction_count: u32,
    /// 赠送礼物次数
    pub gift_count: u32,
    /// 请求帮助次数
    pub help_count: u32,
    /// 上次互动时间
    pub last_interaction: Option<DateTime<Utc>>,
    /// 解锁的故事
    pub unlocked_stories: Vec<String>,
    /// 解锁的记忆碎片
    pub unlocked_memories: Vec<String>,
}

impl NeighborRelation {
    /// 创建新的邻居关系
    pub fn new(neighbor_id: String) -> Self {
        Self {
            neighbor_id,
            affinity: 20, // 初始好感度
            interaction_count: 0,
            gift_count: 0,
            help_count: 0,
            last_interaction: None,
            unlocked_stories: Vec::new(),
            unlocked_memories: Vec::new(),
        }
    }

    /// 增加好感度
    pub fn increase_affinity(&mut self, amount: u32) -> u32 {
        self.affinity = (self.affinity + amount).min(100);
        self.affinity
    }

    /// 减少好感度
    pub fn decrease_affinity(&mut self, amount: u32) -> u32 {
        self.affinity = self.affinity.saturating_sub(amount);
        self.affinity
    }

    /// 获取好感度等级
    pub fn affinity_level(&self) -> AffinityLevel {
        match self.affinity {
            0..=19 => AffinityLevel::Stranger,
            20..=39 => AffinityLevel::Acquaintance,
            40..=59 => AffinityLevel::Friend,
            60..=79 => AffinityLevel::GoodFriend,
            80..=99 => AffinityLevel::CloseFriend,
            100 => AffinityLevel::Family,
            _ => AffinityLevel::Stranger,
        }
    }

    /// 检查是否可以解锁内容
    pub fn can_unlock_content(&self, required_affinity: u32) -> bool {
        self.affinity >= required_affinity
    }

    /// 记录互动
    pub fn record_interaction(&mut self) {
        self.interaction_count += 1;
        self.last_interaction = Some(Utc::now());
    }

    /// 记录礼物
    pub fn record_gift(&mut self) {
        self.gift_count += 1;
        self.last_interaction = Some(Utc::now());
    }

    /// 记录帮助请求
    pub fn record_help(&mut self) {
        self.help_count += 1;
        self.last_interaction = Some(Utc::now());
    }
}

/// 好感度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffinityLevel {
    /// 陌生人 (0-19)
    Stranger,
    /// 熟人 (20-39)
    Acquaintance,
    /// 朋友 (40-59)
    Friend,
    /// 好友 (60-79)
    GoodFriend,
    /// 挚友 (80-99)
    CloseFriend,
    /// 家人 (100)
    Family,
}

impl AffinityLevel {
    /// 获取等级名称
    pub fn name(&self) -> &str {
        match self {
            AffinityLevel::Stranger => "陌生人",
            AffinityLevel::Acquaintance => "熟人",
            AffinityLevel::Friend => "朋友",
            AffinityLevel::GoodFriend => "好友",
            AffinityLevel::CloseFriend => "挚友",
            AffinityLevel::Family => "家人",
        }
    }

    /// 获取等级图标
    pub fn icon(&self) -> &str {
        match self {
            AffinityLevel::Stranger => "🤝",
            AffinityLevel::Acquaintance => "😊",
            AffinityLevel::Friend => "😄",
            AffinityLevel::GoodFriend => "🥰",
            AffinityLevel::CloseFriend => "💝",
            AffinityLevel::Family => "❤️",
        }
    }
}

/// 邻居管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborManager {
    /// 所有邻居
    pub neighbors: Vec<Neighbor>,
    /// 关系列表
    pub relations: Vec<NeighborRelation>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl NeighborManager {
    /// 创建新的邻居管理器
    pub fn new() -> Self {
        let neighbors = Self::create_default_neighbors();
        let relations = neighbors
            .iter()
            .map(|n| NeighborRelation::new(n.id.clone()))
            .collect();

        Self {
            neighbors,
            relations,
            updated_at: Utc::now(),
        }
    }

    /// 创建默认邻居
    fn create_default_neighbors() -> Vec<Neighbor> {
        vec![
            Neighbor::grandma_wang(),
            Neighbor::uncle_li(),
            Neighbor::zhang_vendor(),
            Neighbor::xiao_liu(),
            Neighbor::zhou_critic(),
        ]
    }

    /// 获取邻居
    pub fn get_neighbor(&self, neighbor_id: &str) -> Option<&Neighbor> {
        self.neighbors.iter().find(|n| n.id == neighbor_id)
    }

    /// 获取关系
    pub fn get_relation(&self, neighbor_id: &str) -> Option<&NeighborRelation> {
        self.relations.iter().find(|r| r.neighbor_id == neighbor_id)
    }

    /// 获取关系（可变）
    pub fn get_relation_mut(&mut self, neighbor_id: &str) -> Option<&mut NeighborRelation> {
        self.relations
            .iter_mut()
            .find(|r| r.neighbor_id == neighbor_id)
    }

    /// 更新好感度
    pub fn update_affinity(
        &mut self,
        neighbor_id: &str,
        change: i32,
    ) -> std::result::Result<u32, String> {
        let relation = self
            .get_relation_mut(neighbor_id)
            .ok_or_else(|| "邻居不存在".to_string())?;

        if change >= 0 {
            Ok(relation.increase_affinity(change as u32))
        } else {
            Ok(relation.decrease_affinity((-change) as u32))
        }
    }

    /// 获取所有邻居及关系
    pub fn get_all_with_relations(&self) -> Vec<(&Neighbor, &NeighborRelation)> {
        self.neighbors
            .iter()
            .map(|n| {
                let relation = self.get_relation(&n.id).unwrap();
                (n, relation)
            })
            .collect()
    }

    /// 获取好友列表（好感度 >= 40）
    pub fn get_friends(&self) -> Vec<(&Neighbor, &NeighborRelation)> {
        self.get_all_with_relations()
            .into_iter()
            .filter(|(_, r)| r.affinity >= 40)
            .collect()
    }
}

impl Default for NeighborManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_creation() {
        let neighbor = Neighbor::grandma_wang();
        assert_eq!(neighbor.id, "grandma_wang");
        assert_eq!(neighbor.name, "王奶奶");
        assert!(neighbor.has_ability(NeighborAbility::Gardening));
    }

    #[test]
    fn test_neighbor_relation() {
        let mut relation = NeighborRelation::new("grandma_wang".to_string());
        assert_eq!(relation.affinity, 20);
        assert_eq!(relation.affinity_level(), AffinityLevel::Acquaintance);

        relation.increase_affinity(30);
        assert_eq!(relation.affinity, 50);
        assert_eq!(relation.affinity_level(), AffinityLevel::Friend);
    }

    #[test]
    fn test_affinity_level() {
        assert_eq!(AffinityLevel::Stranger.name(), "陌生人");
        assert_eq!(AffinityLevel::Family.name(), "家人");
    }

    #[test]
    fn test_neighbor_manager() {
        let manager = NeighborManager::new();
        assert_eq!(manager.neighbors.len(), 5);

        let neighbor = manager.get_neighbor("grandma_wang");
        assert!(neighbor.is_some());
    }

    #[test]
    fn test_update_affinity() {
        let mut manager = NeighborManager::new();

        let result = manager.update_affinity("grandma_wang", 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);

        let result = manager.update_affinity("nonexistent", 10);
        assert!(result.is_err());
    }
}
