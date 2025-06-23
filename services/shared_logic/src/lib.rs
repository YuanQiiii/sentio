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
//!     config::initialize_config().await?;
//!     
//!     // 在应用的任何地方访问配置
//!     let config = config::get_config();
//!     println!("Database URL: {}", config.database.url);
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod types;
pub mod database;
pub mod memory_data;

// 重新导出主要的公共接口
pub use config::{get_config, initialize_config, Config, Prompt, PromptsConfig};
pub use types::*;

// 重新导出数据库相关接口
pub use database::{
    initialize_database, get_database, try_get_database, get_collection, check_database_health,
    get_database_stats, DatabaseError, DatabaseResult, DatabaseStats,
};

// 重新导出记忆数据访问接口
pub use memory_data::{
    MemoryDataAccess, MemoryFragment, MemoryCorpus, InteractionLog,
    MemoryType, MemoryQuery, TimeRange, UserStatistics,
    CoreProfile, EpisodicMemory, SemanticMemory, SkillExpertise,
    StrategicInferentialMemory, ActionStateMemory, MessageDirection,
    CommunicationStrategy,
};
