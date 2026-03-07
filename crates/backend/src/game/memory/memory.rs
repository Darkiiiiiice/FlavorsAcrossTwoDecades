//! 记忆内容定义

use serde::{Deserialize, Serialize};

/// 感官类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sense {
    /// 视觉
    Visual,
    /// 听觉
    Auditory,
    /// 嗅觉
    Olfactory,
    /// 味觉
    Gustatory,
    /// 触觉
    Tactile,
}

impl Sense {
    /// 获取感官名称
    pub fn name(&self) -> &str {
        match self {
            Sense::Visual => "视觉",
            Sense::Auditory => "听觉",
            Sense::Olfactory => "嗅觉",
            Sense::Gustatory => "味觉",
            Sense::Tactile => "触觉",
        }
    }

    /// 获取图标
    pub fn icon(&self) -> &str {
        match self {
            Sense::Visual => "👁️",
            Sense::Auditory => "👂",
            Sense::Olfactory => "👃",
            Sense::Gustatory => "👅",
            Sense::Tactile => "✋",
        }
    }
}

/// 感官记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryMemory {
    /// 感官类型
    pub sense: Sense,
    /// 描述
    pub description: String,
}

impl SensoryMemory {
    /// 创建新的感官记忆
    pub fn new(sense: Sense, description: String) -> Self {
        Self { sense, description }
    }

    /// 视觉记忆
    pub fn visual(description: String) -> Self {
        Self::new(Sense::Visual, description)
    }

    /// 听觉记忆
    pub fn auditory(description: String) -> Self {
        Self::new(Sense::Auditory, description)
    }

    /// 嗅觉记忆
    pub fn olfactory(description: String) -> Self {
        Self::new(Sense::Olfactory, description)
    }

    /// 味觉记忆
    pub fn gustatory(description: String) -> Self {
        Self::new(Sense::Gustatory, description)
    }

    /// 触觉记忆
    pub fn tactile(description: String) -> Self {
        Self::new(Sense::Tactile, description)
    }
}

/// 记忆内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContent {
    /// 叙事文本
    pub narrative: String,
    /// 场景描述
    pub scene_description: Option<String>,
    /// 祖父语录
    pub grandfather_quote: Option<String>,
    /// 感官记忆
    pub sensory_memories: Vec<SensoryMemory>,
    /// Panda 的反应
    pub panda_reaction: String,
    /// 解锁的知识
    pub unlocked_knowledge: Option<String>,
    /// 情感标签
    pub emotion_tags: Vec<String>,
    /// 时间线索
    pub time_hint: Option<String>,
    /// 地点线索
    pub place_hint: Option<String>,
}

impl MemoryContent {
    /// 创建新的记忆内容
    pub fn new(narrative: String, panda_reaction: String) -> Self {
        Self {
            narrative,
            scene_description: None,
            grandfather_quote: None,
            sensory_memories: Vec::new(),
            panda_reaction,
            unlocked_knowledge: None,
            emotion_tags: Vec::new(),
            time_hint: None,
            place_hint: None,
        }
    }

    /// 添加场景描述
    pub fn with_scene(mut self, description: String) -> Self {
        self.scene_description = Some(description);
        self
    }

    /// 添加祖父语录
    pub fn with_grandfather_quote(mut self, quote: String) -> Self {
        self.grandfather_quote = Some(quote);
        self
    }

    /// 添加感官记忆
    pub fn add_sensory_memory(&mut self, memory: SensoryMemory) {
        self.sensory_memories.push(memory);
    }

    /// 添加感官记忆（便捷方法）
    pub fn with_sensory(mut self, sense: Sense, description: String) -> Self {
        self.sensory_memories
            .push(SensoryMemory::new(sense, description));
        self
    }

    /// 设置解锁知识
    pub fn with_knowledge(mut self, knowledge: String) -> Self {
        self.unlocked_knowledge = Some(knowledge);
        self
    }

