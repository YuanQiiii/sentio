use anyhow::Result;
use shared_logic::config;
use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};

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

    // 演示 LLM 服务集成
    if let Err(e) = demonstrate_llm_integration().await {
        tracing::warn!(
            error = %e,
            "LLM demonstration failed (this is expected if API key is not configured)"
        );
    }

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

/// 演示 LLM 服务集成
async fn demonstrate_llm_integration() -> Result<()> {
    tracing::info!("Initializing LLM client...");
    
    // 创建 LLM 客户端
    let llm_client = DeepSeekClient::new()?;
    
    // 创建示例请求
    let request = LlmRequest::new(
        "你是 Sentio AI 邮件助手，一个专业、友好的智能邮件伙伴。".to_string(),
        "请简单介绍一下自己，说明你的主要功能和特点。".to_string(),
    );
    
    tracing::info!(
        request_id = %request.id,
        "Sending demo request to LLM"
    );
    
    // 发送请求（仅在有效 API 密钥时执行）
    let response = llm_client.generate_response(&request).await?;
    
    tracing::info!(
        request_id = %response.request_id,
        tokens_used = response.token_usage.total_tokens,
        latency_ms = response.metadata.latency_ms,
        "LLM response received successfully"
    );
    
    // 输出响应内容（在实际应用中这应该保存到日志或返回给调用者）
    println!("\n🤖 Sentio AI 回复:");
    println!("{}", response.content);
    println!();
    
    Ok(())
}
