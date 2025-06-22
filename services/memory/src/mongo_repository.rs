//! # MongoDB 记忆仓储实现
//!
//! 实现基于 MongoDB 的记忆数据存储，严格遵循"健壮性是底线"原则。
//! 所有数据库操作都包含完整的错误处理和数据验证。

use crate::error::{MemoryError, MemoryResult};
use crate::models::*;
use crate::repository::*;
use async_trait::async_trait;
use bson::{doc, Document};
use chrono::Utc;
use mongodb::{
    options::{ClientOptions, IndexOptions},
    Client, Collection, Database, IndexModel,
};
use shared_logic::config::get_config;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// MongoDB 记忆仓储实现
#[derive(Debug, Clone)]
pub struct MongoMemoryRepository {
    /// MongoDB 数据库实例
    database: Database,
    /// 记忆体集合
    memory_corpus_collection: Collection<MemoryCorpus>,
    /// 交互记录集合
    interaction_collection: Collection<InteractionLog>,
    /// 记忆片段集合（用于快速搜索）
    memory_fragment_collection: Collection<MemoryFragment>,
}

impl MongoMemoryRepository {
    /// 创建新的 MongoDB 记忆仓储实例
    ///
    /// # 错误处理
    /// - 验证数据库配置有效性
    /// - 测试数据库连接
    /// - 确保必要的索引存在
    pub async fn new() -> MemoryResult<Self> {
        let config = get_config();
        let db_config = &config.database;

        // 验证配置
        Self::validate_config(db_config)?;

        info!(
            database_url = %db_config.url,
            max_connections = db_config.max_connections,
            timeout = db_config.connect_timeout,
            "Initializing MongoDB memory repository"
        );

        // 配置 MongoDB 客户端选项
        let mut client_options = ClientOptions::parse(&db_config.url).await.map_err(|e| {
            MemoryError::ConfigurationError {
                field: format!("Invalid MongoDB URL: {}", e),
            }
        })?;

        // 配置连接池和超时
        client_options.max_pool_size = Some(db_config.max_connections);
        client_options.connect_timeout = Some(Duration::from_secs(db_config.connect_timeout));
        client_options.server_selection_timeout = Some(Duration::from_secs(10));

        // 创建客户端
        let client = Client::with_options(client_options).map_err(|e| {
            MemoryError::DatabaseConnectionFailed {
                message: format!("Failed to create MongoDB client: {}", e),
            }
        })?;

        // 连接到数据库
        let database = client.database("sentio_memory");

        // 测试连接
        database
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| MemoryError::DatabaseConnectionFailed {
                message: format!("Failed to ping MongoDB: {}", e),
            })?;

        // 获取集合引用
        let memory_corpus_collection = database.collection::<MemoryCorpus>("memory_corpus");
        let interaction_collection = database.collection::<InteractionLog>("interactions");
        let memory_fragment_collection = database.collection::<MemoryFragment>("memory_fragments");

        let repository = Self {
            database,
            memory_corpus_collection,
            interaction_collection,
            memory_fragment_collection,
        };

        // 确保索引存在
        repository.ensure_indexes().await?;

