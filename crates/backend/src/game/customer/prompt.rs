//! 顾客生成提示词模块
//!
//! 统一管理顾客生成相关的 LLM 提示词

use super::customer::CustomerType;
use super::preference::{DietaryRestriction, FlavorPreference};

/// 顾客生成提示词构建器
pub struct CustomerPromptBuilder {
    /// 顾客类型
    customer_type: Option<CustomerType>,
    /// 餐厅风格
    restaurant_style: Option<String>,
    /// 当前季节
    season: Option<String>,
    /// 已有顾客背景故事（用于避免重复）
    existing_stories: Vec<String>,
}

impl CustomerPromptBuilder {
    /// 创建新的提示词构建器
    pub fn new() -> Self {
        Self {
            customer_type: None,
            restaurant_style: None,
            season: None,
            existing_stories: Vec::new(),
        }
    }

    /// 设置顾客类型
    pub fn with_customer_type(mut self, customer_type: CustomerType) -> Self {
        self.customer_type = Some(customer_type);
        self
    }

    /// 设置餐厅风格
    pub fn with_restaurant_style(mut self, style: String) -> Self {
        self.restaurant_style = Some(style);
        self
    }

    /// 设置当前季节
    pub fn with_season(mut self, season: String) -> Self {
        self.season = Some(season);
        self
    }

    /// 设置已有顾客故事
    pub fn with_existing_stories(mut self, stories: Vec<String>) -> Self {
        self.existing_stories = stories;
        self
    }

