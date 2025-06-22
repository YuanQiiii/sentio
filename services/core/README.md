# Sentio Core Service

## 概述

`sentio_core` 是 Sentio AI 邮件伙伴系统的核心入口服务，负责协调各个子服务，管理应用程序的生命周期，并演示全局配置的使用。

## 功能特性

- **应用程序入口**: 作为整个系统的主要入口点
- **服务协调**: 协调各个子服务的初始化和运行
- **全局配置管理**: 演示如何使用 shared_logic 中的全局配置
- **生命周期管理**: 管理应用程序的启动、运行和关闭过程

## 架构职责

### 1. 初始化流程

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 第一步：初始化全局配置
    config::initialize().await?;
    
    // 第二步：基于配置初始化遥测系统
    let global_config = config::get();
    sentio_telemetry::init_telemetry_with_config(&global_config.telemetry)?;
    
    // 第三步：启动各种服务
    // ...
}
```

### 2. 服务集成

Core 服务负责集成以下组件：

- **配置管理**: 通过 `shared_logic::config` 访问全局配置
- **遥测系统**: 初始化和管理日志记录
- **内存服务**: 管理用户记忆数据
- **未来扩展**: 邮件处理、LLM 交互等服务

## 全局配置使用

### 在启动时使用配置

```rust
// 获取全局配置
let global_config = config::get();

// 使用配置信息
tracing::info!(
    log_level = ?global_config.telemetry.log_level,
    database_url = %global_config.database.url,
    llm_provider = %global_config.llm.provider,
    "Configuration loaded successfully"
);
```

### 在运行时访问配置

```rust
/// 演示如何在应用的任何地方访问全局配置
fn demonstrate_global_config_access() {
    let config = config::get();
    tracing::info!(
        "Database max connections: {}",
        config.database.max_connections
    );
}
```

## 启动流程

### 详细步骤

1. **配置初始化**
   - 加载配置文件和环境变量
   - 初始化全局配置单例
   - 验证配置的完整性

2. **遥测初始化**
   - 根据配置设置日志级别
   - 配置日志输出格式
   - 启用结构化日志记录

3. **服务启动**
   - 初始化数据库连接
   - 启动邮件监听服务
   - 初始化 LLM 客户端

4. **运行监控**
   - 健康检查
   - 性能监控
   - 错误处理

## 错误处理

### 启动错误

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 配置初始化失败
    if let Err(e) = config::initialize().await {
        eprintln!("Failed to initialize configuration: {}", e);
        std::process::exit(1);
    }
    
    // 遥测初始化失败
    if let Err(e) = sentio_telemetry::init_telemetry_with_config(&config) {
        eprintln!("Failed to initialize telemetry: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
```

### 运行时错误

```rust
// 使用 tracing 记录错误
if let Err(e) = some_operation().await {
    tracing::error!(
        error = %e,
        operation = "some_operation",
        "Operation failed"
    );
}
```

## 性能监控

### 使用 tracing spans

```rust
use tracing::{info_span, Instrument};

async fn process_request() -> Result<()> {
    let span = info_span!("process_request");
    
    async {
        tracing::info!("Starting request processing");
        
        // 业务逻辑
        
        tracing::info!("Request processing completed");
    }
    .instrument(span)
    .await
}
```

### 配置信息记录

```rust
// 启动时记录关键配置信息
tracing::info!(
    database_url = %config.database.url,
    llm_provider = %config.llm.provider,
    server_host = %config.server.host,
    server_port = %config.server.port,
    "System configuration loaded"
);
```

## 依赖服务

### 内部依赖

- **shared_logic**: 全局配置管理
- **sentio_telemetry**: 日志和遥测
- **sentio_memory**: 数据结构定义

### 外部依赖

- **tokio**: 异步运行时
- **anyhow**: 错误处理
- **tracing**: 结构化日志

## 部署配置

### 环境变量

```bash
# 数据库配置
export SENTIO_DATABASE__URL="mongodb://prod-server:27017/sentio"

# 日志配置
export SENTIO_TELEMETRY__LOG_LEVEL="info"
export SENTIO_TELEMETRY__JSON_FORMAT="true"

# LLM 配置
export SENTIO_LLM__API_KEY="your-production-api-key"
```

### Docker 运行

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin sentio_core

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/sentio_core /usr/local/bin/
CMD ["sentio_core"]
```

## 监控和健康检查

### 日志监控

```rust
// 定期记录系统状态
tokio::spawn(async {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        tracing::info!("System health check - OK");
    }
});
```

### 错误报告

```rust
// 关键错误报告
tracing::error!(
    error = %e,
    component = "email_processor",
    user_id = %user_id,
    "Critical error in email processing"
);
```

## 扩展指南

### 添加新服务

1. 在 `main.rs` 中添加服务初始化代码
2. 更新依赖项配置
3. 添加相应的错误处理
4. 更新文档和示例

### 配置扩展

1. 在 `shared_logic` 中添加新的配置项
2. 在 core 服务中使用新配置
3. 更新环境变量示例
4. 添加配置验证

## 最佳实践

### 1. 启动时间优化

- 并行初始化独立的服务
- 延迟加载非关键组件
- 使用连接池预热

### 2. 资源管理

- 正确处理资源清理
- 实现优雅关闭
- 监控内存使用

### 3. 可观测性

- 记录关键业务指标
- 使用结构化日志
- 实现分布式追踪

## 未来功能

- 健康检查端点
- 指标暴露（Prometheus）
- 配置热重载
- 服务发现集成
- 负载均衡支持
