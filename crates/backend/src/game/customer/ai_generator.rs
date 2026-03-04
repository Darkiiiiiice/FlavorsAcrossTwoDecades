//! AI 顾客生成器
//!
//! 使用 LLM 生成具有丰富背景故事的顾客

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::LlmConfig;
use crate::error::{GameError, Result};
use crate::game::customer::{
    Customer, CustomerType, DietaryRestriction, FlavorPreference, Preference,
};
use crate::game::llm::{create_llm_manager, LlmManager};

/// AI 生成的顾客数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGeneratedCustomer {
    /// 姓名
    pub name: String,
    /// 年龄
    pub age: u32,
    /// 职业
    pub occupation: String,
    /// 顾客类型
    pub customer_type: String,
    /// 口味偏好
    pub flavor_preference: String,
    /// 饮食限制
    pub dietary_restriction: String,
    /// 价格敏感度 (0-100)
    pub price_sensitivity: u32,
    /// 耐心值 (50-100)
    pub patience: u32,
    /// 喜欢的菜品类型
    pub favorite_categories: Vec<String>,
    /// 背景故事
    pub background_story: String,
    /// 来访原因
    pub visit_reason: String,
    /// 性格特点
    pub personality_traits: Vec<String>,
}

/// AI 顾客生成器
#[derive(Debug)]
pub struct AICustomerGenerator {
    llm_manager: Arc<LlmManager>,
}

impl AICustomerGenerator {
    /// 创建新的 AI 顾客生成器
    pub fn new(config: LlmConfig) -> Result<Self> {
        let llm_manager = create_llm_manager(config)?;
        Ok(Self { llm_manager })
    }

    /// 生成顾客
    pub async fn generate_customer(&self, id_num: u32) -> Result<Customer> {
        // 构建 prompt
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt();

        // 调用 LLM 生成文本
        let response = self
            .llm_manager
            .generate_text(system_prompt, user_prompt)
            .await?;

        // 从响应中提取 JSON
        let ai_customer = self.parse_response(&response)?;

        // 转换为 Customer 结构体
        Ok(self.convert_to_customer(ai_customer, id_num))
    }

    /// 构建系统提示词
    fn build_system_prompt(&self) -> String {
        r#"你是一个游戏中的顾客生成器，负责为一家名为"星夜小馆"的餐厅生成有趣且真实的顾客。

## 游戏背景
这是一个关于火星玩家通过 AI 机器人"盼盼"在地球上经营小馆的模拟经营游戏。顾客来自各行各业，有各自的背景故事和喜好。

## 顾客类型
- 普通顾客：一般的食客
- 美食家：对美食有研究
- 评论家：挑剔的美食评论家
- VIP顾客：高级客户
- 邻居：附近的居民

## 口味偏好
- 清淡：喜欢原汁原味
- 适中：平衡口味
- 重口味：喜欢浓郁调味
- 麻辣：喜欢辣味
- 酸甜：喜欢酸甜口

## 饮食限制
- 无限制：什么都能吃
- 素食：只吃素食
- 清真：遵循清真饮食
- 无麸质：不能吃麸质
- 低糖：需要控制糖分

## 输出格式
请以 JSON 格式输出顾客信息：
```json
{
  "name": "张小明",
  "age": 28,
  "occupation": "程序员",
  "customer_type": "普通顾客",
  "flavor_preference": "重口味",
  "dietary_restriction": "无限制",
  "price_sensitivity": 60,
  "patience": 70,
  "favorite_categories": ["川菜", "湘菜"],
  "background_story": "一位热爱美食的程序员...",
  "visit_reason": "听朋友推荐来这里尝尝",
  "personality_traits": ["健谈", "好奇心强"]
}
```

## 要求
1. 生成真实可信的人物
2. 属性之间要有逻辑关联
3. 背景故事要生动有趣
4. 直接输出纯 JSON，不要有任何额外的文字说明"#.to_string()
    }

    /// 构建用户提示词
    fn build_user_prompt(&self) -> String {
        "请生成一个有趣且真实的顾客，直接输出 JSON 格式。".to_string()
    }

    /// 解析 LLM 响应
    fn parse_response(&self, content: &str) -> Result<AIGeneratedCustomer> {
        // 清理 markdown 代码块标记
        let cleaned = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // 解析 JSON
        let ai_customer: AIGeneratedCustomer = serde_json::from_str(cleaned)
            .map_err(|e| GameError::LlmError(format!("Failed to parse customer JSON: {}", e)))?;

        Ok(ai_customer)
    }

