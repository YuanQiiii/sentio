# 项目待办事项 (TODO.md)

## 核心服务 (services/core)

### `services/core/src/lib.rs`
*   **完善 `demonstrate_workflow` 函数**: 目前该函数仅为占位符，需要扩展以实际演示 `EmailWorkflow` 的完整功能，例如调用 `workflow.process_email(...)`。
*   **`MockSmtpClient` 的处理**: `lib.rs` 中定义的 `MockSmtpClient` 仅用于演示。考虑更健壮的测试策略，例如在测试模块中集中管理模拟对象，或引入更通用的模拟框架。

### `services/core/src/workflow.rs`
*   **`EmailWorkflow` 的实际业务逻辑**: `EmailWorkflow` 结构体及其 `new_with_clients` 方法已定义，但其核心业务逻辑（例如处理邮件、调用 LLM 和 Email 客户端）尚未实现。

## 邮件服务 (services/email)

### `services/email/src/client.rs`
*   **实际的 SMTP 发送逻辑**: `SimpleSmtpClient::send_message` 方法中需要实现真正的 SMTP 邮件发送逻辑，目前仅为模拟。
*   **更复杂的邮件地址验证**: `SimpleSmtpClient::verify_address` 方法中的邮件地址验证逻辑可以更复杂和健壮，例如使用正则表达式或进行 DNS MX 记录查询。
*   **实际的连接和断开逻辑**: `SimpleSmtpClient::connect` 和 `disconnect` 方法需要实现与 SMTP 服务器的实际连接和断开逻辑。
*   **测试配置**: `test_smtp_client_creation` 测试需要有效的配置才能通过，应考虑使用模拟配置进行测试。

## LLM 服务 (services/llm)

### `services/llm/src/client.rs`
*   **LLM 客户端的完整性**: `DeepSeekClient` 实现了基本的 `generate_response`，但可能需要根据实际需求扩展更多功能，例如流式响应处理、更复杂的错误分类等。

## 记忆服务 (services/memory)

### `services/memory/src/memory_data.rs`
*   **`update_memory_corpus` 的简化逻辑**: `update_memory_corpus` 函数中的更新逻辑目前是简化的，需要扩展以处理 `MemoryCorpus` 中所有特定字段的更新。
*   **`UserStatistics` 中的 `account_created` 字段**: `get_user_statistics` 函数中 `account_created` 字段目前硬编码为 `Utc::now()`，需要从实际的用户档案或记忆体中获取。
*   **持久化存储**: `MemoryDataRepository` 是一个内存实现。项目结构暗示了对持久化存储（如 MongoDB）的需求。这是一个重要的架构待办事项，需要实现一个真正的数据库集成。

## 遥测服务 (services/telemetry)

### `services/telemetry/src/lib.rs`
*   **文件输出支持**: `init_telemetry_with_config` 函数中 `log_file` 的支持尚未实现。

## 共享逻辑 (services/shared_logic)

### `services/shared_logic/src/config.rs`
*   **`database.max_connections` 配置**: `Config::from_env` 和 `load_config` 中有 `database.max_connections` 的默认配置，但目前没有实际的数据库连接池使用此配置。这与记忆服务的持久化存储待办事项相关。

## 整体改进

*   **错误处理细化**: 检查所有 `Result` 和 `Error` 类型，确保错误信息足够详细，并且错误类型能够准确反映问题。
*   **日志记录增强**: 确保关键操作和潜在问题都有适当的日志记录级别（debug, info, warn, error）。
*   **性能优化**: 对于数据密集型操作（例如记忆搜索），考虑潜在的性能瓶颈并进行优化。
*   **安全审计**: 对所有外部输入和敏感数据处理进行安全审计，确保符合最佳实践。
*   **文档完善**: 补充和完善代码注释，特别是对于复杂逻辑和公共接口。
*   **测试覆盖率**: 评估并提高单元测试和集成测试的覆盖率，确保所有关键路径都经过测试。
