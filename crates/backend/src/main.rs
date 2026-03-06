//! 后端服务入口
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use clap::Parser;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use flavors_backend::api::{
    complete_travel, create_recipe, delete_customer, get_command, get_current_travel,
    get_customer, get_dialogue_history, get_memory, get_message, get_panpan, get_plot, get_recipe,
    get_shop, get_travel, harvest_crop, health_check, list_commands, list_customers, list_memories,
    list_plots, list_recipes, list_travels, liveness_check, plant_crop, purchase_item,
    readiness_check, send_command, send_message, start_travel, unlock_memory, update_customer,
    update_funds, update_panpan, update_recipe_status, water_plot, ws_handler,
};
use flavors_backend::config::Settings;
use flavors_backend::db::DbPool;
use flavors_backend::game::{AppState, GameEngine, create_llm_manager};

/// 命令行参数
#[derive(Parser, Debug)]
#[command(version, about = "味延廿载后端服务")]
struct Args {
    /// 配置文件路径
    #[arg(short, long)]
    config_path: Option<std::path::PathBuf>,

    /// 数据库 URL
    #[arg(short, long)]
    database_url: Option<String>,

    /// 日志级别
    #[arg(short, long)]
    log_level: Option<String>,
}

/// 加载配置
fn load_settings(args: &Args) -> Result<Settings> {
    let mut settings = if let Some(path) = &args.config_path {
        Settings::load_from(path)
    } else {
        Settings::load()
    }
    .map_err(|e| anyhow::anyhow!("Failed to load settings: {}", e))?;

    // 命令行参数覆盖配置文件
    if let Some(db_url) = &args.database_url {
        settings.database.url = db_url.clone();
    }

    Ok(settings)
}

/// 初始化日志
fn init_logging(settings: &Settings) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&settings.logging.level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// 初始化数据库
async fn init_database(settings: &Settings) -> Result<Arc<DbPool>> {
    let db_pool = DbPool::new(&settings.database.url).await?;
    db_pool.run_migrations().await?;

    // 初始化种子数据
    db_pool.initialize_seed_data().await?;

    Ok(Arc::new(db_pool))
}

/// 创建路由
fn create_router(state: Arc<AppState>) -> Router {
    // API 路由（每个数据库就是一个存档，不再需要 save_id）
    let api_routes = Router::new()
        // 健康检查
        .route("/health", axum::routing::get(health_check))
        .route("/health/ready", axum::routing::get(readiness_check))
        .route("/health/live", axum::routing::get(liveness_check))
        // WebSocket
        .route("/ws", axum::routing::get(ws_handler))
        // 指令 API
        .route(
            "/commands",
            axum::routing::get(list_commands).post(send_command),
        )
        .route("/commands/{command_id}", axum::routing::get(get_command))
        // 对话 API
        .route(
            "/dialogues",
            axum::routing::get(get_dialogue_history).post(send_message),
        )
        .route("/dialogues/{message_id}", axum::routing::get(get_message))
        // 菜谱 API
        .route(
            "/recipes",
            axum::routing::get(list_recipes).post(create_recipe),
        )
        .route("/recipes/{recipe_id}", axum::routing::get(get_recipe))
        .route(
            "/recipes/{recipe_id}/status",
            axum::routing::patch(update_recipe_status),
        )
        // 顾客 API
        .route("/customers", axum::routing::get(list_customers))
        .route(
            "/customers/{customer_id}",
            axum::routing::get(get_customer)
                .patch(update_customer)
                .delete(delete_customer),
        )
        // 记忆碎片 API
        .route("/memories", axum::routing::get(list_memories))
        .route("/memories/{memory_id}", axum::routing::get(get_memory))
        .route(
            "/memories/{memory_id}/unlock",
            axum::routing::post(unlock_memory),
        )
        // 盼盼状态 API
        .route(
            "/panpan",
            axum::routing::get(get_panpan).patch(update_panpan),
        )
        // 菜园 API
        .route("/garden/plots", axum::routing::get(list_plots))
        .route("/garden/plots/{plot_id}", axum::routing::get(get_plot))
        .route(
            "/garden/plots/{plot_id}/plant",
            axum::routing::post(plant_crop),
        )
        .route(
            "/garden/plots/{plot_id}/water",
            axum::routing::post(water_plot),
        )
        .route(
            "/garden/plots/{plot_id}/harvest",
            axum::routing::post(harvest_crop),
        )
        // 商店 API
        .route("/shop", axum::routing::get(get_shop))
        .route("/shop/purchase", axum::routing::post(purchase_item))
        .route("/shop/funds", axum::routing::patch(update_funds))
        // 旅行 API
        .route(
            "/travels",
            axum::routing::get(list_travels).post(start_travel),
        )
        .route("/travels/current", axum::routing::get(get_current_travel))
        .route("/travels/{travel_id}", axum::routing::get(get_travel))
        .route(
            "/travels/{travel_id}/complete",
            axum::routing::post(complete_travel),
        );

    // 合并所有路由
    Router::new()
        .nest("/api/v1", api_routes)
        .with_state(state)
}

/// 运行服务
#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 加载配置
    let settings = load_settings(&args)?;

    // 初始化日志
    init_logging(&settings);

    info!("Starting 味延廿载 backend service...");

    // 初始化数据库
    let db_pool = init_database(&settings).await?;

    // 创建 LLM 管理器（失败则不允许启动）
    let llm_manager = create_llm_manager(settings.llm.clone())
        .map_err(|e| anyhow::anyhow!("Failed to initialize LLM: {}", e))?;

    // 创建取消令牌（用于优雅退出）
    let cancel_token = CancellationToken::new();

    // 启动游戏引擎（后台运行）
    let mut engine = GameEngine::new(llm_manager, db_pool.clone(), cancel_token.clone());
    let engine_task = tokio::spawn(async move {
        engine.start().await;
    });
    info!("GameEngine started");

    // 创建 API 状态
    let state = Arc::new(AppState::new(db_pool, settings.llm.clone()));

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let addr = settings.server.addr();
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // 设置 Ctrl+C 信号监听
    let shutdown_token = cancel_token.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C signal, initiating graceful shutdown...");
                shutdown_token.cancel();
            }
            Err(e) => {
                tracing::error!("Failed to listen for Ctrl+C: {}", e);
            }
        }
    });

    // 使用优雅关闭运行服务器
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancel_token.cancelled().await;
            info!("HTTP server shutting down...");
        })
        .await?;

    // 等待游戏引擎完全停止
    info!("Waiting for GameEngine to stop...");
    engine_task.await?;
    info!("GameEngine stopped");

    info!("Server stopped gracefully");
    Ok(())
}
