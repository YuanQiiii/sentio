# Sentio Memory Service

## 概述

`sentio_memory` 是 Sentio AI 邮件伙伴系统的内存管理服务，定义了系统中所有核心数据结构，用于存储和管理用户的记忆数据。

## 功能特性

- **完整的记忆模型**: 实现了技术设计文档中定义的完整记忆体数据结构
- **类型安全**: 使用 Rust 的强类型系统确保数据完整性
- **序列化支持**: 所有数据结构都支持 JSON 序列化/反序列化
- **时间跟踪**: 内置时间戳管理，使用 chrono 库处理时间

## 数据结构概览

### 1. MemoryCorpus（记忆体）

每个用户的完整记忆数据的根结构：

```rust
pub struct MemoryCorpus {
    pub user_id: String,                                    // 用户唯一标识
    pub version: String,                                    // 记忆体格式版本
    pub created_at: chrono::DateTime<chrono::Utc>,         // 创建时间
    pub updated_at: chrono::DateTime<chrono::Utc>,         // 最后更新时间
    pub core_profile: CoreProfile,                          // 核心档案
    pub episodic_memory: EpisodicMemory,                   // 情节记忆
    pub semantic_memory: SemanticMemory,                   // 语义记忆
    pub action_state_memory: ActionStateMemory,           // 行动状态记忆
    pub strategic_inferential_memory: StrategicInferentialMemory, // 战略推断记忆
}
```

### 2. 核心档案（CoreProfile）

用户的基本信息和个人档案：

- 个人基本信息（姓名、年龄、性别、城市、职业）
- 重要人际关系
- 基本个性特征
- 当前生活状态摘要

### 3. 情节记忆（EpisodicMemory）

具体的交互历史记录：

```rust
pub struct InteractionLog {
    pub log_id: String,                                    // 日志唯一ID
    pub email_id: Option<String>,                          // 关联的邮件ID
    pub timestamp: chrono::DateTime<chrono::Utc>,          // 交互时间
    pub direction: MessageDirection,                       // 消息方向（入站/出站）
    pub summary: String,                                   // 交互摘要
    pub emotional_tone: Vec<String>,                       // 情感色调
    pub key_topics: Vec<String>,                           // 关键话题
    pub llm_model_version: String,                         // 使用的LLM模型版本
    pub reasoning_chain_snapshot: Option<String>,          // 思考链快照
    pub cost_usd: Option<f64>,                            // 交互成本
}
```

### 4. 语义记忆（SemanticMemory）

抽象的概念和知识：

- **偏好和厌恶**：喜好、兴趣、食物偏好等
- **习惯和模式**：行为习惯、频率、置信度
- **重要事件**：关键生活事件、情感影响
- **技能专长**：技能水平、相关经验

### 5. 行动状态记忆（ActionStateMemory）

当前和未来的计划：

- **当前任务**：待办事项、优先级、状态
- **未来计划**：计划描述、时间范围、相关目标
- **跟进事项**：需要跟进的内容、建议时间

### 6. 战略推断记忆（StrategicInferentialMemory）

AI的假设和策略：

- **用户模型假设**：对用户的推断、置信度、支持证据
- **关系目标**：短期、中期、长期目标
- **沟通策略**：语气风格、话题偏好
- **自我反思**：AI的反思记录和改进

## 使用方法

### 创建新的记忆体

```rust
use sentio_memory::MemoryCorpus;

// 创建默认的记忆体
let mut memory = MemoryCorpus::default();
memory.user_id = "user@example.com".to_string();

// 或者创建带初始数据的记忆体
let memory = MemoryCorpus {
    user_id: "user@example.com".to_string(),
    version: "2.1".to_string(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    // ... 其他字段
    ..Default::default()
};
```

### 添加交互记录

```rust
use sentio_memory::{InteractionLog, MessageDirection};

let interaction = InteractionLog {
    log_id: uuid::Uuid::new_v4().to_string(),
    email_id: Some("msg-123".to_string()),
    timestamp: chrono::Utc::now(),
    direction: MessageDirection::Inbound,
    summary: "用户询问关于工作压力的建议".to_string(),
    emotional_tone: vec!["stressed".to_string(), "seeking_help".to_string()],
    key_topics: vec!["work".to_string(), "stress".to_string()],
    llm_model_version: "deepseek-v2".to_string(),
    reasoning_chain_snapshot: Some("...".to_string()),
    cost_usd: Some(0.025),
};

memory.episodic_memory.interaction_log.push(interaction);
memory.updated_at = chrono::Utc::now();
```

### 更新用户假设

```rust
use sentio_memory::UserModelHypothesis;

let hypothesis = UserModelHypothesis {
    hypothesis_id: "hyp_001".to_string(),
    hypothesis: "用户正在经历职业倦怠".to_string(),
    confidence: 0.75,
    status: "active".to_string(),
    evidence: vec!["log_001".to_string(), "log_003".to_string()],
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};

memory.strategic_inferential_memory.user_model_hypotheses.push(hypothesis);
```

## 序列化和反序列化

所有数据结构都支持 JSON 序列化：

```rust
use serde_json;

// 序列化为 JSON
let json = serde_json::to_string_pretty(&memory)?;

// 从 JSON 反序列化
let memory: MemoryCorpus = serde_json::from_str(&json)?;

// 保存到文件
std::fs::write("user_memory.json", json)?;

// 从文件加载
let json = std::fs::read_to_string("user_memory.json")?;
let memory: MemoryCorpus = serde_json::from_str(&json)?;
```

## 数据验证

### 时间戳管理

```rust
// 创建新记录时设置时间戳
let now = chrono::Utc::now();
let mut memory = MemoryCorpus::default();
memory.created_at = now;
memory.updated_at = now;

// 更新记录时更新时间戳
memory.updated_at = chrono::Utc::now();
```

### 置信度验证

```rust
// 确保置信度在有效范围内
fn validate_confidence(confidence: f64) -> Result<f64, String> {
    if confidence >= 0.0 && confidence <= 1.0 {
        Ok(confidence)
    } else {
        Err("Confidence must be between 0.0 and 1.0".to_string())
    }
}
```

## 枚举类型

### MessageDirection

```rust
pub enum MessageDirection {
    Inbound,  // 用户发来的消息
    Outbound, // AI发出的回复
}
```

## 最佳实践

### 1. 数据完整性

- 始终保持时间戳的准确性
- 为重要的数据结构分配唯一ID
- 在更新记忆体时更新 `updated_at` 字段

### 2. 内存管理

- 定期清理过期的交互记录
- 合并相似的习惯和偏好记录
- 限制数组大小以避免内存膨胀

### 3. 数据隐私

- 避免在日志中暴露敏感的个人信息
- 实现数据脱敏功能用于调试
- 遵循相关的隐私法规要求

## 性能考虑

- 使用 `Vec` 存储列表数据，对于大量数据考虑使用数据库索引
- JSON 序列化可能对大型记忆体产生性能影响，考虑使用二进制格式
- 实现增量更新而不是全量保存

## 扩展性

数据结构设计考虑了未来的扩展需求：

- 使用 `Option<T>` 字段支持可选数据
- 版本号字段支持数据结构迁移
- 灵活的 HashMap 结构支持动态字段

## 依赖项

- `serde`: JSON 序列化/反序列化
- `chrono`: 时间处理
- `anyhow`: 错误处理

## 数据迁移

当需要升级数据结构时：

1. 更新版本号
2. 实现数据迁移函数
3. 保持向后兼容性
4. 提供迁移工具和文档
