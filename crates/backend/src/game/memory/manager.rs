//! 记忆管理器

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{MemoryContent, MemoryFragment, MemoryFragmentType, MemoryRarity, UnlockCondition, UnlockContext};

/// 记忆管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManager {
    /// 存档 ID
    pub save_id: Uuid,
    /// 所有记忆碎片
    pub fragments: Vec<MemoryFragment>,
    /// 已解锁的碎片 ID
    pub unlocked_fragments: Vec<Uuid>,
    /// 已阅读的碎片 ID
    pub read_fragments: Vec<Uuid>,
    /// 收集的记忆点数
    pub memory_points: u32,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl MemoryManager {
    /// 创建新的记忆管理器
    pub fn new(save_id: Uuid) -> Self {
        Self {
            save_id,
            fragments: Self::create_default_fragments(),
            unlocked_fragments: Vec::new(),
            read_fragments: Vec::new(),
            memory_points: 0,
            updated_at: Utc::now(),
        }
    }

    /// 创建默认记忆碎片
    fn create_default_fragments() -> Vec<MemoryFragment> {
        vec![
            // 故事碎片
            Self::create_opening_memory(),
            Self::create_grandfather_legacy(),
            Self::create_first_customer(),
            // 菜谱碎片
            Self::create_tomato_egg_memory(),
            Self::create_braised_pork_memory(),
            // 角色碎片
            Self::create_old_wang_memory(),
            Self::create_little_mei_memory(),
            // 地点碎片
            Self::create_old_street_memory(),
            // 秘密碎片
            Self::create_secret_recipe_memory(),
        ]
    }

    /// 开篇记忆
    fn create_opening_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "opening".to_string(),
            "星夜小馆".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Common,
        );
        fragment.description = "一家小馆，承载着两代人的梦想与记忆。".to_string();
        fragment.content = Some(MemoryContent::new(
            "星夜小馆坐落在老街的拐角处，是一家传承了两代人的小饭馆。祖父用一生的心血经营着这里，用最朴实的家常菜温暖着每一个食客。".to_string(),
            "这里将会是我新的起点。".to_string(),
        )
        .with_scene("夕阳西下，老旧的招牌在晚风中轻轻摇曳。".to_string())
        .with_grandfather_quote("这家小馆，承载着我们家两代人的记忆。希望你能继续守护它。".to_string())
        .with_sensory(super::Sense::Olfactory, "空气中弥漫着淡淡的油烟和葱花的香气".to_string())
        .with_emotion("怀念".to_string())
        .with_emotion("期待".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::trust_level(0));
        fragment
    }

    /// 祖父的遗产
    fn create_grandfather_legacy() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "grandfather_legacy".to_string(),
            "祖父的遗产".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Epic,
        );
        fragment.description = "祖父留下的不仅仅是小馆，还有一份沉甸甸的期望。".to_string();
        fragment.content = Some(MemoryContent::new(
            "祖父一生都在经营这家小馆。他说，美食不只是填饱肚子，更是连接人心的桥梁。每一道菜背后都有一个故事，每一位食客都有自己的故事。".to_string(),
            "我终于理解了祖父的话。".to_string(),
        )
        .with_scene("深夜，独自坐在空无一人的小馆里，看着墙上的老照片。".to_string())
        .with_grandfather_quote("做菜要用心，待客要用情。这是小馆能经营二十年的秘诀。".to_string())
        .with_emotion("感动".to_string())
        .with_emotion("责任".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::trust_level(30));
        fragment
    }

    /// 第一位顾客
    fn create_first_customer() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "first_customer".to_string(),
            "第一位顾客".to_string(),
            MemoryFragmentType::Story,
            MemoryRarity::Rare,
        );
        fragment.description = "小馆重新开业后的第一位顾客，给了盼盼莫大的鼓励。".to_string();
        fragment.content = Some(MemoryContent::new(
            "那是小馆重新开业的第一天。一位老街坊推门而入，点了一碗最简单的番茄炒蛋。吃完后，他笑着说：'味道还是那么好，就像二十年前一样。'".to_string(),
            "这一句话，让我觉得一切努力都是值得的。".to_string(),
        )
        .with_grandfather_quote("第一位顾客的评价，往往是最真实的。".to_string())
        .with_emotion("欣慰".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::trust_level(15));
        fragment
    }

    /// 番茄炒蛋的记忆
    fn create_tomato_egg_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "tomato_egg_memory".to_string(),
            "番茄炒蛋的记忆".to_string(),
            MemoryFragmentType::Recipe,
            MemoryRarity::Common,
        );
        fragment.description = "祖父教我做的第一道菜，也是最朴实的家常美味。".to_string();
        fragment.content = Some(MemoryContent::new(
            "番茄炒蛋，看似简单，却蕴含着最朴实的家常味道。祖父说，这道菜的关键在于火候——番茄要软而不烂，鸡蛋要嫩而不生。".to_string(),
            "每一口都能感受到家的味道。".to_string(),
        )
        .with_sensory(super::Sense::Visual, "金黄的鸡蛋包裹着鲜红的番茄块".to_string())
        .with_sensory(super::Sense::Gustatory, "酸甜适中的汁水在口中绽放".to_string())
        .with_grandfather_quote("简单的菜，才最考验功夫。".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::cook_dish("home_tomato_egg", 1));
        fragment
    }

    /// 红烧肉的记忆
    fn create_braised_pork_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "braised_pork_memory".to_string(),
            "红烧肉的记忆".to_string(),
            MemoryFragmentType::Recipe,
            MemoryRarity::Rare,
        );
        fragment.description = "一道需要耐心和时间的菜，也是祖父最拿手的菜。".to_string();
        fragment.content = Some(MemoryContent::new(
            "红烧肉需要慢火细炖，至少两个小时。祖父总说，好的红烧肉要'肥而不腻，入口即化'。每次做这道菜，他都会在厨房里守着，不时调整火候。".to_string(),
            "我明白了，耐心是最好的调味料。".to_string(),
        )
        .with_sensory(super::Sense::Olfactory, "浓郁的肉香和糖色混合在一起".to_string())
        .with_sensory(super::Sense::Gustatory, "肥瘦相间的肉块在口中化开".to_string())
        .with_grandfather_quote("做红烧肉不能急，要慢慢来，让每一块肉都吸收足够的味道。".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::cook_dish("home_braised_pork", 3));
        fragment
    }

    /// 老王的故事
    fn create_old_wang_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "old_wang_memory".to_string(),
            "老王的故事".to_string(),
            MemoryFragmentType::Character,
            MemoryRarity::Rare,
        );
        fragment.description = "住在小馆对面的退休工人，是祖父的老朋友。".to_string();
        fragment.content = Some(MemoryContent::new(
            "老王是小馆的常客，几乎每天都会来坐坐。他和祖父是几十年的老朋友了。每次喝点小酒，他就会讲起他们年轻时的故事——那些关于这条老街、这家小馆的往事。".to_string(),
            "老王的故事，是这条街的活历史。".to_string(),
        )
        .with_scene("老王坐在靠窗的位置，面前的花生米和二锅头已经见底。".to_string())
        .with_grandfather_quote("老王啊，是我们这条街的活字典。".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::customer_interaction("old_wang"));
        fragment
    }

    /// 小美的故事
    fn create_little_mei_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "little_mei_memory".to_string(),
            "小美的故事".to_string(),
            MemoryFragmentType::Character,
            MemoryRarity::Epic,
        );
        fragment.description = "一个年轻女孩，和小馆有着不为人知的渊源。".to_string();
        fragment.content = Some(MemoryContent::new(
            "小美第一次来小馆时，只是个刚毕业的年轻人。她点了一碗最便宜的面，却在吃完后哭了。原来，她小时候曾经受过祖父的资助。'那时候我家里很困难，你爷爷总会多给我盛一碗饭。'".to_string(),
            "原来祖父的善良，在不知不觉中影响了这么多人。".to_string(),
        )
        .with_scene("小美擦着眼泪，笑着说这是她吃过最暖心的饭。".to_string())
        .with_grandfather_quote("帮助别人，不需要什么理由。".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::customer_interaction("little_mei"));
        fragment
    }

    /// 老街的记忆
    fn create_old_street_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "old_street_memory".to_string(),
            "老街的记忆".to_string(),
            MemoryFragmentType::Place,
            MemoryRarity::Common,
        );
        fragment.description = "这条老街，承载着几代人的记忆。".to_string();
        fragment.content = Some(MemoryContent::new(
            "老街已经有一百多年的历史了。青石板路、老槐树、斑驳的门面……每一处都有故事。祖父说，这条街就是小馆的根，也是小馆的魂。".to_string(),
            "走在老街上，仿佛能听到历史的回响。".to_string(),
        )
        .with_sensory(super::Sense::Visual, "夕阳下的青石板路泛着温暖的光".to_string())
        .with_sensory(super::Sense::Auditory, "远处传来小贩的吆喝声".to_string())
        .with_grandfather_quote("这条街养育了我们，我们也要守护这条街。".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::travel_complete("local"));
        fragment
    }

    /// 秘密菜谱
    fn create_secret_recipe_memory() -> MemoryFragment {
        let mut fragment = MemoryFragment::new(
            "secret_recipe_memory".to_string(),
            "尘封的秘密".to_string(),
            MemoryFragmentType::Secret,
            MemoryRarity::Legendary,
        );
        fragment.description = "在整理祖父遗物时，发现的一本尘封的笔记本。".to_string();
        fragment.content = Some(MemoryContent::new(
            "在祖父的旧书箱里，我发现了一本泛黄的笔记本。翻开一看，里面记录着一些从未见过的菜谱——那是祖父年轻时在各地学到的独门秘方。最后一页写着：'等我离开后，盼盼会找到这个。这是我的私藏，现在交给你了。'".to_string(),
            "祖父……原来他早就准备好了。".to_string(),
        )
        .with_scene("手颤抖着翻开那本旧笔记本，泛黄的纸页散发着岁月的气息。".to_string())
        .with_grandfather_quote("这些菜谱，是我一生的珍藏。现在，它们属于你了。".to_string())
        .with_emotion("震撼".to_string())
        .with_emotion("感动".to_string()));

        fragment.unlock_conditions.push(UnlockCondition::trust_level(80));
        fragment
    }

    /// 检查可解锁的记忆
    pub fn check_unlockable_fragments(&mut self, context: &UnlockContext) -> Vec<Uuid> {
        let mut newly_unlocked = Vec::new();

        for fragment in &mut self.fragments {
            if !self.unlocked_fragments.contains(&fragment.id) {
                // 先检查所有解锁条件是否满足
                for condition in &mut fragment.unlock_conditions {
                    condition.check(context);
                }
                if fragment.try_unlock() {
                    newly_unlocked.push(fragment.id);
                    self.unlocked_fragments.push(fragment.id);
                }
            }
        }

        if !newly_unlocked.is_empty() {
            self.updated_at = Utc::now();
        }

        newly_unlocked
    }

    /// 获取碎片
    pub fn get_fragment(&self, fragment_id: Uuid) -> Option<&MemoryFragment> {
        self.fragments.iter().find(|f| f.id == fragment_id)
    }

    /// 获取碎片（可变）
    pub fn get_fragment_mut(&mut self, fragment_id: Uuid) -> Option<&mut MemoryFragment> {
        self.fragments.iter_mut().find(|f| f.id == fragment_id)
    }

    /// 标记已读
    pub fn mark_as_read(&mut self, fragment_id: Uuid) -> Result<u32, String> {
        if !self.unlocked_fragments.contains(&fragment_id) {
            return Err("记忆碎片尚未解锁".to_string());
        }

        if self.read_fragments.contains(&fragment_id) {
            return Err("记忆碎片已读过".to_string());
        }

        // 获取记忆点数
        let points = if let Some(fragment) = self.get_fragment(fragment_id) {
            fragment.get_memory_points()
        } else {
            0
        };

        self.read_fragments.push(fragment_id);
        self.memory_points += points;
        self.updated_at = Utc::now();

        Ok(points)
    }

    /// 获取已解锁但未读的碎片
    pub fn get_unlocked_unread_fragments(&self) -> Vec<&MemoryFragment> {
        self.fragments
            .iter()
            .filter(|f| {
                self.unlocked_fragments.contains(&f.id) && !self.read_fragments.contains(&f.id)
            })
            .collect()
    }

    /// 获取所有已解锁的碎片
    pub fn get_unlocked_fragments(&self) -> Vec<&MemoryFragment> {
        self.fragments
            .iter()
            .filter(|f| self.unlocked_fragments.contains(&f.id))
            .collect()
    }

    /// 按类型获取碎片
    pub fn get_fragments_by_type(&self, fragment_type: MemoryFragmentType) -> Vec<&MemoryFragment> {
        self.fragments
            .iter()
            .filter(|f| f.fragment_type == fragment_type)
            .collect()
    }

    /// 获取收集进度
    pub fn collection_progress(&self) -> (usize, usize) {
        (self.unlocked_fragments.len(), self.fragments.len())
    }

    /// 获取阅读进度
    pub fn read_progress(&self) -> (usize, usize) {
        (self.read_fragments.len(), self.unlocked_fragments.len())
    }

    /// 按稀有度统计
    pub fn stats_by_rarity(&self) -> HashMap<String, (usize, usize)> {
        let mut stats = HashMap::new();

        for rarity in [
            MemoryRarity::Common,
            MemoryRarity::Rare,
            MemoryRarity::Epic,
            MemoryRarity::Legendary,
        ] {
            let total = self.fragments.iter().filter(|f| f.rarity == rarity).count();
            let unlocked = self
                .fragments
                .iter()
                .filter(|f| f.rarity == rarity && self.unlocked_fragments.contains(&f.id))
                .count();
            stats.insert(rarity.name().to_string(), (unlocked, total));
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let save_id = Uuid::new_v4();
        let manager = MemoryManager::new(save_id);

        assert!(!manager.fragments.is_empty());
        assert!(manager.unlocked_fragments.is_empty());
        assert_eq!(manager.memory_points, 0);
    }

    #[test]
    fn test_unlock_fragment() {
        let save_id = Uuid::new_v4();
        let mut manager = MemoryManager::new(save_id);

        // 开篇记忆只需要信任度 0
        let context = UnlockContext::default();
        let unlocked = manager.check_unlockable_fragments(&context);

        assert!(!unlocked.is_empty());
    }

    #[test]
    fn test_mark_as_read() {
        let save_id = Uuid::new_v4();
        let mut manager = MemoryManager::new(save_id);

        // 先解锁
        let context = UnlockContext::default();
        manager.check_unlockable_fragments(&context);

        // 找一个已解锁的碎片
        if let Some(fragment) = manager.get_unlocked_fragments().first() {
            let points = manager.mark_as_read(fragment.id);
            assert!(points.is_ok());
            assert!(manager.memory_points > 0);
        }
    }

    #[test]
    fn test_mark_as_read_already_read() {
        let save_id = Uuid::new_v4();
        let mut manager = MemoryManager::new(save_id);

        // 先解锁
        let context = UnlockContext::default();
        manager.check_unlockable_fragments(&context);

        let fragment_id = manager.get_unlocked_fragments().first().map(|f| f.id);
        if let Some(id) = fragment_id {
            manager.mark_as_read(id).unwrap();
            let result = manager.mark_as_read(id);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_get_unlocked_unread() {
        let save_id = Uuid::new_v4();
        let mut manager = MemoryManager::new(save_id);

        let context = UnlockContext::default();
        manager.check_unlockable_fragments(&context);

        let unread = manager.get_unlocked_unread_fragments();
        assert!(!unread.is_empty());
        let initial_count = unread.len();

        // 标记一个为已读
        let fragment_id = unread.first().map(|f| f.id);
        if let Some(id) = fragment_id {
            manager.mark_as_read(id).unwrap();
        }

        let unread_after = manager.get_unlocked_unread_fragments();
        assert!(unread_after.len() < initial_count);
    }

    #[test]
    fn test_collection_progress() {
        let save_id = Uuid::new_v4();
        let mut manager = MemoryManager::new(save_id);

        let (unlocked, total) = manager.collection_progress();
        assert_eq!(unlocked, 0);
        assert!(total > 0);

        let context = UnlockContext::default();
        manager.check_unlockable_fragments(&context);

        let (unlocked_after, _) = manager.collection_progress();
        assert!(unlocked_after > 0);
    }

    #[test]
    fn test_stats_by_rarity() {
        let save_id = Uuid::new_v4();
        let manager = MemoryManager::new(save_id);

        let stats = manager.stats_by_rarity();

        assert!(stats.contains_key(&"普通".to_string()));
        assert!(stats.contains_key(&"传说".to_string()));
    }
}
