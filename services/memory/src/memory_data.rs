//! # 内存数据存储实现
//!
//! 提供线程安全的内存数据存储实现，使用 `Arc<RwLock<T>>` 进行同步。

use crate::error::{MemoryError, MemoryResult};
use crate::models::{InteractionLog, MemoryCorpus};
use crate::repository::{MemoryFragment, MemoryQuery, MemoryRepository, UserStatistics};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{error, info};

/// 内存数据存储实现
#[derive(Debug)]
pub struct MemoryDataRepository {
    /// 用户记忆体存储
    memory_corpus: Arc<RwLock<HashMap<String, MemoryCorpus>>>,
    /// 交互记录存储
    interactions: Arc<RwLock<HashMap<String, Vec<InteractionLog>>>>,
    /// 记忆片段存储
    memory_fragments: Arc<RwLock<HashMap<String, Vec<MemoryFragment>>>>,
    /// 持久化文件路径
    file_path: PathBuf,
}

/// 用于序列化/反序列化的数据结构
#[derive(Debug, Serialize, Deserialize, Default)]
struct PersistentData {
    memory_corpus: HashMap<String, MemoryCorpus>,
    interactions: HashMap<String, Vec<InteractionLog>>,
    memory_fragments: HashMap<String, Vec<MemoryFragment>>,
}

impl MemoryDataRepository {
    /// 创建新的内存数据存储实例
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            memory_corpus: Arc::new(RwLock::new(HashMap::new())),
            interactions: Arc::new(RwLock::new(HashMap::new())),
            memory_fragments: Arc::new(RwLock::new(HashMap::new())),
            file_path,
        }
    }

    /// 从文件加载数据
    async fn load_from_file(&self) -> MemoryResult<()> {
        if !self.file_path.exists() {
            info!("Persistence file not found: {:?}. Starting with empty data.", self.file_path);
            return Ok(());
        }

        let data = fs::read_to_string(&self.file_path).await.map_err(|e| {
            error!("Failed to read persistence file {:?}: {}", self.file_path, e);
            MemoryError::DatabaseOperationFailed { operation: "read_persistence_file".to_string(), details: e.to_string() }
        })?;

        let persistent_data: PersistentData = serde_json::from_str(&data).map_err(|e| {
            error!("Failed to parse persistence data from {:?}: {}", self.file_path, e);
            MemoryError::DatabaseOperationFailed { operation: "parse_persistence_data".to_string(), details: e.to_string() }
        })?;

        *self.memory_corpus.write().await = persistent_data.memory_corpus;
        *self.interactions.write().await = persistent_data.interactions;
        *self.memory_fragments.write().await = persistent_data.memory_fragments;

        info!("Data loaded from persistence file: {:?}", self.file_path);
        Ok(())
    }

    /// 将数据保存到文件
    async fn save_to_file(&self) -> MemoryResult<()> {
        let persistent_data = PersistentData {
            memory_corpus: self.memory_corpus.read().await.clone(),
            interactions: self.interactions.read().await.clone(),
            memory_fragments: self.memory_fragments.read().await.clone(),
        };

        let data = serde_json::to_string_pretty(&persistent_data).map_err(|e| {
            error!("Failed to serialize data for persistence: {}", e);
            MemoryError::DatabaseOperationFailed { operation: "serialize_data".to_string(), details: e.to_string() }
        })?;

        fs::write(&self.file_path, data).await.map_err(|e| {
            error!("Failed to write data to persistence file {:?}: {}", self.file_path, e);
            MemoryError::DatabaseOperationFailed { operation: "write_persistence_file".to_string(), details: e.to_string() }
        })?;

        info!("Data saved to persistence file: {:?}", self.file_path);
        Ok(())
    }
}

#[async_trait]
impl MemoryRepository for MemoryDataRepository {
    async fn save_memory_corpus(&self, corpus: &MemoryCorpus) -> MemoryResult<()> {
        let mut store = self.memory_corpus.write().await;
        store.insert(corpus.user_id.clone(), corpus.clone());
        self.save_to_file().await
    }

    async fn get_memory_corpus(&self, user_id: &str) -> MemoryResult<Option<MemoryCorpus>> {
        let store = self.memory_corpus.read().await;
        Ok(store.get(user_id).cloned())
    }

