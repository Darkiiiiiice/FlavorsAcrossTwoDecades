//! Prompt 模板管理系统

use handlebars::Handlebars;

use crate::error::{GameError, Result};

/// Prompt 模板管理器
pub struct PromptTemplates {
    engine: Handlebars<'static>,
}

impl PromptTemplates {
    /// 创建新的模板管理器
    pub fn new() -> Result<Self> {
        let mut engine = Handlebars::new();

        // 注册内置模板
        Self::register_builtin_templates(&mut engine)?;

        Ok(Self { engine })
    }

    /// 注册内置模板
    fn register_builtin_templates(engine: &mut Handlebars<'static>) -> Result<()> {
        // 系统提示词模板
        engine
            .register_template_string("system_prompt", include_str!("templates/system_prompt.hbs"))
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))?;

        // 指令决策模板
        engine
            .register_template_string(
                "command_decision",
                include_str!("templates/command_decision.hbs"),
            )
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))?;

        // 自主行动模板
        engine
            .register_template_string(
                "autonomous_action",
                include_str!("templates/autonomous_action.hbs"),
            )
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))?;

        // 事件响应模板
        engine
            .register_template_string(
                "event_response",
                include_str!("templates/event_response.hbs"),
            )
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))?;

        // 每日简报模板
        engine
            .register_template_string("daily_report", include_str!("templates/daily_report.hbs"))
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))?;

        Ok(())
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, data: &impl Serialize) -> Result<String> {
        self.engine
            .render(template_name, data)
            .map_err(|e| GameError::LlmError(format!("Template rendering failed: {}", e)))
    }

    /// 注册自定义模板
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.engine
            .register_template_string(name, template)
            .map_err(|e| GameError::LlmError(format!("Failed to register template: {}", e)))
    }
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self::new().expect("Failed to create default PromptTemplates")
    }
}

use serde::Serialize;
