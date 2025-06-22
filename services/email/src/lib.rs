//! # 邮件服务模块
//!
//! 这个模块提供了 SMTP 邮件发送功能，专注于可靠的邮件发送服务。
//!
//! ## 特性
//!
//! - 异步邮件发送
//! - 支持 TLS/SSL 加密连接
//! - 自动错误处理和验证
//! - 邮件内容验证和净化
//! - 线程安全的客户端实现
//!
//! ## 安全考虑
//!
//! - 邮件地址格式验证
//! - 附件类型验证和大小限制
//! - 防止邮件内容注入
//! - SMTP 凭证安全管理
//!
//! ## 使用示例
//!
//! ```rust
//! use sentio_email::{SmtpClient, SimpleSmtpClient, EmailAddress, EmailBody, OutgoingMessage};
//! use shared_logic::config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 初始化配置
//!     shared_logic::config::initialize_config().await?;
//!     
//!     // 创建 SMTP 客户端
//!     let mut client = SimpleSmtpClient::from_config().await?;
//!     
//!     // 连接到 SMTP 服务器
//!     client.connect().await?;
//!     
//!     // 创建邮件
//!     let from = EmailAddress::with_name(
//!         "sender@example.com".to_string(),
//!         "发件人".to_string()
//!     );
//!     let to = vec![EmailAddress::new("recipient@example.com".to_string())];
//!     let body = EmailBody::text("Hello, World!".to_string());
//!     let message = OutgoingMessage::new(from, to, "测试邮件".to_string(), body);
//!     
//!     // 发送邮件
//!     let message_id = client.send_message(&message).await?;
//!     println!("邮件发送成功，ID: {}", message_id);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

// 重新导出主要类型和 trait
pub use client::{create_smtp_client, SimpleSmtpClient, SmtpClient};
pub use error::{EmailError, EmailResult};
pub use types::{EmailAddress, EmailAttachment, EmailBody, MessageId, OutgoingMessage};
