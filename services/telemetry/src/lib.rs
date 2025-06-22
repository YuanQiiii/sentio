use anyhow::Result;
use shared_logic::config::{LogLevel, TelemetryConfig};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 初始化遥测系统
/// 
/// 这个函数会设置基于 tracing 的结构化日志系统。
/// 日志会输出到控制台，格式为人类可读的格式。
/// 日志级别由配置文件或环境变量控制。
pub fn init_telemetry() -> Result<()> {
    // 创建环境过滤器，默认为 info 级别
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 创建格式化层，输出到控制台
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true);

    // 初始化全局订阅者
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

/// 基于配置初始化遥测系统
/// 
/// 这个版本允许更细粒度的控制，包括：
/// - 自定义日志级别
/// - JSON 格式输出
/// - 日志文件输出（未来实现）
pub fn init_telemetry_with_config(config: &TelemetryConfig) -> Result<()> {
    // 根据配置设置日志级别
    let level_filter = match config.log_level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug", 
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level_filter));

    // 创建订阅者注册表
    let registry = tracing_subscriber::registry().with(env_filter);

    if config.console {
        if config.json_format {
            // JSON 格式输出
            let fmt_layer = fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(false)
                .with_span_list(false);
            
            registry.with(fmt_layer).init();
        } else {
            // 人类可读格式
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true);
            
            registry.with(fmt_layer).init();
        }
    } else {
        // 如果禁用控制台输出，至少需要一个空的订阅者
        registry.init();
    }

    // TODO: 未来添加文件输出支持
    if let Some(_log_file) = &config.log_file {
        tracing::warn!("File logging is not yet implemented");
    }

    Ok(())
}
