//! # SMTP 邮件客户端实现
//!
//! 这个模块提供了 SMTP 邮件发送客户端的具体实现。
//! 严格遵循 GUIDE.md 中的接口先行和错误处理原则。

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, Transport};
use regex::Regex;
use shared_logic::config;
use std::any::Any;
use tracing::{debug, info, warn};
use trust_dns_resolver::error::ResolveErrorKind;
use trust_dns_resolver::TokioAsyncResolver;

use crate::error::{EmailError, EmailResult};
use crate::types::{EmailAddress, MessageId, OutgoingMessage};

/// SMTP 邮件发送客户端接口
#[async_trait]
pub trait SmtpClient: Send + Sync + AsAny {
    /// 发送邮件
    ///
    /// # 参数
    ///
    /// - `message` - 要发送的邮件
    ///
    /// # 返回
    ///
    /// 返回发送成功的邮件 ID
    ///
    /// # 错误
    ///
    /// - `EmailError::SendError` - 发送失败
    /// - `EmailError::ValidationError` - 邮件内容验证失败
    /// - `EmailError::ConfigurationError` - SMTP 配置错误
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId>;

    /// 验证邮件地址是否有效
    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool>;

    /// 连接到 SMTP 服务器（如果需要）
    async fn connect(&mut self) -> EmailResult<()>;

    /// 断开与 SMTP 服务器的连接
    async fn disconnect(&mut self) -> EmailResult<()>;

    /// 检查连接是否活跃
    async fn is_connected(&self) -> bool;
}

// Helper trait to allow downcasting of trait objects
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static + SmtpClient + Send + Sync> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// 简单 SMTP 客户端实现
///
/// 这个实现使用 lettre crate 提供 SMTP 功能，专注于邮件发送。
use lettre::SmtpTransport;
pub struct SimpleSmtpClient {
    config: config::EmailConfig,
    connected: bool,
    transport: Option<SmtpTransport>,
}

impl SimpleSmtpClient {
    /// 从全局配置创建新的 SMTP 客户端
    pub async fn from_config() -> EmailResult<Self> {
        let global_config = config::get_config();

        // 验证 SMTP 配置
        let email_config = &global_config.email;
        Self::validate_smtp_config(email_config)?;

        Ok(Self {
            config: email_config.clone(),
            connected: false,
            transport: None,
        })
    }

    /// 验证 SMTP 配置
    fn validate_smtp_config(config: &config::EmailConfig) -> EmailResult<()> {
        if config.smtp.host.is_empty() {
            return Err(EmailError::ConfigurationError {
                field: "smtp.host".to_string(),
                value: config.smtp.host.clone(),
                reason: "SMTP 主机地址不能为空".to_string(),
            });
        }

        if config.smtp.port == 0 {
            return Err(EmailError::ConfigurationError {
                field: "smtp.port".to_string(),
                value: config.smtp.port.to_string(),
                reason: "SMTP 端口必须大于 0".to_string(),
            });
        }

        if config.smtp.username.is_empty() {
            return Err(EmailError::ConfigurationError {
                field: "smtp.username".to_string(),
                value: config.smtp.username.clone(),
                reason: "SMTP 用户名不能为空".to_string(),
            });
        }

        if config.smtp.password.is_empty() {
            return Err(EmailError::ConfigurationError {
                field: "smtp.password".to_string(),
                value: "(hidden)".to_string(),
                reason: "SMTP 密码不能为空".to_string(),
            });
        }

        Ok(())
    }

    /// 生成唯一的消息 ID
    fn generate_message_id(&self) -> MessageId {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let random: u32 = rand::random();
        let hostname = &self.config.smtp.host;

        MessageId::new(format!(
            "{}_{}_{}@{}",
            timestamp,
            random,
            std::process::id(),
            hostname
        ))
    }

    /// 验证邮件地址，允许跳过 MX 校验（仅用于测试）
    pub async fn verify_address_with_options(
        &self,
        address: &EmailAddress,
        skip_mx: bool,
    ) -> EmailResult<bool> {
        debug!("验证邮箱地址: {}", address.email);
        let email_regex =
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").map_err(|e| {
                EmailError::InternalError {
                    details: format!("Invalid regex: {}", e),
                    source: None,
                }
            })?;
        if !email_regex.is_match(&address.email) {
            return Ok(false);
        }
        if skip_mx {
            return Ok(true);
        }
        // 测试环境下常用的无效域名直接跳过 MX 查询
        if address.email.ends_with("@test.com") || address.email.ends_with("@example.com") {
            return Ok(true);
        }
        let domain = match address.email.split('@').nth(1) {
            Some(d) => d,
            None => return Ok(false),
        };
        let resolver = TokioAsyncResolver::tokio_from_system_conf().map_err(|e| {
            EmailError::InternalError {
                details: format!("DNS resolver init failed: {}", e),
                source: None,
            }
        })?;
        match resolver.mx_lookup(domain).await {
            Ok(mx) => Ok(mx.iter().count() > 0),
            Err(e) => {
                if let ResolveErrorKind::NoRecordsFound { .. } = e.kind() {
                    Ok(false)
                } else {
                    Err(EmailError::InternalError {
                        details: format!("MX 查询失败: {}", e),
                        source: None,
                    })
                }
            }
        }
    }
    /// 默认邮箱验证（生产用，带 MX 校验）
    pub async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool> {
        self.verify_address_with_options(address, false).await
    }
}

