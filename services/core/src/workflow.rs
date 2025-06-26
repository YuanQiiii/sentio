
use sentio_llm::LlmClient;
use sentio_email::SmtpClient;

pub struct EmailWorkflow {
    pub llm_client: Box<dyn LlmClient + Send + Sync>,
    pub email_client: Box<dyn SmtpClient + Send + Sync>,
}

impl EmailWorkflow {
    pub fn new_with_clients(
        llm_client: Box<dyn LlmClient + Send + Sync>,
        email_client: Box<dyn SmtpClient + Send + Sync>,
    ) -> Self {
        EmailWorkflow {
            llm_client,
            email_client,
        }
    }
}

#[cfg(test)]
mod tests {
        use sentio_llm::{LlmClient, LlmRequest, LlmResponse, LlmResult, TokenUsage, ResponseMetadata};
    use sentio_email::{SmtpClient, EmailResult, MessageId, OutgoingMessage, EmailAddress};
    
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use super::EmailWorkflow;

    // Mock LlmClient
    pub struct MockLlmClient {
        pub generate_response_calls: std::sync::Mutex<Vec<LlmRequest>>,
        pub mock_response_content: String,
    }

    impl MockLlmClient {
        pub fn new(mock_response_content: &str) -> Self {
            Self {
                generate_response_calls: std::sync::Mutex::new(Vec::new()),
                mock_response_content: mock_response_content.to_string(),
            }
        }
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        

        async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
            self.generate_response_calls.lock().unwrap().push(request.clone());
            Ok(LlmResponse {
                request_id: request.id,
                content: self.mock_response_content.clone(),
                token_usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                metadata: ResponseMetadata {
                    model: "mock-model".to_string(),
                    latency_ms: 100,
                    retry_count: 0,
                    extra: HashMap::new(),
                },
                created_at: Utc::now(),
            })
        }
    }

    

    // Mock SmtpClient
    pub struct MockSmtpClient {
        pub send_message_calls: std::sync::Mutex<Vec<OutgoingMessage>>,
        pub connect_calls: std::sync::Mutex<u32>,
        pub disconnect_calls: std::sync::Mutex<u32>,
        pub is_connected_value: std::sync::Mutex<bool>,
    }

    impl MockSmtpClient {
        pub fn new() -> Self {
            Self {
                send_message_calls: std::sync::Mutex::new(Vec::new()),
                connect_calls: std::sync::Mutex::new(0),
                disconnect_calls: std::sync::Mutex::new(0),
                is_connected_value: std::sync::Mutex::new(false),
            }
        }
    }

    #[async_trait]
    impl SmtpClient for MockSmtpClient {
        async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId> {
            self.send_message_calls.lock().unwrap().push(message.clone());
            Ok(MessageId::new("mock_message_id".to_string()))
        }

        async fn verify_address(&self, _address: &EmailAddress) -> EmailResult<bool> {
            Ok(true)
        }

        async fn connect(&mut self) -> EmailResult<()> {
            *self.connect_calls.lock().unwrap() += 1;
            *self.is_connected_value.lock().unwrap() = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> EmailResult<()> {
            *self.disconnect_calls.lock().unwrap() += 1;
            *self.is_connected_value.lock().unwrap() = false;
            Ok(())
        }

        async fn is_connected(&self) -> bool {
            *self.is_connected_value.lock().unwrap()
        }
    }

    

    #[tokio::test]
    async fn test_email_workflow_new_with_clients() {
        let mock_llm_client = Box::new(MockLlmClient::new("mocked llm response"));
        let mock_email_client = Box::new(MockSmtpClient::new());

        let workflow = EmailWorkflow::new_with_clients(mock_llm_client, mock_email_client);

        // Verify that the clients are correctly set
        let _llm_client_ref = workflow.llm_client.as_any().downcast_ref::<MockLlmClient>().unwrap();
        

        let email_client_ref = workflow.email_client.as_any().downcast_ref::<MockSmtpClient>().unwrap();
        assert_eq!(*email_client_ref.is_connected_value.lock().unwrap(), false); // Should be false as connect() is not called by new_with_clients
    }
}