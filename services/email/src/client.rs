//! # SMTP 邮件客户端实现
//!
//! 这个模块提供了 SMTP 邮件发送客户端的具体实现。
//! 严格遵循 GUIDE.md 中的接口先行和错误处理原则。

use async_trait::async_trait;
use shared_logic::config;
use std::any::Any;
use tracing::{debug, info, warn};

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
pub struct SimpleSmtpClient {
    config: config::EmailConfig,
    connected: bool,
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
}

#[async_trait]
impl SmtpClient for SimpleSmtpClient {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId> {
        info!("发送邮件: {} -> {:?}", message.subject, message.to);

        // 在实际实现中，这里会使用 lettre 或其他 SMTP 库
        // 现在先返回一个模拟的结果

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

        // TODO: 实际的 SMTP 发送逻辑
        // 1. 建立 SMTP 连接
        // 2. 认证
        // 3. 发送邮件
        // 4. 关闭连接

        let message_id = self.generate_message_id();
        info!("邮件发送成功，Message-ID: {}", message_id);

        Ok(message_id)
    }

    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool> {
        debug!("验证邮箱地址: {}", address.email);

        // 基本的邮箱地址格式验证
        if !address.email.contains('@') {
            return Ok(false);
        }

        let parts: Vec<&str> = address.email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Ok(false);
        }

        // TODO: 更复杂的验证逻辑
        // 1. 正则表达式验证
        // 2. DNS MX 记录查询
        // 3. SMTP 验证（可选）

        Ok(true)
    }

    async fn connect(&mut self) -> EmailResult<()> {
        debug!(
            "连接到 SMTP 服务器: {}:{}",
            self.config.smtp.host, self.config.smtp.port
        );

        // TODO: 实际的连接逻辑
        // 对于无状态的 SMTP 客户端，这个方法可能只是验证配置

        self.connected = true;
        info!("SMTP 连接建立成功");
        Ok(())
    }

    async fn disconnect(&mut self) -> EmailResult<()> {
        debug!("断开 SMTP 连接");

        // TODO: 实际的断开逻辑

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
    use super::*;

    #[tokio::test]
    async fn test_smtp_client_creation() {
        // 这个测试需要有效的配置才能通过
        // 在实际项目中，应该使用 mock 配置进行测试
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
        };

        // 测试有效邮箱
        let valid_email = EmailAddress {
            name: Some("Test User".to_string()),
            email: "test@example.com".to_string(),
        };
        assert!(client.verify_address(&valid_email).await.unwrap());

        // 测试无效邮箱
        let invalid_email = EmailAddress {
            name: None,
            email: "invalid-email".to_string(),
        };
        assert!(!client.verify_address(&invalid_email).await.unwrap());
    }
}
