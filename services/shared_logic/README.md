# Shared Logic Service

## 概述

shared_logic 是 Sentio AI 邮件伙伴系统的共享逻辑服务，提供了所有服务和组件之间共享的核心功能。

## 主要功能

### 1. 全局配置管理

- **单例模式**: 全局配置在应用启动时加载一次，确保所有组件使用相同的配置
- **线程安全**: 支持多线程环境下的安全访问
- **环境变量支持**: 支持通过环境变量覆盖配置文件设置
- **类型安全**: 使用强类型配置结构，编译时检查配置完整性

### 2. 共享类型定义

- **标准结果类型**: 统一的错误处理方式
- **服务状态管理**: 标准化的服务状态枚举
- **健康检查**: 统一的服务健康检查接口

## 配置系统

### 配置文件格式

系统支持 TOML 格式的配置文件 (`Config.toml`)：

```toml
[database]
url = "mongodb://localhost:27017/sentio"
max_connections = 10
connect_timeout = 30

[email.imap]
host = "imap.gmail.com"
port = 993
username = "your-email@example.com"
password = "your-app-password"
use_tls = true

[email.smtp]
host = "smtp.gmail.com"
port = 587
username = "your-email@example.com"
password = "your-app-password"
use_tls = true

[llm]
provider = "deepseek"
api_key = "your-deepseek-api-key"
base_url = "https://api.deepseek.com"
model = "deepseek-chat"
timeout = 120
max_retries = 3

[telemetry]
log_level = "info"
console = true
json_format = false

[server]
host = "127.0.0.1"
port = 8080
workers = 4
```

### 环境变量覆盖

使用 `SENTIO_` 前缀，嵌套字段用双下划线 `__` 分隔：

```bash
# 数据库配置
SENTIO_DATABASE__URL=mongodb://prod-server:27017/sentio
SENTIO_DATABASE__MAX_CONNECTIONS=20

# LLM 配置
SENTIO_LLM__API_KEY=sk-your-production-key
SENTIO_LLM__MODEL=deepseek-chat-v2

# 遥测配置
SENTIO_TELEMETRY__LOG_LEVEL=debug
SENTIO_TELEMETRY__JSON_FORMAT=true
```

## 使用示例

### 基本用法

```rust
use shared_logic::config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 在应用启动时初始化全局配置
    config::initialize_config().await?;
    
    // 在应用的任何地方访问配置
    let config = config::get_config();
    println!("Database URL: {}", config.database.url);
    println!("LLM Provider: {}", config.llm.provider);
    
    Ok(())
}
```

### 条件初始化

```rust
use shared_logic::config;

// 检查配置是否已初始化
if !config::is_initialized() {
    config::initialize_config().await?;
}

// 安全地尝试获取配置
if let Some(config) = config::try_get_config() {
    println!("Server port: {}", config.server.port);
} else {
    println!("Configuration not yet initialized");
}
```

### 在其他服务中使用

```rust
// 在其他服务的 lib.rs 中
use shared_logic::config;

pub fn some_service_function() -> anyhow::Result<()> {
    let config = config::get_config();
    
    // 使用配置中的数据库连接信息
    let db_url = &config.database.url;
    
    // 使用配置中的 LLM 设置
    let llm_timeout = config.llm.timeout;
    
    Ok(())
}
```

## API 参考

### 主要函数

#### `initialize_config() -> Result<()>`

初始化全局配置。必须在应用启动时调用一次。

**错误情况:**

- 配置文件格式错误
- 环境变量格式错误
- 全局配置已经初始化

#### `get_config() -> &'static Config`

获取全局配置的只读引用。如果配置未初始化会 panic。

#### `try_get_config() -> Option<&'static Config>`

安全地尝试获取全局配置引用。如果未初始化返回 None。

#### `is_initialized() -> bool`

检查全局配置是否已经初始化。

## 配置优先级

配置系统按以下优先级加载设置（从低到高）：

1. **代码中的默认值** - 硬编码的后备值
2. **配置文件** - `Config.toml` 文件中的设置
3. **环境变量** - `SENTIO_*` 环境变量（最高优先级）

## 最佳实践

### 1. 配置初始化

- 总是在 `main()` 函数的开始处初始化配置
- 在初始化遥测系统之前初始化配置
- 使用 `is_initialized()` 在测试中避免重复初始化

### 2. 敏感信息处理

- 将敏感信息（API 密钥、密码）通过环境变量传递
- 不要在配置文件中硬编码生产环境的敏感信息
- 使用 `.env.example` 文件提供环境变量模板

### 3. 配置验证

- 配置结构使用强类型，利用编译器检查
- 在配置加载后立即验证关键配置项
- 为必需的配置项提供合理的默认值

## 错误处理

所有配置相关的错误都使用 `anyhow::Result` 返回，提供详细的错误信息：

```rust
use shared_logic::config;

match config::initialize_config().await {
    Ok(()) => println!("Configuration loaded successfully"),
    Err(e) => {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }
}
```

## 线程安全

- 全局配置使用 `std::sync::OnceLock` 实现，保证线程安全
- 配置在初始化后是只读的，可以安全地在多线程间共享
- 不需要额外的锁或同步机制

## 测试支持

在测试中使用配置系统：

```rust
#[tokio::test]
async fn test_with_config() -> anyhow::Result<()> {
    // 测试环境下的配置初始化
    if !shared_logic::config::is_initialized() {
        shared_logic::config::initialize_config().await?;
    }
    
    let config = shared_logic::config::get_config();
    assert_eq!(config.server.port, 8080);
    
    Ok(())
}
```
