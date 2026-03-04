//! 邻居互动定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 互动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// 赠送礼物
    Gift,
    /// 请求帮助
    RequestHelp,
    /// 闲聊
    Chat,
    /// 交易
    Trade,
    /// 请教
    Consult,
}

impl InteractionType {
    /// 获取互动名称
    pub fn name(&self) -> &str {
        match self {
            InteractionType::Gift => "赠送礼物",
            InteractionType::RequestHelp => "请求帮助",
            InteractionType::Chat => "闲聊",
            InteractionType::Trade => "交易",
            InteractionType::Consult => "请教",
        }
    }

    /// 获取基础好感度变化
    pub fn base_affinity_change(&self) -> i32 {
        match self {
            InteractionType::Gift => 5,
            InteractionType::RequestHelp => -2, // 请求帮助会消耗人情
            InteractionType::Chat => 2,
            InteractionType::Trade => 1,
            InteractionType::Consult => 3,
        }
    }
}

/// 互动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    /// 是否成功
    pub success: bool,
    /// 结果描述
    pub description: String,
    /// 好感度变化
    pub affinity_change: i32,
    /// 获得的物品/效果
    pub rewards: Vec<String>,
    /// 解锁的内容
    pub unlocked_content: Vec<String>,
}

impl InteractionResult {
    /// 创建成功的互动结果
    pub fn success(description: String, affinity_change: i32) -> Self {
        Self {
            success: true,
            description,
            affinity_change,
            rewards: Vec::new(),
            unlocked_content: Vec::new(),
        }
    }

    /// 创建失败的互动结果
    pub fn failure(description: String) -> Self {
        Self {
            success: false,
            description,
            affinity_change: 0,
            rewards: Vec::new(),
            unlocked_content: Vec::new(),
        }
    }

    /// 添加奖励
    pub fn with_reward(mut self, reward: String) -> Self {
        self.rewards.push(reward);
        self
    }

    /// 添加解锁内容
    pub fn with_unlocked(mut self, content: String) -> Self {
        self.unlocked_content.push(content);
        self
    }
}

/// 互动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// 互动 ID
    pub id: Uuid,
    /// 邻居 ID
    pub neighbor_id: String,
    /// 互动类型
    pub interaction_type: InteractionType,
    /// 互动结果
    pub result: InteractionResult,
    /// 互动时间
    pub timestamp: DateTime<Utc>,
    /// 相关物品（礼物、交易品等）
    pub items: Vec<String>,
    /// 对话内容
    pub dialogue: Option<String>,
}

impl Interaction {
    /// 创建新互动
    pub fn new(neighbor_id: String, interaction_type: InteractionType) -> Self {
        Self {
            id: Uuid::new_v4(),
            neighbor_id,
            interaction_type,
            result: InteractionResult::success("互动成功".to_string(), interaction_type.base_affinity_change()),
            timestamp: Utc::now(),
            items: Vec::new(),
            dialogue: None,
        }
    }

    /// 设置结果
    pub fn with_result(mut self, result: InteractionResult) -> Self {
        self.result = result;
        self
    }

    /// 添加物品
    pub fn with_item(mut self, item: String) -> Self {
        self.items.push(item);
        self
    }

    /// 设置对话
    pub fn with_dialogue(mut self, dialogue: String) -> Self {
        self.dialogue = Some(dialogue);
        self
    }
}

/// 互动管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionManager {
    /// 互动历史
    pub history: Vec<Interaction>,
    /// 每日互动限制
    pub daily_limits: std::collections::HashMap<String, u32>,
    /// 当前已用次数
    pub daily_used: std::collections::HashMap<String, u32>,
    /// 上次重置日期
    pub last_reset: DateTime<Utc>,
}

