//! # 核心工作流程模块
//!
//! 实现 Sentio AI 的主要业务流程，包括：
//! - 邮件分析和处理
//! - 记忆检索和更新
//! - 智能回复生成
//! - 服务间协调

use anyhow::Result;
use sentio_email::{EmailAddress, EmailBody, OutgoingMessage, SimpleSmtpClient, SmtpClient};
use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};
use serde_json::json;
use shared_logic::{get_config, InteractionLog, MemoryCorpus, MemoryDataAccess, MessageDirection};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 邮件处理工作流程
pub struct EmailWorkflow {
    llm_client: DeepSeekClient,
    #[allow(dead_code)]
    email_client: SimpleSmtpClient,
}

impl EmailWorkflow {
    /// 创建新的邮件工作流程实例
    pub async fn new() -> Result<Self> {
        info!("Initializing email workflow...");

        let llm_client = DeepSeekClient::new()?;
        let mut email_client = SimpleSmtpClient::from_config().await?;
        email_client.connect().await?;

        Ok(Self {
            llm_client,
            email_client,
        })
    }

    /// 处理收到的邮件
    ///
    /// 实现里程碑 4 的主工作流程：
    /// 1. 分析邮件内容
    /// 2. 检索相关记忆
    /// 3. 生成智能回复
    /// 4. 更新记忆数据
    pub async fn process_incoming_email(
        &self,
        user_id: &str,
        email_content: &str,
        sender_email: &str,
    ) -> Result<String> {
        info!(
            user_id = %user_id,
            sender = %sender_email,
            content_length = email_content.len(),
            "Starting email processing workflow"
        );

        // 步骤 1: 分析邮件内容
        let analysis_result = self.analyze_email_content(email_content).await?;
        debug!(analysis = %analysis_result, "Email analysis completed");

        // 步骤 2: 检索用户记忆
        let user_memory = self.retrieve_user_memory(user_id).await?;
        debug!(
            memory_exists = user_memory.is_some(),
            "User memory retrieval completed"
        );

        // 步骤 3: 生成智能回复
        let reply_content = self
            .generate_smart_reply(email_content, &analysis_result, user_memory.as_ref())
            .await?;

        // 步骤 4: 记录交互历史
        self.record_interaction(user_id, email_content, &reply_content, &analysis_result)
            .await?;

        info!(
            user_id = %user_id,
            reply_length = reply_content.len(),
            "Email processing workflow completed successfully"
        );

        Ok(reply_content)
    }

    /// 发送邮件回复
    #[allow(dead_code)]
    pub async fn send_email_reply(
        &self,
        to_email: &str,
        subject: &str,
        reply_content: &str,
        from_name: Option<&str>,
    ) -> Result<String> {
        info!(
            to = %to_email,
            subject = %subject,
            "Sending email reply"
        );

        let config = get_config();
        let from_email = &config.email.smtp.username;

        let from = match from_name {
            Some(name) => EmailAddress::with_name(from_email.clone(), name.to_string()),
            None => EmailAddress::new(from_email.clone()),
        };

        let to = vec![EmailAddress::new(to_email.to_string())];
        let body = EmailBody::text(reply_content.to_string());
        let message = OutgoingMessage::new(from, to, subject.to_string(), body);

        let message_id = self
            .email_client
            .send_message(&message)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        info!(
            message_id = %message_id,
            "Email sent successfully"
        );

        Ok(message_id.to_string())
    }

    /// 分析邮件内容
    async fn analyze_email_content(&self, email_content: &str) -> Result<String> {
        let mut context = HashMap::new();
        context.insert("email_body".to_string(), json!(email_content));

        let request = LlmRequest::new("email_analysis.default".to_string(), context);
        let response = self.llm_client.generate_response(&request).await?;

        Ok(response.content)
    }

    /// 检索用户记忆
    async fn retrieve_user_memory(&self, user_id: &str) -> Result<Option<MemoryCorpus>> {
        match MemoryDataAccess::get_memory_corpus_by_user_id(user_id).await {
            Ok(memory) => Ok(memory),
            Err(e) => {
                warn!(
                    user_id = %user_id,
                    error = %e,
                    "Failed to retrieve user memory, will proceed without memory context"
                );
                Ok(None)
            }
        }
    }

