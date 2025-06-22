# Sentio AI 文档索引

本目录包含 Sentio AI 邮件伙伴系统的完整技术文档。

## 📋 核心文档

### 项目概览
- **[README.md](../README.md)** - 项目主文档，快速开始指南
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - 项目当前状态和完成度
- **[TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md)** - 技术架构和设计文档

### 实现总结
- **[MEMORY_SERVICE_COMPLETION_SUMMARY.md](MEMORY_SERVICE_COMPLETION_SUMMARY.md)** - 记忆服务完整实现总结
- **[LLM_SERVICE_IMPLEMENTATION_SUMMARY.md](LLM_SERVICE_IMPLEMENTATION_SUMMARY.md)** - LLM 服务实现总结

## 🏗️ 服务文档

每个微服务都有独立的 README 文档：

| 服务 | 文档路径 | 功能描述 |
|------|----------|----------|
| **Core** | [services/core/README.md](../services/core/README.md) | 核心业务逻辑和服务协调 |
| **Memory** | [services/memory/README.md](../services/memory/README.md) | 记忆数据管理 (MongoDB) |
| **LLM** | [services/llm/README.md](../services/llm/README.md) | 大语言模型集成 |
| **Email** | [services/email/README.md](../services/email/README.md) | 邮件发送服务 (SMTP) |
| **Telemetry** | [services/telemetry/README.md](../services/telemetry/README.md) | 日志和监控 |
| **Shared Logic** | [services/shared_logic/README.md](../services/shared_logic/README.md) | 配置管理和共享工具 |

## 📚 开发历史文档

以下文档记录了项目的开发过程和历史状态：

### 完成的里程碑
- **[FINAL_PROJECT_COMPLETION_SUMMARY.md](FINAL_PROJECT_COMPLETION_SUMMARY.md)** - 项目最终完成总结
- **[DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md)** - 原始开发计划

### 历史记录
- **[CLEANUP_SUMMARY.md](CLEANUP_SUMMARY.md)** - 项目清理过程记录
- **[DOCUMENTATION_SYNC_SUMMARY.md](DOCUMENTATION_SYNC_SUMMARY.md)** - 文档同步更新记录
- **[DOC_TEST_FIX_SUMMARY.md](DOC_TEST_FIX_SUMMARY.md)** - 文档测试修复记录
- **[IMAP_REMOVAL_SUMMARY.md](IMAP_REMOVAL_SUMMARY.md)** - IMAP 功能移除记录

## 🎯 文档使用指南

### 快速开始
1. 从 [README.md](../README.md) 开始了解项目概况
2. 查看 [PROJECT_STATUS.md](PROJECT_STATUS.md) 了解当前状态
3. 根据需要查阅具体服务的文档

### 开发者指南
1. 阅读 [TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md) 了解架构设计
2. 查看相关服务的 README 了解具体实现
3. 参考实现总结文档了解关键实现细节

### 部署运维
1. 查看 [README.md](../README.md) 的部署章节
2. 参考各服务文档的配置说明
3. 查看 [PROJECT_STATUS.md](PROJECT_STATUS.md) 的部署检查清单

## 📖 文档规范

### 文档结构
- 每个文档都有清晰的标题和目录
- 使用 Markdown 格式，支持代码高亮
- 包含实际的代码示例和配置

### 更新频率
- 核心文档在重大功能完成时更新
- 服务文档在对应服务修改时同步更新
- 状态文档定期更新以反映最新进展

### 维护原则
- 保持文档的准确性和时效性
- 删除过时和冗余的文档
- 确保文档间的一致性和关联性

---

**最后更新**: 2025年6月22日  
**文档版本**: v2.0  
**维护状态**: 🟢 活跃维护
