# Sentio AI 邮件伙伴系统

> 基于推理增强型 LLM 的个性化记忆 AI 邮件伙伴系统

## 🎯 项目概述

Sentio 是一个智能邮件助手系统，能够：

- 理解和记忆用户的个人信息、偏好和历史交互
- 基于深度思考链（Chain of Thought）生成个性化回复
- 维护长期的用户关系模型和交互策略
- 提供前瞻性的建议和深刻的洞察

## 🏗️ 项目架构

```
services/
├── shared_logic/     # 共享逻辑和全局配置管理
├── core/            # 核心服务 (邮件处理和 LLM 交互)
├── telemetry/       # 遥测和日志服务
└── memory/          # 记忆数据模型和存储
```

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- MongoDB (可选，默认使用本地实例)
- DeepSeek API 密钥 (或其他 LLM API)

### 安装和配置

1. **克隆项目**

   ```bash
   git clone https://github.com/your-org/sentio-ai.git
   cd sentio-ai
   ```

2. **配置环境变量**

   ```bash
   cp .env.example .env
   # 编辑 .env 文件，填入您的配置
   ```

3. **构建项目**

   ```bash
   cargo build --workspace
   ```

4. **运行系统**

   ```bash
   cargo run --bin sentio_core
   ```

### 配置说明

系统支持通过配置文件 (`Config.toml`) 和环境变量进行配置。环境变量具有更高优先级。

#### 主要配置项

| 配置项 | 环境变量 | 默认值 | 说明 |
|--------|----------|--------|------|
| 数据库 URL | `SENTIO_DATABASE__URL` | `mongodb://localhost:27017/sentio` | MongoDB 连接字符串 |
| LLM API 密钥 | `SENTIO_LLM__API_KEY` | - | DeepSeek 或其他 LLM 服务的 API 密钥 |
| 日志级别 | `SENTIO_TELEMETRY__LOG_LEVEL` | `info` | 日志级别 (trace/debug/info/warn/error) |
| 服务器端口 | `SENTIO_SERVER__PORT` | `8080` | HTTP 服务器监听端口 |

#### 邮件配置示例

```bash
# IMAP 配置
SENTIO_EMAIL__IMAP__HOST=imap.gmail.com
SENTIO_EMAIL__IMAP__PORT=993
SENTIO_EMAIL__IMAP__USERNAME=your-email@gmail.com
SENTIO_EMAIL__IMAP__PASSWORD=your-app-password
SENTIO_EMAIL__IMAP__USE_TLS=true

# SMTP 配置
SENTIO_EMAIL__SMTP__HOST=smtp.gmail.com
SENTIO_EMAIL__SMTP__PORT=587
SENTIO_EMAIL__SMTP__USERNAME=your-email@gmail.com
SENTIO_EMAIL__SMTP__PASSWORD=your-app-password
SENTIO_EMAIL__SMTP__USE_TLS=true
```

## 📚 详细文档

### 服务文档

- [Shared Logic 服务](services/shared_logic/README.md) - 全局配置和共享逻辑
- [Core 服务](services/core/README.md) - 核心邮件处理逻辑
- [Telemetry 服务](services/telemetry/README.md) - 日志和监控
- [Memory 服务](services/memory/README.md) - 数据模型和记忆管理

### 技术文档

- [技术设计文档](TECHNICAL_DESIGN.md) - 完整的技术架构和设计说明
- [开发指南](GUIDE.md) - LLM 代码生成参考指令集

## 🛠️ 开发

### 项目结构

```
├── services/              # 微服务模块
│   ├── shared_logic/      # 共享逻辑
│   │   ├── src/
│   │   │   ├── config.rs  # 全局配置管理
│   │   │   ├── types.rs   # 共享类型定义
│   │   │   └── lib.rs     # 模块入口
│   │   └── README.md
│   ├── core/              # 核心服务
│   ├── telemetry/         # 遥测服务
│   └── memory/            # 记忆管理
├── Config.toml            # 配置文件
├── .env.example           # 环境变量模板
└── Cargo.toml             # Workspace 配置
```

### 构建和测试

```bash
# 构建所有服务
cargo build --workspace

# 运行测试
cargo test --workspace

# 检查代码质量
cargo clippy --workspace

# 格式化代码
cargo fmt --workspace
```

### 添加新服务

1. 在 `services/` 目录下创建新的服务目录
2. 更新根目录的 `Cargo.toml` 中的 `members` 列表
3. 在新服务中添加对 `shared_logic` 的依赖以访问全局配置

## 🔧 架构设计

### 核心原则

1. **模块化设计** - 每个服务都有明确的职责边界
2. **配置驱动** - 所有配置通过统一的配置系统管理
3. **类型安全** - 使用 Rust 的类型系统确保运行时安全
4. **可观测性** - 内置完整的日志和监控系统

### 数据流

```
邮件接收 → 内容解析 → 记忆检索 → LLM 推理 → 回复生成 → 邮件发送
     ↓
记忆更新 ← 交互记录 ← 策略调整 ← 反思分析 ←
```

## 🤝 贡献

欢迎贡献代码！请确保：

1. 遵循项目的代码风格和命名约定
2. 为新功能添加相应的测试
3. 更新相关文档
4. 提交前运行 `cargo clippy` 和 `cargo fmt`

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## ⭐ 致谢

- [DeepSeek](https://www.deepseek.com/) - 提供强大的 LLM API
- [Tokio](https://tokio.rs/) - 异步运行时
- [Serde](https://serde.rs/) - 序列化框架
- [Tracing](https://tracing.rs/) - 结构化日志
