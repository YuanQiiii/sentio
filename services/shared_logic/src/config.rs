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
//! SENTIO_DATABASE__URL=mongodb://localhost:27017/sentio
//! SENTIO_LLM__API_KEY=your-api-key
//! SENTIO_TELEMETRY__LOG_LEVEL=debug
//! ```

use anyhow::Result;
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::OnceLock;

// 全局配置实例
static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

/// 系统配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 邮件服务配置  
    pub email: EmailConfig,
    /// LLM API配置
    pub llm: LlmConfig,
    /// 遥测配置
    pub telemetry: TelemetryConfig,
    /// 服务器配置
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库连接URL
    pub url: String,
    /// 连接池最大连接数
    pub max_connections: u32,
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
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
    /// 从环境变量和配置文件加载配置
    pub fn from_env() -> Result<Self> {
        let settings = ConfigBuilder::builder()
            // 首先设置默认值
            .set_default("database.url", "mongodb://localhost:27017/sentio")?
            .set_default("database.max_connections", 10)?
            .set_default("database.connect_timeout", 30)?
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
            // 遥测默认配置
            .set_default("telemetry.log_level", "info")?
            .set_default("telemetry.console", true)?
            .set_default("telemetry.json_format", false)?
            // 服务器默认配置
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("server.workers", 4)?
            // 从配置文件加载（可选）
            .add_source(File::with_name("Config").required(false))
            // 从环境变量覆盖，使用 SENTIO_ 前缀（环境变量优先级最高）
            .add_source(
                Environment::with_prefix("SENTIO")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let config: Config = settings.try_deserialize()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "mongodb://localhost:27017/sentio".to_string(),
                max_connections: 10,
                connect_timeout: 30,
            },
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
        }
    }
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
///     println!("Database URL: {}", config.database.url);
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

/// 获取全局配置的只读引用
///
/// 这个函数提供对全局配置的线程安全访问。配置必须先通过
/// [`initialize_config`] 初始化，否则会 panic。
///
/// # Panics
///
/// 如果全局配置尚未初始化，这个函数会 panic。
///
/// # 示例
///
/// ```rust
/// use shared_logic::config;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     config::initialize_config().await?;
///     let config = config::get_config();
///     println!("LLM Provider: {}", config.llm.provider);
///     Ok(())
/// }
/// ```
pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG
        .get()
        .expect("Global config has not been initialized. Call initialize_config() first.")
}

/// 尝试获取全局配置的只读引用
///
/// 这是 [`get_config`] 的安全版本，如果配置未初始化会返回 None
/// 而不是 panic。
///
/// # 返回值
///
/// - `Some(&Config)` - 如果配置已初始化
/// - `None` - 如果配置尚未初始化
///
/// # 示例
///
/// ```rust
/// use shared_logic::config;
///
/// if let Some(config) = config::try_get_config() {
///     println!("Database URL: {}", config.database.url);
/// } else {
///     println!("Configuration not yet initialized");
/// }
/// ```
pub fn try_get_config() -> Option<&'static Config> {
    GLOBAL_CONFIG.get()
}

/// 检查全局配置是否已经初始化
///
/// # 返回值
///
/// - `true` - 如果配置已初始化
/// - `false` - 如果配置尚未初始化
///
/// # 示例
///
/// ```rust
/// use shared_logic::config;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     if !config::is_initialized() {
///         config::initialize_config().await?;
///     }
///     Ok(())
/// }
/// ```
pub fn is_initialized() -> bool {
    GLOBAL_CONFIG.get().is_some()
}