    /// 生成智能回复
    async fn generate_smart_reply(
        &self,
        original_email: &str,
        analysis_result: &str,
        user_memory: Option<&MemoryCorpus>,
    ) -> Result<String> {
        let mut context = HashMap::new();
        context.insert("email_body".to_string(), json!(original_email));
        context.insert("analysis".to_string(), json!(analysis_result));

        // 如果有用户记忆，添加个人化上下文
        if let Some(memory) = user_memory {
            context.insert("user_profile".to_string(), json!(memory.core_profile));
            // 注意：episodic_memory 是 Vec<EpisodicMemory>，不是 interaction_log
            context.insert(
                "recent_memories".to_string(),
                json!(memory.episodic_memory.iter().take(5).collect::<Vec<_>>()),
            );
        }

        let request = LlmRequest::new("smart_reply.default".to_string(), context);
        let response = self.llm_client.generate_response(&request).await?;

        Ok(response.content)
    }

    /// 记录交互历史
    async fn record_interaction(
        &self,
        user_id: &str,
        original_email: &str,
        reply_content: &str,
        _analysis_result: &str,
    ) -> Result<()> {
        // 创建交互记录
        let inbound_interaction = InteractionLog {
            id: None,
            user_id: user_id.to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            direction: MessageDirection::UserToSystem,
            content: original_email.to_string(),
            metadata: HashMap::new(),
        };

        let outbound_interaction = InteractionLog {
            id: None,
            user_id: user_id.to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            direction: MessageDirection::SystemToUser,
            content: reply_content.to_string(),
            metadata: HashMap::new(),
        };

        // 保存交互记录 - 使用静态方法
        if let Err(e) = MemoryDataAccess::log_interaction(&inbound_interaction).await {
            warn!(
                user_id = %user_id,
                error = %e,
                "Failed to save inbound interaction"
            );
        }

        if let Err(e) = MemoryDataAccess::log_interaction(&outbound_interaction).await {
            warn!(
                user_id = %user_id,
                error = %e,
                "Failed to save outbound interaction"
            );
        }

        info!(
            user_id = %user_id,
            "Interaction history recorded"
        );

        Ok(())
    }

    /// 创建或更新用户档案
    pub async fn create_or_update_user_profile(
        &self,
        user_id: &str,
        name: Option<String>,
        _email: String,
    ) -> Result<()> {
        info!(
            user_id = %user_id,
            "Creating or updating user profile"
        );

        // 检查是否已存在用户记忆
        match MemoryDataAccess::get_memory_corpus_by_user_id(user_id).await {
            Ok(Some(_)) => {
                debug!(user_id = %user_id, "User memory corpus already exists");
                return Ok(());
            }
            Ok(None) => {
                debug!(user_id = %user_id, "Creating new user memory corpus");
            }
            Err(e) => {
                warn!(
                    user_id = %user_id,
                    error = %e,
                    "Error checking existing memory corpus, will create simple profile"
                );
            }
        }

        // 暂时记录用户信息到日志，完整的用户档案创建将在后续版本实现
        info!(
            user_id = %user_id,
            name = ?name,
            "User profile information recorded (full persistence pending)"
        );

        Ok(())
    }
}

/// 简化的工作流程演示
pub async fn demonstrate_workflow() -> Result<()> {
    info!("Starting workflow demonstration...");

    // 初始化工作流程
    let workflow = match EmailWorkflow::new().await {
        Ok(w) => w,
        Err(e) => {
            warn!(
                error = %e,
                "Failed to initialize workflow (expected if services are not configured)"
            );
            return Ok(());
        }
    };

    // 演示邮件处理流程
    let demo_user = "demo@example.com";
    let demo_email = "你好！我想了解一下贵公司的产品服务，请问可以安排一个会议吗？";

    // 创建用户档案
    if let Err(e) = workflow
        .create_or_update_user_profile(
            demo_user,
            Some("Demo User".to_string()),
            demo_user.to_string(),
        )
        .await
    {
        warn!(error = %e, "Failed to create user profile in demo");
    }

    // 处理邮件
    match workflow
        .process_incoming_email(demo_user, demo_email, demo_user)
        .await
    {
        Ok(reply) => {
            info!(
                reply_preview = %reply.chars().take(100).collect::<String>(),
                "Workflow demonstration completed successfully"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                "Workflow demonstration failed (expected if LLM API is not configured)"
            );
        }
    }

    Ok(())
}