    async fn update_memory_corpus(
        &self,
        user_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> MemoryResult<()> {
        let mut store = self.memory_corpus.write().await;
        if let Some(corpus) = store.get_mut(user_id) {
            for (key, value) in updates {
                match key.as_str() {
                    "core_profile.name" => {
                        if let Some(name) = value.as_str() {
                            corpus.core_profile.name = Some(name.to_string());
                        }
                    }
                    "core_profile.age" => {
                        if let Some(age) = value.as_u64() {
                            corpus.core_profile.age = Some(age as u32);
                        }
                    }
                    "core_profile.city" => {
                        if let Some(city) = value.as_str() {
                            corpus.core_profile.city = Some(city.to_string());
                        }
                    }
                    "core_profile.occupation" => {
                        if let Some(occupation) = value.as_str() {
                            corpus.core_profile.occupation = Some(occupation.to_string());
                        }
                    }
                    "core_profile.current_life_summary" => {
                        if let Some(summary) = value.as_str() {
                            corpus.core_profile.current_life_summary = Some(summary.to_string());
                        }
                    }
                    // Add more fields here as needed
                    _ => {
                        // Optionally log or handle unknown keys
                    }
                }
            }
            corpus.updated_at = Utc::now();
            self.save_to_file().await
        } else {
            Err(MemoryError::DocumentNotFound {
                document_type: "MemoryCorpus".to_string(),
                id: user_id.to_string(),
            })
        }
    }

    async fn save_interaction(
        &self,
        user_id: &str,
        interaction: &InteractionLog,
    ) -> MemoryResult<()> {
        let mut store = self.interactions.write().await;
        store
            .entry(user_id.to_string())
            .or_default()
            .push(interaction.clone());
        self.save_to_file().await
    }

    async fn search_memories(&self, query: &MemoryQuery) -> MemoryResult<Vec<MemoryFragment>> {
        let store = self.memory_fragments.read().await;
        let mut results = Vec::new();

        if let Some(user_id) = &query.user_id {
            if let Some(fragments) = store.get(user_id) {
                let lower_query_text = query.query_text.to_lowercase();
                let keywords: Vec<&str> = lower_query_text.split_whitespace().collect();

                results.extend(
                    fragments
                        .iter()
                        .filter(|f| {
                            let lower_content = f.content.to_lowercase();
                            keywords.iter().all(|&keyword| lower_content.contains(keyword))
                        })
                        .cloned(),
                );
            }
        }

        Ok(results)
    }

    async fn get_recent_interactions(
        &self,
        user_id: &str,
        limit: u32,
    ) -> MemoryResult<Vec<InteractionLog>> {
        let store = self.interactions.read().await;
        if let Some(interactions) = store.get(user_id) {
            Ok(interactions.iter().take(limit as usize).cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_user_statistics(&self, user_id: &str) -> MemoryResult<UserStatistics> {
        let corpus_store = self.memory_corpus.read().await;
        let interaction_store = self.interactions.read().await;
        let fragment_store = self.memory_fragments.read().await;

        let account_created = corpus_store
            .get(user_id)
            .map(|c| c.created_at)
            .unwrap_or_else(Utc::now);

        let total_interactions = interaction_store.get(user_id).map_or(0, |v| v.len()) as u64;
        let total_memories = fragment_store.get(user_id).map_or(0, |v| v.len()) as u64;

        let first_interaction = interaction_store
            .get(user_id)
            .and_then(|v| v.last())
            .map(|i| i.timestamp)
            .unwrap_or_else(Utc::now);

        let last_interaction = interaction_store
            .get(user_id)
            .and_then(|v| v.first())
            .map(|i| i.timestamp)
            .unwrap_or_else(Utc::now);

        Ok(UserStatistics {
            user_id: user_id.to_string(),
            account_created,
            total_interactions,
            first_interaction,
            last_interaction,
            total_memories,
            memory_type_distribution: HashMap::new(),
        })
    }

    async fn delete_user_data(&self, user_id: &str) -> MemoryResult<()> {
        let mut corpus_store = self.memory_corpus.write().await;
        let mut interaction_store = self.interactions.write().await;
        let mut fragment_store = self.memory_fragments.write().await;

        corpus_store.remove(user_id);
        interaction_store.remove(user_id);
        fragment_store.remove(user_id);

        self.save_to_file().await
    }

    async fn health_check(&self) -> MemoryResult<bool> {
        Ok(true)
    }

    async fn initialize(&self) -> MemoryResult<()> {
        self.load_from_file().await
    }
}
