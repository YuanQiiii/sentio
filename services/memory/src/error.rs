//! # 记忆服务错误处理
//!
//! 定义记忆服务的所有错误类型，严格遵循"健壮性是底线"原则。
//! 所有数据库操作和外部依赖都被视为可能失败的操作。

use thiserror::Error;

/// 记忆服务错误类型
#[derive(Error, Debug)]
pub enum MemoryError {
    /// 数据库连接失败
    #[error("Database connection failed: {message}")]
    DatabaseConnectionFailed { message: String },

    /// 数据库操作失败
    #[error("Database operation failed: {operation} - {details}")]
    DatabaseOperationFailed { operation: String, details: String },

    /// 文档未找到
    #[error("Document not found: {document_type} with id {id}")]
    DocumentNotFound { document_type: String, id: String },

    /// 数据序列化错误
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: bson::ser::Error,
    },

    /// 数据反序列化错误
    #[error("Deserialization error: {source}")]
    DeserializationError {
        #[from]
        source: bson::de::Error,
    },

    /// MongoDB 特定错误
    #[error("MongoDB error: {source}")]
    MongoError {
        #[from]
        source: mongodb::error::Error,
    },

    /// 配置错误
    #[error("Configuration error: {field}")]
    ConfigurationError { field: String },

    /// 数据验证错误
    #[error("Data validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    /// 并发冲突错误
    #[error("Concurrent modification detected for {resource}")]
    ConcurrencyConflict { resource: String },

    /// 存储容量错误
    #[error("Storage limit exceeded: {limit} for {resource}")]
    StorageLimitExceeded { limit: String, resource: String },

    /// 索引错误
    #[error("Index operation failed: {index_name} - {details}")]
    IndexError { index_name: String, details: String },
}

/// 记忆服务操作结果类型
pub type MemoryResult<T> = Result<T, MemoryError>;

impl MemoryError {
    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            MemoryError::DatabaseConnectionFailed { .. } => true,
            MemoryError::DatabaseOperationFailed { .. } => false, // 需要根据具体错误判断
            MemoryError::MongoError { source } => {
                // 网络错误通常可以重试 - 使用字符串检查替代私有方法
                let error_str = source.to_string().to_lowercase();
                error_str.contains("network")
                    || error_str.contains("timeout")
                    || error_str.contains("connection")
            }
            MemoryError::ConcurrencyConflict { .. } => true,
            _ => false,
        }
    }

    /// 获取错误代码，用于日志记录和监控
    pub fn error_code(&self) -> &'static str {
        match self {
            MemoryError::DatabaseConnectionFailed { .. } => "DB_CONNECTION_FAILED",
            MemoryError::DatabaseOperationFailed { .. } => "DB_OPERATION_FAILED",
            MemoryError::DocumentNotFound { .. } => "DOCUMENT_NOT_FOUND",
            MemoryError::SerializationError { .. } => "SERIALIZATION_ERROR",
            MemoryError::DeserializationError { .. } => "DESERIALIZATION_ERROR",
            MemoryError::MongoError { .. } => "MONGO_ERROR",
            MemoryError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            MemoryError::ValidationError { .. } => "VALIDATION_ERROR",
            MemoryError::ConcurrencyConflict { .. } => "CONCURRENCY_CONFLICT",
            MemoryError::StorageLimitExceeded { .. } => "STORAGE_LIMIT_EXCEEDED",
            MemoryError::IndexError { .. } => "INDEX_ERROR",
        }
    }

    /// 检查是否为严重错误，需要立即报警
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            MemoryError::DatabaseConnectionFailed { .. }
                | MemoryError::ConfigurationError { .. }
                | MemoryError::StorageLimitExceeded { .. }
        )
    }
}