impl InteractionManager {
    /// 创建新的互动管理器
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            daily_limits: Self::create_default_limits(),
            daily_used: std::collections::HashMap::new(),
            last_reset: Utc::now(),
        }
    }

    /// 创建默认限制
    fn create_default_limits() -> std::collections::HashMap<String, u32> {
        let mut limits = std::collections::HashMap::new();
        limits.insert("gift".to_string(), 3);       // 每天最多送3次礼物
        limits.insert("request_help".to_string(), 2); // 每天最多请求2次帮助
        limits.insert("chat".to_string(), 5);       // 每天最多闲聊5次
        limits.insert("trade".to_string(), 3);      // 每天最多交易3次
        limits.insert("consult".to_string(), 3);    // 每天最多请教3次
        limits
    }

    /// 检查是否可以互动
    pub fn can_interact(&self, interaction_type: InteractionType) -> bool {
        let key = match interaction_type {
            InteractionType::Gift => "gift",
            InteractionType::RequestHelp => "request_help",
            InteractionType::Chat => "chat",
            InteractionType::Trade => "trade",
            InteractionType::Consult => "consult",
        };

        let limit = self.daily_limits.get(key).copied().unwrap_or(999);
        let used = self.daily_used.get(key).copied().unwrap_or(0);

        used < limit
    }

    /// 记录互动
    pub fn record_interaction(&mut self, interaction: &Interaction) {
        let key = match interaction.interaction_type {
            InteractionType::Gift => "gift",
            InteractionType::RequestHelp => "request_help",
            InteractionType::Chat => "chat",
            InteractionType::Trade => "trade",
            InteractionType::Consult => "consult",
        };

        *self.daily_used.entry(key.to_string()).or_insert(0) += 1;
        self.history.push(interaction.clone());
    }

    /// 重置每日限制
    pub fn reset_daily(&mut self) {
        self.daily_used.clear();
        self.last_reset = Utc::now();
    }

    /// 获取与某邻居的互动历史
    pub fn get_history_with(&self, neighbor_id: &str) -> Vec<&Interaction> {
        self.history.iter()
            .filter(|i| i.neighbor_id == neighbor_id)
            .collect()
    }

    /// 获取最近的互动
    pub fn get_recent_interactions(&self, count: usize) -> Vec<&Interaction> {
        self.history.iter().rev().take(count).collect()
    }
}

impl Default for InteractionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 礼物效果计算
pub struct GiftEffect;

impl GiftEffect {
    /// 计算礼物好感度加成
    pub fn calculate_affinity(gift: &str, favorite_gifts: &[String], disliked_items: &[String]) -> i32 {
        if favorite_gifts.iter().any(|f| gift.contains(f)) {
            10 // 喜欢的礼物 +10
        } else if disliked_items.iter().any(|d| gift.contains(d)) {
            -5 // 讨厌的东西 -5
        } else {
            3 // 普通礼物 +3
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_type() {
        assert_eq!(InteractionType::Gift.name(), "赠送礼物");
        assert_eq!(InteractionType::Chat.base_affinity_change(), 2);
    }

    #[test]
    fn test_interaction_result() {
        let result = InteractionResult::success("互动成功".to_string(), 5);
        assert!(result.success);
        assert_eq!(result.affinity_change, 5);

        let result = InteractionResult::failure("好感度不足".to_string());
        assert!(!result.success);
    }

    #[test]
    fn test_interaction_creation() {
        let interaction = Interaction::new("grandma_wang".to_string(), InteractionType::Chat);
        assert_eq!(interaction.neighbor_id, "grandma_wang");
        assert_eq!(interaction.interaction_type, InteractionType::Chat);
    }

    #[test]
    fn test_interaction_manager() {
        let manager = InteractionManager::new();
        assert!(manager.can_interact(InteractionType::Gift));
        assert!(manager.can_interact(InteractionType::Chat));
    }

    #[test]
    fn test_interaction_limit() {
        let mut manager = InteractionManager::new();

        // 应该可以互动
        assert!(manager.can_interact(InteractionType::Gift));

        // 模拟多次互动
        for _ in 0..3 {
            let interaction = Interaction::new("test".to_string(), InteractionType::Gift);
            manager.record_interaction(&interaction);
        }

        // 达到限制后不能互动
        assert!(!manager.can_interact(InteractionType::Gift));
    }

    #[test]
    fn test_gift_effect() {
        let favorite = vec!["花种".to_string(), "茶叶".to_string()];
        let disliked = vec!["快餐".to_string()];

        // 喜欢的礼物
        assert_eq!(GiftEffect::calculate_affinity("玫瑰花种", &favorite, &disliked), 10);

        // 讨厌的礼物
        assert_eq!(GiftEffect::calculate_affinity("快餐外卖", &favorite, &disliked), -5);

        // 普通礼物
        assert_eq!(GiftEffect::calculate_affinity("水果", &favorite, &disliked), 3);
    }
}
