//! # 记忆数据访问层
//!
//! 提供统一的记忆数据 CRUD 操作接口，所有记忆相关的数据访问都通过此模块进行。
//! 遵循"健壮性是底线"和"零信任"原则。

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// In-memory database for development/testing
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref IN_MEMORY_DB: Arc<RwLock<InMemoryDb>> = Arc::new(RwLock::new(InMemoryDb::default()));
}

#[derive(Default)]
struct InMemoryDb {
    memory_corpus: Vec<MemoryCorpus>,
    memory_fragments: Vec<MemoryFragment>,
    interaction_logs: Vec<InteractionLog>,
    metrics: DbMetrics,
}

#[derive(Default)]
struct DbMetrics {
    reads: u64,
    writes: u64,
    query_hits: u64,
    query_misses: u64,
}

fn get_db() -> Arc<RwLock<InMemoryDb>> {
    IN_MEMORY_DB.clone()
}

/// 记忆类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// 情景记忆 - 具体的经历和事件
    Episodic,
    /// 语义记忆 - 事实、概念和知识
    Semantic,
    /// 程序性记忆 - 技能和习惯
    Procedural,
    /// 战略推理记忆 - 长期目标和计划
    Strategic,
    /// 行动状态记忆 - 当前任务和状态
    ActionState,
}

/// 记忆片段 - 用于快速搜索和检索
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub keywords: Vec<String>,
    pub importance_score: f64,
    pub created_at: DateTime<Utc>,
    pub source_id: Option<Uuid>, // 指向原始记忆体的 ID
}

/// 记忆体 - 完整的用户记忆数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCorpus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    pub core_profile: CoreProfile,
    pub episodic_memory: Vec<EpisodicMemory>,
    pub semantic_memory: Vec<SemanticMemory>,
    pub procedural_memory: SkillExpertise,
    pub strategic_memory: StrategicInferentialMemory,
    pub action_state_memory: ActionStateMemory,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 交互记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub direction: MessageDirection,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

