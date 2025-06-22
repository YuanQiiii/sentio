# Email 服务

`sentio_email` 是 Sentio AI 邮件伙伴系统的邮件发送服务，专注于 SMTP 邮件发送功能。

## 🎯 功能特性

### 📤 邮件发送 (SMTP)

- 异步邮件发送
- 支持多种邮件格式（纯文本/HTML）
- 附件支持和安全检查
- 发送状态跟踪
- 支持 TLS/SSL 加密连接

### 🔒 安全特性

- 邮件内容验证和清理
- 附件类型和大小限制
- API 凭证安全管理
- 防止邮件滥用

## 🏗️ 架构设计

### 核心接口

```rust
#[async_trait]
pub trait SmtpClient {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId>;
    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool>;
}
```

### 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<Attachment>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}
```

## 🚀 使用示例

### 基本邮件发送

```rust
use sentio_email::{SmtpClient, EmailMessage};
use shared_logic::config::get_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化配置
    shared_logic::config::initialize_config().await?;
    
    // 获取配置
    let config = get_config();
    
    // 创建 SMTP 客户端
    let smtp_client = SmtpClient::new(&config.email.smtp).await?;
    
    // 构建邮件消息
    let message = EmailMessage::builder()
        .from("sender@example.com")?
        .to("recipient@example.com")?
        .subject("测试邮件")?
        .body_text("这是一封测试邮件")?
        .build()?;
    
    // 发送邮件
    let message_id = smtp_client.send_message(&message).await?;
    println!("邮件发送成功，ID: {}", message_id);
    
    Ok(())
}
```

### HTML 邮件发送

```rust
let html_message = EmailMessage::builder()
    .from("sender@example.com")?
    .to("recipient@example.com")?
    .subject("HTML 邮件")?
    .body_html(r#"
        <h1>欢迎使用 Sentio AI</h1>
        <p>这是一封 <strong>HTML 格式</strong> 的邮件。</p>
    "#)?
    .body_text("这是纯文本版本的内容")?
    .build()?;

smtp_client.send_message(&html_message).await?;
```

## ⚙️ 配置

### 环境变量配置

```bash
# SMTP 服务器配置
SENTIO_EMAIL_SMTP_HOST=smtp.gmail.com
SENTIO_EMAIL_SMTP_PORT=587
SENTIO_EMAIL_SMTP_USERNAME=your-email@gmail.com
SENTIO_EMAIL_SMTP_PASSWORD=your-app-password
SENTIO_EMAIL_SMTP_USE_TLS=true
```

### Config.toml 配置

```toml
[email.smtp]
host = "smtp.gmail.com"
port = 587
username = "your-email@example.com"
password = "your-app-password"
use_tls = true
```

## 🧪 测试

### 单元测试

```bash
cargo test --package sentio_email
```

### 集成测试

```bash
# 设置测试环境变量
export SENTIO_EMAIL_SMTP_HOST=smtp.example.com
export SENTIO_EMAIL_SMTP_USERNAME=test@example.com
export SENTIO_EMAIL_SMTP_PASSWORD=test_password

# 运行集成测试
cargo test --package sentio_email --test integration
```

## 🔧 开发

### 添加新功能

1. 在 `src/types.rs` 中定义相关类型
2. 在 `src/client.rs` 中实现客户端逻辑
3. 添加相应的测试用例
4. 更新文档和示例

### 错误处理

所有公共接口都返回 `EmailResult<T>`，这是 `Result<T, EmailError>` 的类型别名：

```rust
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("SMTP connection error: {0}")]
    SmtpConnection(String),
    
    #[error("Message format error: {0}")]
    MessageFormat(String),
    
    #[error("Authentication failed")]
    Authentication,
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

## 📊 性能和监控

### 指标

- 邮件发送成功率
- 平均发送时间
- 错误率和错误类型分布
- 连接池状态

### 日志

使用结构化日志记录所有重要操作：

```rust
tracing::info!(
    message_id = %message_id,
    recipient = %to_address,
    subject = %subject,
    "邮件发送成功"
);
```

## 🤝 集成

### 与其他服务集成

- **shared_logic**: 获取全局配置
- **telemetry**: 记录结构化日志
- **core**: 接收邮件发送请求

### API 兼容性

遵循语义版本控制，确保向后兼容性。

## 📄 许可证

MIT License - 详见项目根目录的 LICENSE 文件。
