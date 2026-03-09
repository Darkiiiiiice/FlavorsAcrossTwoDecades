#![allow(clippy::module_inception)]
//! 顾客系统模块

mod customer;
pub mod preference;
mod prompt;

pub use customer::{Customer, CustomerType};
pub use preference::{DietaryRestriction, FlavorPreference, Preference};
pub use prompt::{CustomerLlmResponse, CustomerPromptBuilder};

// 重新导出 Review（来自 order 模块）
pub use crate::game::order::Review;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use rand::RngExt;

use crate::config::LlmConfig;
use crate::db::DbPool;
use crate::db::repositories::customer::CustomerRepository;
use crate::error::{GameError, GameResult};
use crate::game::LlmManager;

/// 顾客生成最小间隔（秒）
const MIN_GENERATION_INTERVAL: i64 = 12;
/// 顾客生成最大间隔（秒）
const MAX_GENERATION_INTERVAL: i64 = 60;

/// 后台顾客生成任务的结果
type CustomerGenerationResult = GameResult<Customer>;

/// 顾客管理器
#[derive(Debug)]
pub struct CustomerManager {
    /// 活跃顾客列表
    pub active_customers: Vec<i64>,
    /// 历史顾客记录
    pub customer_history: Vec<i64>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 下次生成顾客的时间戳（游戏秒）
    next_generation_time: Option<i64>,
    /// 数据库连接池
    db_pool: Option<Arc<DbPool>>,
    /// LLM 管理器（可选）
    llm_manager: Option<Arc<LlmManager>>,
    /// 后台生成任务句柄
    generation_task: Option<tokio::task::JoinHandle<CustomerGenerationResult>>,
}

impl CustomerManager {
    /// 创建新的顾客管理器
    pub fn new() -> Self {
        Self {
            active_customers: Vec::new(),
            customer_history: Vec::new(),
            updated_at: Utc::now(),
            next_generation_time: None,
            db_pool: None,
            llm_manager: None,
            generation_task: None,
        }
    }

    /// 设置数据库连接池
    pub fn with_db_pool(mut self, db_pool: Arc<DbPool>) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    /// 设置 LLM 管理器
    pub fn with_llm(mut self, llm_manager: Arc<LlmManager>) -> Self {
        self.llm_manager = Some(llm_manager);
        self
    }

    /// 创建带 AI 生成器的顾客管理器（兼容旧接口）
    pub fn with_ai(_llm_config: LlmConfig) -> GameResult<Self> {
        Ok(Self::new())
    }

    /// 生成随机时间间隔（120-600秒）
    fn random_generation_interval() -> i64 {
        let mut rng = rand::rng();
        rng.random_range(MIN_GENERATION_INTERVAL..=MAX_GENERATION_INTERVAL)
    }

    /// 设置下次生成时间
    fn schedule_next_generation(&mut self, current_time: i64) {
        let interval = Self::random_generation_interval();
        self.next_generation_time = Some(current_time + interval);
        tracing::debug!(
            "Scheduled next customer generation in {} seconds (at {})",
            interval,
            self.next_generation_time.unwrap()
        );
    }

    /// 生成顾客（使用 LLM 生成）
    pub async fn generate_customer(
        &mut self,
        customer_type: Option<CustomerType>,
    ) -> GameResult<Customer> {
        // 必须有 LLM 管理器
        let llm = self.llm_manager.as_ref().ok_or_else(|| GameError::LlmError(
            "LLM manager not configured".to_string()
        ))?;

        self.generate_customer_with_llm(llm.clone(), customer_type).await
    }

    /// 使用 LLM 生成顾客
    async fn generate_customer_with_llm(
        &self,
        llm_manager: Arc<LlmManager>,
        customer_type: Option<CustomerType>,
    ) -> GameResult<Customer> {
        // 获取已有顾客的故事背景（用于避免重复）
        let existing_stories = self.get_existing_stories().await;

        // 构建提示词
        let builder = CustomerPromptBuilder::new().with_existing_stories(existing_stories);

        let builder = match customer_type {
            Some(ct) => builder.with_customer_type(ct),
            None => builder,
        };

        let system_prompt = builder.build_system_prompt();
        let user_message = builder.build_user_message();

        tracing::debug!("Generating customer with LLM...");
        tracing::debug!("System prompt:\n{}", system_prompt);
        tracing::debug!("User message:\n{}", user_message);

        let response = llm_manager
            .generate_text(system_prompt.to_string(), user_message)
            .await?;

        tracing::debug!("LLM response:\n{}", response);

        // 解析响应
        let llm_response = CustomerLlmResponse::parse(&response).ok_or_else(|| {
            GameError::LlmError(format!("Failed to parse LLM response: {}", response))
        })?;

        // 构建 Customer
        let customer = Customer {
            id: 0, // 将在保存时由数据库分配
            name: llm_response.name.clone(),
            age: llm_response.age,
            occupation: llm_response.occupation.clone(),
            customer_type: llm_response.to_customer_type(),
            preference: Preference {
                flavor: llm_response.to_flavor(),
                dietary: llm_response.to_dietary(),
                price_sensitivity: llm_response.preference.price_sensitivity.min(100),
                patience: llm_response.preference.patience.min(100),
                favorite_categories: llm_response.preference.favorite_categories.clone(),
            },
            affinity: 0,
            visit_count: 0,
            story_background: llm_response.story_background.clone(),
        };

        Ok(customer)
    }

