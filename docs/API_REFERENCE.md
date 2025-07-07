# Sentio API 参考文档

## 核心 API

### EmailWorkflow

邮件处理工作流的主要接口。

```rust
pub struct EmailWorkflow {
    llm_client: Box<dyn LlmClient>,
    email_client: Box<dyn SmtpClient>,
}
```

#### 方法

##### `new() -> Result<Self, Box<dyn std::error::Error>>`
创建新的邮件工作流实例，使用默认的客户端。

##### `new_with_clients(llm_client: Box<dyn LlmClient>, email_client: Box<dyn SmtpClient>) -> Self`
使用自定义客户端创建工作流实例。

##### `process_email(&self, message: &OutgoingMessage) -> Result<String, Box<dyn std::error::Error>>`
处理邮件消息，执行完整的分析和回复流程。

### LlmClient Trait

LLM 客户端接口，用于与大语言模型交互。

```rust
#[async_trait]
pub trait LlmClient: Send + Sync + AsAny {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
}
```

#### 实现

##### `DeepSeekClient`
DeepSeek API 的客户端实现。

```rust
impl DeepSeekClient {
    pub fn new() -> LlmResult<Self>
}
```

### SmtpClient Trait

SMTP 客户端接口，用于发送邮件。

```rust
#[async_trait]
pub trait SmtpClient: Send + Sync + AsAny {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId>;
    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool>;
    async fn connect(&mut self) -> EmailResult<()>;
    async fn disconnect(&mut self) -> EmailResult<()>;
    async fn is_connected(&self) -> bool;
}
```

#### 实现

##### `SimpleSmtpClient`
基于 lettre 的 SMTP 客户端实现。

```rust
impl SimpleSmtpClient {
    pub async fn from_config() -> EmailResult<Self>
}
```

### MemoryDataAccess

记忆数据访问接口，用于存储和检索用户交互记录。

```rust
pub struct MemoryDataAccess;
```

#### 方法

##### `initialize(persistence_file: PathBuf) -> Result<(), MemoryError>`
初始化记忆服务，指定持久化文件路径。

##### `log_interaction(interaction: &InteractionLog) -> Result<uuid::Uuid, MemoryError>`
记录用户交互。

##### `get_user_interactions(user_id: &str, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<InteractionLog>, MemoryError>`
获取用户交互记录。

##### `get_user_statistics(user_id: &str) -> Result<UserStatistics, MemoryError>`
获取用户统计信息。

## 数据类型

### 邮件相关类型

#### `EmailAddress`
```rust
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: String) -> Self
    pub fn with_name(email: String, name: String) -> Self
    pub fn is_valid(&self) -> bool
}
```

#### `EmailBody`
```rust
pub struct EmailBody {
    pub text: Option<String>,
    pub html: Option<String>,
    pub content_type: String,
}

impl EmailBody {
    pub fn text(content: String) -> Self
    pub fn html(content: String) -> Self
    pub fn get_display_content(&self) -> Option<&String>
    pub fn is_empty(&self) -> bool
}
```

#### `OutgoingMessage`
```rust
pub struct OutgoingMessage {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: EmailBody,
    pub attachments: Vec<EmailAttachment>,
    pub in_reply_to: Option<MessageId>,
    pub headers: HashMap<String, String>,
}

impl OutgoingMessage {
    pub fn new(from: EmailAddress, to: Vec<EmailAddress>, subject: String, body: EmailBody) -> Self
    pub fn add_cc(self, cc: EmailAddress) -> Self
    pub fn add_bcc(self, bcc: EmailAddress) -> Self
    pub fn reply_to(self, original_message_id: MessageId) -> Self
    pub fn add_attachment(self, attachment: EmailAttachment) -> Self
    pub fn add_header(self, key: String, value: String) -> Self
    pub fn validate(&self) -> Result<(), String>
}
```

#### `EmailAttachment`
```rust
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub content_id: Option<String>,
    pub is_inline: bool,
}

impl EmailAttachment {
    pub fn is_safe_type(&self) -> bool
    pub fn is_reasonable_size(&self) -> bool
}
```

### LLM 相关类型

