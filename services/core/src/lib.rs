//! # Sentio Core 服务
//!
//! 核心业务协调服务，负责整合所有其他服务来实现完整的邮件智能处理流程。
//!
//! ## 主要功能
//!
//! - **邮件工作流程**: 完整的邮件接收、分析、回复生成流程
//! - **服务编排**: 协调 memory、llm、email 等服务
//! - **配置管理**: 统一的配置加载和管理
//! - **错误处理**: 优雅的错误处理和恢复机制
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Email Input   │───▶│  Core Workflow  │───▶│  Email Output   │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!                                │
//!                                ▼
//!                        ┌───────────────┐
//!                        │   Services    │
//!                        │               │
//!                        │ ┌───────────┐ │
//!                        │ │ Memory    │ │
//!                        │ └───────────┘ │
//!                        │ ┌───────────┐ │
//!                        │ │ LLM       │ │
//!                        │ └───────────┘ │
//!                        │ ┌───────────┐ │
//!                        │ │ Telemetry │ │
//!                        │ └───────────┘ │
//!                        └───────────────┘
//! ```

use anyhow::Result;
use async_trait::async_trait;
use sentio_email::{EmailAddress, EmailBody, EmailResult, MessageId, OutgoingMessage, SmtpClient};
use sentio_llm::DeepSeekClient;
use std::sync::Mutex;
use tracing::info;

pub struct MockSmtpClient {
    pub send_message_calls: Mutex<Vec<OutgoingMessage>>,
    pub connect_calls: Mutex<u32>,
    pub disconnect_calls: Mutex<u32>,
    pub is_connected_value: Mutex<bool>,
}

impl MockSmtpClient {
    pub fn new() -> Self {
        Self {
            send_message_calls: Mutex::new(Vec::new()),
            connect_calls: Mutex::new(0),
            disconnect_calls: Mutex::new(0),
            is_connected_value: Mutex::new(false),
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

pub mod workflow;
pub use workflow::EmailWorkflow;

pub async fn demonstrate_workflow() -> Result<()> {
    info!("Demonstrating email workflow...");

    // Initialize LLM client
    let llm_client = Box::new(DeepSeekClient::new()?);

    // Initialize Email client (using a mock for demonstration)
    let email_client = Box::new(MockSmtpClient::new());

    let workflow = EmailWorkflow::new_with_clients(llm_client, email_client);

    // Create a mock incoming email
    let from_addr = EmailAddress::new("sender@example.com".to_string());
    let to_addr = vec![EmailAddress::new("recipient@example.com".to_string())];
    let email_body = EmailBody::text(
        "Hello, I need help with my account. Can you reset my password?".to_string(),
    );
    let incoming_message = OutgoingMessage::new(
        from_addr,
        to_addr,
        "Password Reset Request".to_string(),
        email_body,
    );

    // Process the email
    workflow.process_email(&incoming_message).await?;

    info!("Email workflow demonstration complete.");
    Ok(())
}
