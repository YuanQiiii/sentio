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
//! - **配置驱动**: 所有提示词外置于 `prompts.yaml`，支持多种 LLM 提供商
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};
//! use serde_json::json;
//! use std::collections::HashMap;
//! use shared_logic::config;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 在应用启动时必须先初始化配置
//!     // 这会加载 config/default.toml 和 config/prompts.yaml
//!     config::initialize_config().await?;
//!
//!     // 初始化客户端
//!     let client = DeepSeekClient::new()?;
//!     
//!     // 准备请求上下文
//!     let mut context = HashMap::new();
//!     context.insert("email_body".to_string(), json!("你好，下周的会议时间可以调整到周三下午吗？"));
//!
//!     // 创建请求，使用在 prompts.yaml 中定义的提示词
//!     let request = LlmRequest::new("email_analysis".to_string(), context);
//!     
//!     // 生成响应
//!     let response = client.generate_response(&request).await?;
//!     println!("分析结果: {}", response.content);
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
