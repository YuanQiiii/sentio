use anyhow::Result;
use std::path::PathBuf;

mod config;
mod telemetry;
mod workflow;
mod email;
mod llm;
mod memory;

use crate::memory::MemoryStore;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("🚀 程序开始启动...");

    // 初始化配置
    eprintln!("📝 开始初始化配置...");
    config::initialize().await?;
    eprintln!("✅ 配置初始化完成");

    // 初始化日志
    let config = config::get();
    let log_dir = PathBuf::from("logs");
    telemetry::init(&config.telemetry, Some(&log_dir))?;

    // 初始化记忆存储
    eprintln!("💾 开始初始化记忆服务...");
    let memory_file_path = PathBuf::from("memory.json");
    MemoryStore::initialize(memory_file_path).await?;
    eprintln!("✅ 记忆服务初始化完成");

    // 打印启动日志
    tracing::info!(
        log_level = ?config.telemetry.log_level,
        llm_provider = %config.llm.provider,
        allowed_sender = %config.email.allowed_sender,
        "Configuration loaded successfully. System starting."
    );

    // 创建工作流
    let _workflow = workflow::create_workflow().await?;
    tracing::info!("Email workflow created. Waiting for emails from {}", config.email.allowed_sender);

    // TODO: 实现邮件监听逻辑
    // 目前只是创建了工作流，实际的邮件接收需要通过IMAP或其他方式实现

    // 程序正常退出
    tracing::info!("System shutdown completed.");
    Ok(())
}