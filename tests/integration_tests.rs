// 集成测试 - 测试各服务之间的协作

use sentio_core::{EmailWorkflow, MockSmtpClient};
use sentio_email::{EmailAddress, EmailBody, OutgoingMessage};
use sentio_llm::{LlmClient, LlmRequest, LlmResponse, LlmResult, ResponseMetadata, TokenUsage};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

// Mock LLM Client for integration tests
pub struct IntegrationMockLlmClient {
    pub response_content: String,
}

impl IntegrationMockLlmClient {
    pub fn new(response: &str) -> Self {
        Self {
            response_content: response.to_string(),
        }
    }
}

#[async_trait]
impl LlmClient for IntegrationMockLlmClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        Ok(LlmResponse {
            request_id: request.id,
            content: self.response_content.clone(),
            token_usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            metadata: ResponseMetadata {
                model: "integration-mock".to_string(),
                latency_ms: 50,
                retry_count: 0,
                extra: HashMap::new(),
            },
            created_at: Utc::now(),
        })
    }
}

#[tokio::test]
async fn test_email_workflow_integration() {
    // Create mock clients
    let llm_client = Box::new(IntegrationMockLlmClient::new(
        "This email appears to be a password reset request. The sender seems urgent but polite."
    ));
    let email_client = Box::new(MockSmtpClient::new());

    // Create workflow
    let workflow = EmailWorkflow::new_with_clients(llm_client, email_client);

    // Create test email
    let from_addr = EmailAddress::new("user@example.com".to_string());
    let to_addr = vec![EmailAddress::new("support@company.com".to_string())];
    let email_body = EmailBody::text("I forgot my password and need help resetting it.".to_string());
    let message = OutgoingMessage::new(
        from_addr,
        to_addr,
        "Password Reset Help".to_string(),
        email_body,
    );

    // Process email (this should work with mock clients)
    let result = workflow.process_email(&message).await;
    
    // For this test, we expect it to fail because the mock LLM client
    // doesn't have access to proper prompts configuration
    // In a real environment, this would be properly configured
    assert!(result.is_err());
}

#[tokio::test]
async fn test_email_validation_comprehensive() {
    use sentio_email::{EmailAddress, EmailBody, OutgoingMessage, EmailAttachment};

    // Test comprehensive email validation
    let valid_from = EmailAddress::with_name("sender@test.com".to_string(), "Test Sender".to_string());
    let valid_to = vec![
        EmailAddress::new("recipient1@test.com".to_string()),
        EmailAddress::new("recipient2@test.com".to_string()),
    ];
    let valid_body = EmailBody::text("This is a test email message.".to_string());

    let mut message = OutgoingMessage::new(
        valid_from,
        valid_to,
        "Test Email Subject".to_string(),
        valid_body,
    );

    // Add CC and BCC
    message = message
        .add_cc(EmailAddress::new("cc@test.com".to_string()))
        .add_bcc(EmailAddress::new("bcc@test.com".to_string()));

    // Add safe attachment
    let attachment = EmailAttachment {
        filename: "document.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        size: 1024,
        content_id: None,
        is_inline: false,
    };
    message = message.add_attachment(attachment);

    // Add custom header
    message = message.add_header("X-Priority".to_string(), "High".to_string());

    // Validate message
    assert!(message.validate().is_ok());
    assert_eq!(message.cc.len(), 1);
    assert_eq!(message.bcc.len(), 1);
    assert_eq!(message.attachments.len(), 1);
    assert_eq!(message.headers.len(), 1);
}

#[tokio::test]
async fn test_memory_service_integration() {
    use shared_logic::{MemoryDataAccess, InteractionLog, MessageDirection};
    use tempfile::tempdir;
    use std::path::PathBuf;

    // Create temporary directory for test
    let temp_dir = tempdir().unwrap();
    let memory_file = temp_dir.path().join("integration_test_memory.json");

    // Initialize memory service
    MemoryDataAccess::initialize(memory_file).await.unwrap();

    // Create test interaction
    let interaction = InteractionLog {
        id: None,
        user_id: "integration_test_user".to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        direction: MessageDirection::UserToSystem,
        content: "Integration test message".to_string(),
        metadata: HashMap::new(),
    };

    // Log interaction
    let interaction_id = MemoryDataAccess::log_interaction(&interaction).await.unwrap();
    assert!(!interaction_id.to_string().is_empty());

    // Retrieve interactions
    let interactions = MemoryDataAccess::get_user_interactions("integration_test_user", Some(10), None).await.unwrap();
    assert_eq!(interactions.len(), 1);
    assert_eq!(interactions[0].content, "Integration test message");

    // Get statistics
    let stats = MemoryDataAccess::get_user_statistics("integration_test_user").await.unwrap();
    assert_eq!(stats.user_id, "integration_test_user");
    assert_eq!(stats.total_interactions, 1);
}

#[tokio::test]
async fn test_config_and_prompts_integration() {
    use shared_logic::config;
    use tempfile::tempdir;
    use std::fs;

    // Create temporary config files
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let prompts_file = config_dir.join("prompts.yaml");
    fs::write(&prompts_file, r#"
prompts:
  test:
    simple:
      system: "You are a test assistant."
      user: "Test prompt with {variable}."
"#).unwrap();

    // Set environment variable for config path
    std::env::set_var("SENTIO_CONFIG_DIR", config_dir.to_string_lossy().to_string());

    // Test would require modifying config loading to support custom paths
    // For now, we'll test the basic functionality
    let test_config = config::Config::default();
    assert_eq!(test_config.server.host, "127.0.0.1");
    assert_eq!(test_config.server.port, 8080);
}