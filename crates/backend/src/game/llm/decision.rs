//! 盼盼决策系统

use serde::{Deserialize, Serialize};

use crate::error::{GameError, Result};

/// 决策类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    /// 玩家指令
    Command(String),
    /// 自主行动
    AutonomousAction,
    /// 事件处理
    Event(String),
    /// 旅行决策
    Travel,
    /// 实验策略
    Experiment,
    /// 简报生成
    DailyReport,
    /// 旅行日志
    TravelLog,
}

/// 性格参数变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityChange {
    /// 经营风格变化
    pub business_style: Option<f32>,
    /// 创新倾向变化
    pub innovation: Option<f32>,
    /// 独立倾向变化
    pub independence: Option<f32>,
}

/// 盼盼的决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// 是否理解
    pub understood: bool,
    /// 理解/解释
    pub interpretation: String,
    /// 是否执行
    pub will_execute: bool,
    /// 执行计划
    pub execution_plan: String,
    /// 修改建议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modification: Option<String>,
    /// 对玩家的回复
    pub response_to_player: String,
    /// 性格参数变化
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality_changes: Option<PersonalityChange>,
}

impl Decision {
    /// 从 LLM 响应解析决策
    pub fn parse_from_response(response: &str, _decision_type: DecisionType) -> Result<Self> {
        // 尝试提取 JSON
        let json_str = Self::extract_json(response)?;

        // 解析 JSON
        let decision: Decision = serde_json::from_str(&json_str)
            .map_err(|e| GameError::LlmError(format!("Failed to parse decision JSON: {}", e)))?;

        Ok(decision)
    }

    /// 从响应文本中提取 JSON
    fn extract_json(response: &str) -> Result<String> {
        // 尝试找到 JSON 代码块
        if let Some(start) = response.find("```json") {
            let json_start = start + 7; // "```json" 的长度
            if let Some(end) = response[json_start..].find("```") {
                let json_end = json_start + end;
                return Ok(response[json_start..json_end].trim().to_string());
            }
        }

        // 尝试找到裸 JSON（以 { 开始，以 } 结束）
        if let Some(start) = response.find('{')
            && let Some(end) = response.rfind('}')
            && end > start
        {
            return Ok(response[start..=end].trim().to_string());
        }

        // 如果找不到 JSON，返回错误
        Err(GameError::LlmError(
            "No valid JSON found in LLM response".to_string(),
        ))
    }

    /// 创建简单的确认决策
    pub fn simple_confirm(response: &str) -> Self {
        Self {
            understood: true,
            interpretation: "理解指令".to_string(),
            will_execute: true,
            execution_plan: "执行指令".to_string(),
            modification: None,
            response_to_player: response.to_string(),
            personality_changes: None,
        }
    }

    /// 创建拒绝决策
    pub fn reject(reason: &str) -> Self {
        Self {
            understood: true,
            interpretation: "理解但拒绝执行".to_string(),
            will_execute: false,
            execution_plan: String::new(),
            modification: None,
            response_to_player: reason.to_string(),
            personality_changes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_with_code_block() {
        let response = r#"
Here is the decision:
```json
{
  "understood": true,
  "interpretation": "test",
  "will_execute": true,
  "execution_plan": "plan",
  "response_to_player": "ok"
}
```
"#;
        let json = Decision::extract_json(response).unwrap();
        assert!(json.contains("understood"));
    }

    #[test]
    fn test_extract_json_bare() {
        let response = r#"
Here is the decision:
{
  "understood": true,
  "interpretation": "test",
  "will_execute": true,
  "execution_plan": "plan",
  "response_to_player": "ok"
}
"#;
        let json = Decision::extract_json(response).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
