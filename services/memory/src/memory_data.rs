//! # 内存数据存储实现
//!
//! 提供线程安全的内存数据存储实现，使用 `Arc<RwLock<T>>` 进行同步。

use crate::error::{MemoryError, MemoryResult};
use crate::models::*;
use crate::repository::*;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 内存数据存储实现
#[derive(Debug, Default)]
pub struct MemoryDataRepository {
    /// 用户记忆体存储
    memory_corpus: Arc<RwLock<HashMap<String, MemoryCorpus>>>,
    /// 交互记录存储
    interactions: Arc<RwLock<HashMap<String, Vec<InteractionLog>>>>,
    /// 记忆片段存储
    memory_fragments: Arc<RwLock<HashMap<String, Vec<MemoryFragment>>>>,
}

impl MemoryDataRepository {
    /// 创建新的内存数据存储实例
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MemoryRepository for MemoryDataRepository {
    async fn save_memory_corpus(&self, corpus: &MemoryCorpus) -> MemoryResult<()> {
        let mut store = self.memory_corpus.write().await;
        store.insert(corpus.user_id.clone(), corpus.clone());
        Ok(())
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
                // Simplified update logic - actual implementation would need to handle specific fields
                if key == "core_profile.current_life_summary" {
                    if let Some(summary) = value.as_str() {
                        corpus.core_profile.current_life_summary = Some(summary.to_string());
                    }
                }
            }
            corpus.updated_at = Utc::now();
            Ok(())
        } else {
            Err(MemoryError::DocumentNotFound {
                document_type: "MemoryCorpus".to_string(),
                id: user_id.to_string(),
            })
        }
    }

    async fn save_interaction(&self, user_id: &str, interaction: &InteractionLog) -> MemoryResult<()> {
        let mut store = self.interactions.write().await;
        store.entry(user_id.to_string())
            .or_default()
            .push(interaction.clone());
        Ok(())
    }

    async fn search_memories(&self, query: &MemoryQuery) -> MemoryResult<Vec<MemoryFragment>> {
        let store = self.memory_fragments.read().await;
        let mut results = Vec::new();
        
        if let Some(user_id) = &query.user_id {
            if let Some(fragments) = store.get(user_id) {
                results.extend(fragments.iter().filter(|f| {
                    f.content.contains(&query.query_text)
                }).cloned());
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
        let _corpus_store = self.memory_corpus.read().await;
        let interaction_store = self.interactions.read().await;
        let fragment_store = self.memory_fragments.read().await;
        
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
        
        Ok(())
    }

    async fn health_check(&self) -> MemoryResult<bool> {
        Ok(true)
    }

    async fn initialize(&self) -> MemoryResult<()> {
        Ok(())
    }
}