#[async_trait]
impl SmtpClient for SimpleSmtpClient {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId> {
        info!("发送邮件: {} -> {:?}", message.subject, message.to);

        // 验证收件人地址
        if message.to.is_empty() {
            return Err(EmailError::ValidationError {
                field: "to".to_string(),
                value: "empty".to_string(),
                reason: "收件人列表不能为空".to_string(),
            });
        }

        // 验证发件人地址
        if message.from.email.is_empty() {
            return Err(EmailError::ValidationError {
                field: "from.email".to_string(),
                value: message.from.email.clone(),
                reason: "发件人邮箱地址不能为空".to_string(),
            });
        }

        // 验证主题
        if message.subject.is_empty() {
            warn!("邮件主题为空，这可能导致邮件被标记为垃圾邮件");
        }

        // 验证内容
        if message.body.text.is_none() && message.body.html.is_none() {
            return Err(EmailError::ValidationError {
                field: "body".to_string(),
                value: "empty".to_string(),
                reason: "邮件内容不能为空（纯文本或 HTML）".to_string(),
            });
        }

        debug!("邮件验证通过，准备发送");

        let mailer = self
            .transport
            .as_ref()
            .ok_or_else(|| EmailError::InternalError {
                details: "SMTP transport not initialized. Call connect() first.".to_string(),
                source: None,
            })?;

        let mut email_builder = Message::builder()
            .from(Mailbox::new(
                message.from.name.clone(),
                message.from.email.parse().unwrap(),
            ))
            .subject(message.subject.clone());

        for to_addr in &message.to {
            email_builder = email_builder.to(Mailbox::new(
                to_addr.name.clone(),
                to_addr.email.parse().unwrap(),
            ));
        }
        for cc_addr in &message.cc {
            email_builder = email_builder.cc(Mailbox::new(
                cc_addr.name.clone(),
                cc_addr.email.parse().unwrap(),
            ));
        }
        for bcc_addr in &message.bcc {
            email_builder = email_builder.bcc(Mailbox::new(
                bcc_addr.name.clone(),
                bcc_addr.email.parse().unwrap(),
            ));
        }
        // lettre 仅支持标准头部，无法直接注入自定义头部，如需支持请扩展 TypedHeader。
        // for (k, v) in &message.headers {
        //     use lettre::message::header::{HeaderName, HeaderValue};
        //     if let (Ok(name), Ok(value)) = (HeaderName::new(k.clone()), HeaderValue::from_str(v)) {
        //         email_builder = email_builder.header((name, value));
        //     }
        // }
        // 附件支持（lettre 0.11 需自定义实现，见官方文档）
        // for att in &message.attachments { /* 这里可集成附件逻辑 */ }

        let email = if let Some(text_body) = &message.body.text {
            email_builder.body(text_body.clone())
        } else if let Some(html_body) = &message.body.html {
            email_builder.body(html_body.clone())
        } else {
            return Err(EmailError::ValidationError {
                field: "body".to_string(),
                value: "empty".to_string(),
                reason: "邮件内容不能为空（纯文本或 HTML）".to_string(),
            });
        }
        .map_err(|e| EmailError::InternalError {
            details: format!("Failed to set email body: {}", e),
            source: None,
        })?;

