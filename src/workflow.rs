use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::config;
use crate::email::{EmailClient, EmailMessage};
use crate::llm::{LlmClient, LlmRequest};
use crate::memory::{MemoryStore, MemoryType, InteractionLog, MessageDirection};

#[allow(dead_code)]
pub struct EmailWorkflow {
    llm_client: Box<dyn LlmClient>,
    email_client: Box<dyn EmailClient>,
    allowed_sender: String,
}

impl EmailWorkflow {
    pub fn new(llm_client: Box<dyn LlmClient>, email_client: Box<dyn EmailClient>, allowed_sender: String) -> Self {
        Self {
            llm_client,
            email_client,
            allowed_sender,
        }
    }

    pub async fn process_incoming_email(&self, message: &EmailMessage) -> Result<()> {
        // 检查发件人是否是允许的邮箱
        if message.from.email != self.allowed_sender {
            warn!(
                "Ignoring email from unauthorized sender: {}. Only {} is allowed.",
                message.from.email, self.allowed_sender
            );
            return Ok(());
        }

        info!("Processing email from authorized sender {}: {}", message.from.email, message.subject);

        // 记录交互
        let _ = MemoryStore::log_interaction(&InteractionLog {
            id: None,
            user_id: message.from.email.clone(),
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            direction: MessageDirection::UserToSystem,
            content: format!("Subject: {}\n\n{}", message.subject, message.body),
            metadata: HashMap::new(),
        }).await;

        // 分析邮件
        let mut context = HashMap::new();
        context.insert("email_content".to_string(), 
            serde_json::json!(format!("Subject: {}\n\n{}", message.subject, message.body)));

        let analysis_request = LlmRequest::new("email_analysis".to_string(), context);
        let analysis = self.llm_client.generate_response(&analysis_request).await?;
        
        debug!("Email analysis: {}", analysis.content);

        // 存储分析结果
        let memory_store = MemoryStore::get();
        memory_store.add_memory(
            &message.from.email,
            MemoryType::Event,
            format!("Email analysis for '{}': {}", message.subject, analysis.content)
        ).await?;

        // 生成回复
        let mut reply_context = HashMap::new();
        reply_context.insert("original_email".to_string(), 
            serde_json::json!(format!("Subject: {}\n\n{}", message.subject, message.body)));
        reply_context.insert("analysis_result".to_string(), 
            serde_json::json!(analysis.content));

        let reply_request = LlmRequest::new("email_reply".to_string(), reply_context);
        let reply = self.llm_client.generate_response(&reply_request).await?;

        // 发送回复
        let reply_message = EmailMessage {
            from: message.to[0].clone(),
            to: vec![message.from.clone()],
            subject: format!("Re: {}", message.subject),
            body: reply.content.clone(),
            is_html: false,
        };

        self.email_client.send(reply_message).await?;

        // 记录回复
        let _ = MemoryStore::log_interaction(&InteractionLog {
            id: None,
            user_id: message.from.email.clone(),
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            direction: MessageDirection::SystemToUser,
            content: reply.content,
            metadata: HashMap::new(),
        }).await;

        info!("Email processed and reply sent successfully");
        Ok(())
    }
}

pub async fn create_workflow() -> Result<EmailWorkflow> {
    use crate::email::SmtpClient;
    use crate::llm::DeepSeekClient;

    let config = config::get();
    
    // 创建客户端
    let llm_client = Box::new(DeepSeekClient::new()?);
    let email_client = Box::new(SmtpClient::new(config.email.smtp.clone())?);

    // 创建工作流
    Ok(EmailWorkflow::new(llm_client, email_client, config.email.allowed_sender.clone()))
}