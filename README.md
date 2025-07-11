# Sentio

智能邮件处理系统 - 一个使用 Rust 构建的邮件自动化工具，专门处理来自特定邮箱的邮件。

## 特性

- 🔒 仅处理指定邮箱发送的邮件（可配置）
- 🤖 AI 驱动的邮件分析和自动回复
- 📧 SMTP 邮件发送支持
- 💾 持久化内存存储
- 🔧 简单的模块化架构

## 快速开始

### 配置

1. 复制示例配置文件：

```bash
cp sentio.example.toml sentio.toml
```

2. 编辑 `sentio.toml` 文件，设置你的配置：

```toml
[email]
# 仅处理来自此邮箱的邮件
allowed_sender = "1607033217@qq.com"

[email.smtp]
host = "smtp.gmail.com"
port = 587
username = "your-email@example.com"
password = "your-app-password"

[llm]
api_key = "your-deepseek-api-key"
```

### 构建和运行

```bash
# 构建项目
cargo build --release

# 运行
cargo run
```

## 工作原理

1. **邮件过滤**: 系统只处理来自配置中 `allowed_sender` 指定邮箱的邮件
2. **智能分析**: 使用 AI 分析邮件内容并进行分类
3. **自动回复**: 基于分析结果生成合适的回复
4. **记忆存储**: 保存交互历史以提供上下文

## 架构

项目采用单一 crate 的扁平化设计，遵循 YAGNI 原则：

```
src/
├── main.rs       # 应用入口
├── config.rs     # 配置管理
├── workflow.rs   # 核心邮件处理流程
├── email/        # SMTP 邮件客户端
├── llm/          # AI 集成 (DeepSeek)
└── memory/       # 持久化存储
```

## 注意事项

- 目前系统创建工作流后等待邮件输入，实际的邮件接收需要通过 IMAP 或其他方式实现
- 确保 `allowed_sender` 配置正确，系统会忽略其他邮箱的邮件

## 开发

```bash
# 格式化代码
cargo fmt

# 运行 linter
cargo clippy

# 运行测试
cargo test
```

## License

MIT