use anyhow::Result;
use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};
use shared_logic::{config, InteractionLog, MemoryDataAccess, MessageDirection};
use std::collections::HashMap;

// 导入本地模块
mod workflow;
use sentio_core::demonstrate_workflow;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("🚀 程序开始启动...");

    // 第零步：加载 .env 文件
    dotenv::dotenv().ok(); // 忽略错误，因为 .env 文件可能不存在
    eprintln!("✅ .env 文件处理完成");

    // 第一步：初始化全局配置
    eprintln!("📝 开始初始化配置...");
    config::initialize_config().await?;
    eprintln!("✅ 配置初始化完成");

    // 第三步：基于配置初始化遥测系统
    let global_config = config::get_config();
    sentio_telemetry::init_telemetry_with_config(&global_config.telemetry)?;

    // 第三步：打印启动日志
    tracing::info!(
        log_level = ?global_config.telemetry.log_level,
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

    // 演示记忆服务集成
    if let Err(e) = demonstrate_memory_integration().await {
        tracing::warn!(
            error = %e,
            "Memory service demonstration failed"
        );
    }

    // 演示完整的邮件工作流程
    if let Err(e) = demonstrate_workflow().await {
        tracing::warn!(
            error = %e,
            "Workflow demonstration failed (this is expected if services are not fully configured)"
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
        "Demonstrating global config access - Server workers: {}",
        config.server.workers
    );

    // 演示访问提示词配置
    match config.get_prompt("introduction.default") {
        Ok(prompt) => tracing::info!(
            "Successfully accessed prompt 'introduction.default': User prompt starts with '{}...'",
            prompt.user.chars().take(50).collect::<String>()
        ),
        Err(e) => tracing::error!("Failed to access prompt 'introduction.default': {}", e),
    }
}

/// 演示 LLM 服务集成
async fn demonstrate_llm_integration() -> Result<()> {
    tracing::info!("Initializing LLM client...");

    // 创建 LLM 客户端
    let llm_client = DeepSeekClient::new()?;

    // 创建示例请求，使用在 prompts.yaml 中定义的名称
    // 这里我们使用 "introduction.default"，并且不需要任何上下文变量
    let request = LlmRequest::new("introduction.default".to_string(), HashMap::new());

    tracing::info!(
        request_id = %request.id,
        prompt_name = %request.prompt_name,
        "Sending demo request to LLM using configuration-driven prompt"
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

/// 演示记忆服务集成
async fn demonstrate_memory_integration() -> Result<()> {
    tracing::info!("Testing memory service with unified data access...");

    tracing::info!("Memory service demonstration starting");

    // 创建示例交互记录
    use chrono::Utc;

    let interaction = InteractionLog {
        id: None, // 将由数据库自动生成
        user_id: "demo_user_001".to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        direction: MessageDirection::UserToSystem,
        content: "你好，我是新用户，请问你能帮我管理邮件吗？".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    tracing::info!(
        user_id = %interaction.user_id,
        session_id = %interaction.session_id,
        direction = ?interaction.direction,
        "Creating demo interaction log"
    );

    // 保存交互记录
    let interaction_id = MemoryDataAccess::log_interaction(&interaction).await?;

    tracing::info!(
        interaction_id = %interaction_id,
        "Interaction saved successfully"
    );

    // 检索用户的最近交互
    let recent_interactions =
        MemoryDataAccess::get_user_interactions(&interaction.user_id, Some(5), None).await?;

    tracing::info!(
        user_id = %interaction.user_id,
        count = recent_interactions.len(),
        "Retrieved user interactions"
    );

    // 获取用户统计信息
    let user_stats = MemoryDataAccess::get_user_statistics(&interaction.user_id).await?;

    // 输出演示结果
    println!("\n💾 记忆服务演示:");
    println!("用户 ID: {}", interaction.user_id);
    println!("交互 ID: {}", interaction_id);
    println!("历史交互数量: {}", recent_interactions.len());
    println!("交互内容: {}", interaction.content);
    println!("用户总记忆数: {}", user_stats.total_memories);
    println!("用户总交互数: {}", user_stats.total_interactions);
    println!();

    Ok(())
}
