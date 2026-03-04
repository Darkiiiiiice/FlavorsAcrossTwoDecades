//! 数据库连接池

use std::path::Path;

use sqlx::sqlite::SqlitePool;

use crate::db::MIGRATIONS;
use crate::error::{DatabaseError, GameError, GameResult};

/// 从 SQLite URL 中提取文件路径
/// 支持格式：
/// - `sqlite:path/to/db.sqlite` - 相对路径
/// - `sqlite:///absolute/path/db.sqlite` - 绝对路径
/// - `sqlite::memory:` - 内存数据库（返回 None）
fn extract_db_path(database_url: &str) -> Option<String> {
    let url = database_url.trim();

    // 移除 sqlite: 前缀
    let path_part = url.strip_prefix("sqlite:")?;

    // 内存数据库
    if path_part == ":memory:" {
        return None;
    }

    // 处理绝对路径 (sqlite:///path/to/db)
    if let Some(absolute_path) = path_part.strip_prefix("///") {
        // 移除查询参数
        let path = absolute_path.split('?').next().unwrap_or(absolute_path);
        return Some(path.to_string());
    }

    // 处理相对路径 (sqlite:path/to/db 或 sqlite://path/to/db)
    let path = if let Some(relative_path) = path_part.strip_prefix("//") {
        relative_path.split('?').next().unwrap_or(relative_path)
    } else {
        path_part.split('?').next().unwrap_or(path_part)
    };

    Some(path.to_string())
}

/// 使用 sqlite3 命令创建空数据库文件
fn create_empty_database(db_path: &str) -> GameResult<()> {
    let parent = Path::new(db_path).parent();

    // 确保父目录存在
    if let Some(parent_dir) = parent
        && !parent_dir.exists()
    {
        std::fs::create_dir_all(parent_dir).map_err(|e| {
            GameError::Database(DatabaseError::ConnectionFailed(format!(
                "Failed to create database directory: {}",
                e
            )))
        })?;
        tracing::info!("Created database directory: {:?}", parent_dir);
    }

    // 使用 sqlite3 命令创建空数据库
    let output = std::process::Command::new("sqlite3")
        .arg(db_path)
        .arg("SELECT 1;")
        .output()
        .map_err(|e| {
            GameError::Database(DatabaseError::ConnectionFailed(format!(
                "Failed to execute sqlite3 command: {}. Please ensure sqlite3 is installed.",
                e
            )))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GameError::Database(DatabaseError::ConnectionFailed(
            format!("Failed to create database file: {}", stderr),
        )));
    }

    tracing::info!("Created new database file: {}", db_path);
    Ok(())
}

/// 数据库连接池
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    /// 创建新的数据库连接池
    pub async fn new(database_url: &str) -> GameResult<Self> {
        // 检查数据库文件是否存在，如果不存在则创建
        if let Some(db_path) = extract_db_path(database_url)
            && !Path::new(&db_path).exists()
        {
            tracing::info!("Database file not found, creating: {}", db_path);
            create_empty_database(&db_path)?;
        }

        // 连接到数据库
        let pool = SqlitePool::connect(database_url).await.map_err(|e| {
            GameError::Database(DatabaseError::ConnectionFailed(format!(
                "Database connection failed: {}",
                e
            )))
        })?;

        Ok(Self { pool })
    }

    /// 获取数据库连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// 运行数据库迁移
    pub async fn run_migrations(&self) -> GameResult<()> {
        tracing::info!("Running {} embedded migrations...", MIGRATIONS.len());

        for (name, sql) in MIGRATIONS {
            tracing::info!("Running migration: {}", name);

            // 分割 SQL 语句（处理多个语句）
            for statement in sql.split(';') {
                let statement = statement.trim();
                if statement.is_empty() {
                    continue;
                }

                if let Err(e) = sqlx::query(statement).execute(self.pool()).await {
                    // 忽略"表已存在"等重复执行错误
                    let error_msg = e.to_string();
                    if error_msg.contains("already exists") {
                        tracing::debug!("Migration already applied: {}", name);
                        continue;
                    }

                    tracing::error!("Migration {} failed: {}", name, e);
                    return Err(GameError::Database(DatabaseError::QueryFailed(format!(
                        "Failed to execute migration '{}': {}",
                        name, e
                    ))));
                }
            }

            tracing::info!("Migration completed: {}", name);
        }

        tracing::info!("All migrations completed successfully");
        Ok(())
    }

    /// 初始化种子数据
    pub async fn initialize_seed_data(&self) -> GameResult<()> {
        crate::db::seed::initialize_seed_data(&self.pool).await
    }
}
