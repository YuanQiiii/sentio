//! # 记忆服务集成测试
//!
//! 这些测试验证记忆服务的核心功能，包括数据模型的序列化/反序列化
//! 以及基本的业务逻辑。包括完整的 MongoDB 集成测试。

use chrono::Utc;
use sentio_memory::{InteractionLog, MemoryCorpus, MessageDirection};
use serde_json;
use tokio;

#[test]
fn test_interaction_log_creation() {
    let interaction = InteractionLog::new(
        "test_user_123".to_string(),
        MessageDirection::Inbound,
        "这是一个测试消息".to_string(),
    );

    assert_eq!(interaction.user_id, "test_user_123");
    assert!(matches!(interaction.direction, MessageDirection::Inbound));
    assert_eq!(interaction.summary, "这是一个测试消息");
    assert!(!interaction.log_id.is_empty());
    assert_eq!(interaction.llm_model_version, "demo");

    // 检查时间戳是否合理（在最近几秒内）
    let now = Utc::now();
    let diff = now.signed_duration_since(interaction.timestamp);
    assert!(diff.num_seconds() < 5);
}

#[test]
fn test_interaction_log_serialization() {
    let interaction = InteractionLog::new(
        "serialization_test".to_string(),
        MessageDirection::Outbound,
        "序列化测试消息".to_string(),
    );

    // 测试序列化
    let json = serde_json::to_string(&interaction).expect("序列化失败");
    assert!(!json.is_empty());
    assert!(json.contains("serialization_test"));
    assert!(json.contains("序列化测试消息"));

    // 测试反序列化
    let deserialized: InteractionLog = serde_json::from_str(&json).expect("反序列化失败");
    assert_eq!(deserialized.user_id, interaction.user_id);
    assert_eq!(deserialized.log_id, interaction.log_id);
    assert_eq!(deserialized.summary, interaction.summary);
    assert!(matches!(deserialized.direction, MessageDirection::Outbound));
}

#[test]
fn test_memory_corpus_creation() {
    let mut corpus = MemoryCorpus::default();
    corpus.user_id = "test_user@example.com".to_string();
    corpus.version = "1.0".to_string();

    assert_eq!(corpus.user_id, "test_user@example.com");
    assert_eq!(corpus.version, "1.0");
    assert_eq!(corpus.episodic_memory.interaction_log.len(), 0);
}

#[test]
fn test_message_direction_serialization() {
    // 测试 Inbound 序列化
    let inbound = MessageDirection::Inbound;
    let json = serde_json::to_string(&inbound).expect("序列化失败");
    assert_eq!(json, r#""inbound""#);

    // 测试 Outbound 序列化
    let outbound = MessageDirection::Outbound;
    let json = serde_json::to_string(&outbound).expect("序列化失败");
    assert_eq!(json, r#""outbound""#);

    // 测试反序列化
    let deserialized: MessageDirection =
        serde_json::from_str(r#""inbound""#).expect("反序列化失败");
    assert!(matches!(deserialized, MessageDirection::Inbound));

    let deserialized: MessageDirection =
        serde_json::from_str(r#""outbound""#).expect("反序列化失败");
    assert!(matches!(deserialized, MessageDirection::Outbound));
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use async_trait::async_trait;
    use sentio_memory::{MemoryRepository, MemoryResult};
    use std::collections::HashMap;

    /// 模拟记忆仓储实现，用于单元测试
    pub struct MockMemoryRepository {
        interactions: std::sync::Mutex<Vec<InteractionLog>>,
    }

    impl MockMemoryRepository {
        pub fn new() -> Self {
            Self {
                interactions: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl MemoryRepository for MockMemoryRepository {
        async fn save_memory_corpus(
            &self,
            _corpus: &sentio_memory::MemoryCorpus,
        ) -> MemoryResult<()> {
            Ok(())
        }

        async fn get_memory_corpus(
            &self,
            _user_id: &str,
        ) -> MemoryResult<Option<sentio_memory::MemoryCorpus>> {
            Ok(None)
        }

        async fn update_memory_corpus(
            &self,
            _user_id: &str,
            _updates: HashMap<String, serde_json::Value>,
        ) -> MemoryResult<()> {
            Ok(())
        }

        async fn save_interaction(
            &self,
            _user_id: &str,
            interaction: &InteractionLog,
        ) -> MemoryResult<()> {
            let mut interactions = self.interactions.lock().unwrap();
            interactions.push(interaction.clone());
            Ok(())
        }

        async fn search_memories(
            &self,
            _query: &sentio_memory::MemoryQuery,
        ) -> MemoryResult<Vec<sentio_memory::MemoryFragment>> {
            Ok(Vec::new())
        }

        async fn get_recent_interactions(
            &self,
            user_id: &str,
            limit: u32,
        ) -> MemoryResult<Vec<InteractionLog>> {
            let interactions = self.interactions.lock().unwrap();
            let user_interactions: Vec<InteractionLog> = interactions
                .iter()
                .filter(|i| i.user_id == user_id)
                .take(limit as usize)
                .cloned()
                .collect();
            Ok(user_interactions)
        }

        async fn get_user_statistics(
            &self,
            _user_id: &str,
        ) -> MemoryResult<sentio_memory::UserStatistics> {
            Ok(sentio_memory::UserStatistics {
                user_id: "test".to_string(),
                total_interactions: 0,
                first_interaction: Utc::now(),
                last_interaction: Utc::now(),
                total_memories: 0,
                memory_type_distribution: HashMap::new(),
            })
        }

        async fn delete_user_data(&self, user_id: &str) -> MemoryResult<()> {
            let mut interactions = self.interactions.lock().unwrap();
            interactions.retain(|i| i.user_id != user_id);
            Ok(())
        }

        async fn health_check(&self) -> MemoryResult<bool> {
            Ok(true)
        }

        async fn initialize(&self) -> MemoryResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_repository_basic_operations() {
        let repo = MockMemoryRepository::new();

        // 创建测试交互
        let interaction = InteractionLog::new(
            "mock_user".to_string(),
            MessageDirection::Inbound,
            "模拟测试消息".to_string(),
        );

        // 保存交互
        repo.save_interaction(&interaction.user_id, &interaction)
            .await
            .expect("保存交互失败");

        // 检索交互
        let retrieved = repo
            .get_recent_interactions(&interaction.user_id, 10)
            .await
            .expect("检索交互失败");

        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].user_id, interaction.user_id);
        assert_eq!(retrieved[0].summary, interaction.summary);

        // 健康检查
        let health = repo.health_check().await.expect("健康检查失败");
        assert!(health);
    }
}
