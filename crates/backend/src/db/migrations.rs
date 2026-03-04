//! 数据库迁移
//!
//! SQL 迁移文件在编译时嵌入到二进制中

/// 嵌入的迁移文件
/// 格式：(文件名, SQL内容)
pub const MIGRATIONS: &[(&str, &str)] = &[(
    "001_initial.sql",
    include_str!("../../migrations/001_initial.sql"),
)];
