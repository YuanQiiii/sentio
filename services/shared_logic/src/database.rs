//! # 内存数据库模块
//!
//! 提供线程安全的内存数据库实现，替代 MongoDB 依赖。

use anyhow::Result;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

/// 内存数据库状态
#[derive(Debug, Default)]
pub struct MemoryDatabase {
    pub memory_corpus: Vec<crate::MemoryCorpus>,
    pub memory_fragments: Vec<crate::MemoryFragment>,
    pub interaction_logs: Vec<crate::InteractionLog>,

    // 指标统计
    pub metrics: DatabaseMetrics,
}

/// 数据库操作指标
#[derive(Debug, Default)]
pub struct DatabaseMetrics {
    pub reads: u64,
    pub writes: u64,
    pub query_hits: u64,
    pub query_misses: u64,
}

// 全局内存数据库实例
static GLOBAL_DB: OnceLock<Arc<RwLock<MemoryDatabase>>> = OnceLock::new();

/// 初始化内存数据库
///
/// # 错误
///
/// - 如果全局数据库已经初始化过
pub async fn initialize_database() -> Result<()> {
    let db = Arc::new(RwLock::new(MemoryDatabase::default()));

    GLOBAL_DB
        .set(db)
        .map_err(|_| anyhow::anyhow!("Global database has already been initialized"))?;

    tracing::info!("In-memory database initialized successfully");
    Ok(())
}

/// 获取数据库统计信息
///
/// # 错误
///
/// - 如果数据库未初始化
pub async fn get_database_stats() -> Result<()> {
    let db = GLOBAL_DB
        .get()
        .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

    let db = db.read().await;
    tracing::debug!(
        memory_corpus_count = db.memory_corpus.len(),
        memory_fragments_count = db.memory_fragments.len(),
        interaction_logs_count = db.interaction_logs.len(),
        "In-memory database statistics"
    );

    Ok(())
}

/// 获取数据库实例
///
/// # Panics
///
/// 如果数据库未初始化
pub fn get_db() -> Arc<RwLock<MemoryDatabase>> {
    GLOBAL_DB.get().expect("Database not initialized").clone()
}
