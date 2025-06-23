//! # LLM 服务错误处理
//!
//! 定义了 LLM 服务的所有错误类型，遵循"健壮性是底线"的原则。
//! 所有外部 API 响应都被视为不可信任，需要验证。

use thiserror::Error;

/// LLM 服务错误类型
#[derive(Error, Debug)]
pub enum LlmError {
    /// API 请求失败
    #[error("API request failed: {message}")]
    ApiRequestFailed { message: String },

    /// API 响应无效
    #[error("Invalid API response: {details}")]
    InvalidApiResponse { details: String },

    /// 认证失败
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// 配置错误
    #[error("Configuration error: {field}")]
    ConfigurationError { field: String },

    /// 提示词未找到
    #[error("Prompt not found: {name}")]
    PromptNotFound { name: String },

    /// 内部错误，用于包装来自其他模块的错误
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// 序列化/反序列化错误
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    /// 网络错误
    #[error("Network error: {source}")]
    NetworkError {
        #[from]
        source: reqwest::Error,
    },

    /// 超时错误
    #[error("Request timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// 重试次数耗尽
    #[error("Max retries ({max_retries}) exceeded")]
    MaxRetriesExceeded { max_retries: u32 },

    /// 内容过滤错误
    #[error("Content filtered by safety system: {reason}")]
    ContentFiltered { reason: String },

    /// 令牌限制错误
    #[error("Token limit exceeded: {limit}")]
    TokenLimitExceeded { limit: u32 },
}

/// LLM 服务操作结果类型
pub type LlmResult<T> = Result<T, LlmError>;

impl From<anyhow::Error> for LlmError {
    fn from(err: anyhow::Error) -> Self {
        LlmError::InternalError { message: err.to_string() }
    }
}

impl LlmError {
    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::NetworkError { .. }
                | LlmError::Timeout { .. }
                | LlmError::ApiRequestFailed { .. }
        )
    }

    /// 获取错误代码，用于日志记录
    pub fn error_code(&self) -> &'static str {
        match self {
            LlmError::ApiRequestFailed { .. } => "API_REQUEST_FAILED",
            LlmError::InvalidApiResponse { .. } => "INVALID_API_RESPONSE",
            LlmError::AuthenticationFailed { .. } => "AUTHENTICATION_FAILED",
            LlmError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            LlmError::PromptNotFound { .. } => "PROMPT_NOT_FOUND",
            LlmError::InternalError { .. } => "INTERNAL_ERROR",
            LlmError::SerializationError { .. } => "SERIALIZATION_ERROR",
            LlmError::NetworkError { .. } => "NETWORK_ERROR",
            LlmError::Timeout { .. } => "TIMEOUT",
            LlmError::MaxRetriesExceeded { .. } => "MAX_RETRIES_EXCEEDED",
            LlmError::ContentFiltered { .. } => "CONTENT_FILTERED",
            LlmError::TokenLimitExceeded { .. } => "TOKEN_LIMIT_EXCEEDED",
        }
    }
}
