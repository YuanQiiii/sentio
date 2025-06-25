//! # 记忆仓储接口定义
//!
//! 定义记忆数据访问的核心接口，遵循"接口先行"设计原则。
//! 这些接口支持多种存储后端实现。

use crate::error::MemoryResult;
use crate::models::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 记忆查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// 查询文本（用于语义搜索）
    pub query_text: String,
    /// 用户ID过滤
    pub user_id: Option<String>,
    /// 时间范围过滤
    pub time_range: Option<TimeRange>,
    /// 记忆类型过滤
    pub memory_types: Vec<MemoryType>,
    /// 最大结果数量
    pub limit: Option<u32>,
    /// 相关性阈值 (0.0-1.0)
    pub relevance_threshold: Option<f64>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 记忆类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Episodic,
    Semantic,
    ActionState,
    StrategicInferential,
}

/// 记忆片段（查询结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// 片段唯一标识
    pub id: Uuid,
    /// 关联的用户ID
    pub user_id: String,
    /// 记忆类型
    pub memory_type: MemoryType,
    /// 记忆内容
    pub content: String,
    /// 关联的标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 相关性评分 (0.0-1.0)
    pub relevance_score: f64,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 用户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    /// 用户ID
    pub user_id: String,
    /// 总交互次数
    pub total_interactions: u64,
    /// 首次交互时间
    pub first_interaction: DateTime<Utc>,
    /// 最后交互时间
    pub last_interaction: DateTime<Utc>,
    /// 记忆片段总数
    pub total_memories: u64,
    /// 各类型记忆数量分布
    pub memory_type_distribution: HashMap<String, u64>,
}

/// 记忆仓储核心接口
///
/// 这个 trait 定义了所有记忆数据访问操作的核心接口。
/// 遵循"零信任"原则，所有输入数据都将被验证。
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// 保存完整的用户记忆体
    ///
    /// # 参数
    /// - `corpus`: 用户的完整记忆数据
    ///
    /// # 错误
    /// - `ValidationError`: 数据验证失败
    /// - `DatabaseOperationFailed`: 数据库操作失败
    /// - `StorageLimitExceeded`: 存储容量超限
    async fn save_memory_corpus(&self, corpus: &MemoryCorpus) -> MemoryResult<()>;

    /// 根据用户ID获取完整的记忆体
    ///
    /// # 参数
    /// - `user_id`: 用户唯一标识符
    ///
    /// # 返回
    /// - `Some(MemoryCorpus)`: 找到用户记忆体
    /// - `None`: 用户不存在
    async fn get_memory_corpus(&self, user_id: &str) -> MemoryResult<Option<MemoryCorpus>>;

    /// 更新用户记忆体（部分更新）
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    /// - `updates`: 需要更新的字段
    ///
    /// # 错误
    /// - `DocumentNotFound`: 用户不存在
    /// - `ConcurrencyConflict`: 并发修改冲突
    async fn update_memory_corpus(
        &self,
        user_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> MemoryResult<()>;

    /// 保存单个交互记录
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    /// - `interaction`: 交互记录
    async fn save_interaction(
        &self,
        user_id: &str,
        interaction: &InteractionLog,
    ) -> MemoryResult<()>;

    /// 语义搜索相关记忆
    ///
    /// # 参数
    /// - `query`: 搜索查询参数
    ///
    /// # 返回
    /// 按相关性排序的记忆片段列表
    async fn search_memories(&self, query: &MemoryQuery) -> MemoryResult<Vec<MemoryFragment>>;

    /// 获取用户的最近交互记录
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    /// - `limit`: 最大返回数量
    async fn get_recent_interactions(
        &self,
        user_id: &str,
        limit: u32,
    ) -> MemoryResult<Vec<InteractionLog>>;

    /// 获取用户统计信息
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    async fn get_user_statistics(&self, user_id: &str) -> MemoryResult<UserStatistics>;

    /// 删除用户的所有数据（GDPR 合规）
    ///
    /// # 参数
    /// - `user_id`: 用户ID
    ///
    /// # 安全注意
    /// 这是一个不可逆操作，调用前需要额外验证
    async fn delete_user_data(&self, user_id: &str) -> MemoryResult<()>;

    /// 健康检查 - 验证存储连接和基本功能
    async fn health_check(&self) -> MemoryResult<bool>;

    /// 初始化存储（初始化时调用）
    async fn initialize(&self) -> MemoryResult<()>;
}

/// 记忆仓储工厂接口
///
/// 用于创建不同类型的记忆仓储实现
#[async_trait]
pub trait MemoryRepositoryFactory {
    /// 创建内存存储实例
    async fn create_memory_repository(&self) -> MemoryResult<Box<dyn MemoryRepository>>;
}

impl MemoryQuery {
    /// 创建简单的文本搜索查询
    pub fn simple_text_search(query_text: String, user_id: String) -> Self {
        Self {
            query_text,
            user_id: Some(user_id),
            time_range: None,
            memory_types: vec![
                MemoryType::Episodic,
                MemoryType::Semantic,
                MemoryType::ActionState,
                MemoryType::StrategicInferential,
            ],
            limit: Some(10),
            relevance_threshold: Some(0.3),
        }
    }

    /// 创建最近记忆查询
    pub fn recent_memories(user_id: String, days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);

        Self {
            query_text: String::new(),
            user_id: Some(user_id),
            time_range: Some(TimeRange { start, end }),
            memory_types: vec![MemoryType::Episodic],
            limit: Some(20),
            relevance_threshold: None,
        }
    }
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            query_text: String::new(),
            user_id: None,
            time_range: None,
            memory_types: Vec::new(),
            limit: Some(10),
            relevance_threshold: Some(0.5),
        }
    }
}
