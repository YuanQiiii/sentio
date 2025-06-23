//! # 统一数据库访问层
//!
//! 提供集中化的数据库连接管理和数据访问功能，遵循"健壮性是底线"原则。
//! 所有服务通过此模块访问数据库，避免重复的连接管理和配置。

use crate::config::get_config;
use anyhow::{Context, Result};
use bson::{doc, Document};
use mongodb::{
    options::{ClientOptions, IndexOptions},
    Client, Collection, Database, IndexModel,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{error, info};

// 全局数据库实例
static GLOBAL_DATABASE: OnceLock<Database> = OnceLock::new();

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Configuration error: {field}")]
    ConfigurationError { field: String },

    #[error("Database connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Database operation failed: {message}")]
    OperationFailed { message: String },

    #[error("Data validation error: {details}")]
    ValidationError { details: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// 初始化全局数据库连接
///
/// 此函数应在应用启动时调用一次，会创建并测试数据库连接。
/// 后续所有数据库操作都将使用这个全局连接。
pub async fn initialize_database() -> Result<()> {
    let config = get_config();
    let db_config = &config.database;

    info!(
        database_url = %db_config.url,
        max_connections = db_config.max_connections,
        timeout = db_config.connect_timeout,
        "Initializing global database connection"
    );

    // 验证配置
    if db_config.url.is_empty() {
        return Err(DatabaseError::ConfigurationError {
            field: "database.url is empty".to_string(),
        }
        .into());
    }

    // 配置 MongoDB 客户端选项
    let mut client_options = ClientOptions::parse(&db_config.url)
        .await
        .context("Failed to parse MongoDB URL")?;

    // 配置连接池和超时
    client_options.max_pool_size = Some(db_config.max_connections);
    client_options.connect_timeout = Some(Duration::from_secs(db_config.connect_timeout));
    client_options.server_selection_timeout = Some(Duration::from_secs(30));

    // 创建客户端
    let client = Client::with_options(client_options).context("Failed to create MongoDB client")?;

    // 连接到数据库
    let database = client.database("sentio");

    // 测试连接
    database
        .run_command(doc! { "ping": 1 }, None)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed {
            message: format!("Failed to ping MongoDB: {}", e),
        })?;

    // 确保索引存在
    ensure_database_indexes(&database).await?;

    // 设置全局数据库实例
    GLOBAL_DATABASE
        .set(database)
        .map_err(|_| anyhow::anyhow!("Global database already initialized"))?;

    info!("Global database connection initialized successfully");
    Ok(())
}

/// 获取全局数据库实例（安全版本）
///
/// 如果数据库未初始化返回 None 而不是 panic
pub fn try_get_database() -> Option<&'static Database> {
    GLOBAL_DATABASE.get()
}

/// 获取全局数据库实例
///
/// # Panics
/// 如果数据库未初始化则会 panic，确保在调用前已调用 `initialize_database()`
pub fn get_database() -> &'static Database {
    GLOBAL_DATABASE
        .get()
        .expect("Database not initialized. Call initialize_database() first.")
}

/// 获取指定集合的引用
pub fn get_collection<T>(collection_name: &str) -> Collection<T>
where
    T: Send + Sync,
{
    get_database().collection::<T>(collection_name)
}

/// 确保数据库索引存在
async fn ensure_database_indexes(database: &Database) -> Result<()> {
    info!("Creating database indexes...");

    // 记忆体集合索引
    let memory_corpus_collection: Collection<Document> = database.collection("memory_corpus");
    let memory_indexes = vec![
        IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "created_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_created_at_-1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "updated_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_updated_at_-1".to_string())
                    .build(),
            )
            .build(),
    ];

    memory_corpus_collection
        .create_indexes(memory_indexes, None)
        .await
        .context("Failed to create memory_corpus indexes")?;

    // 交互记录集合索引
    let interaction_collection: Collection<Document> = database.collection("interactions");
    let interaction_indexes = vec![
        IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "timestamp": -1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_timestamp_-1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "session_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("session_id_1".to_string())
                    .build(),
            )
            .build(),
    ];

    interaction_collection
        .create_indexes(interaction_indexes, None)
        .await
        .context("Failed to create interactions indexes")?;

    // 记忆片段集合索引
    let memory_fragment_collection: Collection<Document> = database.collection("memory_fragments");
    let fragment_indexes = vec![
        IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "memory_type": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_memory_type_1".to_string())
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "created_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("user_id_created_at_-1".to_string())
                    .build(),
            )
            .build(),
    ];

    memory_fragment_collection
        .create_indexes(fragment_indexes, None)
        .await
        .context("Failed to create memory_fragments indexes")?;

    info!("Database indexes created successfully");
    Ok(())
}

/// 检查数据库连接健康状态
pub async fn check_database_health() -> DatabaseResult<()> {
    let database = try_get_database().ok_or_else(|| DatabaseError::ConnectionFailed {
        message: "Database not initialized".to_string(),
    })?;

    database
        .run_command(doc! { "ping": 1 }, None)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed {
            message: format!("Database health check failed: {}", e),
        })?;

    Ok(())
}

/// 数据库统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_users: u64,
    pub total_memory_corpus: u64,
    pub total_interactions: u64,
    pub total_memory_fragments: u64,
}

/// 获取数据库统计信息
pub async fn get_database_stats() -> DatabaseResult<DatabaseStats> {
    let database = try_get_database().ok_or_else(|| DatabaseError::ConnectionFailed {
        message: "Database not initialized".to_string(),
    })?;

    // 获取用户统计 (通过 memory_corpus 集合中的唯一 user_id)
    let total_users = database
        .collection::<Document>("memory_corpus")
        .distinct("user_id", doc! {}, None)
        .await
        .map_err(|e| DatabaseError::OperationFailed {
            message: format!("Failed to count users: {}", e),
        })?
        .len() as u64;

    // 获取记忆体总数
    let total_memory_corpus = database
        .collection::<Document>("memory_corpus")
        .count_documents(doc! {}, None)
        .await
        .map_err(|e| DatabaseError::OperationFailed {
            message: format!("Failed to count memory corpus: {}", e),
        })?;

    // 获取交互记录总数
    let total_interactions = database
        .collection::<Document>("interactions")
        .count_documents(doc! {}, None)
        .await
        .map_err(|e| DatabaseError::OperationFailed {
            message: format!("Failed to count interactions: {}", e),
        })?;

    // 获取记忆片段总数
    let total_memory_fragments = database
        .collection::<Document>("memory_fragments")
        .count_documents(doc! {}, None)
        .await
        .map_err(|e| DatabaseError::OperationFailed {
            message: format!("Failed to count memory fragments: {}", e),
        })?;

    Ok(DatabaseStats {
        total_users,
        total_memory_corpus,
        total_interactions,
        total_memory_fragments,
    })
}
