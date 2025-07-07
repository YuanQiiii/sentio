# Sentio 使用文档

## 简介

Sentio 是一个智能邮件处理系统，基于 Rust 构建，集成了人工智能技术来分析和处理邮件。系统采用微服务架构，包含邮件服务、LLM 服务、记忆服务等模块。

## 快速开始

### 1. 环境准备

#### 系统要求
- Rust 1.70 或更高版本
- Linux/macOS/Windows 操作系统
- 互联网连接（用于访问 LLM API）

#### 安装 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 项目设置

#### 克隆或下载项目
```bash
git clone <your-repo-url>
cd sentio
```

#### 配置环境变量
创建 `.env` 文件（可选，用于覆盖默认配置）：
```env
# DeepSeek API 配置
DEEPSEEK_API_KEY=your_api_key_here

# 邮件配置
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

#### 配置文件
默认配置文件位于 `config/default.toml`，包含所有服务的基本配置。您可以根据需要修改：

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4

[llm]
provider = "deepseek"
api_key = "your_api_key"
base_url = "https://api.deepseek.com"
model = "deepseek-chat"
timeout = 120
max_retries = 3

[email.smtp]
host = "smtp.gmail.com"
port = 587
username = "your_email@gmail.com"
password = "your_password"
use_tls = true

[telemetry]
level = "info"
```

### 3. 构建和运行

#### 构建项目
```bash
cargo build --release
```

#### 运行主服务
```bash
cargo run --package sentio_core
```

#### 运行测试
```bash
# 运行所有测试
cargo test --all

# 运行特定服务测试
cargo test --package sentio_email
cargo test --package sentio_llm
cargo test --package sentio_memory
```

## 核心功能

### 1. 邮件处理工作流

Sentio 提供完整的邮件处理流程：

1. **接收邮件** - 接收来自用户的邮件
2. **AI 分析** - 使用 DeepSeek LLM 分析邮件内容
3. **生成回复** - 基于分析结果生成智能回复
4. **发送邮件** - 通过 SMTP 发送回复邮件
5. **记忆存储** - 将交互记录存储到本地文件

### 2. 服务架构

#### 核心服务 (Core Service)
- 主要的应用程序入口点
- 协调各个服务之间的交互
- 提供邮件处理工作流

#### 邮件服务 (Email Service)
- SMTP 邮件发送功能
- 邮件地址验证
- 邮件内容验证
- 附件处理（安全检查）

#### LLM 服务 (LLM Service)
- 与 DeepSeek API 集成
- 提供智能文本分析
- 支持自定义提示词模板
- 自动重试和错误处理

#### 记忆服务 (Memory Service)
- 用户交互记录存储
- 基于 JSON 文件的持久化
- 用户统计信息
- 查询和检索功能

#### 遥测服务 (Telemetry Service)
- 结构化日志记录
- 性能监控
- 错误追踪

## 使用示例

### 基本邮件处理

```rust
use sentio_core::{EmailWorkflow, MockSmtpClient};
use sentio_email::{EmailAddress, EmailBody, OutgoingMessage};
use sentio_llm::DeepSeekClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建邮件工作流
    let workflow = EmailWorkflow::new().await?;
    
    // 创建测试邮件
    let from = EmailAddress::new("user@example.com".to_string());
    let to = vec![EmailAddress::new("support@company.com".to_string())];
    let body = EmailBody::text("我需要帮助重置密码".to_string());
    let message = OutgoingMessage::new(from, to, "密码重置请求".to_string(), body);
    
    // 处理邮件
    let result = workflow.process_email(&message).await?;
    println!("邮件处理完成: {:?}", result);
    
    Ok(())
}
```

### 自定义 LLM 客户端

```rust
use sentio_llm::{LlmClient, LlmRequest, LlmParameters};
use std::collections::HashMap;

async fn analyze_email_content(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = sentio_llm::DeepSeekClient::new()?;
    
    let mut context = HashMap::new();
    context.insert("email_content".to_string(), serde_json::json!(content));
    
    let request = LlmRequest {
        id: uuid::Uuid::new_v4(),
        prompt_name: "email_analysis.classify".to_string(),
        context,
        parameters: LlmParameters::default(),
        created_at: chrono::Utc::now(),
    };
    
    let response = client.generate_response(&request).await?;
    Ok(response.content)
}
```

