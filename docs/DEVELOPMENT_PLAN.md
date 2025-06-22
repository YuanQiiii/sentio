# 开发计划 - 阶段二：核心功能实现

> 严格遵循 GUIDE.md 中的开发流程和原则

## 🎯 阶段目标

基于 GUIDE.md 的**从零创建新功能**策略，实现核心邮件处理和 LLM 交互功能。

## 📐 设计原则遵循

### 1. 情境至上 (Context is King)

- ✅ **已分析**: 现有项目结构、依赖关系、配置系统
- ✅ **风格一致性**: 遵循现有 Rust 项目的命名约定和模式

### 2. 健壮性是底线 (Robustness is Non-Negotiable)

- 🎯 **悲观主义编程**: 所有外部输入（邮件内容、API响应）都需验证
- 🎯 **无副作用**: 核心逻辑与副作用（IO、网络）解耦

### 3. 开发者体验优先 (Developer Experience First)

- 🎯 **代码即文档**: 清晰的函数和变量命名
- 🎯 **避免魔法**: 使用主流 Rust 惯用法

### 4. 安全是内置属性 (Security is Built-in)

- 🎯 **零信任**: 验证所有邮件内容和 API 响应
- 🎯 **凭证管理**: API 密钥通过环境变量管理

## 🏗️ 模块化设计

### 阶段二-A：邮件服务 (`services/email`)

#### 接口先行设计

```rust
// services/email/src/lib.rs
pub mod client;
pub mod types;
pub mod error;

// 核心接口
pub trait EmailClient {
    async fn connect(&mut self) -> EmailResult<()>;
    async fn fetch_messages(&self, folder: &str) -> EmailResult<Vec<EmailMessage>>;
    async fn send_message(&self, message: &OutgoingMessage) -> EmailResult<MessageId>;
    async fn mark_as_read(&self, message_id: &MessageId) -> EmailResult<()>;
    async fn disconnect(&mut self) -> EmailResult<()>;
}

// 数据结构
pub struct EmailMessage {
    pub id: MessageId,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub body: EmailBody,
    pub received_at: DateTime<Utc>,
}
```

#### 错误处理策略

- 网络连接错误
- 认证失败
- 邮件解析错误
- 配置错误

#### 配置外置

```toml
[email.imap]
host = "imap.gmail.com"
port = 993
use_tls = true
timeout_seconds = 30

[email.smtp]
host = "smtp.gmail.com"
port = 587
use_tls = true
timeout_seconds = 30
```

### 阶段二-B：LLM 服务 (`services/llm`)

#### 接口先行设计

```rust
// services/llm/src/lib.rs
pub mod client;
pub mod types;
pub mod reasoning;

// 核心接口
pub trait LlmClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
    async fn analyze_email(&self, email: &EmailMessage) -> LlmResult<EmailAnalysis>;
    async fn generate_reply(&self, context: &ReplyContext) -> LlmResult<EmailReply>;
}

// 思考链接口
pub trait ReasoningChain {
    async fn execute(&self, input: &ReasoningInput) -> ReasoningResult<ReasoningOutput>;
    fn add_step(&mut self, step: Box<dyn ReasoningStep>);
}
```

#### 思考链实现

- 邮件内容分析步骤
- 记忆检索步骤
- 策略选择步骤
- 回复生成步骤

### 阶段二-C：记忆服务增强 (`services/memory`)

#### 数据库集成

```rust
// services/memory/src/repository.rs
pub trait MemoryRepository {
    async fn save_interaction(&self, interaction: &InteractionLog) -> MemoryResult<()>;
    async fn find_relevant_memories(&self, query: &MemoryQuery) -> MemoryResult<Vec<Memory>>;
    async fn update_user_model(&self, user_id: &UserId, model: &UserModel) -> MemoryResult<()>;
}

// MongoDB 实现
pub struct MongoMemoryRepository {
    database: Database,
}
```

### 阶段二-D：工具集 API (`services/tools`)

#### Memory_Interface 实现

```rust
// services/tools/src/memory_interface.rs
pub struct MemoryInterface {
    repository: Arc<dyn MemoryRepository>,
}

impl MemoryInterface {
    pub async fn search_memories(&self, query: &str) -> ToolResult<Vec<Memory>>;
    pub async fn save_interaction(&self, interaction: &InteractionLog) -> ToolResult<()>;
    pub async fn get_user_profile(&self, user_id: &UserId) -> ToolResult<UserProfile>;
}
```

## 📋 实施计划

### 第1周：邮件服务基础

- [ ] 创建 `services/email` 模块
- [ ] 实现 IMAP 客户端（只读功能）
- [ ] 添加邮件解析功能
- [ ] 编写单元测试（Mock IMAP 服务器）

### 第2周：LLM 集成基础

- [ ] 创建 `services/llm` 模块
- [ ] 实现 DeepSeek API 客户端
- [ ] 添加重试和错误处理机制
- [ ] 实现基础的思考链框架

### 第3周：记忆系统增强

- [ ] 添加 MongoDB 依赖和配置
- [ ] 实现 `MemoryRepository` trait
- [ ] 添加数据迁移脚本
- [ ] 编写集成测试

### 第4周：工具集和集成

- [ ] 创建 `services/tools` 模块
- [ ] 实现 Memory_Interface
- [ ] 集成所有服务到 core
- [ ] 端到端测试

## 🔒 安全考虑

### 邮件处理安全

- 邮件内容 HTML 净化
- 附件类型验证
- 防止邮件炸弹攻击

### API 安全

- API 密钥安全存储
- 请求频率限制
- 响应内容验证

### 数据安全

- 敏感信息加密存储
- 用户数据隔离
- 访问权限控制

## 🧪 测试策略

### 单元测试

- 每个模块都有对应的测试
- Mock 外部依赖（邮件服务器、LLM API、数据库）
- 覆盖边界条件和错误情况

### 集成测试

- 端到端邮件处理流程
- LLM 调用链测试
- 数据库操作测试

### 性能测试

- 大量邮件处理性能
- LLM API 调用延迟
- 内存使用监控

## 📝 文档要求

每个新服务都需要：

- README.md 文档
- API 文档
- 配置说明
- 故障排除指南

## 🚀 验收标准

每个阶段完成后需要：

- [ ] 所有测试通过
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码格式化 (`cargo fmt`)
- [ ] 文档完整且最新
- [ ] 配置项都有环境变量支持
- [ ] 错误处理覆盖所有可能的失败情况

---

**注意**: 严格按照 GUIDE.md 的原则执行，每个模块都要先设计接口，再实现功能，最后添加测试。
