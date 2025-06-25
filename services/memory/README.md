# Sentio Memory Service

## 🎯 概述

`sentio_memory` 是 Sentio AI 邮件伙伴系统的记忆管理服务，提供完整的用户记忆数据存储、检索和管理功能。

## ✨ 功能特性

- **🧠 完整记忆模型**: 5种记忆类型的完整实现
- **🗄️ MongoDB 集成**: 高性能的 NoSQL 数据库后端
- **🔒 类型安全**: Rust 强类型系统保证数据完整性
- **⚡ 异步操作**: 基于 Tokio 的高性能异步 I/O
- **🔄 自动索引**: 智能的数据库索引优化
- **📊 序列化支持**: 完整的 JSON/BSON 序列化

## 🏗️ 核心架构

### 记忆仓储模式

```rust
// 抽象仓储接口
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    async fn save_memory_corpus(&self, corpus: &MemoryCorpus) -> MemoryResult<()>;
    async fn get_memory_corpus(&self, user_id: &str) -> MemoryResult<Option<MemoryCorpus>>;
    async fn save_interaction(&self, user_id: &str, interaction: &InteractionLog) -> MemoryResult<()>;
    // ... 更多方法
}

// MongoDB 具体实现
pub struct MongoMemoryRepository {
    database: Database,
    memory_corpus_collection: Collection<MemoryCorpus>,
    interaction_collection: Collection<InteractionLog>,
    // ...
}
```

### 数据模型层次

```text
MemoryCorpus (用户完整记忆)
├── CoreProfile (个人档案)
├── EpisodicMemory (情节记忆)
│   └── InteractionLog[] (交互历史)
├── SemanticMemory (语义记忆)
│   ├── PreferencesAndDislikes (偏好)
│   ├── HabitPattern[] (习惯模式)
│   └── SignificantEvent[] (重要事件)
├── ActionStateMemory (行动记忆)
│   ├── Task[] (待办事项)
│   └── Plan[] (未来计划)
└── StrategicInferentialMemory (策略记忆)
    ├── UserModelHypothesis[] (用户假设)
    └── CommunicationStrategy (沟通策略)
```

## 🚀 快速开始

### 基本使用

```rust
use sentio_memory::{MongoMemoryRepository, InteractionLog, MessageDirection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建仓储实例
    let repo = MongoMemoryRepository::new().await?;
    
    // 创建交互记录
    let interaction = InteractionLog::new(
        "user123".to_string(),
        MessageDirection::Inbound,
        "用户的邮件内容摘要".to_string(),
    );
    
    // 保存交互
    repo.save_interaction("user123", &interaction).await?;
    
    // 检索最近交互
    let recent = repo.get_recent_interactions("user123", 10).await?;
    println!("最近 {} 条交互", recent.len());
    
    Ok(())
}
```

### 配置要求

```bash
# 环境变量
SENTIO_DATABASE_URL=
SENTIO_DATABASE_MAX_CONNECTIONS=10
SENTIO_DATABASE_CONNECT_TIMEOUT=30
```

## 📊 数据模型详解

### 1. 交互记录 (InteractionLog)

记录用户与 AI 的每次交互：

```rust
pub struct InteractionLog {
    pub log_id: String,                         // 唯一标识
    pub user_id: String,                        // 用户 ID
    pub timestamp: DateTime<Utc>,               // 时间戳
    pub direction: MessageDirection,            // 方向 (Inbound/Outbound)
    pub summary: String,                        // 内容摘要
    pub emotional_tone: Vec<String>,            // 情感色调
    pub key_topics: Vec<String>,                // 关键话题
    pub llm_model_version: String,              // 模型版本
    pub cost_usd: Option<f64>,                  // 成本
}

// 支持便捷创建
let log = InteractionLog::new(user_id, direction, summary);
```

### 2. 核心档案 (CoreProfile)

用户的基本信息和人格特征：

```rust
pub struct CoreProfile {
    pub name: Option<String>,                   // 姓名
    pub age: Option<u32>,                       // 年龄
    pub occupation: Option<String>,             // 职业
    pub relationships: Vec<Relationship>,       // 人际关系
    pub personality_traits: Vec<String>,        // 性格特征
    pub current_life_summary: Option<String>,   // 生活状态
}
```

### 3. 语义记忆 (SemanticMemory)

抽象概念和长期知识：

```rust
pub struct SemanticMemory {
    pub preferences_and_dislikes: PreferencesAndDislikes,
    pub habits_and_patterns: Vec<HabitPattern>,
    pub significant_events: Vec<SignificantEvent>,
    pub skills_and_expertise: Vec<SkillExpertise>,
    pub values_and_beliefs: Vec<String>,
}
```

## 🛠️ 开发指南

### 添加新的记忆类型

1. **定义数据结构**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMemoryType {
    pub field1: String,
    pub field2: i32,
    // ...
}
```

2. **集成到主记忆体**:

```rust
pub struct MemoryCorpus {
    // ... 现有字段
    pub new_memory: NewMemoryType,
}
```

3. **更新仓储接口**:

```rust
#[async_trait]
pub trait MemoryRepository {
    // ... 现有方法
    async fn update_new_memory(&self, user_id: &str, data: &NewMemoryType) -> MemoryResult<()>;
}
```

### 性能优化建议

- **批量操作**: 使用 `save_interactions` 批量保存
- **查询优化**: 利用数据库索引进行高效查询
- **连接池**: 调整 `max_connections` 参数
- **内存管理**: 定期清理旧数据

## 🧪 测试

### 运行测试

```bash
# 单元测试
cargo test -p sentio_memory

# 集成测试
cargo test -p sentio_memory --test integration_tests

# 所有测试
cargo test --workspace
```

### 测试覆盖

- ✅ 数据模型序列化/反序列化
- ✅ 仓储接口模拟实现
- ✅ MongoDB 连接和基本操作
- ✅ 错误处理和重试机制

## 📈 性能指标

### 操作延迟

| 操作 | 本地 MongoDB | 云端 MongoDB |
|------|-------------|-------------|
| 保存交互 | < 5ms | < 50ms |
| 查询用户记忆 | < 10ms | < 100ms |
| 批量插入 | < 20ms | < 200ms |

### 存储效率

- **平均用户记忆体**: ~50KB
- **单次交互记录**: ~2KB
- **索引开销**: ~20% 额外存储

## 🔧 故障排除

### 常见问题

1. **连接超时**:

   ```bash
   # 检查 MongoDB 服务状态
   systemctl status mongod
   
   # 调整超时配置
   SENTIO_DATABASE_CONNECT_TIMEOUT=60
   ```

2. **内存使用过高**:

   ```rust
   // 限制查询结果数量
   let recent = repo.get_recent_interactions(user_id, 100).await?;
   ```

3. **索引性能问题**:

   ```javascript
   // MongoDB shell 中检查索引
   db.interactions.getIndexes()
   ```

## 📚 相关文档

- [技术设计文档](../../docs/TECHNICAL_DESIGN.md)
- [项目主文档](../../README.md)

---

**维护状态**: 🟢 生产就绪  
**最后更新**: 2025年6月22日