### 记忆服务使用

```rust
use shared_logic::{MemoryDataAccess, InteractionLog, MessageDirection};
use std::collections::HashMap;

async fn log_user_interaction(user_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let interaction = InteractionLog {
        id: None,
        user_id: user_id.to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        direction: MessageDirection::UserToSystem,
        content: content.to_string(),
        metadata: HashMap::new(),
    };
    
    let interaction_id = MemoryDataAccess::log_interaction(&interaction).await?;
    println!("交互记录已保存: {}", interaction_id);
    
    Ok(())
}
```

## 配置说明

### 提示词配置

提示词模板存储在 `config/prompts.yaml` 文件中：

```yaml
prompts:
  introduction:
    default:
      system: "你是一个专业的邮件助手，能够分析和回复各种类型的邮件。"
      user: "请简单介绍一下自己，说明你的主要功能和特点。"
  
  email_analysis:
    classify:
      system: "你是一个邮件分类专家，能够分析邮件内容并进行分类。"
      user: "请分析以下邮件内容：{email_content}"
    
    summarize_thread:
      system: "你是一个邮件总结专家，能够总结邮件线程的要点。"
      user: "请总结以下邮件线程：{thread_content}"
```

### 环境变量覆盖

您可以使用环境变量来覆盖配置文件中的设置：

```bash
# LLM 配置
export DEEPSEEK_API_KEY="your_api_key"
export LLM_MODEL="deepseek-chat"
export LLM_TIMEOUT=120

# 邮件配置
export SMTP_HOST="smtp.gmail.com"
export SMTP_PORT=587
export SMTP_USERNAME="your_email@gmail.com"
export SMTP_PASSWORD="your_app_password"

# 服务器配置
export SERVER_HOST="0.0.0.0"
export SERVER_PORT=8080
```

## 故障排除

### 常见问题

#### 1. LLM API 连接失败
```
Error: AuthenticationFailed { reason: "Invalid API key" }
```
**解决方案**：检查 DeepSeek API 密钥是否正确设置。

#### 2. 邮件发送失败
```
Error: SendError { recipient: "user@example.com", details: "Connection refused" }
```
**解决方案**：检查 SMTP 服务器配置和网络连接。

#### 3. 配置文件加载失败
```
Error: Config file not found: config/default.toml
```
**解决方案**：确保配置文件存在且格式正确。

### 调试模式

启用详细日志记录：
```bash
RUST_LOG=debug cargo run --package sentio_core
```

### 测试邮件功能

使用测试邮件地址验证功能：
```bash
cargo test --package sentio_email test_email_validation
```

## 扩展开发

### 添加新的 LLM 提供商

1. 在 `services/llm/src/client.rs` 中实现 `LlmClient` trait
2. 添加相应的配置选项
3. 更新工厂函数以支持新的提供商

### 自定义邮件处理器

1. 创建新的邮件处理器结构体
2. 实现 `EmailProcessor` trait
3. 在工作流中注册新的处理器

### 扩展记忆服务

1. 实现新的存储后端（如数据库）
2. 扩展 `MemoryDataAccess` 接口
3. 添加新的查询和分析功能

## 生产部署

### Docker 部署

创建 `Dockerfile`：
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/sentio_core /usr/local/bin/
CMD ["sentio_core"]
```

### 系统服务配置

创建 systemd 服务文件 `/etc/systemd/system/sentio.service`：
```ini
[Unit]
Description=Sentio Email Processing Service
After=network.target

[Service]
Type=simple
User=sentio
WorkingDirectory=/opt/sentio
ExecStart=/opt/sentio/target/release/sentio_core
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 监控和日志

- 日志文件位于 `logs/` 目录
- 使用 `journalctl -u sentio` 查看系统日志
- 配置日志轮转以管理磁盘空间

## 许可证

本项目遵循 MIT 许可证。详情请参阅 LICENSE 文件。

## 支持

如有问题或建议，请：
1. 查看本文档的故障排除部分
2. 检查项目的 GitHub Issues
3. 创建新的 Issue 描述问题

---

*最后更新时间：2025-07-07*