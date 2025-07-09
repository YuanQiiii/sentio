//! # 配置管理模块
//!
//! 这个模块提供了全局的配置管理功能。配置在应用初始化时加载一次，
//! 然后作为只读的全局变量供所有组件使用。
//!
//! ## 特性
//!
//! - 从配置文件和环境变量加载配置
//! - 全局单例模式，保证配置的一致性
//! - 线程安全的配置访问
//! - 环境变量优先级高于配置文件
//!
//! ## 环境变量格式
//!
//! 使用 `SENTIO_` 前缀，嵌套字段用双下划线 `__` 分隔：
//!
//! ```bash
//! SENTIO_LLM__API_KEY=your-api-key
//! SENTIO_LLM__BASE_URL=https://api.example.com
//! SENTIO_TELEMETRY__LOG_LEVEL=debug
//! ```

use anyhow::Result;
use config::{Config as ConfigBuilder, Environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

// 全局配置实例
static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

/// 系统配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 邮件服务配置
    pub email: EmailConfig,
    /// LLM API配置
    pub llm: LlmConfig,
    /// 遥测配置
    pub telemetry: TelemetryConfig,
    /// 服务器配置
    pub server: ServerConfig,
    /// LLM 提示词配置
    #[serde(default)]
    pub prompts: PromptsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP服务器配置
    pub smtp: SmtpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// SMTP服务器地址
    pub host: String,
    /// SMTP服务器端口
    pub port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否使用TLS
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API提供商 (deepseek, openai, etc.)
    pub provider: String,
    /// API密钥
    pub api_key: String,
    /// API基础URL
    pub base_url: String,
    /// 默认模型名称
    pub model: String,
    /// 请求超时时间（秒）
    pub timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

/// LLM 提示词配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptsConfig {
    #[serde(flatten, default)]
    pub prompts: HashMap<String, HashMap<String, Prompt>>,
}

