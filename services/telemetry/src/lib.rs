//! # 遥测服务
//!
//! 提供日志记录、度量和追踪功能，帮助监控和调试应用。

use anyhow::Result;
use shared_logic::config::{LogLevel, TelemetryConfig};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use std::path::PathBuf;

/// 初始化遥测系统
///
/// 这个函数会设置基于 tracing 的结构化日志系统。
/// 日志会输出到控制台，格式为人类可读的格式。
/// 日志级别由配置文件或环境变量控制。
pub fn init_telemetry() -> Result<()> {
    // 创建环境过滤器，默认为 info 级别
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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
/// - 日志文件输出
pub fn init_telemetry_with_config(config: &TelemetryConfig, log_dir_path: Option<&PathBuf>) -> Result<(Box<dyn tracing::Subscriber + Send + Sync>, Option<WorkerGuard>)> {
    // 根据配置设置日志级别
    let level_filter = match config.log_level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level_filter));

    let console_layer = if config.console {
        if config.json_format {
            Some(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(false)
                    .with_span_list(false)
                    .boxed(),
            )
        } else {
            Some(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_file(true)
                    .with_line_number(true)
                    .boxed(),
            )
        }
    } else {
        None
    };

    let mut guard = None;
    let file_layer = if let Some(log_file) = &config.log_file {
        let appender_dir = if let Some(dir) = log_dir_path {
            dir
        } else {
            // Fallback to a default directory if not provided (e.g., for non-test calls)
            &PathBuf::from("logs")
        };
        let file_appender = tracing_appender::rolling::daily(appender_dir, log_file);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        guard = Some(_guard);
        Some(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .json()
                .boxed(),
        )
    } else {
        None
    };

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    Ok((Box::new(registry), guard))
}


/// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use shared_logic::config::TelemetryConfig;
    use std::fs;
    use tempfile::tempdir;
    use tracing::info;

    #[test]
    fn test_init_telemetry_with_config_console_human() {
        let config = TelemetryConfig {
            log_level: LogLevel::Info,
            console: true,
            log_file: None,
            json_format: false,
        };

        let (registry, _guard) = init_telemetry_with_config(&config, None).unwrap();
        tracing::subscriber::with_default(registry, || {
            info!("This is an info message");
        });
    }

    #[test]
    fn test_init_telemetry_with_config_console_json() {
        let config = TelemetryConfig {
            log_level: LogLevel::Debug,
            console: true,
            log_file: None,
            json_format: true,
        };

        let (registry, _guard) = init_telemetry_with_config(&config, None).unwrap();
        tracing::subscriber::with_default(registry, || {
            tracing::debug!("This is a debug message in JSON");
        });
    }

    #[test]
    fn test_init_telemetry_with_config_file_logging() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().join("logs");
        fs::create_dir_all(&log_dir).unwrap(); // Ensure log directory exists
        let log_file_name = "app.log";
        let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let full_log_path = log_dir.join(format!("{}.{}", log_file_name, current_date));

        let config = TelemetryConfig {
            log_level: LogLevel::Info,
            console: false,
            log_file: Some(log_file_name.to_string()),
            json_format: true,
        };

        let (registry, guard_option) = init_telemetry_with_config(&config, Some(&log_dir)).unwrap();
        tracing::subscriber::with_default(registry, || {
            info!(message = "log to file", component = "test");
        });

        // 注意：由于日志记录是异步的，我们可能需要稍微等待一下
        std::thread::sleep(std::time::Duration::from_millis(100));

        let log_content = fs::read_to_string(&full_log_path)
            .expect("Failed to read log file");
        
        assert!(log_content.contains("log to file"));
        assert!(log_content.contains(r#""component":"test""#));
        drop(guard_option); // Ensure guard is dropped to flush logs
    }
}