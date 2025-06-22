//! # 邮件服务类型定义
//! 
//! 这个模块定义了邮件服务中使用的所有数据类型。
//! 专注于 SMTP 邮件发送功能，遵循 GUIDE.md 中的类型安全和清晰命名原则。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 邮件地址
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress {
    /// 邮件地址 (example@domain.com)
    pub email: String,
    /// 显示名称 (可选)
    pub name: Option<String>,
}

impl EmailAddress {
    /// 创建一个新的邮件地址
    pub fn new(email: String) -> Self {
        Self { email, name: None }
    }
    
    /// 创建一个带显示名称的邮件地址
    pub fn with_name(email: String, name: String) -> Self {
        Self { 
            email, 
            name: Some(name),
        }
    }
    
    /// 验证邮件地址格式是否有效
    pub fn is_valid(&self) -> bool {
        // 简单的邮件地址验证
        self.email.contains('@') && 
        self.email.contains('.') &&
        !self.email.starts_with('@') &&
        !self.email.ends_with('@')
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}

/// 邮件消息 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl MessageId {
    /// 创建新的消息 ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 邮件正文内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBody {
    /// 纯文本内容
    pub text: Option<String>,
    /// HTML 内容（已净化）
    pub html: Option<String>,
    /// 原始内容类型
    pub content_type: String,
}

impl EmailBody {
    /// 创建纯文本邮件正文
    pub fn text(content: String) -> Self {
        Self {
            text: Some(content),
            html: None,
            content_type: "text/plain".to_string(),
        }
    }
    
    /// 创建 HTML 邮件正文
    pub fn html(content: String) -> Self {
        Self {
            text: None,
            html: Some(content),
            content_type: "text/html".to_string(),
        }
    }
    
    /// 获取最佳显示内容
    pub fn get_display_content(&self) -> Option<&String> {
        self.text.as_ref().or(self.html.as_ref())
    }
    
    /// 检查是否有内容
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.html.is_none()
    }
}

/// 邮件附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    /// 文件名
    pub filename: String,
    /// MIME 类型
    pub content_type: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 内容 ID（用于内嵌图片等）
    pub content_id: Option<String>,
    /// 是否是内嵌附件
    pub is_inline: bool,
}

impl EmailAttachment {
    /// 检查附件类型是否安全
    pub fn is_safe_type(&self) -> bool {
        // 定义安全的文件类型
        const SAFE_TYPES: &[&str] = &[
            "text/plain",
            "text/html",
            "image/jpeg",
            "image/png",
            "image/gif",
            "application/pdf",
        ];
        
        SAFE_TYPES.contains(&self.content_type.as_str())
    }
    
    /// 检查文件大小是否合理（< 10MB）
    pub fn is_reasonable_size(&self) -> bool {
        self.size < 10 * 1024 * 1024 // 10MB
    }
}

/// 要发送的邮件消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    /// 发件人
    pub from: EmailAddress,
    /// 收件人列表
    pub to: Vec<EmailAddress>,
    /// 抄送列表
    pub cc: Vec<EmailAddress>,
    /// 密送列表
    pub bcc: Vec<EmailAddress>,
    /// 邮件主题
    pub subject: String,
    /// 邮件正文
    pub body: EmailBody,
    /// 附件列表
    pub attachments: Vec<EmailAttachment>,
    /// 回复的原始邮件 ID
    pub in_reply_to: Option<MessageId>,
    /// 自定义邮件头
    pub headers: HashMap<String, String>,
}

impl OutgoingMessage {
    /// 创建一个新的待发送邮件
    pub fn new(from: EmailAddress, to: Vec<EmailAddress>, subject: String, body: EmailBody) -> Self {
        Self {
            from,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            subject,
            body,
            attachments: Vec::new(),
            in_reply_to: None,
            headers: HashMap::new(),
        }
    }
    
    /// 添加抄送收件人
    pub fn add_cc(mut self, cc: EmailAddress) -> Self {
        self.cc.push(cc);
        self
    }
    
    /// 添加密送收件人
    pub fn add_bcc(mut self, bcc: EmailAddress) -> Self {
        self.bcc.push(bcc);
        self
    }
    
    /// 设置为回复邮件
    pub fn reply_to(mut self, original_message_id: MessageId) -> Self {
        self.in_reply_to = Some(original_message_id);
        // 如果主题不是以 "Re:" 开头，则添加
        if !self.subject.to_lowercase().starts_with("re:") {
            self.subject = format!("Re: {}", self.subject);
        }
        self
    }
    
    /// 添加附件
    pub fn add_attachment(mut self, attachment: EmailAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }
    
    /// 添加自定义邮件头
    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    /// 验证邮件是否可以发送
    pub fn validate(&self) -> Result<(), String> {
        if !self.from.is_valid() {
            return Err(format!("无效的发件人地址: {}", self.from.email));
        }
        
        if self.to.is_empty() {
            return Err("收件人列表不能为空".to_string());
        }
        
        for addr in &self.to {
            if !addr.is_valid() {
                return Err(format!("无效的收件人地址: {}", addr.email));
            }
        }
        
        for addr in &self.cc {
            if !addr.is_valid() {
                return Err(format!("无效的抄送地址: {}", addr.email));
            }
        }
        
        for addr in &self.bcc {
            if !addr.is_valid() {
                return Err(format!("无效的密送地址: {}", addr.email));
            }
        }
        
        if self.subject.trim().is_empty() {
            return Err("邮件主题不能为空".to_string());
        }
        
        if self.body.is_empty() {
            return Err("邮件内容不能为空".to_string());
        }
        
        // 检查附件
        for attachment in &self.attachments {
            if !attachment.is_safe_type() {
                return Err(format!("不安全的附件类型: {}", attachment.content_type));
            }
            
            if !attachment.is_reasonable_size() {
                return Err(format!("附件 {} 过大", attachment.filename));
            }
        }
        
        Ok(())
    }
}