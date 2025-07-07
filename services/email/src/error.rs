//! # 邮件服务错误类型定义
//!
//! 这个模块定义了邮件服务中可能出现的所有错误类型。
//! 遵循 GUIDE.md 中的错误处理原则，提供清晰的错误信息和恢复建议。

use thiserror::Error;

/// 邮件服务的结果类型
pub type EmailResult<T> = Result<T, EmailError>;

/// 邮件服务错误类型
#[derive(Error, Debug)]
pub enum EmailError {
    /// 连接错误 - 无法连接到邮件服务器
    #[error("无法连接到邮件服务器 {server}:{port}")]
    ConnectionError {
        server: String,
        port: u16,
        #[source] // 使用 #[source] 标记底层错误
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 认证错误 - 用户名或密码错误
    #[error("邮件服务器 {server} 认证失败，用户名: {username}")]
    AuthenticationError {
        username: String,
        server: String,
        #[source] // 使用 #[source] 标记底层错误
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 邮件解析错误 - 无法解析邮件内容
    #[error("解析邮件失败{}: {details}", message_id.as_ref().map_or("".to_string(), |id| format!(" (ID: {})", id)))]
    ParseError {
        message_id: Option<String>,
        details: String,
        #[source] // 使用 #[source] 标记底层错误
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 发送错误 - 无法发送邮件
    #[error("发送邮件到 {recipient} 失败: {details}")]
    SendError {
        recipient: String,
        details: String,
        #[source] // 使用 #[source] 标记底层错误
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 配置错误 - 邮件服务配置无效
    #[error("邮件配置错误，字段 '{field}' 值 '{value}': {reason}")]
    ConfigurationError {
        field: String,
        value: String,
        reason: String,
    },

    /// 网络超时错误
    #[error("操作 '{operation}' 超时 ({timeout_seconds}秒)")]
    TimeoutError {
        operation: String,
        timeout_seconds: u64,
    },

    /// TLS/SSL 错误
    #[error("TLS/SSL 错误: {details}")]
    TlsError {
        details: String,
        #[source] // 使用 #[source] 标记底层错误
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 邮件服务器错误 - 服务器返回错误
    #[error("邮件服务器 {server} 错误{}: {message}", code.as_ref().map_or("".to_string(), |c| format!(" [{}]", c)))]
    ServerError {
        code: Option<String>,
        message: String,
        server: String,
    },

    /// 邮件内容验证错误
    #[error("邮件验证错误，字段 '{field}' 值 '{value}': {reason}")]
    ValidationError {
        field: String,
        value: String,
        reason: String,
    },

    /// 权限错误 - 没有权限访问某个邮箱或文件夹
    #[error("没有权限对 '{resource}' 执行 '{operation}' 操作")]
    PermissionError { resource: String, operation: String },

    /// 内部错误 - 不应该发生的错误
    #[error("内部错误: {details}")]
    InternalError {
        details: String,
        #[source] // 使用 #[source] 标记底层错误
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
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
