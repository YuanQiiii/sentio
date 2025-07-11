pub mod client;

pub use client::{EmailClient, SmtpClient, EmailMessage};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self { 
            email: email.into(), 
            name: None 
        }
    }

    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}