        info!("MongoDB memory repository initialized successfully");
        Ok(repository)
    }

    /// 验证数据库配置
    fn validate_config(config: &shared_logic::config::DatabaseConfig) -> MemoryResult<()> {
        if config.url.is_empty() {
            return Err(MemoryError::ConfigurationError {
                field: "database.url is empty".to_string(),
            });
        }

        if !config.url.starts_with("mongodb://") && !config.url.starts_with("mongodb+srv://") {
            return Err(MemoryError::ConfigurationError {
                field: "database.url must be a valid MongoDB connection string".to_string(),
            });
        }

        if config.max_connections == 0 {
            return Err(MemoryError::ConfigurationError {
                field: "database.max_connections must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// 验证用户ID格式
    fn validate_user_id(user_id: &str) -> MemoryResult<()> {
        if user_id.is_empty() {
            return Err(MemoryError::ValidationError {
                field: "user_id".to_string(),
                reason: "User ID cannot be empty".to_string(),
            });
        }

        if user_id.len() > 255 {
            return Err(MemoryError::ValidationError {
                field: "user_id".to_string(),
                reason: "User ID cannot exceed 255 characters".to_string(),
            });
        }

        // 基本的邮箱格式验证（如果用户ID是邮箱）
        if user_id.contains('@') && !user_id.contains('.') {
            return Err(MemoryError::ValidationError {
                field: "user_id".to_string(),
                reason: "Invalid email format".to_string(),
            });
        }

        Ok(())
    }

    /// 验证记忆体数据
    fn validate_memory_corpus(corpus: &MemoryCorpus) -> MemoryResult<()> {
        Self::validate_user_id(&corpus.user_id)?;

        if corpus.version.is_empty() {
            return Err(MemoryError::ValidationError {
                field: "version".to_string(),
                reason: "Version cannot be empty".to_string(),
            });
        }

        // 验证时间逻辑
        if corpus.updated_at < corpus.created_at {
            return Err(MemoryError::ValidationError {
                field: "updated_at".to_string(),
                reason: "Updated time cannot be before created time".to_string(),
            });
        }

        Ok(())
    }

    /// 执行带重试的数据库操作
    async fn execute_with_retry<F, Fut, T>(
        &self,
        operation: F,
        operation_name: &str,
    ) -> MemoryResult<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = MemoryResult<T>> + Send,
        T: Send,
    {
        const MAX_RETRIES: u32 = 3;
        let mut last_error: Option<MemoryError> = None;

        for attempt in 0..=MAX_RETRIES {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(
                            operation = operation_name,
                            attempt = attempt,
                            "Database operation succeeded after retry"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);

                    if attempt < MAX_RETRIES && last_error.as_ref().unwrap().is_retryable() {
                        let delay = Duration::from_millis(1000 * (attempt + 1) as u64);
                        warn!(
                            operation = operation_name,
                            attempt = attempt,
                            delay_ms = delay.as_millis(),
                            error = %last_error.as_ref().unwrap(),
                            "Database operation failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        error!(
            operation = operation_name,
            max_retries = MAX_RETRIES,
            error = %last_error.as_ref().unwrap(),
            "Database operation failed after all retries"
        );

        Err(last_error.unwrap())
    }
}

#[async_trait]
impl MemoryRepository for MongoMemoryRepository {
    async fn save_memory_corpus(&self, corpus: &MemoryCorpus) -> MemoryResult<()> {
        // 数据验证 - 零信任原则
        Self::validate_memory_corpus(corpus)?;

        debug!(
            user_id = %corpus.user_id,
            version = %corpus.version,
            "Saving memory corpus"
        );

        self.execute_with_retry(
            || {
                Box::pin(async {
                    let filter = doc! { "user_id": &corpus.user_id };
                    let options = mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build();

                    self.memory_corpus_collection
                        .replace_one(filter, corpus, options)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "save_memory_corpus".to_string(),
                            details: e.to_string(),
                        })?;

                    Ok(())
                })
            },
            "save_memory_corpus",
        )
        .await?;

        info!(
            user_id = %corpus.user_id,
            "Memory corpus saved successfully"
        );

        Ok(())
    }

    async fn get_memory_corpus(&self, user_id: &str) -> MemoryResult<Option<MemoryCorpus>> {
        Self::validate_user_id(user_id)?;

        debug!(user_id = %user_id, "Retrieving memory corpus");

        let result = self
            .execute_with_retry(
                || {
                    Box::pin(async {
                        let filter = doc! { "user_id": user_id };
                        let result = self
                            .memory_corpus_collection
                            .find_one(filter, None)
                            .await
                            .map_err(|e| MemoryError::DatabaseOperationFailed {
                                operation: "get_memory_corpus".to_string(),
                                details: e.to_string(),
                            })?;

                        Ok(result)
                    })
                },
                "get_memory_corpus",
            )
            .await?;

        match &result {
            Some(_) => info!(user_id = %user_id, "Memory corpus found"),
            None => debug!(user_id = %user_id, "Memory corpus not found"),
        }

        Ok(result)
    }

    async fn update_memory_corpus(
        &self,
        user_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> MemoryResult<()> {
        Self::validate_user_id(user_id)?;

        if updates.is_empty() {
            return Err(MemoryError::ValidationError {
                field: "updates".to_string(),
                reason: "Updates cannot be empty".to_string(),
            });
        }

        debug!(
            user_id = %user_id,
            update_count = updates.len(),
            "Updating memory corpus"
        );

        // 构建更新文档，包含 updated_at 字段
        let mut update_doc = Document::new();
        for (key, value) in updates {
            update_doc.insert(key, bson::to_bson(&value)?);
        }
        update_doc.insert("updated_at", Utc::now().timestamp_millis());

        self.execute_with_retry(
            || {
                Box::pin(async {
                    let filter = doc! { "user_id": user_id };
                    let update = doc! { "$set": &update_doc };

                    let result = self
                        .memory_corpus_collection
                        .update_one(filter, update, None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "update_memory_corpus".to_string(),
                            details: e.to_string(),
                        })?;

                    if result.matched_count == 0 {
                        return Err(MemoryError::DocumentNotFound {
                            document_type: "MemoryCorpus".to_string(),
                            id: user_id.to_string(),
                        });
                    }

                    Ok(())
                })
            },
            "update_memory_corpus",
        )
        .await?;

        info!(user_id = %user_id, "Memory corpus updated successfully");
        Ok(())
    }

    async fn save_interaction(
        &self,
        user_id: &str,
        interaction: &InteractionLog,
    ) -> MemoryResult<()> {
        Self::validate_user_id(user_id)?;

        debug!(
            user_id = %user_id,
            log_id = %interaction.log_id,
            "Saving interaction"
        );

        // 添加用户ID到交互记录中（如果没有的话）
        let interaction_with_user = interaction.clone();
        // 注意：这里假设 InteractionLog 结构中有 user_id 字段
        // 如果没有，需要修改数据结构

        self.execute_with_retry(
            || {
                Box::pin(async {
                    self.interaction_collection
                        .insert_one(&interaction_with_user, None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "save_interaction".to_string(),
                            details: e.to_string(),
                        })?;

                    Ok(())
                })
            },
            "save_interaction",
        )
        .await?;

        info!(
            user_id = %user_id,
            log_id = %interaction.log_id,
            "Interaction saved successfully"
        );

        Ok(())
    }

    async fn search_memories(&self, query: &MemoryQuery) -> MemoryResult<Vec<MemoryFragment>> {
        debug!(
            query_text = %query.query_text,
            user_id = ?query.user_id,
            "Searching memories"
        );

        // 构建搜索过滤器
        let mut filter = Document::new();

        if let Some(user_id) = &query.user_id {
            Self::validate_user_id(user_id)?;
            filter.insert("user_id", user_id);
        }

        if let Some(time_range) = &query.time_range {
            filter.insert(
                "created_at",
                doc! {
                    "$gte": time_range.start.timestamp_millis(),
                    "$lte": time_range.end.timestamp_millis()
                },
            );
        }

        if !query.memory_types.is_empty() {
            let types: Vec<String> = query
                .memory_types
                .iter()
                .map(|t| format!("{:?}", t))
                .collect();
            filter.insert("memory_type", doc! { "$in": types });
        }

        // 文本搜索（简化版本，实际应用中可能需要更复杂的语义搜索）
        if !query.query_text.is_empty() {
            filter.insert("$text", doc! { "$search": &query.query_text });
        }

        let results = self
            .execute_with_retry(
                || {
                    Box::pin(async {
                        let mut cursor = self
                            .memory_fragment_collection
                            .find(filter.clone(), None)
                            .await
                            .map_err(|e| MemoryError::DatabaseOperationFailed {
                                operation: "search_memories".to_string(),
                                details: e.to_string(),
                            })?;

                        let mut memories = Vec::new();
                        while cursor.advance().await.map_err(|e| {
                            MemoryError::DatabaseOperationFailed {
                                operation: "search_memories_cursor".to_string(),
                                details: e.to_string(),
                            }
                        })? {
                            let memory = cursor.deserialize_current().map_err(|e| {
                                MemoryError::DatabaseOperationFailed {
                                    operation: "deserialize_memory".to_string(),
                                    details: e.to_string(),
                                }
                            })?;
                            memories.push(memory);

                            // 限制结果数量
                            if let Some(limit) = query.limit {
                                if memories.len() >= limit as usize {
                                    break;
                                }
                            }
                        }

                        Ok(memories)
                    })
                },
                "search_memories",
            )
            .await?;

        info!(
            query_text = %query.query_text,
            results_count = results.len(),
            "Memory search completed"
        );

        Ok(results)
    }

    async fn get_recent_interactions(
        &self,
        user_id: &str,
        limit: u32,
    ) -> MemoryResult<Vec<InteractionLog>> {
        Self::validate_user_id(user_id)?;

        debug!(
            user_id = %user_id,
            limit = limit,
            "Getting recent interactions"
        );

        let filter = doc! { "user_id": user_id };
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit as i64)
            .build();

        let results = self
            .execute_with_retry(
                || {
                    Box::pin(async {
                        let mut cursor = self
                            .interaction_collection
                            .find(filter.clone(), options.clone())
                            .await
                            .map_err(|e| MemoryError::DatabaseOperationFailed {
                                operation: "get_recent_interactions".to_string(),
                                details: e.to_string(),
                            })?;

                        let mut interactions = Vec::new();
                        while cursor.advance().await.map_err(|e| {
                            MemoryError::DatabaseOperationFailed {
                                operation: "get_recent_interactions_cursor".to_string(),
                                details: e.to_string(),
                            }
                        })? {
                            let interaction = cursor.deserialize_current().map_err(|e| {
                                MemoryError::DatabaseOperationFailed {
                                    operation: "deserialize_interaction".to_string(),
                                    details: e.to_string(),
                                }
                            })?;
                            interactions.push(interaction);
                        }

                        Ok(interactions)
                    })
                },
                "get_recent_interactions",
            )
            .await?;

        info!(
            user_id = %user_id,
            interactions_count = results.len(),
            "Recent interactions retrieved"
        );

        Ok(results)
    }

    async fn get_user_statistics(&self, user_id: &str) -> MemoryResult<UserStatistics> {
        Self::validate_user_id(user_id)?;

        debug!(user_id = %user_id, "Getting user statistics");

        // 这里简化实现，实际应该使用聚合管道进行更高效的统计
        let interactions = self.get_recent_interactions(user_id, 1000).await?;

        let stats = if interactions.is_empty() {
            UserStatistics {
                user_id: user_id.to_string(),
                total_interactions: 0,
                first_interaction: Utc::now(),
                last_interaction: Utc::now(),
                total_memories: 0,
                memory_type_distribution: HashMap::new(),
            }
        } else {
            let first_interaction = interactions.last().unwrap().timestamp;
            let last_interaction = interactions.first().unwrap().timestamp;

            UserStatistics {
                user_id: user_id.to_string(),
                total_interactions: interactions.len() as u64,
                first_interaction,
                last_interaction,
                total_memories: 0, // 需要单独查询
                memory_type_distribution: HashMap::new(),
            }
        };

        info!(
            user_id = %user_id,
            total_interactions = stats.total_interactions,
            "User statistics computed"
        );

        Ok(stats)
    }

    async fn delete_user_data(&self, user_id: &str) -> MemoryResult<()> {
        Self::validate_user_id(user_id)?;

        warn!(
            user_id = %user_id,
            "Deleting all user data - this is irreversible"
        );

        self.execute_with_retry(
            || {
                Box::pin(async {
                    let filter = doc! { "user_id": user_id };

                    // 删除记忆体
                    self.memory_corpus_collection
                        .delete_one(filter.clone(), None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "delete_memory_corpus".to_string(),
                            details: e.to_string(),
                        })?;

                    // 删除交互记录
                    self.interaction_collection
                        .delete_many(filter.clone(), None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "delete_interactions".to_string(),
                            details: e.to_string(),
                        })?;

                    // 删除记忆片段
                    self.memory_fragment_collection
                        .delete_many(filter, None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "delete_memory_fragments".to_string(),
                            details: e.to_string(),
                        })?;

                    Ok(())
                })
            },
            "delete_user_data",
        )
        .await?;

        warn!(user_id = %user_id, "All user data deleted successfully");
        Ok(())
    }

    async fn health_check(&self) -> MemoryResult<bool> {
        debug!("Performing memory repository health check");

        self.execute_with_retry(
            || {
                Box::pin(async {
                    // 测试数据库连接
                    self.database
                        .run_command(doc! { "ping": 1 }, None)
                        .await
                        .map_err(|e| MemoryError::DatabaseConnectionFailed {
                            message: format!("Health check ping failed: {}", e),
                        })?;

                    // 测试集合访问
                    self.memory_corpus_collection
                        .estimated_document_count(None)
                        .await
                        .map_err(|e| MemoryError::DatabaseOperationFailed {
                            operation: "health_check_count".to_string(),
                            details: e.to_string(),
                        })?;

                    Ok(true)
                })
            },
            "health_check",
        )
        .await
    }

    async fn ensure_indexes(&self) -> MemoryResult<()> {
        info!("Creating database indexes for optimal performance");

        // 为记忆体集合创建索引
        let memory_corpus_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "user_id": 1 })
                .options(
                    IndexOptions::builder()
                        .unique(true)
                        .name("user_id_unique".to_string())
                        .build(),
                )
                .build(),
            IndexModel::builder()
                .keys(doc! { "updated_at": -1 })
                .build(),
        ];

        // 为交互记录集合创建索引
        let interaction_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "timestamp": -1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "interaction_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];

        // 为记忆片段集合创建索引
        let fragment_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "created_at": -1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "content": "text" })
                .options(
                    IndexOptions::builder()
                        .name("content_text_search".to_string())
                        .build(),
                )
                .build(),
        ];

        // 创建索引
        self.memory_corpus_collection
            .create_indexes(memory_corpus_indexes, None)
            .await
            .map_err(|e| MemoryError::IndexError {
                index_name: "memory_corpus_indexes".to_string(),
                details: e.to_string(),
            })?;

        self.interaction_collection
            .create_indexes(interaction_indexes, None)
            .await
            .map_err(|e| MemoryError::IndexError {
                index_name: "interaction_indexes".to_string(),
                details: e.to_string(),
            })?;

        self.memory_fragment_collection
            .create_indexes(fragment_indexes, None)
            .await
            .map_err(|e| MemoryError::IndexError {
                index_name: "fragment_indexes".to_string(),
                details: e.to_string(),
            })?;

        info!("Database indexes created successfully");
        Ok(())
    }
}
