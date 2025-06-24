//! # 记忆数据访问层
//!
//! 提供统一的记忆数据 CRUD 操作接口，所有记忆相关的数据访问都通过此模块进行。
//! 遵循"健壮性是底线"和"零信任"原则。

use crate::database::{get_collection, DatabaseError, DatabaseResult};
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use mongodb::{
    options::{FindOptions, ReplaceOptions},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

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
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub keywords: Vec<String>,
    pub importance_score: f64,
    pub created_at: DateTime<Utc>,
    pub source_id: Option<ObjectId>, // 指向原始记忆体的 ID
}

/// 记忆体 - 完整的用户记忆数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCorpus {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
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
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
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
    pub async fn save_memory_corpus(corpus: &MemoryCorpus) -> DatabaseResult<ObjectId> {
        let collection: Collection<MemoryCorpus> = get_collection("memory_corpus");

        // 验证数据
        Self::validate_memory_corpus(corpus)?;

        let mut corpus_to_save = corpus.clone();
        corpus_to_save.updated_at = Utc::now();

        match &corpus.id {
            Some(id) => {
                // 更新现有记忆体
                let options = ReplaceOptions::builder().upsert(false).build();
                let result = collection
                    .replace_one(doc! { "_id": id }, &corpus_to_save, options)
                    .await
                    .map_err(|e| DatabaseError::OperationFailed {
                        message: format!("Failed to update memory corpus: {}", e),
                    })?;

                if result.matched_count == 0 {
                    return Err(DatabaseError::NotFound {
                        resource: format!("Memory corpus with id: {}", id),
                    });
                }

                Ok(*id)
            }
            None => {
                // 创建新记忆体
                corpus_to_save.created_at = Utc::now();
                let result = collection
                    .insert_one(&corpus_to_save, None)
                    .await
                    .map_err(|e| DatabaseError::OperationFailed {
                        message: format!("Failed to insert memory corpus: {}", e),
                    })?;

                Ok(result.inserted_id.as_object_id().unwrap())
            }
        }
    }

    /// 根据用户ID获取记忆体
    pub async fn get_memory_corpus_by_user_id(
        user_id: &str,
    ) -> DatabaseResult<Option<MemoryCorpus>> {
        let collection: Collection<MemoryCorpus> = get_collection("memory_corpus");

        let result = collection
            .find_one(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to find memory corpus: {}", e),
            })?;

        Ok(result)
    }

    /// 添加记忆片段
    pub async fn add_memory_fragment(fragment: &MemoryFragment) -> DatabaseResult<ObjectId> {
        let collection: Collection<MemoryFragment> = get_collection("memory_fragments");

        // 验证数据
        Self::validate_memory_fragment(fragment)?;

        let mut fragment_to_save = fragment.clone();
        fragment_to_save.created_at = Utc::now();

        let result = collection
            .insert_one(&fragment_to_save, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to insert memory fragment: {}", e),
            })?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// 搜索记忆片段
    pub async fn search_memory_fragments(
        query: &MemoryQuery,
    ) -> DatabaseResult<Vec<MemoryFragment>> {
        let collection: Collection<MemoryFragment> = get_collection("memory_fragments");

        // 构建查询文档
        let mut filter = doc! { "user_id": &query.user_id };

        if let Some(ref memory_types) = query.memory_types {
            let type_names: Vec<String> = memory_types
                .iter()
                .map(|t| {
                    serde_json::to_string(t)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
                .collect();
            filter.insert("memory_type", doc! { "$in": type_names });
        }

        if let Some(ref keywords) = query.keywords {
            filter.insert("keywords", doc! { "$in": keywords });
        }

        if let Some(min_importance) = query.min_importance {
            filter.insert("importance_score", doc! { "$gte": min_importance });
        }

        if let Some(ref time_range) = query.time_range {
            filter.insert(
                "created_at",
                doc! {
                    "$gte": time_range.start,
                    "$lte": time_range.end
                },
            );
        }

        // 构建查询选项
        let options = FindOptions::builder()
            .sort(doc! { "importance_score": -1, "created_at": -1 })
            .limit(query.limit.unwrap_or(100))
            .build();

        let mut results = Vec::new();
        use futures::TryStreamExt;

        let mut cursor =
            collection
                .find(filter, options)
                .await
                .map_err(|e| DatabaseError::OperationFailed {
                    message: format!("Failed to search memory fragments: {}", e),
                })?;

        while let Some(doc) =
            cursor
                .try_next()
                .await
                .map_err(|e| DatabaseError::OperationFailed {
                    message: format!("Failed to iterate cursor: {}", e),
                })?
        {
            results.push(doc);
        }

        Ok(results)
    }

    /// 记录交互日志
    pub async fn log_interaction(interaction: &InteractionLog) -> DatabaseResult<ObjectId> {
        let collection: Collection<InteractionLog> = get_collection("interactions");

        // 验证数据
        Self::validate_interaction_log(interaction)?;

        let result = collection
            .insert_one(interaction, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to insert interaction log: {}", e),
            })?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// 获取用户交互历史
    pub async fn get_user_interactions(
        user_id: &str,
        limit: Option<i64>,
        session_id: Option<&str>,
    ) -> DatabaseResult<Vec<InteractionLog>> {
        let collection: Collection<InteractionLog> = get_collection("interactions");

        let mut filter = doc! { "user_id": user_id };
        if let Some(session) = session_id {
            filter.insert("session_id", session);
        }

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit.unwrap_or(100))
            .build();

        let mut results = Vec::new();
        use futures::TryStreamExt;

        let mut cursor =
            collection
                .find(filter, options)
                .await
                .map_err(|e| DatabaseError::OperationFailed {
                    message: format!("Failed to get user interactions: {}", e),
                })?;

        while let Some(doc) =
            cursor
                .try_next()
                .await
                .map_err(|e| DatabaseError::OperationFailed {
                    message: format!("Failed to iterate cursor: {}", e),
                })?
        {
            results.push(doc);
        }

        Ok(results)
    }

    /// 获取用户统计信息
    pub async fn get_user_statistics(user_id: &str) -> DatabaseResult<UserStatistics> {
        // 获取记忆体信息
        let memory_corpus_collection: Collection<MemoryCorpus> = get_collection("memory_corpus");
        let memory_corpus = memory_corpus_collection
            .find_one(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to get memory corpus: {}", e),
            })?;

        // 统计记忆片段
        let fragment_collection: Collection<MemoryFragment> = get_collection("memory_fragments");
        let total_memories = fragment_collection
            .count_documents(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to count memory fragments: {}", e),
            })? as u64;

        // 统计各类型记忆数量
        let mut memory_type_counts = HashMap::new();
        for memory_type in [
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Strategic,
            MemoryType::ActionState,
        ] {
            let count = fragment_collection
                .count_documents(doc! {
                    "user_id": user_id,
                    "memory_type": serde_json::to_string(&memory_type).unwrap().trim_matches('"')
                }, None)
                .await
                .map_err(|e| DatabaseError::OperationFailed {
                    message: format!("Failed to count memory type: {}", e),
                })? as u64;
            memory_type_counts.insert(memory_type, count);
        }

        // 统计交互记录
        let interaction_collection: Collection<InteractionLog> = get_collection("interactions");
        let total_interactions = interaction_collection
            .count_documents(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to count interactions: {}", e),
            })? as u64;

        // 获取最后交互时间
        use mongodb::options::FindOneOptions;
        let last_interaction = interaction_collection
            .find_one(
                doc! { "user_id": user_id },
                FindOneOptions::builder()
                    .sort(doc! { "timestamp": -1 })
                    .build(),
            )
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to get last interaction: {}", e),
            })?
            .map(|log| log.timestamp);

        let account_created = memory_corpus
            .map(|mc| mc.created_at)
            .unwrap_or_else(Utc::now);

        Ok(UserStatistics {
            user_id: user_id.to_string(),
            total_memories,
            memory_type_counts,
            total_interactions,
            last_interaction,
            account_created,
        })
    }

    /// 删除用户的所有数据 (GDPR 合规)
    pub async fn delete_user_data(user_id: &str) -> DatabaseResult<u64> {
        let mut total_deleted = 0u64;

        // 删除记忆体
        let memory_corpus_collection: Collection<MemoryCorpus> = get_collection("memory_corpus");
        let memory_result = memory_corpus_collection
            .delete_many(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to delete memory corpus: {}", e),
            })?;
        total_deleted += memory_result.deleted_count;

        // 删除记忆片段
        let fragment_collection: Collection<MemoryFragment> = get_collection("memory_fragments");
        let fragment_result = fragment_collection
            .delete_many(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to delete memory fragments: {}", e),
            })?;
        total_deleted += fragment_result.deleted_count;

        // 删除交互记录
        let interaction_collection: Collection<InteractionLog> = get_collection("interactions");
        let interaction_result = interaction_collection
            .delete_many(doc! { "user_id": user_id }, None)
            .await
            .map_err(|e| DatabaseError::OperationFailed {
                message: format!("Failed to delete interactions: {}", e),
            })?;
        total_deleted += interaction_result.deleted_count;

        info!(
            user_id = user_id,
            total_deleted = total_deleted,
            "User data deleted successfully"
        );

        Ok(total_deleted)
    }

    // 私有验证方法
    fn validate_memory_corpus(corpus: &MemoryCorpus) -> DatabaseResult<()> {
        if corpus.user_id.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "user_id cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    fn validate_memory_fragment(fragment: &MemoryFragment) -> DatabaseResult<()> {
        if fragment.user_id.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "user_id cannot be empty".to_string(),
            });
        }

        if fragment.content.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "content cannot be empty".to_string(),
            });
        }

        if fragment.importance_score < 0.0 || fragment.importance_score > 1.0 {
            return Err(DatabaseError::ValidationError {
                details: "importance_score must be between 0.0 and 1.0".to_string(),
            });
        }

        Ok(())
    }

    fn validate_interaction_log(interaction: &InteractionLog) -> DatabaseResult<()> {
        if interaction.user_id.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "user_id cannot be empty".to_string(),
            });
        }

        if interaction.session_id.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "session_id cannot be empty".to_string(),
            });
        }

        if interaction.content.is_empty() {
            return Err(DatabaseError::ValidationError {
                details: "content cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}