/// 单个提示词模板
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prompt {
    /// 系统提示词
    pub system: String,
    /// 用户提示词
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// 日志级别
    pub log_level: LogLevel,
    /// 是否输出到控制台
    pub console: bool,
    /// 日志文件路径（可选）
    pub log_file: Option<String>,
    /// 是否启用JSON格式日志
    pub json_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务器监听地址
    pub host: String,
    /// 服务器监听端口
    pub port: u16,
    /// 工作线程数
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let mut settings = ConfigBuilder::builder()
            // 邮件默认配置
            .set_default("email.imap.host", "imap.gmail.com")?
            .set_default("email.imap.port", 993)?
            .set_default("email.imap.username", "your-email@example.com")?
            .set_default("email.imap.password", "your-app-password")?
            .set_default("email.imap.use_tls", true)?
            .set_default("email.smtp.host", "smtp.gmail.com")?
            .set_default("email.smtp.port", 587)?
            .set_default("email.smtp.username", "your-email@example.com")?
            .set_default("email.smtp.password", "your-app-password")?
            .set_default("email.smtp.use_tls", true)?
            // LLM默认配置
            .set_default("llm.provider", "deepseek")?
            .set_default("llm.api_key", "your-deepseek-api-key")?
            .set_default("llm.base_url", "https://api.deepseek.com")?
            .set_default("llm.model", "deepseek-chat")?
            .set_default("llm.timeout", 120)?
            .set_default("llm.max_retries", 3)?
            // 数据库默认配置
            .set_default("database.max_connections", 10)?
            // 遥测默认配置
            .set_default("telemetry.log_level", "info")?
            .set_default("telemetry.console", true)?
            .set_default("telemetry.json_format", false)?
            // 服务器默认配置
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("server.workers", 4)?;
            
        // 添加环境变量源，应该覆盖默认值
        settings = settings.add_source(
            Environment::with_prefix("SENTIO")
                .separator("__")  // 使用双下划线作为嵌套字段分隔符
                .prefix_separator("_"), // 前缀与字段之间使用单下划线
        );

        let settings = settings.build()?;
        
        // 加载基本配置
        let mut config: Config = settings.try_deserialize()?;
        
        // 硬编码提示词配置，因为不再从文件加载
        config.prompts = PromptsConfig {
            prompts: Self::default_prompts(),
        };
        
        Ok(config)
    }
    
    /// 获取默认的提示词配置
    fn default_prompts() -> HashMap<String, HashMap<String, Prompt>> {
        let mut prompts = HashMap::new();
        
        // introduction 类别
        let mut introduction = HashMap::new();
        introduction.insert(
            "default".to_string(),
            Prompt {
                system: "你是一个专业的邮件助手，能够分析和回复各种类型的邮件。你具备以下特点：\n\n1. 专业性：能够准确理解邮件内容，识别邮件意图\n2. 高效性：快速生成合适的回复建议\n3. 个性化：根据不同场景调整回复风格\n4. 安全性：保护用户隐私，不泄露敏感信息".to_string(),
                user: "请简单介绍一下自己，说明你的主要功能和特点。".to_string(),
            },
        );
        prompts.insert("introduction".to_string(), introduction);
        
        // email_analysis 类别
        let mut email_analysis = HashMap::new();
        email_analysis.insert(
            "classify".to_string(),
            Prompt {
                system: "你是一个邮件分类专家。请分析邮件内容并进行分类。\n\n分类包括：\n- 工作相关\n- 个人事务\n- 营销推广\n- 系统通知\n- 垃圾邮件\n- 其他".to_string(),
                user: "请分析以下邮件并进行分类：\n\n{email_content}".to_string(),
            },
        );
        email_analysis.insert(
            "extract_key_info".to_string(),
            Prompt {
                system: "你是一个信息提取专家。请从邮件中提取关键信息。".to_string(),
                user: "请从以下邮件中提取关键信息（如日期、地点、人物、事件等）：\n\n{email_content}".to_string(),
            },
        );
        email_analysis.insert(
            "summarize_thread".to_string(),
            Prompt {
                system: "你是一个邮件总结专家。请总结邮件线程的主要内容。".to_string(),
                user: "请总结以下邮件线程的主要内容：\n\n{thread_content}".to_string(),
            },
        );
        prompts.insert("email_analysis".to_string(), email_analysis);
        
        // email_reply 类别
        let mut email_reply = HashMap::new();
        email_reply.insert(
            "generate_response".to_string(),
            Prompt {
                system: "你是一位专业的邮件回复助手。请根据邮件内容生成合适的回复。回复应当：\n1. 礼貌专业\n2. 简洁明了\n3. 针对性强\n4. 符合邮件往来的语境".to_string(),
                user: "请为以下邮件生成合适的回复：\n\n原始邮件：\n{original_email}\n\n分析结果：\n{analysis_result}".to_string(),
            },
        );
        email_reply.insert(
            "suggest_actions".to_string(),
            Prompt {
                system: "你是一个行动建议专家。请根据邮件内容建议合适的后续行动。".to_string(),
                user: "基于以下邮件内容，请建议合适的后续行动：\n\n{email_content}".to_string(),
            },
        );
        prompts.insert("email_reply".to_string(), email_reply);
        
        prompts
    }

    /// 获取指定名称的提示词
    ///
    /// # Panics
    ///
    /// 如果找不到指定名称的提示词，则会 panic。
    pub fn get_prompt(&self, name: &str) -> Result<&Prompt> {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid prompt name format: '{}'. Expected 'category.name'.",
                name
            ));
        }
        let category = parts[0];
        let prompt_name = parts[1];

        self.prompts
            .prompts
            .get(category)
            .and_then(|p| p.get(prompt_name))
            .ok_or_else(|| anyhow::anyhow!("Prompt '{}' not found", name))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            email: EmailConfig {
                smtp: SmtpConfig {
                    host: "smtp.gmail.com".to_string(),
                    port: 587,
                    username: "your-email@example.com".to_string(),
                    password: "your-app-password".to_string(),
                    use_tls: true,
                },
            },
            llm: LlmConfig {
                provider: "deepseek".to_string(),
                api_key: "your-api-key".to_string(),
                base_url: "https://api.deepseek.com".to_string(),
                model: "deepseek-chat".to_string(),
                timeout: 120,
                max_retries: 3,
            },
            telemetry: TelemetryConfig {
                log_level: LogLevel::Info,
                console: true,
                log_file: None,
                json_format: false,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
            },
            prompts: PromptsConfig {
                prompts: HashMap::new(),
            },
        }
    }
}