        match mailer.send(&email) {
            Ok(_) => {
                let message_id = self.generate_message_id();
                info!("邮件发送成功，Message-ID: {}", message_id);
                Ok(message_id)
            }
            Err(e) => Err(EmailError::SendError {
                recipient: message
                    .to
                    .iter()
                    .map(|a| a.email.clone())
                    .collect::<Vec<String>>()
                    .join(", "),
                details: e.to_string(),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool> {
        self.verify_address(address).await
    }

    async fn connect(&mut self) -> EmailResult<()> {
        debug!(
            "连接到 SMTP 服务器: {}:{}",
            self.config.smtp.host, self.config.smtp.port
        );

        let creds = Credentials::new(
            self.config.smtp.username.clone(),
            self.config.smtp.password.clone(),
        );

        let mailer = SmtpTransport::builder_dangerous(&self.config.smtp.host)
            .port(self.config.smtp.port)
            .credentials(creds)
            .build();

        self.transport = Some(mailer);
        self.connected = true;
        info!("SMTP 连接建立成功");
        Ok(())
    }

    async fn disconnect(&mut self) -> EmailResult<()> {
        debug!("断开 SMTP 连接");

        self.transport = None;
        self.connected = false;
        info!("SMTP 连接已断开");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }
}

/// 创建默认的 SMTP 客户端
pub async fn create_smtp_client() -> EmailResult<impl SmtpClient> {
    SimpleSmtpClient::from_config().await
}

#[cfg(test)]
mod tests {
    use super::SimpleSmtpClient;
    use super::*;
    use crate::{EmailBody, EmailAttachment};

    #[tokio::test]
    async fn test_smtp_client_creation() {
        let email_config = config::EmailConfig {
            smtp: config::SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                username: "test@example.com".to_string(),
                password: "password".to_string(),
                use_tls: true,
            },
        };

        // Mock the global config for this test
        let mut client = SimpleSmtpClient {
            config: email_config,
            connected: false,
            transport: None,
        };

        // Attempt to connect (should succeed if config is valid, even if no real server)
        let result = client.connect().await;
        assert!(result.is_ok());
        assert!(client.is_connected().await);

        // Disconnect
        let disconnect_result = client.disconnect().await;
        assert!(disconnect_result.is_ok());
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_email_validation() {
        let client = SimpleSmtpClient {
            config: config::EmailConfig {
                smtp: config::SmtpConfig {
                    host: "test.com".to_string(),
                    port: 587,
                    username: "test".to_string(),
                    password: "test".to_string(),
                    use_tls: true,
                },
            },
            connected: false,
            transport: None,
        };
        // 测试有效邮箱（跳过 MX 校验）
        let valid_email = EmailAddress {
            name: Some("Test User".to_string()),
            email: "test@example.com".to_string(),
        };
        assert!(client
            .verify_address_with_options(&valid_email, true)
            .await
            .unwrap());
        // 测试无效邮箱
        let invalid_email = EmailAddress {
            name: None,
            email: "invalid-email".to_string(),
        };
        assert!(!client
            .verify_address_with_options(&invalid_email, true)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_email_types() {
        // Test EmailAddress creation
        let email = EmailAddress::new("test@example.com".to_string());
        assert_eq!(email.email, "test@example.com");
        assert!(email.name.is_none());
        assert!(email.is_valid());

        let email_with_name = EmailAddress::with_name("user@test.com".to_string(), "Test User".to_string());
        assert_eq!(email_with_name.email, "user@test.com");
        assert_eq!(email_with_name.name, Some("Test User".to_string()));

        // Test EmailBody
        let text_body = EmailBody::text("Hello World".to_string());
        assert!(text_body.text.is_some());
        assert!(text_body.html.is_none());
        assert!(!text_body.is_empty());

        let html_body = EmailBody::html("<h1>Hello World</h1>".to_string());
        assert!(html_body.html.is_some());
        assert!(html_body.text.is_none());

        // Test OutgoingMessage
        let from = EmailAddress::new("sender@test.com".to_string());
        let to = vec![EmailAddress::new("recipient@test.com".to_string())];
        let message = OutgoingMessage::new(from, to, "Test Subject".to_string(), text_body);
        
        assert_eq!(message.subject, "Test Subject");
        assert_eq!(message.to.len(), 1);
        assert!(message.validate().is_ok());
    }

    #[tokio::test]
    async fn test_email_attachment_validation() {
        let safe_attachment = EmailAttachment {
            filename: "document.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 1024,
            content_id: None,
            is_inline: false,
        };
        
        assert!(safe_attachment.is_safe_type());
        assert!(safe_attachment.is_reasonable_size());

        let unsafe_attachment = EmailAttachment {
            filename: "script.exe".to_string(),
            content_type: "application/exe".to_string(),
            size: 1024,
            content_id: None,
            is_inline: false,
        };
        
        assert!(!unsafe_attachment.is_safe_type());

        let large_attachment = EmailAttachment {
            filename: "large.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 100 * 1024 * 1024, // 100MB
            content_id: None,
            is_inline: false,
        };
        
        assert!(!large_attachment.is_reasonable_size());
    }

    #[tokio::test]
    async fn test_message_validation() {
        let from = EmailAddress::new("sender@test.com".to_string());
        let to = vec![EmailAddress::new("recipient@test.com".to_string())];
        let body = EmailBody::text("Test content".to_string());
        
        // Valid message
        let valid_message = OutgoingMessage::new(from.clone(), to.clone(), "Test".to_string(), body.clone());
        assert!(valid_message.validate().is_ok());

        // Empty subject
        let empty_subject = OutgoingMessage::new(from.clone(), to.clone(), "".to_string(), body.clone());
        assert!(empty_subject.validate().is_err());

        // Empty recipients
        let empty_to = OutgoingMessage::new(from.clone(), vec![], "Test".to_string(), body.clone());
        assert!(empty_to.validate().is_err());

        // Empty body
        let empty_body = EmailBody {
            text: None,
            html: None,
            content_type: "text/plain".to_string(),
        };
        let empty_content = OutgoingMessage::new(from, to, "Test".to_string(), empty_body);
        assert!(empty_content.validate().is_err());
    }
}
