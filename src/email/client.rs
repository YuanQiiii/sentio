use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use thiserror::Error;

use super::EmailAddress;
use crate::config::SmtpConfig;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Send error: {0}")]
    Send(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type EmailResult<T> = Result<T, EmailError>;

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
    pub is_html: bool,
}

#[async_trait]
pub trait EmailClient: Send + Sync {
    async fn send(&self, message: EmailMessage) -> EmailResult<String>;
}

pub struct SmtpClient {
    transport: SmtpTransport,
}

impl SmtpClient {
    pub fn new(config: SmtpConfig) -> EmailResult<Self> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());
        
        let transport = if config.use_tls {
            SmtpTransport::relay(&config.host)
                .map_err(|e| EmailError::Connection(e.to_string()))?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.host)
                .port(config.port)
                .credentials(creds)
                .build()
        };

        Ok(Self { transport })
    }

    fn build_message(&self, msg: &EmailMessage) -> EmailResult<Message> {
        let from_mailbox = Mailbox::new(
            msg.from.name.clone(),
            msg.from.email.parse().map_err(|e| EmailError::Validation(format!("Invalid from address: {}", e)))?,
        );

        let mut message = Message::builder()
            .from(from_mailbox)
            .subject(&msg.subject);

        for to_addr in &msg.to {
            let to_mailbox = Mailbox::new(
                to_addr.name.clone(),
                to_addr.email.parse().map_err(|e| EmailError::Validation(format!("Invalid to address: {}", e)))?,
            );
            message = message.to(to_mailbox);
        }

        let content_type = if msg.is_html {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        message
            .header(content_type)
            .body(msg.body.clone())
            .map_err(|e| EmailError::Validation(e.to_string()))
    }
}

#[async_trait]
impl EmailClient for SmtpClient {
    async fn send(&self, message: EmailMessage) -> EmailResult<String> {
        let email = self.build_message(&message)?;
        
        self.transport
            .send(&email)
            .map(|_| "Email sent successfully".to_string())
            .map_err(|e| EmailError::Send(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_creation() {
        let addr = EmailAddress::new("test@example.com");
        assert_eq!(addr.email, "test@example.com");
        assert!(addr.name.is_none());

        let addr_with_name = EmailAddress::with_name("test@example.com", "Test User");
        assert_eq!(addr_with_name.email, "test@example.com");
        assert_eq!(addr_with_name.name.as_deref(), Some("Test User"));
    }
}