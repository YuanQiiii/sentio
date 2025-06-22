//! # Shared Logic Crate
//! 
//! 这个 crate 包含了 Sentio AI 邮件伙伴系统的共享逻辑组件。
//! 它提供了在不同模块和服务之间共享的通用功能，包括全局配置管理。
//! 
//! ## 主要模块
//! 
//! - [`config`] - 全局配置管理，提供只读的全局配置访问
//! - [`types`] - 共享的数据类型定义
//! 
//! ## 使用示例
//! 
//! ```rust
//! use shared_logic::config;
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 在应用启动时初始化全局配置
//!     config::initialize().await?;
//!     
//!     // 在应用的任何地方访问配置
//!     let config = config::get();
//!     println!("Database URL: {}", config.database.url);
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod types;

// 重新导出主要的公共接口
pub use config::{Config, initialize_config, get_config};
pub use types::*;
