//! 数据库迁移
//! SQL 迁移文件在编译时嵌入到二进制中

/// 嵌入的迁移文件
/// 格式：(文件名, SQL内容)
pub const MIGRATIONS: &[(&str, &str)] = &[(
    "intialization.sql",
    include_str!("../../migrations/initial.sql"),
)];