// 重新导出 memory 服务中的类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreProfile {
    pub name: Option<String>,
    pub age_range: Option<String>,
    pub location: Option<String>,
    pub occupation: Option<String>,
    pub interests: Vec<String>,
    pub communication_style: CommunicationStrategy,
    pub values_and_beliefs: Vec<String>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub id: String,
    pub event_description: String,
    pub timestamp: DateTime<Utc>,
    pub location: Option<String>,
    pub people_involved: Vec<String>,
    pub emotional_context: String,
    pub significance_level: u8,
    pub related_goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub id: String,
    pub concept: String,
    pub definition: String,
    pub examples: Vec<String>,
    pub related_concepts: Vec<String>,
    pub confidence_level: f64,
    pub source: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExpertise {
    pub programming_languages: Vec<String>,
    pub frameworks_and_tools: Vec<String>,
    pub domain_expertise: Vec<String>,
    pub learning_preferences: Vec<String>,
    pub problem_solving_approach: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInferentialMemory {
    pub user_model_hypotheses: Vec<UserModelHypothesis>,
    pub relationship_dynamics: Vec<Relationship>,
    pub long_term_plans: Vec<Plan>,
    pub self_reflection: Vec<SelfReflectionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStateMemory {
    pub current_tasks: Vec<Task>,
    pub pending_follow_ups: Vec<FollowUp>,
    pub active_goals: Vec<String>,
    pub context_switches: Vec<String>,
    pub mood_and_energy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModelHypothesis {
    pub hypothesis: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_type: String,
    pub description: String,
    pub strength: f64,
    pub recent_interactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub milestones: Vec<String>,
    pub timeline: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReflectionEntry {
    pub id: String,
    pub reflection: String,
    pub insights: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUp {
    pub id: String,
    pub content: String,
    pub scheduled_for: DateTime<Utc>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    UserToSystem,
    SystemToUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationStrategy {
    Direct,
    Collaborative,
    Analytical,
    Creative,
    Supportive,
}

/// 记忆查询参数
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub user_id: String,
    pub memory_types: Option<Vec<MemoryType>>,
    pub keywords: Option<Vec<String>>,
    pub min_importance: Option<f64>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 用户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub user_id: String,
    pub total_memories: u64,
    pub memory_type_counts: HashMap<MemoryType, u64>,
    pub total_interactions: u64,
    pub last_interaction: Option<DateTime<Utc>>,
    pub account_created: DateTime<Utc>,
}

/// 记忆数据访问接口
pub struct MemoryDataAccess;

impl MemoryDataAccess {
    /// 创建或更新用户的记忆体
    pub async fn save_memory_corpus(corpus: &MemoryCorpus) -> Result<Uuid> {
        Self::validate_memory_corpus(corpus)?;

        let db = get_db();
        let mut db = db.write().await;

        let corpus_id = corpus.id.unwrap_or_else(Uuid::new_v4);
        let mut new_corpus = corpus.clone();
        new_corpus.id = Some(corpus_id);

        if let Some(existing) = db.memory_corpus.iter_mut().find(|c| c.id == new_corpus.id) {
            *existing = new_corpus;
        } else {
            db.memory_corpus.push(new_corpus);
        }

        Ok(corpus_id)
    }

    /// 根据用户ID获取记忆体
    pub async fn get_memory_corpus_by_user_id(user_id: &str) -> Result<Option<MemoryCorpus>> {
        let db = get_db();
        let mut db = db.write().await;

        db.metrics.reads += 1;

        let result = db
            .memory_corpus
            .iter()
            .find(|c| c.user_id == user_id)
            .cloned();

        if result.is_some() {
            db.metrics.query_hits += 1;
        } else {
            db.metrics.query_misses += 1;
        }

        Ok(result)
    }

    /// 添加记忆片段
    pub async fn add_memory_fragment(fragment: &MemoryFragment) -> Result<Uuid> {
        Self::validate_memory_fragment(fragment)?;

        let db = get_db();
        let mut db = db.write().await;

        db.metrics.writes += 1;

        let fragment_id = fragment.id.unwrap_or_else(Uuid::new_v4);
        let mut new_fragment = fragment.clone();
        new_fragment.id = Some(fragment_id);

        if let Some(existing) = db
            .memory_fragments
            .iter_mut()
            .find(|f| f.id == new_fragment.id)
        {
            *existing = new_fragment;
        } else {
            db.memory_fragments.push(new_fragment);
        }

        Ok(fragment_id)
    }

    /// 搜索记忆片段
    pub async fn search_memory_fragments(query: &MemoryQuery) -> Result<Vec<MemoryFragment>> {
        let db = get_db();
        let mut db = db.write().await;

        db.metrics.reads += 1;

        let results: Vec<_> = db
            .memory_fragments
            .iter()
            .filter(|f| f.user_id == query.user_id)
            .filter(|f| {
                query
                    .memory_types
                    .as_ref()
                    .map_or(true, |types| types.contains(&f.memory_type))
            })
            .filter(|f| {
                query.keywords.as_ref().map_or(true, |kws| {
                    kws.iter()
                        .any(|kw| f.keywords.contains(kw) || f.content.contains(kw))
                })
            })
            .filter(|f| {
                query
                    .min_importance
                    .map_or(true, |min| f.importance_score >= min)
            })
            .filter(|f| {
                query.time_range.as_ref().map_or(true, |tr| {
                    f.created_at >= tr.start && f.created_at <= tr.end
                })
            })
            .cloned()
            .collect();

        if results.is_empty() {
            db.metrics.query_misses += 1;
        } else {
            db.metrics.query_hits += results.len() as u64;
        }

        Ok(results)
    }

    /// 记录交互日志
    pub async fn log_interaction(interaction: &InteractionLog) -> Result<Uuid> {
        Self::validate_interaction_log(interaction)?;

        let db = get_db();
        let mut db = db.write().await;

        db.metrics.writes += 1;

        let interaction_id = interaction.id.unwrap_or_else(Uuid::new_v4);
        let mut new_interaction = interaction.clone();
        new_interaction.id = Some(interaction_id);

        db.interaction_logs.push(new_interaction);

        Ok(interaction_id)
    }

    /// 获取用户交互历史
    pub async fn get_user_interactions(
        user_id: &str,
        limit: Option<i64>,
        session_id: Option<&str>,
    ) -> Result<Vec<InteractionLog>> {
        let db = get_db();
        let mut db = db.write().await;

        db.metrics.reads += 1;

        let mut logs: Vec<_> = db
            .interaction_logs
            .iter()
            .filter(|log| log.user_id == user_id)
            .filter(|log| session_id.map_or(true, |sid| log.session_id == sid))
            .cloned()
            .collect();

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            logs.truncate(limit as usize);
        }

        if logs.is_empty() {
            db.metrics.query_misses += 1;
        } else {
            db.metrics.query_hits += logs.len() as u64;
        }

        Ok(logs)
    }

    /// 获取用户统计信息
    pub async fn get_user_statistics(user_id: &str) -> Result<UserStatistics> {
        let db = get_db();
        let mut db = db.write().await;

        db.metrics.reads += 1;

        let memory_type_counts = db
            .memory_fragments
            .iter()
            .filter(|f| f.user_id == user_id)
            .fold(HashMap::new(), |mut acc, f| {
                *acc.entry(f.memory_type.clone()).or_insert(0) += 1;
                acc
            });

        let last_interaction = db
            .interaction_logs
            .iter()
            .filter(|log| log.user_id == user_id)
            .max_by_key(|log| log.timestamp)
            .map(|log| log.timestamp);

        Ok(UserStatistics {
            user_id: user_id.to_string(),
            total_memories: db
                .memory_fragments
                .iter()
                .filter(|f| f.user_id == user_id)
                .count() as u64,
            memory_type_counts,
            total_interactions: db
                .interaction_logs
                .iter()
                .filter(|log| log.user_id == user_id)
                .count() as u64,
            last_interaction,
            account_created: Utc::now(), // TODO: 需要从用户档案获取
        })
    }

    /// 删除用户的所有数据 (GDPR 合规)
    pub async fn delete_user_data(user_id: &str) -> Result<u64> {
        let db = get_db();
        let mut db = db.write().await;

        db.metrics.writes += 1;

        let before_memories = db.memory_fragments.len();
        let before_logs = db.interaction_logs.len();

        db.memory_fragments.retain(|f| f.user_id != user_id);
        db.interaction_logs.retain(|log| log.user_id != user_id);

        let deleted_memories = before_memories - db.memory_fragments.len();
        let deleted_logs = before_logs - db.interaction_logs.len();

        Ok((deleted_memories + deleted_logs) as u64)
    }

    // 私有验证方法
    fn validate_memory_corpus(corpus: &MemoryCorpus) -> Result<()> {
        if corpus.user_id.is_empty() {
            return Err(anyhow::anyhow!("user_id cannot be empty"));
        }

        Ok(())
    }

    fn validate_memory_fragment(fragment: &MemoryFragment) -> Result<()> {
        if fragment.user_id.is_empty() {
            return Err(anyhow::anyhow!("user_id cannot be empty"));
        }

        if fragment.content.is_empty() {
            return Err(anyhow::anyhow!("content cannot be empty"));
        }

        if fragment.importance_score < 0.0 || fragment.importance_score > 1.0 {
            return Err(anyhow::anyhow!(
                "importance_score must be between 0.0 and 1.0"
            ));
        }

        Ok(())
    }

    fn validate_interaction_log(interaction: &InteractionLog) -> Result<()> {
        if interaction.user_id.is_empty() {
            return Err(anyhow::anyhow!("user_id cannot be empty"));
        }

        if interaction.session_id.is_empty() {
            return Err(anyhow::anyhow!("session_id cannot be empty"));
        }

        if interaction.content.is_empty() {
            return Err(anyhow::anyhow!("content cannot be empty"));
        }

        Ok(())
    }
}
