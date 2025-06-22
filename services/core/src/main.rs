use anyhow::Result;
use sentio_telemetry;
use shared_logic::config;

#[tokio::main]
async fn main() -> Result<()> {
    // 第一步：初始化全局配置
    config::initialize_config().await?;

    // 第二步：基于配置初始化遥测系统
    let global_config = config::get_config();
    sentio_telemetry::init_telemetry_with_config(&global_config.telemetry)?;

    // 第三步：打印启动日志
    tracing::info!(
        log_level = ?global_config.telemetry.log_level,
        database_url = %global_config.database.url,
        llm_provider = %global_config.llm.provider,
        server_host = %global_config.server.host,
        server_port = %global_config.server.port,
        "Configuration loaded successfully. System starting."
    );

    // 展示一些配置加载的详细信息
    tracing::debug!(
        smtp_host = %global_config.email.smtp.host,
        "Email configuration loaded"
    );

    tracing::debug!(
        model = %global_config.llm.model,
        timeout = %global_config.llm.timeout,
        max_retries = %global_config.llm.max_retries,
        "LLM configuration loaded"
    );

    // 演示在程序其他地方如何访问全局配置
    demonstrate_global_config_access();

    // 程序正常退出
    tracing::info!("System shutdown completed.");
    Ok(())
}

/// 演示如何在应用的任何地方访问全局配置
fn demonstrate_global_config_access() {
    let config = config::get_config();
    tracing::info!(
        "Demonstrating global config access - Database max connections: {}",
        config.database.max_connections
    );
}