/// 加载配置
///
/// # 错误
///
/// - 如果配置文件格式错误
/// - 如果环境变量格式错误
#[deprecated(note = "使用 Config::from_env() 代替")]
pub fn load_config() -> Result<Config> {
    Config::from_env()
}

/// 初始化全局配置
///
/// 这个函数应该在应用启动时调用一次。它会加载配置并设置全局配置实例。
/// 如果配置已经初始化过，再次调用会返回错误。
///
/// # 错误
///
/// - 如果配置文件格式错误
/// - 如果环境变量格式错误
/// - 如果全局配置已经被初始化过
///
/// # 示例
///
/// ```rust
/// use shared_logic::config;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     config::initialize_config().await?;
///     
///     let config = config::get_config();
///     println!("Server host: {}", config.server.host);
///     
///     Ok(())
/// }
/// ```
pub async fn initialize_config() -> Result<()> {
    let config = Config::from_env()?;

    GLOBAL_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Global config has already been initialized"))?;

    tracing::info!("Global configuration initialized successfully");
    Ok(())
}

/// 获取对全局配置的引用
///
/// # Panics
///
/// 如果配置尚未初始化，则会 panic。
pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG
        .get()
        .expect("全局配置尚未初始化，请先调用 load_config()")
}

/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // 测试基本的环境变量配置加载
        std::env::set_var("SENTIO_SERVER__HOST", "test-host");
        std::env::set_var("SENTIO_SERVER__PORT", "9999");
        
        let config = Config::from_env().unwrap();
        
        // 清理环境变量
        std::env::remove_var("SENTIO_SERVER__HOST");
        std::env::remove_var("SENTIO_SERVER__PORT");
        
        // 验证环境变量覆盖了默认值
        assert_eq!(config.server.host, "test-host");
        assert_eq!(config.server.port, 9999);
    }

    #[test]
    fn test_llm_config_from_env() {
        // 测试嵌套的 LLM 配置环境变量
        std::env::set_var("SENTIO_LLM__API_KEY", "test-api-key-12345");
        std::env::set_var("SENTIO_LLM__BASE_URL", "https://test.api.com");
        std::env::set_var("SENTIO_LLM__MODEL", "test-model");
        
        let config = Config::from_env().unwrap();
        
        // 清理环境变量
        std::env::remove_var("SENTIO_LLM__API_KEY");
        std::env::remove_var("SENTIO_LLM__BASE_URL");
        std::env::remove_var("SENTIO_LLM__MODEL");
        
        // 验证环境变量覆盖了默认值
        assert_eq!(config.llm.api_key, "test-api-key-12345");
        assert_eq!(config.llm.base_url, "https://test.api.com");
        assert_eq!(config.llm.model, "test-model");
    }

    #[test]
    fn test_default_prompts() {
        let config = Config::from_env().unwrap();
        
        // 测试 introduction.default
        let prompt = config.get_prompt("introduction.default").unwrap();
        assert!(prompt.system.contains("专业的邮件助手"));
        assert!(prompt.user.contains("请简单介绍一下自己"));
        
        // 测试 email_analysis.classify
        let prompt = config.get_prompt("email_analysis.classify").unwrap();
        assert!(prompt.system.contains("邮件分类专家"));
        assert!(prompt.user.contains("{email_content}"));
        
        // 测试 email_reply.generate_response
        let prompt = config.get_prompt("email_reply.generate_response").unwrap();
        assert!(prompt.system.contains("邮件回复助手"));
        assert!(prompt.user.contains("{original_email}"));
    }

    #[test]
    fn test_get_prompt_not_found() {
        let config = Config::from_env().unwrap();
        let result = config.get_prompt("non_existent.prompt");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Prompt 'non_existent.prompt' not found"
        );
    }

    #[test]
    fn test_get_prompt_invalid_format() {
        let config = Config::from_env().unwrap();
        let result = config.get_prompt("invalid_format");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid prompt name format: 'invalid_format'. Expected 'category.name'."
        );
    }
}