#### `LlmRequest`
```rust
pub struct LlmRequest {
    pub id: uuid::Uuid,
    pub prompt_name: String,
    pub context: HashMap<String, serde_json::Value>,
    pub parameters: LlmParameters,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

#### `LlmResponse`
```rust
pub struct LlmResponse {
    pub request_id: uuid::Uuid,
    pub content: String,
    pub token_usage: TokenUsage,
    pub metadata: ResponseMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

#### `LlmParameters`
```rust
pub struct LlmParameters {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub stream: bool,
}

impl Default for LlmParameters {
    fn default() -> Self {
        Self {
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            stream: false,
        }
    }
}
```

### 记忆相关类型

#### `InteractionLog`
```rust
pub struct InteractionLog {
    pub id: Option<uuid::Uuid>,
    pub user_id: String,
    pub session_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub direction: MessageDirection,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### `MessageDirection`
```rust
pub enum MessageDirection {
    UserToSystem,
    SystemToUser,
}
```

#### `UserStatistics`
```rust
pub struct UserStatistics {
    pub user_id: String,
    pub total_interactions: u64,
    pub first_interaction: Option<chrono::DateTime<chrono::Utc>>,
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    pub session_count: u64,
}
```

## 错误处理

### 错误类型

#### `EmailError`
```rust
pub enum EmailError {
    SendError { recipient: String, details: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },
    ValidationError { field: String, value: String, reason: String },
    ConfigurationError { field: String, value: String, reason: String },
    InternalError { details: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },
}
```

#### `LlmError`
```rust
pub enum LlmError {
    ApiRequestFailed { message: String },
    AuthenticationFailed { reason: String },
    RateLimited { retry_after_seconds: u64 },
    TokenLimitExceeded { limit: u32 },
    ContentFiltered { reason: String },
    InvalidApiResponse { details: String },
    NetworkError { details: String },
    ConfigurationError { field: String },
    InternalError { details: String },
}
```

#### `MemoryError`
```rust
pub enum MemoryError {
    DatabaseError { details: String },
    NotFound { resource: String, id: String },
    ValidationError { field: String, reason: String },
    SerializationError { details: String },
    IoError { details: String },
}
```

## 配置

### 配置结构

#### `Config`
```rust
pub struct Config {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    pub email: EmailConfig,
    pub telemetry: TelemetryConfig,
    pub prompts: PromptsConfig,
}
```

#### `ServerConfig`
```rust
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}
```

#### `LlmConfig`
```rust
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub timeout: u64,
    pub max_retries: u32,
}
```

#### `EmailConfig`
```rust
pub struct EmailConfig {
    pub smtp: SmtpConfig,
}

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}
```

### 配置函数

#### `get_config() -> &'static Config`
获取全局配置实例。

#### `init_config() -> Result<(), Box<dyn std::error::Error>>`
初始化全局配置。

## 实用工具

### 模板渲染

```rust
fn render_template(template: &str, context: &HashMap<String, serde_json::Value>) -> String
```

在模板字符串中替换 `{variable}` 占位符。

### 日志记录

使用 `tracing` crate 进行结构化日志记录：

```rust
use tracing::{info, warn, error, debug};

info!("应用程序启动");
warn!("配置文件未找到，使用默认配置");
error!("数据库连接失败: {}", error_message);
debug!("处理请求: {:?}", request);
```

## 测试工具

### Mock 客户端

#### `MockSmtpClient`
用于测试的 SMTP 客户端模拟。

```rust
pub struct MockSmtpClient {
    pub sent_messages: Arc<Mutex<Vec<OutgoingMessage>>>,
}

impl MockSmtpClient {
    pub fn new() -> Self
    pub fn get_sent_messages(&self) -> Vec<OutgoingMessage>
}
```

#### `MockLlmClient`
用于测试的 LLM 客户端模拟。

```rust
pub struct MockLlmClient {
    pub response_content: String,
}

impl MockLlmClient {
    pub fn new(response: &str) -> Self
}
```

### 测试辅助函数

```rust
pub fn create_test_email() -> OutgoingMessage
pub fn create_test_llm_request() -> LlmRequest
pub fn setup_test_config() -> Config
```

## 扩展接口

### 自定义 LLM 客户端

要实现自定义的 LLM 客户端，需要实现 `LlmClient` trait：

```rust
use async_trait::async_trait;
use sentio_llm::{LlmClient, LlmRequest, LlmResponse, LlmResult};

pub struct CustomLlmClient {
    // 自定义字段
}

#[async_trait]
impl LlmClient for CustomLlmClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        // 自定义实现
    }
}
```

### 自定义邮件客户端

要实现自定义的邮件客户端，需要实现 `SmtpClient` trait：

```rust
use async_trait::async_trait;
use sentio_email::{SmtpClient, OutgoingMessage, EmailAddress, MessageId, EmailResult};

pub struct CustomSmtpClient {
    // 自定义字段
}

#[async_trait]
impl SmtpClient for CustomSmtpClient {
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId> {
        // 自定义实现
    }
    
    async fn verify_address(&self, address: &EmailAddress) -> EmailResult<bool> {
        // 自定义实现
    }
    
    async fn connect(&mut self) -> EmailResult<()> {
        // 自定义实现
    }
    
    async fn disconnect(&mut self) -> EmailResult<()> {
        // 自定义实现
    }
    
    async fn is_connected(&self) -> bool {
        // 自定义实现
    }
}
```

---

*API 文档版本：v1.0.0*
*最后更新时间：2025-07-07*