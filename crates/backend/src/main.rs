//! 后端服务入口
use std::sync::Arc;

use anyhow::Result;
use axum::{Router, routing::get};
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use flavors_backend::api::{
    create_save, delete_save, get_save, health_check, list_saves, liveness_check, readiness_check,
    ws_handler, ApiDoc,
};
use flavors_backend::config::Settings;
use flavors_backend::db::DbPool;
use flavors_backend::game::AppState;

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
    // API 路由
    let api_routes = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // WebSocket
        .route("/ws", get(ws_handler))
        // 存档 API
        .route("/saves", get(list_saves).post(create_save))
        .route("/saves/{save_id}", get(get_save).delete(delete_save));

    // 合并所有路由
    Router::new()
        .nest("/api", api_routes)
        // Swagger UI
        .merge(ApiDoc::swagger_ui())
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

    // 创建 API 状态
    let state = Arc::new(AppState::new(db_pool, settings.llm.clone()));

    // 创建路由
    let app = create_router(state);

    // 启动服务器
    let addr = settings.server.addr();
    info!("Server listening on {}", addr);
    info!("Swagger UI: http://{}/swagger-ui/", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    info!("Server stopped");
    Ok(())
}