    /// 转换为 Customer 结构体
    fn convert_to_customer(&self, ai_customer: AIGeneratedCustomer, id_num: u32) -> Customer {
        // 创建偏好
        let preference = Preference {
            flavor: self.parse_flavor_preference(&ai_customer.flavor_preference),
            dietary: self.parse_dietary_restriction(&ai_customer.dietary_restriction),
            price_sensitivity: ai_customer.price_sensitivity.clamp(0, 100),
            patience: ai_customer.patience.clamp(50, 100),
            favorite_categories: ai_customer.favorite_categories,
        };

        // 创建顾客
        Customer {
            id: Uuid::new_v4(),
            id_num,
            name: ai_customer.name,
            customer_type: self.parse_customer_type(&ai_customer.customer_type),
            preference,
            vip_status: Default::default(),
            affinity: 0,
            visit_count: 0,
            current_order: None,
            reviews: Vec::new(),
            arrived_at: chrono::Utc::now(),
            finished: false,
        }
    }

    /// 解析顾客类型
    fn parse_customer_type(&self, type_str: &str) -> CustomerType {
        match type_str.trim() {
            "普通顾客" | "普通" => CustomerType::Normal,
            "美食家" => CustomerType::Foodie,
            "评论家" => CustomerType::Critic,
            "VIP顾客" | "VIP" => CustomerType::VIP,
            "邻居" => CustomerType::Neighbor,
            _ => CustomerType::Normal,
        }
    }

    /// 解析口味偏好
    fn parse_flavor_preference(&self, flavor_str: &str) -> FlavorPreference {
        match flavor_str.trim() {
            "清淡" => FlavorPreference::Light,
            "适中" => FlavorPreference::Medium,
            "重口味" => FlavorPreference::Heavy,
            "麻辣" => FlavorPreference::Spicy,
            "酸甜" => FlavorPreference::SweetSour,
            _ => FlavorPreference::Medium,
        }
    }

    /// 解析饮食限制
    fn parse_dietary_restriction(&self, restriction_str: &str) -> DietaryRestriction {
        match restriction_str.trim() {
            "无限制" | "无" => DietaryRestriction::None,
            "素食" => DietaryRestriction::Vegetarian,
            "清真" => DietaryRestriction::Halal,
            "无麸质" => DietaryRestriction::GlutenFree,
            "低糖" => DietaryRestriction::LowSugar,
            _ => DietaryRestriction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> LlmConfig {
        let mut config = LlmConfig::default();
        config.base_url = "http://localhost".to_string();
        config.port = 11434;
        config
    }

    #[test]
    fn test_parse_customer_type() {
        let config = create_test_config();
        let generator = AICustomerGenerator::new(config).unwrap();

        assert_eq!(generator.parse_customer_type("普通顾客"), CustomerType::Normal);
        assert_eq!(generator.parse_customer_type("美食家"), CustomerType::Foodie);
        assert_eq!(generator.parse_customer_type("VIP"), CustomerType::VIP);
    }

    #[test]
    fn test_parse_flavor_preference() {
        let config = create_test_config();
        let generator = AICustomerGenerator::new(config).unwrap();

        assert_eq!(
            generator.parse_flavor_preference("清淡"),
            FlavorPreference::Light
        );
        assert_eq!(
            generator.parse_flavor_preference("麻辣"),
            FlavorPreference::Spicy
        );
    }

    #[test]
    fn test_parse_dietary_restriction() {
        let config = create_test_config();
        let generator = AICustomerGenerator::new(config).unwrap();

        assert_eq!(
            generator.parse_dietary_restriction("素食"),
            DietaryRestriction::Vegetarian
        );
        assert_eq!(
            generator.parse_dietary_restriction("清真"),
            DietaryRestriction::Halal
        );
    }

    #[test]
    fn test_parse_json_response() {
        let config = create_test_config();
        let generator = AICustomerGenerator::new(config).unwrap();

        let json = r#"{
            "name": "张小明",
            "age": 28,
            "occupation": "程序员",
            "customer_type": "普通顾客",
            "flavor_preference": "重口味",
            "dietary_restriction": "无限制",
            "price_sensitivity": 60,
            "patience": 70,
            "favorite_categories": ["川菜", "湘菜"],
            "background_story": "一位热爱美食的程序员",
            "visit_reason": "听朋友推荐",
            "personality_traits": ["健谈"]
        }"#;

        let result = generator.parse_response(json);
        assert!(result.is_ok());

        let customer = result.unwrap();
        assert_eq!(customer.name, "张小明");
        assert_eq!(customer.age, 28);
    }

    #[test]
    fn test_parse_json_with_markdown() {
        let config = create_test_config();
        let generator = AICustomerGenerator::new(config).unwrap();

        let markdown_json = r#"```json
{
  "name": "李小红",
  "age": 25,
  "occupation": "设计师",
  "customer_type": "美食家",
  "flavor_preference": "清淡",
  "dietary_restriction": "素食",
  "price_sensitivity": 50,
  "patience": 80,
  "favorite_categories": ["粤菜"],
  "background_story": "热爱美食的设计师",
  "visit_reason": "路过",
  "personality_traits": ["安静"]
}
```"#;

        let result = generator.parse_response(markdown_json);
        assert!(result.is_ok());

        let customer = result.unwrap();
        assert_eq!(customer.name, "李小红");
        assert_eq!(customer.customer_type, "美食家");
    }
}
