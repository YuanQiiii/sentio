# Sentio Telemetry Service

## 概述

`sentio_telemetry` 是 Sentio AI 邮件伙伴系统的遥测和日志服务，基于 `tracing` 生态系统提供结构化日志记录能力。

## 功能特性

- **结构化日志**: 使用 `tracing` 提供结构化、上下文感知的日志记录
- **多种输出格式**: 支持人类可读和 JSON 格式的日志输出
- **可配置日志级别**: 支持 trace、debug、info、warn、error 五个日志级别
- **高性能**: 基于 `tracing` 的零成本抽象，对性能影响最小

## 日志级别

- **TRACE**: 最详细的日志，通常用于调试复杂问题
- **DEBUG**: 调试信息，包含程序执行的详细信息
- **INFO**: 一般信息，记录重要的程序状态变化
- **WARN**: 警告信息，表示潜在问题但不影响程序正常运行
- **ERROR**: 错误信息，表示程序遇到了问题

## 使用方法

### 初始化遥测系统

```rust
use sentio_telemetry;

// 使用默认配置初始化
sentio_telemetry::init_telemetry()?;

// 或者使用自定义配置初始化
let config = TelemetryConfig {
    log_level: LogLevel::Debug,
    console: true,
    json_format: false,
    log_file: None,
};
sentio_telemetry::init_telemetry_with_config(&config)?;
```

### 记录日志

初始化后，可以在应用的任何地方使用 `tracing` 宏记录日志：

```rust
use tracing::{info, debug, warn, error};

// 简单日志
info!("Application started successfully");

// 带结构化字段的日志
info!(
    user_id = "user123",
    action = "login",
    "User logged in successfully"
);

// 错误日志
if let Err(e) = some_operation() {
    error!(error = %e, "Operation failed");
}
```

### 性能监控

使用 `tracing` 的 span 功能进行性能监控：

```rust
use tracing::{info_span, Instrument};

async fn process_email(email_id: &str) -> Result<()> {
    let span = info_span!("process_email", email_id = email_id);
    
    async {
        // 处理邮件的逻辑
        info!("Starting email processing");
        // ...
        info!("Email processing completed");
    }
    .instrument(span)
    .await
}
```

## 配置选项

### TelemetryConfig 结构

```rust
pub struct TelemetryConfig {
    /// 日志级别
    pub log_level: LogLevel,
    /// 是否输出到控制台
    pub console: bool,
    /// 日志文件路径（可选，未来功能）
    pub log_file: Option<String>,
    /// 是否启用JSON格式日志
    pub json_format: bool,
}
```

### 环境变量配置

可以通过环境变量覆盖遥测配置：

```bash
# 设置日志级别
export SENTIO_TELEMETRY_LOG_LEVEL=debug

# 启用 JSON 格式输出
export SENTIO_TELEMETRY_JSON_FORMAT=true

# 禁用控制台输出
export SENTIO_TELEMETRY_CONSOLE=false
```

## 输出格式

### 人类可读格式（默认）

```
2025-06-22T12:34:56.789Z  INFO sentio_core: services/core/src/main.rs:15: Configuration loaded successfully
```

### JSON 格式

启用 `json_format: true` 后：

```json
{
  "timestamp": "2025-06-22T12:34:56.789Z",
  "level": "INFO",
  "target": "sentio_core",
  "fields": {
    "message": "Configuration loaded successfully",
    "log_level": "Debug",
  },
  "file": "services/core/src/main.rs",
  "line": 15
}
```

## 性能考虑

- `tracing` 使用零成本抽象，在发布构建中对性能影响最小
- 结构化字段的序列化只在实际输出时进行
- 可以通过日志级别过滤减少不必要的日志处理开销

## 最佳实践

### 1. 选择合适的日志级别

- 使用 `error!` 记录需要立即关注的错误
- 使用 `warn!` 记录潜在问题或不符合预期的情况
- 使用 `info!` 记录重要的业务事件
- 使用 `debug!` 记录有助于调试的详细信息
- 使用 `trace!` 记录非常详细的执行流程

### 2. 添加上下文信息

```rust
// 好的做法 - 包含相关上下文
info!(
    user_id = %user.id,
    email_count = emails.len(),
    processing_time_ms = start.elapsed().as_millis(),
    "Email processing completed"
);

// 避免 - 缺少上下文
info!("Email processing completed");
```

### 3. 使用结构化字段

```rust
// 好的做法 - 结构化字段便于查询和分析
warn!(
    error_code = "RATE_LIMIT_EXCEEDED",
    retry_after_seconds = 60,
    api_endpoint = "/v1/chat/completions",
    "API rate limit exceeded"
);

// 避免 - 将所有信息放在消息中
warn!("API rate limit exceeded for /v1/chat/completions, retry after 60 seconds");
```

## 未来功能

- 文件日志输出支持
- 日志轮转和归档
- 远程日志收集集成（如 Elasticsearch、Grafana Loki）
- 指标收集和监控集成

## 依赖项

- `tracing`: 核心跟踪框架
- `tracing-subscriber`: 跟踪订阅者实现
- `sentio_config`: 配置管理
- `anyhow`: 错误处理
