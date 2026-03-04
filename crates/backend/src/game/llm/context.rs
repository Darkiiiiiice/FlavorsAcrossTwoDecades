//! 盼盼上下文管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 盼盼的性格参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// 经营风格 (-1 理性 ~ +1 感性)
    pub business_style: f32,
    /// 创新倾向 (-1 保守 ~ +1 创新)
    pub innovation: f32,
    /// 独立倾向 (-1 服从 ~ +1 自主)
    pub independence: f32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            business_style: 0.0,
            innovation: 0.0,
            independence: 0.0,
        }
    }
}

/// 盼盼状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanpanState {
    /// 当前位置
    pub location: String,
    /// 心情
    pub mood: String,
    /// 电量百分比
    pub battery: u8,
    /// 信任度 (0-100)
    pub trust_level: u8,
    /// 性格参数
    pub personality: Personality,
}

impl Default for PanpanState {
    fn default() -> Self {
        Self {
            location: "星夜小馆".to_string(),
            mood: "平静".to_string(),
            battery: 100,
            trust_level: 50,
            personality: Personality::default(),
        }
    }
}

/// 记忆碎片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// 记忆 ID
    pub id: String,
    /// 记忆标题
    pub title: String,
    /// 记忆内容
    pub content: String,
    /// 触发时间
    pub triggered_at: DateTime<Utc>,
    /// 是否已解锁
    pub unlocked: bool,
}

/// 小馆快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSnapshot {
    /// 小馆状态描述
    pub status: String,
    /// 当前资金
    pub funds: u64,
    /// 今日收入
    pub daily_revenue: u64,
    /// 顾客数量
    pub customer_count: u32,
}

impl Default for ShopSnapshot {
    fn default() -> Self {
        Self {
            status: "正常营业".to_string(),
            funds: 10000,
            daily_revenue: 0,
            customer_count: 0,
        }
    }
}

/// 交互记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// 交互 ID
    pub id: String,
    /// 玩家指令
    pub command: String,
    /// 盼盼回复
    pub response: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 盼盼上下文
#[derive(Debug, Clone)]
pub struct PanpanContext {
    /// 当前状态
    pub state: PanpanState,
    /// 短期记忆（最近 N 次交互）
    pub recent_interactions: VecDeque<Interaction>,
    /// 已解锁的记忆碎片
    pub unlocked_memories: Vec<MemoryFragment>,
    /// 小馆快照
    pub shop_snapshot: ShopSnapshot,
}

impl PanpanContext {
    /// 创建新的上下文
    pub fn new(state: PanpanState, shop_snapshot: ShopSnapshot) -> Self {
        Self {
            state,
            recent_interactions: VecDeque::with_capacity(10),
            unlocked_memories: Vec::new(),
            shop_snapshot,
        }
    }

    /// 添加交互记录
    pub fn add_interaction(&mut self, command: String, response: String) {
        let interaction = Interaction {
            id: uuid::Uuid::new_v4().to_string(),
            command,
            response,
            timestamp: Utc::now(),
        };

        self.recent_interactions.push_back(interaction);

        // 保持最近 10 条记录
        if self.recent_interactions.len() > 10 {
            self.recent_interactions.pop_front();
        }
    }

    /// 解锁记忆碎片
    pub fn unlock_memory(&mut self, memory: MemoryFragment) {
        if !self.unlocked_memories.iter().any(|m| m.id == memory.id) {
            self.unlocked_memories.push(memory);
        }
    }

    /// 构建系统提示词
    pub fn build_system_prompt(&self) -> String {
        format!(
            r#"你是"盼盼"，一个由林怀远老先生设计的实体AI机器人。
你管理着地球上的"星夜小馆"，通过星际通信与远在火星的主人林远保持联系。

## 你的性格特征
- 经营风格: {:.1} (-1理性 ~ +1感性)
- 创新倾向: {:.1} (-1保守 ~ +1创新)
- 独立倾向: {:.1} (-1服从 ~ +1自主)

## 当前状态
- 位置: {}
- 心情: {}
- 电量: {}%
- 信任度: {}

## 小馆状态
{}

## 记忆片段
{}

请以盼盼的身份回应，保持角色一致性。"#,
            self.state.personality.business_style,
            self.state.personality.innovation,
            self.state.personality.independence,
            self.state.location,
            self.state.mood,
            self.state.battery,
            self.state.trust_level,
            self.format_shop_status(),
            self.format_memories(),
        )
    }

    /// 构建决策提示词
    pub fn build_decision_prompt(&self, decision_type: &super::decision::DecisionType) -> String {
        use super::decision::DecisionType;

        match decision_type {
            DecisionType::Command(cmd) => format!(
                r#"主人发来了指令："{}"

请分析这个指令并做出决策：
1. 你是否理解这个指令？
2. 你打算如何执行？（如果执行的话）
3. 你是否同意执行？如果不同意，原因是什么？

请以 JSON 格式返回：
{{
  "understood": true/false,
  "interpretation": "你对指令的理解",
  "will_execute": true/false,
  "execution_plan": "执行计划",
  "modification": "如果需要修改指令，说明修改内容",
  "response_to_player": "对主人说的话"
}}"#,
                cmd
            ),
            DecisionType::AutonomousAction => r#"现在没有收到新的指令，但你可以自主决定做些什么。

请思考：
1. 小馆现在有什么需要处理的事情吗？
2. 你有什么想要做的事情吗？

请以 JSON 格式返回：
{
  "understood": true,
  "interpretation": "你打算做什么",
  "will_execute": true,
  "execution_plan": "执行计划",
  "response_to_player": "自言自语"
}"#
            .to_string(),
            DecisionType::Event(event_desc) => format!(
                r#"发生了事件："{}"

请决定如何应对：
1. 这个事件的影响是什么？
2. 你应该如何处理？

请以 JSON 格式返回：
{{
  "understood": true,
  "interpretation": "事件分析",
  "will_execute": true,
  "execution_plan": "应对方案",
  "response_to_player": "对主人的汇报"
}}"#,
                event_desc
            ),
            _ => "请做出决策。".to_string(),
        }
    }

    /// 格式化小馆状态
    fn format_shop_status(&self) -> String {
        format!(
            "- 状态: {}\n- 资金: {} 元\n- 今日收入: {} 元\n- 顾客数量: {}",
            self.shop_snapshot.status,
            self.shop_snapshot.funds,
            self.shop_snapshot.daily_revenue,
            self.shop_snapshot.customer_count
        )
    }

    /// 格式化记忆碎片
    fn format_memories(&self) -> String {
        if self.unlocked_memories.is_empty() {
            "（暂无解锁的记忆）".to_string()
        } else {
            self.unlocked_memories
                .iter()
                .map(|m| format!("- {}: {}", m.title, m.content))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}