    /// 获取已有顾客的故事背景
    async fn get_existing_stories(&self) -> Vec<String> {
        if let Some(ref db_pool) = self.db_pool {
            let repo = CustomerRepository::new(db_pool.pool().clone());
            match repo.find_all().await {
                Ok(customers) => customers
                    .iter()
                    .map(|c| c.story_background.clone())
                    .collect(),
                Err(e) => {
                    tracing::warn!("Failed to get existing customers: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// 启动后台顾客生成任务
    fn start_background_generation(&mut self, customer_type: Option<CustomerType>) {
        if self.generation_task.is_some() {
            tracing::debug!("Customer generation task already running");
            return;
        }

        let llm_manager = self.llm_manager.clone();
        let db_pool = self.db_pool.clone();

        let task = tokio::spawn(async move {
            // 创建临时管理器进行生成
            let mut temp_manager = CustomerManager::new();
            if let Some(db) = db_pool {
                temp_manager = temp_manager.with_db_pool(db);
            }
            if let Some(llm) = llm_manager {
                temp_manager = temp_manager.with_llm(llm);
            }

            temp_manager.generate_customer(customer_type).await
        });

        self.generation_task = Some(task);
        tracing::info!("Started background customer generation");
    }

    /// 检查后台生成任务是否完成
    async fn check_background_generation(&mut self) -> Option<GameResult<Customer>> {
        if let Some(task) = self.generation_task.take() {
            if task.is_finished() {
                match task.await {
                    Ok(result) => {
                        self.generation_task = None;
                        return Some(result);
                    }
                    Err(e) => {
                        tracing::error!("Background customer generation task failed: {:?}", e);
                        self.generation_task = None;
                        return Some(Err(GameError::Internal {
                            request_id: format!(
                                "background_task_{}",
                                chrono::Utc::now().timestamp()
                            ),
                        }));
                    }
                }
            } else {
                // 任务仍在运行，放回句柄
                self.generation_task = Some(task);
            }
        }
        None
    }

    /// 更新顾客状态（每秒调用）
    ///
    /// # 参数
    /// - `current_time`: 当前游戏时间戳（秒）
    /// - `is_restaurant_open`: 餐厅是否营业中
    pub async fn tick(&mut self, current_time: i64, is_restaurant_open: bool) {
        tracing::debug!(
            "Customer tick at {}, restaurant open: {}, next_generation_at: {:?}",
            current_time,
            is_restaurant_open,
            self.next_generation_time,
        );

        // 检查后台生成任务
        if let Some(result) = self.check_background_generation().await {
            match result {
                Ok(customer) => {
                    tracing::info!("New customer arrived: {}", customer.name);
                    // 保存顾客到数据库
                    match self.save_customer(&customer).await {
                        Ok(customer_id) => {
                            // 添加到活跃顾客列表
                            self.active_customers.push(customer_id);
                            self.updated_at = Utc::now();
                            tracing::info!(
                                "Customer {} saved with id {}",
                                customer.name,
                                customer_id
                            );
                            // 安排下次生成
                            self.schedule_next_generation(current_time);
                        }
                        Err(e) => {
                            tracing::error!("Failed to save customer: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Background generation failed: {}", e);
                    // 即使失败也要安排下次生成
                    self.schedule_next_generation(current_time);
                }
            }
            return;
        }

        // 如果餐厅不营业，不生成新顾客
        if !is_restaurant_open {
            tracing::trace!("Restaurant is closed, skipping customer generation");
            return;
        }

        // 检查是否到了生成时间
        let should_generate = match self.next_generation_time {
            Some(next_time) => current_time >= next_time,
            None => {
                // 首次运行，立即安排生成
                self.schedule_next_generation(current_time);
                false
            }
        };

        if should_generate && self.generation_task.is_none() {
            tracing::info!(
                "Time to generate new customer at {} (scheduled for {})",
                current_time,
                self.next_generation_time.unwrap()
            );
            // 启动后台生成任务
            self.start_background_generation(None);
        }
    }

    /// 保存顾客到数据库
    pub async fn save_customer(&self, customer: &Customer) -> GameResult<i64> {
        let db_pool = self.db_pool.as_ref().ok_or_else(|| GameError::Internal {
            request_id: "save_customer".to_string(),
        })?;

        let repo = CustomerRepository::new(db_pool.pool().clone());
        repo.create(customer).await
    }

    /// 强制立即生成一个顾客（用于测试或特殊事件）
    pub async fn force_generate(&mut self, current_time: i64) -> GameResult<Customer> {
        let customer = self.generate_customer(None).await?;

        // 保存顾客到数据库
        let customer_id = self.save_customer(&customer).await?;

        // 添加到活跃顾客列表
        self.active_customers.push(customer_id);
        self.updated_at = Utc::now();

        // 安排下次生成
        self.schedule_next_generation(current_time);

        // 返回带有正确 ID 的顾客
        let mut customer = customer;
        customer.id = customer_id;
        Ok(customer)
    }

    /// 获取距离下次生成的时间（秒）
    pub fn time_until_next_generation(&self, current_time: i64) -> Option<i64> {
        self.next_generation_time.map(|t| (t - current_time).max(0))
    }
}

impl Default for CustomerManager {
    fn default() -> Self {
        Self::new()
    }
}
