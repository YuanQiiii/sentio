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
fn test_memory_corpus_bson_serialization() {
    let mut corpus = MemoryCorpus::default();
    corpus.user_id = "bson_test@example.com".to_string();
    corpus.version = "1.0".to_string();

    // 测试 BSON 序列化
    let bson_doc = bson::to_document(&corpus).expect("BSON 序列化失败");
    assert!(bson_doc.contains_key("user_id"));
    assert!(bson_doc.contains_key("version"));
    assert!(bson_doc.contains_key("core_profile"));

    // 测试 BSON 反序列化
    let deserialized: MemoryCorpus = bson::from_document(bson_doc).expect("BSON 反序列化失败");
    assert_eq!(deserialized.user_id, corpus.user_id);
    assert_eq!(deserialized.version, corpus.version);
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

        async fn ensure_indexes(&self) -> MemoryResult<()> {
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

#[cfg(test)]
mod mongo_integration_tests {
    use super::*; // Imports Utc, etc. from parent module.
    use sentio_memory::{repository::MemoryRepository, MemoryCorpus, MongoMemoryRepository};
    use shared_logic::config;
    use tokio::sync::OnceCell;
    use uuid::Uuid;

    static INIT: OnceCell<()> = OnceCell::const_new();

    async fn setup() -> MongoMemoryRepository {
        // Ensure config is initialized only once for all tests
        INIT.get_or_init(|| async {
            config::initialize_config()
                .await
                .expect("Failed to initialize config for integration tests");
        })
        .await;

        MongoMemoryRepository::new()
            .await
            .expect("Failed to connect to MongoDB for integration tests")
    }

    fn create_test_user_id() -> String {
        format!("test_user_{}", Uuid::new_v4())
    }

    fn create_test_corpus(user_id: &str) -> MemoryCorpus {
        MemoryCorpus {
            user_id: user_id.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_mongo_connection_and_health_check() {
        let repo = setup().await;
        let health = repo.health_check().await.expect("Health check failed");
        assert!(health);
    }

    #[tokio::test]
    async fn test_save_and_get_memory_corpus() {
        let repo = setup().await;
        let user_id = create_test_user_id();
        let corpus = create_test_corpus(&user_id);

        repo.save_memory_corpus(&corpus)
            .await
            .expect("Failed to save memory corpus");

        let retrieved_corpus = repo
            .get_memory_corpus(&user_id)
            .await
            .expect("Failed to get memory corpus")
            .expect("Corpus not found after saving");

        assert_eq!(retrieved_corpus.user_id, corpus.user_id);
        assert_eq!(retrieved_corpus.version, corpus.version);

        // Cleanup
        repo.delete_user_data(&user_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_memory_corpus() {
        let repo = setup().await;
        let user_id = create_test_user_id();
        let corpus = create_test_corpus(&user_id);

        repo.save_memory_corpus(&corpus)
            .await
            .expect("Failed to save initial memory corpus");

        let new_summary = "A new life summary from the test".to_string();
        let mut updates = std::collections::HashMap::new();
        updates.insert(
            "core_profile.current_life_summary".to_string(),
            serde_json::json!(new_summary),
        );

        repo.update_memory_corpus(&user_id, updates)
            .await
            .expect("Failed to update corpus");

        let updated_corpus = repo
            .get_memory_corpus(&user_id)
            .await
            .expect("Failed to get updated corpus")
            .expect("Updated corpus not found");

        assert_eq!(
            updated_corpus.core_profile.current_life_summary.unwrap(),
            new_summary
        );
        assert!(updated_corpus.updated_at > corpus.updated_at);

        // Cleanup
        repo.delete_user_data(&user_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_save_and_get_interactions() {
        let repo = setup().await;
        let user_id = create_test_user_id();

        let interaction1 = InteractionLog::new(
            user_id.clone(),
            MessageDirection::Inbound,
            "First test message".to_string(),
        );
        // Ensure distinct timestamps for ordering test
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let interaction2 = InteractionLog::new(
            user_id.clone(),
            MessageDirection::Outbound,
            "First test reply".to_string(),
        );

        repo.save_interaction(&user_id, &interaction1)
            .await
            .expect("Failed to save interaction 1");
        repo.save_interaction(&user_id, &interaction2)
            .await
            .expect("Failed to save interaction 2");

        let retrieved_interactions = repo
            .get_recent_interactions(&user_id, 5)
            .await
            .expect("Failed to get interactions");

        assert_eq!(retrieved_interactions.len(), 2);
        // get_recent_interactions should return newest first
        assert_eq!(retrieved_interactions[0].summary, "First test reply");
        assert_eq!(retrieved_interactions[1].summary, "First test message");

        // Cleanup
        repo.delete_user_data(&user_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_user_data() {
        let repo = setup().await;
        let user_id = create_test_user_id();
        let corpus = create_test_corpus(&user_id);
        let interaction = InteractionLog::new(
            user_id.clone(),
            MessageDirection::Inbound,
            "This message will be deleted".to_string(),
        );

        repo.save_memory_corpus(&corpus)
            .await
            .expect("Failed to save corpus for deletion test");
        repo.save_interaction(&user_id, &interaction)
            .await
            .expect("Failed to save interaction for deletion test");

        assert!(
            repo.get_memory_corpus(&user_id).await.unwrap().is_some(),
            "Corpus should exist before deletion"
        );
        assert!(
            !repo
                .get_recent_interactions(&user_id, 1)
                .await
                .unwrap()
                .is_empty(),
            "Interaction should exist before deletion"
        );

        repo.delete_user_data(&user_id)
            .await
            .expect("Failed to delete user data");

        assert!(
            repo.get_memory_corpus(&user_id).await.unwrap().is_none(),
            "Corpus should not exist after deletion"
        );
        assert!(
            repo.get_recent_interactions(&user_id, 1)
                .await
                .unwrap()
                .is_empty(),
            "Interaction should not exist after deletion"
        );
    }
}

// ============================================
// 数据库集成测试 (需要 MongoDB 运行)
// ============================================

/// 数据库连接和基本操作集成测试
/// 注意：这个测试需要 MongoDB 服务运行
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_mongodb_connection_and_basic_operations() {
    use sentio_memory::{MemoryRepository, MongoMemoryRepository};

    // 初始化配置 (需要测试配置)
    std::env::set_var(
        "SENTIO_DATABASE__URL",
        "mongodb://localhost:27017/sentio_test",
    );
    std::env::set_var("SENTIO_DATABASE__MAX_CONNECTIONS", "5");
    std::env::set_var("SENTIO_DATABASE__CONNECT_TIMEOUT", "10");

    // 初始化配置
    if let Err(_) = shared_logic::initialize_config().await {
        // 如果配置已经初始化，忽略错误
    }

    // 创建 MongoDB 仓储
    let repository = match MongoMemoryRepository::new().await {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("MongoDB 连接失败，跳过集成测试: {}", e);
            return;
        }
    };

    // 测试健康检查
    let health = repository.health_check().await.expect("健康检查失败");
    assert!(health, "数据库健康检查应该返回 true");

    // 测试保存和获取记忆体
    let test_user_id = "integration_test_user@example.com";

    // 清理可能存在的测试数据
    let _ = repository.delete_user_data(test_user_id).await;

    // 创建测试记忆体
    let mut corpus = MemoryCorpus::default();
    corpus.user_id = test_user_id.to_string();
    corpus.version = "test_1.0".to_string();

    // 保存记忆体
    repository
        .save_memory_corpus(&corpus)
        .await
        .expect("保存记忆体失败");

    // 获取记忆体
    let retrieved = repository
        .get_memory_corpus(test_user_id)
        .await
        .expect("获取记忆体失败");
    assert!(retrieved.is_some(), "应该能够获取刚保存的记忆体");

    let retrieved_corpus = retrieved.unwrap();
    assert_eq!(retrieved_corpus.user_id, test_user_id);
    assert_eq!(retrieved_corpus.version, "test_1.0");

    // 测试交互记录保存
    let interaction = InteractionLog::new(
        test_user_id.to_string(),
        MessageDirection::Inbound,
        "集成测试消息".to_string(),
    );

    repository
        .save_interaction(test_user_id, &interaction)
        .await
        .expect("保存交互记录失败");

    // 获取最近交互记录
    let recent_interactions = repository
        .get_recent_interactions(test_user_id, 10)
        .await
        .expect("获取交互记录失败");
    assert!(!recent_interactions.is_empty(), "应该有至少一条交互记录");
    assert_eq!(recent_interactions[0].summary, "集成测试消息");

    // 获取用户统计信息
    let stats = repository
        .get_user_statistics(test_user_id)
        .await
        .expect("获取用户统计失败");
    assert_eq!(stats.user_id, test_user_id);
    assert!(stats.total_interactions > 0, "应该有交互记录计数");

    // 清理测试数据
    repository
        .delete_user_data(test_user_id)
        .await
        .expect("清理测试数据失败");

    // 验证数据已被删除
    let after_delete = repository
        .get_memory_corpus(test_user_id)
        .await
        .expect("删除后查询应该成功");
    assert!(after_delete.is_none(), "删除后应该查询不到数据");
}

/// 测试数据库配置验证
#[tokio::test]
async fn test_database_config_validation() {
    use sentio_memory::MongoMemoryRepository;

    // 测试无效的数据库 URL
    std::env::set_var("SENTIO_DATABASE__URL", "invalid-url");

    // 重新初始化配置
    let _ = shared_logic::initialize_config().await;

    // 尝试创建仓储应该失败
    let result = MongoMemoryRepository::new().await;
    assert!(result.is_err(), "无效的数据库 URL 应该导致连接失败");
}