    /// 构建系统提示词
    pub fn build_system_prompt(&self) -> &'static str {
        r#"你是一个餐厅模拟游戏中的顾客生成系统。请根据要求生成一位具有独特个性的顾客。

输出格式要求（必须严格遵守 JSON 格式）：
{
    "name": "<顾客姓名，2-4个中文字符>",
    "age": <年龄，18-70之间的整数>,
    "occupation": "<职业，1-10个中文字符>",
    "customer_type": <顾客类型: 0=普通, 1=美食家, 2=评论家>,
    "story_background": "<故事背景，100-200字，描述顾客的身份、经历或来餐厅的原因>",
    "preference": {
        "flavor": <口味偏好: 0=清淡, 1=适中, 2=重口味, 3=麻辣, 4=酸甜>,
        "dietary": <饮食限制: 0=无, 1=素食, 2=清真, 3=无麸质, 4=低糖>,
        "price_sensitivity": <价格敏感度: 0-100的整数，越高越在意价格>,
        "patience": <耐心值: 0-100的整数，越高越有耐心>,
        "favorite_categories": ["<喜欢的菜系1>", "<喜欢的菜系2>"]
    }
}

规则：
1. 顾客姓名应该是真实的中文名字，避免使用"顾客#123"这样的占位符
2. 年龄应该与职业相匹配（例如：大学生约20岁，退休老人约65岁）
3. 职业应该多样化，可以是：程序员、教师、医生、学生、退休老人、自由职业者等
4. 故事背景应该有趣且富有情感，让顾客显得生动真实
5. 故事背景可以包含：职业、家庭、心情、特殊事件、对美食的热爱等元素
6. 偏好设置应该与故事背景和职业相符（例如：健身教练可能偏好低糖饮食）
7. 美食家(customer_type=1)通常有更高的价格承受能力和耐心
8. 评论家(customer_type=2)通常更挑剔，耐心值可能较低
9. 只输出 JSON，不要有任何其他文字或解释
10. 不要使用 markdown 格式"#
    }

    /// 构建用户消息
    pub fn build_user_message(&self) -> String {
        let type_desc = self.customer_type_desc();
        let style_desc = self.restaurant_style.as_deref().unwrap_or("传统中餐厅");
        let season_desc = self.season.as_deref().unwrap_or("春季");
        let stories_desc = self.format_existing_stories();

        format!(
            r#"请生成一位顾客：

顾客类型要求：{}
餐厅风格：{}
当前季节：{}

已有顾客的故事背景（请参考下面顾客的背景故事，尽量避免重复类似的设定）：
{}

请生成一位独特有趣的顾客。"#,
            type_desc, style_desc, season_desc, stories_desc
        )
    }

    /// 获取顾客类型描述
    fn customer_type_desc(&self) -> String {
        match self.customer_type {
            Some(CustomerType::Normal) => "普通顾客（可以随机生成任何类型）".to_string(),
            Some(CustomerType::Foodie) => "美食家（对美食有热情，愿意尝试新菜品）".to_string(),
            Some(CustomerType::Critic) => "评论家（挑剔，对餐厅评价有影响力）".to_string(),
            None => "随机类型（可以是普通顾客、美食家或评论家）".to_string(),
        }
    }

    /// 格式化已有故事
    fn format_existing_stories(&self) -> String {
        if self.existing_stories.is_empty() {
            "暂无已有顾客".to_string()
        } else {
            self.existing_stories
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, s)| {
                    let truncated: String = s.chars().take(50).collect();
                    format!("{}. {}...", i + 1, truncated)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

impl Default for CustomerPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// LLM 响应解析结果
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CustomerLlmResponse {
    /// 顾客姓名
    pub name: String,
    /// 年龄
    pub age: u32,
    /// 职业
    pub occupation: String,
    /// 顾客类型
    pub customer_type: i32,
    /// 故事背景
    pub story_background: String,
    /// 偏好
    pub preference: PreferenceLlmResponse,
}

/// 偏好 LLM 响应
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PreferenceLlmResponse {
    /// 口味偏好
    pub flavor: i32,
    /// 饮食限制
    pub dietary: i32,
    /// 价格敏感度
    pub price_sensitivity: u32,
    /// 耐心值
    pub patience: u32,
    /// 喜欢的菜品类型
    pub favorite_categories: Vec<String>,
}

impl CustomerLlmResponse {
    /// 尝试解析 LLM 返回的 JSON
    pub fn parse(response: &str) -> Option<Self> {
        // 尝试提取 JSON 部分
        let json_str = Self::extract_json(response)?;
        serde_json::from_str(&json_str).ok()
    }

    /// 从响应中提取 JSON
    fn extract_json(response: &str) -> Option<String> {
        let trimmed = response.trim();

        // 尝试直接解析
        if trimmed.starts_with('{') {
            return Some(trimmed.to_string());
        }

        // 尝试提取 ```json ... ``` 块
        if let Some(start) = trimmed.find("```json") {
            let rest = &trimmed[start + 7..];
            if let Some(end) = rest.find("```") {
                return Some(rest[..end].trim().to_string());
            }
        }

        // 尝试提取 ``` ... ``` 块
        if let Some(start) = trimmed.find("```") {
            let rest = &trimmed[start + 3..];
            if let Some(end) = rest.find("```") {
                let content = &rest[..end];
                // 跳过可能的语言标识符
                let content = content.trim_start_matches("json").trim();
                return Some(content.to_string());
            }
        }

        // 尝试找到第一个 { 和最后一个 }
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if end > start {
                    return Some(trimmed[start..=end].to_string());
                }
            }
        }

        None
    }

    /// 转换为 CustomerType
    pub fn to_customer_type(&self) -> CustomerType {
        CustomerType::try_from(self.customer_type).unwrap_or(CustomerType::Normal)
    }

    /// 转换为 FlavorPreference
    pub fn to_flavor(&self) -> FlavorPreference {
        let flavor = self.preference.flavor;
        match flavor {
            0 => FlavorPreference::Light,
            1 => FlavorPreference::Medium,
            2 => FlavorPreference::Heavy,
            3 => FlavorPreference::Spicy,
            4 => FlavorPreference::SweetSour,
            _ => FlavorPreference::Medium,
        }
    }

    /// 转换为 DietaryRestriction
    pub fn to_dietary(&self) -> DietaryRestriction {
        let dietary = self.preference.dietary;
        match dietary {
            0 => DietaryRestriction::None,
            1 => DietaryRestriction::Vegetarian,
            2 => DietaryRestriction::Halal,
            3 => DietaryRestriction::GlutenFree,
            4 => DietaryRestriction::LowSugar,
            _ => DietaryRestriction::None,
        }
    }
}
