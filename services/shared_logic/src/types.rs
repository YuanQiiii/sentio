//! # 共享类型定义模块
//!
//! 这个模块包含了在多个服务和组件之间共享的类型定义。
//! 包括常用的结果类型、错误类型等。

use serde::{Deserialize, Serialize};

/// 应用程序的标准结果类型
pub type AppResult<T> = anyhow::Result<T>;

/// 应用程序的标准错误类型
pub type AppError = anyhow::Error;

/// 服务状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// 服务正在启动
    Starting,
    /// 服务正在运行
    Running,
    /// 服务正在停止
    Stopping,
    /// 服务已停止
    Stopped,
    /// 服务出现错误
    Error,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Starting => write!(f, "starting"),
            ServiceStatus::Running => write!(f, "running"),
            ServiceStatus::Stopping => write!(f, "stopping"),
            ServiceStatus::Stopped => write!(f, "stopped"),
            ServiceStatus::Error => write!(f, "error"),
        }
    }
}

/// 服务健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 服务状态
    pub status: ServiceStatus,
    /// 服务版本
    pub version: String,
    /// 启动时间
    pub uptime: chrono::Duration,
    /// 额外的健康信息
    pub details: std::collections::HashMap<String, String>,
}

impl HealthCheck {
    /// 创建一个新的健康检查结果
    pub fn new(status: ServiceStatus, version: String) -> Self {
        Self {
            status,
            version,
            uptime: chrono::Duration::zero(),
            details: std::collections::HashMap::new(),
        }
    }

    /// 添加详细信息
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// 设置运行时间
    pub fn with_uptime(mut self, uptime: chrono::Duration) -> Self {
        self.uptime = uptime;
        self
    }
}
