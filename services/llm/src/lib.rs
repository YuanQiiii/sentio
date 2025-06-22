//! # Sentio LLM 服务
//!
//! 提供与大语言模型交互的核心功能，包括：
//! - 文本生成和对话
//! - 邮件内容分析
//! - 智能回复生成
//! - 推理链执行
//!
//! ## 设计原则
//!
//! - **健壮性是底线**: 所有外部 API 调用都有重试机制和错误处理
//! - **类型安全**: 使用强类型确保运行时安全
//! - **可观测性**: 完整的日志记录和性能监控
//! - **配置驱动**: 支持多种 LLM 提供商，配置外置
//!
//! ## 快速开始
//!
//! ```rust
//! use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 初始化客户端
//!     let client = DeepSeekClient::new()?;
//!     
//!     // 创建请求
//!     let request = LlmRequest::new(
//!         "你是一个专业的助手".to_string(),
//!         "请介绍一下 Rust 语言的特点".to_string(),
//!     );
//!     
//!     // 生成响应
//!     let response = client.generate_response(&request).await?;
//!     println!("回复: {}", response.content);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

// 重新导出核心类型和功能
pub use client::{DeepSeekClient, LlmClient};
pub use error::{LlmError, LlmResult};
pub use types::*;
