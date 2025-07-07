//! # 邮件工作流模块
//! 
//! `workflow` 模块定义了 `EmailWorkflow` 结构体，负责协调 LLM 和邮件客户端
//! 来处理邮件。它封装了邮件分析、回复生成和发送的端到端流程。

use anyhow::Result;
use sentio_email::{OutgoingMessage, SmtpClient};
use sentio_llm::{LlmClient, LlmRequest, LlmResponse};
use tracing::{debug, info, trace};

/// `EmailWorkflow` 结构体负责协调 LLM 和邮件客户端来处理邮件。
/// 它封装了邮件分析、回复生成和发送的端到端流程。
#[allow(dead_code)]
pub struct EmailWorkflow {
    /// 用于与大型语言模型 (LLM) 交互的客户端。
    pub llm_client: Box<dyn LlmClient + Send + Sync>,
    /// 用于发送邮件的 SMTP 客户端。
    pub email_client: Box<dyn SmtpClient + Send + Sync>,
}

#[allow(dead_code)]
impl EmailWorkflow {
    /// 使用提供的 LLM 和邮件客户端创建一个新的 `EmailWorkflow` 实例。
    ///
    /// # 参数
    ///
    /// * `llm_client` - 一个实现了 `LlmClient` trait 的 Boxed 客户端实例。
    /// * `email_client` - 一个实现了 `SmtpClient` trait 的 Boxed 客户端实例。
    ///
    /// # 返回
    ///
    /// 一个新的 `EmailWorkflow` 实例。
    pub fn new_with_clients(
        llm_client: Box<dyn LlmClient + Send + Sync>,
        email_client: Box<dyn SmtpClient + Send + Sync>,
    ) -> Self {
        debug!("Initializing EmailWorkflow with provided LLM and Email clients.");
        EmailWorkflow {
            llm_client,
            email_client,
        }
    }

    /// 处理传入的邮件：使用 LLM 分析邮件内容，然后模拟生成并发送回复。
    ///
    /// # 参数
    ///
    /// * `message` - 要处理的传入邮件，类型为 `&OutgoingMessage`。
    ///
    /// # 返回
    ///
    /// 如果邮件处理成功，则返回 `Ok(())`；否则返回 `anyhow::Error`。
    pub async fn process_email(&self, message: &OutgoingMessage) -> Result<()> {
        info!("开始处理邮件: {}", message.subject);
        debug!("Incoming email details: from={:?}, to={:?}, subject={}",
            message.from,
            message.to,
            message.subject
        );

        // 1. 使用 LLM 客户端分析邮件内容
        trace!("Preparing LLM request for email analysis.");
        let llm_request = LlmRequest::new(
            "email_analysis.summarize_thread".to_string(), // 假设有这个提示词
            std::collections::HashMap::from([(
                "email_content".to_string(),
                serde_json::json!(message.body.text.clone().unwrap_or_default()),
            )]),
        );
        debug!("Sending LLM request: prompt_name={}, request_id={}",
            llm_request.prompt_name,
            llm_request.id
        );
        let llm_response: LlmResponse = self.llm_client.generate_response(&llm_request).await?;
        info!("LLM 分析结果: {}", llm_response.content);
        debug!("LLM response metadata: model={}, latency={}ms, retry_count={}",
            llm_response.metadata.model,
            llm_response.metadata.latency_ms,
            llm_response.metadata.retry_count
        );

        // 2. 模拟发送回复邮件
        trace!("Preparing reply email based on LLM analysis.");
        // 实际应用中，这里会根据 LLM 分析结果生成回复内容
        let reply_body = sentio_email::EmailBody::text(format!(
            "这是对您邮件的回复：\n\n{}",
            llm_response.content
        ));
        let reply_message = sentio_email::OutgoingMessage::new(
            message.to[0].clone(),      // 假设回复给第一个收件人
            vec![message.from.clone()], // 回复给发件人
            format!("Re: {}", message.subject),
            reply_body,
        );

        debug!("Sending reply email: subject={}, to={:?}",
            reply_message.subject,
            reply_message.to
        );
        self.email_client.send_message(&reply_message).await?;
        info!("已发送回复邮件给: {:?}", reply_message.to);
        trace!("Email processing complete for subject: {}", message.subject);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sentio_email::{EmailAddress, EmailResult, MessageId, OutgoingMessage, SmtpClient};
    use sentio_llm::{LlmClient, LlmRequest, LlmResponse, LlmResult, ResponseMetadata, TokenUsage};

    use super::EmailWorkflow;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;

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
            self.generate_response_calls
                .lock()
                .unwrap()
                .push(request.clone());
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
    // Note: This MockSmtpClient is duplicated from services/core/src/test_utils.rs
    // For actual project, ensure consistency or use the one from test_utils.
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
            self.send_message_calls
                .lock()
                .unwrap()
                .push(message.clone());
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
        let _llm_client_ref = workflow
            .llm_client
            .as_any()
            .downcast_ref::<MockLlmClient>()
            .unwrap();

        let email_client_ref = workflow
            .email_client
            .as_any()
            .downcast_ref::<MockSmtpClient>()
            .unwrap();
        assert_eq!(*email_client_ref.is_connected_value.lock().unwrap(), false);
        // Should be false as connect() is not called by new_with_clients
    }
}