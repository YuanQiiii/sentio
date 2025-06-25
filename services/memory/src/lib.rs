//! # Sentio 记忆服务
//!
//! 提供用户记忆数据的存储、检索和管理功能，遵循"零信任"和"健壮性是底线"原则。
//!
//! ## 核心功能
//!
//! - 记忆体的持久化存储(内存存储)
//! - 交互记录的完整追踪
//! - 语义搜索和记忆检索
//! - 用户数据的 GDPR 合规管理
//!
//! ## 设计原则
//!
//! - **健壮性是底线**: 所有数据库操作都有完整的错误处理和重试机制
//! - **安全是内置属性**: 使用参数化查询，严格的数据验证
//! - **零信任**: 验证所有输入数据的有效性
//! - **配置驱动**: 数据库连接和行为通过配置外置

// 模块声明
pub mod error;
pub mod models;
pub mod memory_repository;
pub mod repository;
pub mod memory_data;

// 导出错误类型
pub use error::{MemoryError, MemoryResult};

// 导出模型类型
pub use models::{
    ActionStateMemory, CommunicationStrategy, CoreProfile, EpisodicMemory, FollowUp, HabitPattern,
    InteractionLog, MemoryCorpus, MessageDirection, Plan, PreferencesAndDislikes, RelationalGoals,
    Relationship, SelfReflectionEntry, SemanticMemory, SignificantEvent, SkillExpertise,
    StrategicInferentialMemory, Task, UserModelHypothesis,
};

// 导出仓储接口类型
pub use repository::{
    MemoryFragment, MemoryQuery, MemoryRepository, MemoryRepositoryFactory, MemoryType, TimeRange,
    UserStatistics,
};

// 导出内存存储实现
pub use memory_repository::MemoryRepositoryImpl;
pub use memory_data::MemoryDataRepository;
