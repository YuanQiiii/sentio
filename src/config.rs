use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::path::Path;

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub email: EmailConfig,
    pub llm: LlmConfig,
    pub telemetry: TelemetryConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp: SmtpConfig,
    pub allowed_sender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub timeout: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub log_level: String,
    pub console: bool,
    pub log_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn load() -> Result<Self> {
        let config_path = Path::new("sentio.toml");
        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file 'sentio.toml' not found. Please copy sentio.example.toml to sentio.toml and configure it."
            ));
        }
        
        Self::from_file(config_path)
    }

    pub fn get_email_analysis_prompt(&self) -> (&str, &str) {
        (
            "你是一个邮件分类专家。请分析邮件内容并进行分类。\n\n分类包括：\n- 工作相关\n- 个人事务\n- 营销推广\n- 系统通知\n- 垃圾邮件\n- 其他",
            "请分析以下邮件并进行分类：\n\n{email_content}"
        )
    }

    pub fn get_email_reply_prompt(&self) -> (&str, &str) {
        (
            "你是一位专业的邮件回复助手。请根据邮件内容生成合适的回复。",
            "请为以下邮件生成合适的回复：\n\n原始邮件：\n{original_email}\n\n分析结果：\n{analysis_result}"
        )
    }
}

pub async fn initialize() -> Result<()> {
    let config = Config::load()?;
    GLOBAL_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Config already initialized"))?;
    Ok(())
}

pub fn get() -> &'static Config {
    GLOBAL_CONFIG.get().expect("Config not initialized")
}