    /// 添加情感标签
    pub fn add_emotion_tag(&mut self, tag: String) {
        if !self.emotion_tags.contains(&tag) {
            self.emotion_tags.push(tag);
        }
    }

    /// 添加情感标签（便捷方法）
    pub fn with_emotion(mut self, tag: String) -> Self {
        self.add_emotion_tag(tag);
        self
    }

    /// 设置时间线索
    pub fn with_time_hint(mut self, hint: String) -> Self {
        self.time_hint = Some(hint);
        self
    }

    /// 设置地点线索
    pub fn with_place_hint(mut self, hint: String) -> Self {
        self.place_hint = Some(hint);
        self
    }

    /// 获取完整描述
    pub fn full_description(&self) -> String {
        let mut result = String::new();

        // 添加场景描述
        if let Some(ref scene) = self.scene_description {
            result.push_str("【场景】\n");
            result.push_str(scene);
            result.push_str("\n\n");
        }

        // 添加叙事
        result.push_str(self.narrative.as_str());
        result.push_str("\n\n");

        // 添加感官记忆
        if !self.sensory_memories.is_empty() {
            result.push_str("【记忆中的感觉】\n");
            for memory in &self.sensory_memories {
                result.push_str(&format!(
                    "{}: {}\n",
                    memory.sense.icon(),
                    memory.description
                ));
            }
            result.push('\n');
        }

        // 添加祖父语录
        if let Some(ref quote) = self.grandfather_quote {
            result.push_str("【祖父说】\n\"");
            result.push_str(quote);
            result.push_str("\"\n\n");
        }

        // 添加 Panda 反应
        result.push_str("【Panda 的感想】\n");
        result.push_str(&self.panda_reaction);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensory_memory_creation() {
        let visual = SensoryMemory::visual("阳光透过窗户洒在餐桌上".to_string());
        assert_eq!(visual.sense, Sense::Visual);
        assert_eq!(visual.description, "阳光透过窗户洒在餐桌上");

        let olfactory = SensoryMemory::olfactory("刚出锅的红烧肉香气".to_string());
        assert_eq!(olfactory.sense, Sense::Olfactory);
    }

    #[test]
    fn test_memory_content_creation() {
        let content = MemoryContent::new(
            "祖父教我做的第一道菜是番茄炒蛋。".to_string(),
            "这道菜承载着太多的回忆。".to_string(),
        );

        assert!(content.scene_description.is_none());
        assert!(content.sensory_memories.is_empty());
    }

    #[test]
    fn test_memory_content_builder() {
        let content = MemoryContent::new(
            "那是一个阳光明媚的下午。".to_string(),
            "这个场景让我想起了过去。".to_string(),
        )
        .with_scene("厨房里弥漫着温暖的气息。".to_string())
        .with_grandfather_quote("做菜最重要的是用心。".to_string())
        .with_sensory(Sense::Olfactory, "油烟和葱姜蒜的香气".to_string())
        .with_emotion("温暖".to_string())
        .with_time_hint("二十年前".to_string());

        assert!(content.scene_description.is_some());
        assert!(content.grandfather_quote.is_some());
        assert_eq!(content.sensory_memories.len(), 1);
        assert!(content.emotion_tags.contains(&"温暖".to_string()));
    }

    #[test]
    fn test_full_description() {
        let content = MemoryContent::new(
            "祖父教我做的第一道菜。".to_string(),
            "这是我最珍贵的回忆之一。".to_string(),
        )
        .with_scene("老旧的厨房里，阳光透过窗户洒进来。".to_string())
        .with_grandfather_quote("记住，火候是关键。".to_string())
        .with_sensory(Sense::Visual, "金黄色的蛋液在锅中翻滚".to_string());

        let full = content.full_description();
        assert!(full.contains("【场景】"));
        assert!(full.contains("【祖父说】"));
        assert!(full.contains("【Panda 的感想】"));
    }
}
