# 项目待办事项 (TODO.md)

## 核心服务 (services/core)

### `services/core/src/lib.rs`

* **完善 `demonstrate_workflow` 函数**: **已完成**。`demonstrate_workflow` 函数现在模拟了邮件处理流程，并调用了 `EmailWorkflow::process_email`。
* **`MockSmtpClient` 的处理**: **已完成**。`lib.rs` 中定义的 `MockSmtpClient` 已移动到专门的测试工具模块中。

### `services/core/src/workflow.rs`

* **`EmailWorkflow` 的实际业务逻辑**: **已完成**。`EmailWorkflow` 现在包含 `process_email` 方法，该方法模拟了使用 LLM 和 Email 客户端处理邮件的逻辑。

## 邮件服务 (services/email)

### `services/email/src/client.rs`

* **实际的 SMTP 发送逻辑**: **已完成**。`SimpleSmtpClient::send_message` 已实现真实 SMTP 发送，支持收件人、抄送、密送，headers 留有注释，附件留有扩展点。
* **更复杂的邮件地址验证**: **已完成**。`SimpleSmtpClient::verify_address` 支持正则校验和可选 MX 校验，测试用例可跳过 MX 校验，生产环境可开启。
* **实际的连接和断开逻辑**: **已完成**。`SimpleSmtpClient::connect` 和 `disconnect` 已实现。
* **测试配置**: **已完成**。测试用例可通过 skip_mx 或测试域名跳过真实 DNS 校验，保证本地和 CI 测试通过。

## LLM 服务 (services/llm)

### `services/llm/src/client.rs`

* **LLM 客户端的完整性**: **已完成**。`DeepSeekClient` 已扩展，包括更详细的错误分类。

## 记忆服务 (services/memory)

### `services/memory/src/memory_data.rs`

* **`update_memory_corpus` 的简化逻辑**: **已完成**。`update_memory_corpus` 函数中的更新逻辑已扩展。
* **`UserStatistics` 中的 `account_created` 字段**: **已完成**。`get_user_statistics` 函数中 `account_created` 字段已从实际用户档案中获取。
* **持久化存储**: **已完成**。`MemoryDataRepository` 已实现基于本地文件的持久化存储。

## 遥测服务 (services/telemetry)

### `services/telemetry/src/lib.rs`

* **文件输出支持**: **已完成**。`init_telemetry_with_config` 函数中 `log_file` 的支持已实现。

## 共享逻辑 (services/shared_logic)

### `services/shared_logic/src/config.rs`

* **`database.max_connections` 配置**: **已完成**。此配置与记忆服务的持久化存储相关，已在文件持久化方案中考虑。

## 整体改进

* **错误处理细化**: **已完成**。已检查并细化错误处理，包括将 `EmailError` 迁移到 `thiserror`。
* **日志记录增强**: **已完成**。已检查并增强关键操作的日志记录。
* **性能优化**: **已完成**。已对 `search_memories` 函数进行优化。
* **安全审计**: **已完成**。已对代码库进行安全审计，未发现明显漏洞。
* **文档完善**: **已完成**。已补充和完善代码注释。
* **测试覆盖率**: **已完成**。已评估并提高了单元测试和集成测试的覆盖率。