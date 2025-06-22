# Sentio LLM 服务

> 提供与大语言模型交互的核心功能，支持邮件分析和智能回复生成

## 🎯 功能特性

### 核心功能

- **文本生成**: 基于提示词生成高质量文本内容
- **邮件分析**: 深度分析邮件的情感、意图和关键信息
- **智能回复**: 基于分析结果生成个性化邮件回复
- **推理链**: 支持复杂的多步推理任务

### 技术特性

- **多 LLM 支持**: 支持 DeepSeek、OpenAI 等多种 LLM 提供商
- **健壮性设计**: 自动重试、超时控制、错误处理
- **类型安全**: 强类型接口，编译时验证
- **可观测性**: 完整的请求追踪和性能监控

## 🏗️ 架构设计

```text
LLM 服务架构
├── client.rs          # LLM 客户端实现
│   ├── LlmClient      # 客户端 trait 接口
│   └── DeepSeekClient # DeepSeek API 实现
├── types.rs           # 核心数据类型
│   ├── LlmRequest     # 请求结构
│   ├── LlmResponse    # 响应结构
│   └── EmailAnalysis  # 邮件分析结果
└── error.rs           # 错误处理
    ├── LlmError       # 错误类型定义
    └── 重试逻辑        # 自动重试机制
```

## 📖 使用指南

### 基础用法

```rust
use sentio_llm::{DeepSeekClient, LlmClient, LlmRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化 LLM 客户端
    let client = DeepSeekClient::new()?;
    
    // 2. 创建请求
    let request = LlmRequest::new(
        "你是一个专业的助手".to_string(),
        "请介绍一下 Rust 语言的特点".to_string(),
    );
    
    // 3. 生成响应
    let response = client.generate_response(&request).await?;
    println!("回复: {}", response.content);
    
    Ok(())
}
```

### 邮件分析

```rust
use sentio_llm::{DeepSeekClient, LlmClient, EmailAnalysisRequest};
use chrono::Utc;

async fn analyze_email() -> Result<(), Box<dyn std::error::Error>> {
    let client = DeepSeekClient::new()?;
    
    let email_request = EmailAnalysisRequest {
        email_id: "email-123".to_string(),
        sender: "user@example.com".to_string(),
        subject: "紧急：项目进度讨论".to_string(),
        body: "您好，我想了解一下项目的最新进度...".to_string(),
        received_at: Utc::now(),
    };
    
    let analysis = client.analyze_email(&email_request).await?;
    
    println!("情感分析: {:?}", analysis.sentiment);
    println!("意图识别: {:?}", analysis.intent);
    println!("紧急程度: {:?}", analysis.urgency_level);
    
    Ok(())
}
```

### 智能回复生成

```rust
async fn generate_reply() -> Result<(), Box<dyn std::error::Error>> {
    let client = DeepSeekClient::new()?;
    
    // 假设已经有了邮件分析结果
    let analysis = get_email_analysis();
    let context = "用户询问项目进度，需要提供详细的状态更新";
    
    let reply = client.generate_reply(&analysis, context).await?;
    println!("生成的回复: {}", reply);
    
    Ok(())
}
```

## ⚙️ 配置说明

LLM 服务通过 `shared_logic` 模块获取全局配置。支持的配置项：

### 环境变量配置

```bash
# LLM 服务配置
SENTIO_LLM__PROVIDER=deepseek           # LLM 提供商
SENTIO_LLM__API_KEY=your-api-key        # API 密钥
SENTIO_LLM__BASE_URL=https://api.deepseek.com  # API 基础 URL
SENTIO_LLM__MODEL=deepseek-chat         # 默认模型
SENTIO_LLM__TIMEOUT=120                 # 请求超时（秒）
SENTIO_LLM__MAX_RETRIES=3               # 最大重试次数
```

### 配置文件

```toml
[llm]
provider = "deepseek"
api_key = "your-api-key"
base_url = "https://api.deepseek.com"
model = "deepseek-chat"
timeout = 120
max_retries = 3
```

## 🔧 错误处理

服务提供了完整的错误处理机制：

### 错误类型

- `ApiRequestFailed`: API 请求失败
- `InvalidApiResponse`: API 响应格式无效
- `AuthenticationFailed`: 认证失败
- `NetworkError`: 网络连接错误
- `Timeout`: 请求超时
- `MaxRetriesExceeded`: 重试次数耗尽

### 重试策略

- 自动重试可恢复的错误（网络、超时等）
- 指数退避算法减少服务压力
- 可配置的最大重试次数

## 📊 监控和日志

### 结构化日志

```rust
// 请求开始
tracing::debug!(
    request_id = %request.id,
    model = %request.parameters.model,
    "Sending request to LLM API"
);

// 请求完成
tracing::info!(
    request_id = %request.id,
    latency_ms = latency.as_millis(),
    tokens_used = response.token_usage.total_tokens,
    "LLM request completed"
);

// 错误记录
tracing::error!(
    error = %error,
    request_id = %request.id,
    "LLM request failed"
);
```

### 性能指标

- 请求延迟 (latency_ms)
- 令牌使用量 (token_usage)
- 重试次数 (retry_count)
- 成功率统计

## 🧪 测试

### 单元测试

```bash
# 运行所有测试
cargo test -p sentio_llm

# 运行特定测试
cargo test -p sentio_llm test_deepseek_client

# 运行文档测试
cargo test -p sentio_llm --doc
```

### 集成测试

```bash
# 需要设置真实的 API 密钥
export SENTIO_LLM__API_KEY=your-test-api-key
cargo test -p sentio_llm --test integration
```

## 🤝 扩展指南

### 添加新的 LLM 提供商

1. 实现 `LlmClient` trait
2. 添加对应的配置选项
3. 在客户端工厂中注册新的提供商

```rust
pub struct OpenAIClient {
    // 实现细节
}

#[async_trait::async_trait]
impl LlmClient for OpenAIClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        // OpenAI API 实现
    }
    
    // 其他方法实现...
}
```

### 自定义推理步骤

```rust
pub struct CustomReasoningStep {
    // 步骤配置
}

#[async_trait::async_trait]
impl ReasoningStep for CustomReasoningStep {
    async fn execute(&self, input: &ReasoningInput) -> ReasoningResult<ReasoningOutput> {
        // 自定义推理逻辑
    }
}
```

## 📄 API 参考

详细的 API 文档请参考代码中的文档注释，或运行：

```bash
cargo doc -p sentio_llm --open
```
