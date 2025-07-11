use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use crate::config::TelemetryConfig;

pub fn init(config: &TelemetryConfig, log_dir: Option<&Path>) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Some(log_file) = &config.log_file {
        let file_path = if let Some(dir) = log_dir {
            dir.join(log_file)
        } else {
            std::path::PathBuf::from(log_file)
        };
        
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
            
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file)
            .with_ansi(false);
            
        subscriber.with(file_layer).init();
    } else {
        subscriber.init();
    }

    tracing::info!("Telemetry initialized with log level: {}", config.log_level);
    Ok(())
}