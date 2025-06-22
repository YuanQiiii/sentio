//! # 邮件服务错误类型定义
//! 
//! 这个模块定义了邮件服务中可能出现的所有错误类型。
//! 遵循 GUIDE.md 中的错误处理原则，提供清晰的错误信息和恢复建议。

use std::fmt;

/// 邮件服务的结果类型
pub type EmailResult<T> = Result<T, EmailError>;

/// 邮件服务错误类型
#[derive(Debug)]
pub enum EmailError {
    /// 连接错误 - 无法连接到邮件服务器
    ConnectionError {
        server: String,
        port: u16,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 认证错误 - 用户名或密码错误
    AuthenticationError {
        username: String,
        server: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// 邮件解析错误 - 无法解析邮件内容
    ParseError {
        message_id: Option<String>,
        details: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// 发送错误 - 无法发送邮件
    SendError {
        recipient: String,
        details: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// 配置错误 - 邮件服务配置无效
    ConfigurationError {
        field: String,
        value: String,
        reason: String,
    },
    
    /// 网络超时错误
    TimeoutError {
        operation: String,
        timeout_seconds: u64,
    },
    
    /// TLS/SSL 错误
    TlsError {
        details: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 邮件服务器错误 - 服务器返回错误
    ServerError {
        code: Option<String>,
        message: String,
        server: String,
    },
    
    /// 邮件内容验证错误
    ValidationError {
        field: String,
        value: String,
        reason: String,
    },
    
    /// 权限错误 - 没有权限访问某个邮箱或文件夹
    PermissionError {
        resource: String,
        operation: String,
    },
    
    /// 内部错误 - 不应该发生的错误
    InternalError {
        details: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::ConnectionError { server, port, .. } => {
                write!(f, "无法连接到邮件服务器 {}:{}", server, port)
            }
            
            EmailError::AuthenticationError { username, server, .. } => {
                write!(f, "邮件服务器 {} 认证失败，用户名: {}", server, username)
            }
            
            EmailError::ParseError { message_id, details, .. } => {
                match message_id {
                    Some(id) => write!(f, "解析邮件失败 (ID: {}): {}", id, details),
                    None => write!(f, "解析邮件失败: {}", details),
                }
            }
            
            EmailError::SendError { recipient, details, .. } => {
                write!(f, "发送邮件到 {} 失败: {}", recipient, details)
            }
            
            EmailError::ConfigurationError { field, value, reason } => {
                write!(f, "邮件配置错误，字段 '{}' 值 '{}': {}", field, value, reason)
            }
            
            EmailError::TimeoutError { operation, timeout_seconds } => {
                write!(f, "操作 '{}' 超时 ({}秒)", operation, timeout_seconds)
            }
            
            EmailError::TlsError { details, .. } => {
                write!(f, "TLS/SSL 错误: {}", details)
            }
            
            EmailError::ServerError { code, message, server } => {
                match code {
                    Some(c) => write!(f, "邮件服务器 {} 错误 [{}]: {}", server, c, message),
                    None => write!(f, "邮件服务器 {} 错误: {}", server, message),
                }
            }
            
            EmailError::ValidationError { field, value, reason } => {
                write!(f, "邮件验证错误，字段 '{}' 值 '{}': {}", field, value, reason)
            }
            
            EmailError::PermissionError { resource, operation } => {
                write!(f, "没有权限对 '{}' 执行 '{}' 操作", resource, operation)
            }
            
            EmailError::InternalError { details, .. } => {
                write!(f, "内部错误: {}", details)
            }
        }
    }
}

impl std::error::Error for EmailError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EmailError::ConnectionError { source, .. } 
            | EmailError::TlsError { source, .. } => Some(source.as_ref()),
            
            EmailError::AuthenticationError { source: Some(source), .. }
            | EmailError::ParseError { source: Some(source), .. }
            | EmailError::SendError { source: Some(source), .. }
            | EmailError::InternalError { source: Some(source), .. } => Some(source.as_ref()),
            
            _ => None,
        }
    }
}

impl EmailError {
    /// 检查错误是否可以重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            EmailError::ConnectionError { .. }
            | EmailError::TimeoutError { .. }
            | EmailError::ServerError { .. }
        )
    }
    
    /// 获取重试建议的等待时间（秒）
    pub fn retry_delay_seconds(&self) -> u64 {
        match self {
            EmailError::ConnectionError { .. } => 5,
            EmailError::TimeoutError { .. } => 3,
            EmailError::ServerError { .. } => 10,
            _ => 0,
        }
    }
    
    /// 检查是否是致命错误（不应该重试）
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            EmailError::AuthenticationError { .. }
            | EmailError::ConfigurationError { .. }
            | EmailError::ValidationError { .. }
            | EmailError::PermissionError { .. }
        )
    }
}
