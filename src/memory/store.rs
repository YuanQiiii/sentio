use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

static MEMORY_STORE: tokio::sync::OnceCell<Arc<MemoryStore>> = tokio::sync::OnceCell::const_new();

#[derive(Debug)]
pub struct MemoryStore {
    data: Arc<RwLock<MemoryData>>,
    file_path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MemoryData {
    memories: HashMap<String, Vec<Memory>>,
    interactions: HashMap<String, Vec<InteractionLog>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub user_id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Event,
    Knowledge,
    Task,
    Relationship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionLog {
    pub id: Option<String>,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    UserToSystem,
    SystemToUser,
}

impl MemoryStore {
    pub async fn initialize(file_path: PathBuf) -> Result<()> {
        let store = Arc::new(Self::new(file_path).await?);
        MEMORY_STORE.set(store)
            .map_err(|_| anyhow::anyhow!("Memory store already initialized"))?;
        Ok(())
    }

    pub fn get() -> &'static Arc<MemoryStore> {
        MEMORY_STORE.get().expect("Memory store not initialized")
    }

    async fn new(file_path: PathBuf) -> Result<Self> {
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path).await?;
            serde_json::from_str(&content)?
        } else {
            MemoryData::default()
        };

        Ok(Self {
            data: Arc::new(RwLock::new(data)),
            file_path,
        })
    }

    async fn save(&self) -> Result<()> {
        let data = self.data.read().await;
        let json = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.file_path, json).await?;
        Ok(())
    }

    pub async fn add_memory(&self, user_id: &str, memory_type: MemoryType, content: String) -> Result<String> {
        let memory = Memory {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            memory_type,
            content,
            created_at: Utc::now(),
        };

        let mut data = self.data.write().await;
        data.memories
            .entry(user_id.to_string())
            .or_default()
            .push(memory.clone());
        drop(data);

        self.save().await?;
        Ok(memory.id)
    }

    #[allow(dead_code)]
    pub async fn get_user_memories(&self, user_id: &str) -> Result<Vec<Memory>> {
        let data = self.data.read().await;
        Ok(data.memories
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn log_interaction(interaction: &InteractionLog) -> Result<String> {
        let store = Self::get();
        let id = interaction.id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let mut log = interaction.clone();
        log.id = Some(id.clone());

        let mut data = store.data.write().await;
        data.interactions
            .entry(interaction.user_id.clone())
            .or_default()
            .push(log);
        drop(data);

        store.save().await?;
        Ok(id)
    }

    #[allow(dead_code)]
    pub async fn get_user_interactions(user_id: &str, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<InteractionLog>> {
        let store = Self::get();
        let data = store.data.read().await;
        let interactions = data.interactions
            .get(user_id)
            .cloned()
            .unwrap_or_default();
        
        let start = offset.unwrap_or(0);
        let end = if let Some(limit) = limit {
            (start + limit).min(interactions.len())
        } else {
            interactions.len()
        };

        Ok(interactions[start..end].to_vec())
    }

    #[allow(dead_code)]
    pub async fn get_user_statistics(user_id: &str) -> Result<UserStats> {
        let store = Self::get();
        let data = store.data.read().await;
        
        let total_memories = data.memories
            .get(user_id)
            .map(|m| m.len())
            .unwrap_or(0);
            
        let total_interactions = data.interactions
            .get(user_id)
            .map(|i| i.len())
            .unwrap_or(0);

        Ok(UserStats {
            total_memories,
            total_interactions,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub total_memories: usize,
    pub total_interactions: usize,
}