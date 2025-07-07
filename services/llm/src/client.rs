//! # LLM 客户端实现
//!
//! 提供与 LLM API 交互的核心功能，遵循"健壮性是底线"的原则。
//! 实现重试逻辑、超时控制和错误处理。

use crate::error::{LlmError, LlmResult};
use crate::types::*;
use chrono::Utc;
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde_json::{json, Value};
use shared_logic::config::{get_config, LlmConfig};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// 简单的模板渲染函数
///
/// 替换字符串中 `{{key}}` 格式的占位符。
///
/// # 参数
///
/// * `template` - 包含占位符的字符串模板。
/// * `context` - 包含占位符对应值的 HashMap。键是占位符名称（不含 `{{}}`），值是 `serde_json::Value`。
///
/// # 返回
///
/// 渲染后的字符串，所有占位符都被替换为对应的值。
fn render_template(template: &str, context: &HashMap<String, Value>) -> String {
    let mut result = template.to_string();
    for (key, value) in context {
        let placeholder = format!("{{{}}}", key);
        // 将 JSON Value 转换为字符串，移除引号
        let value_str = value.to_string().trim_matches('"').to_string();
        result = result.replace(&placeholder, &value_str);
    }
    result
}

/// LLM 客户端接口 trait
///
/// 定义了与大型语言模型 (LLM) 交互的通用接口。
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync + AsAny {
    /// 生成响应。
    ///
    /// 根据给定的 `LlmRequest` 向 LLM 发送请求并获取响应。
    ///
    /// # 参数
    ///
    /// * `request` - 包含 LLM 请求详细信息的 `LlmRequest` 实例。
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `LlmResponse`；否则返回 `LlmError`。
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
}

// Helper trait to allow downcasting of trait objects
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static + LlmClient + Send + Sync> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// `DeepSeekClient` 是 `LlmClient` trait 的一个实现，用于与 DeepSeek LLM API 交互。
/// 它处理 API 请求的构建、发送、响应解析以及错误处理和重试逻辑。
#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    /// 用于发送 HTTP 请求的 `reqwest::Client` 实例。
    http_client: Client,
    /// 存储 DeepSeek API 配置的 `Arc<LlmConfig>` 实例。
    config: Arc<LlmConfig>,
}

impl DeepSeekClient {
    /// 创建一个新的 `DeepSeekClient` 实例。
    ///
    /// 从全局配置中加载 LLM 配置，并构建一个配置了认证头和超时设置的 `reqwest::Client`。
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `DeepSeekClient` 实例；否则返回 `LlmError`。
    pub fn new() -> LlmResult<Self> {
        let global_config = get_config();
        let config = Arc::new(global_config.llm.clone());

        // 验证配置
        Self::validate_config(&config)?;

        // 构建 HTTP 客户端
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", config.api_key).parse().map_err(|_| {
                LlmError::ConfigurationError {
                    field: "api_key".to_string(),
                }
            })?,
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let http_client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| LlmError::ConfigurationError {
                field: format!("http_client: {}", e),
            })?;

        info!(
            provider = %config.provider,
            model = %config.model,
            timeout = %config.timeout,
            "DeepSeek LLM client initialized"
        );

        Ok(Self {
            http_client,
            config,
        })
    }

    /// 验证 `LlmConfig` 实例的有效性。
    ///
    /// 检查 API 密钥、基础 URL 和模型名称是否为空。
    ///
    /// # 参数
    ///
    /// * `config` - 要验证的 `LlmConfig` 引用。
    ///
    /// # 返回
    ///
    /// 如果配置有效，返回 `Ok(())`；否则返回 `LlmError::ConfigurationError`。
    fn validate_config(config: &LlmConfig) -> LlmResult<()> {
        if config.api_key.is_empty() {
            return Err(LlmError::ConfigurationError {
                field: "api_key is empty".to_string(),
            });
        }

        if config.base_url.is_empty() {
            return Err(LlmError::ConfigurationError {
                field: "base_url is empty".to_string(),
            });
        }

        if config.model.is_empty() {
            return Err(LlmError::ConfigurationError {
                field: "model is empty".to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl LlmClient for DeepSeekClient {
    /// 使用配置好的提示词和上下文生成响应。
    ///
    /// 该方法构建 LLM API 请求体，发送请求，并处理响应，包括重试逻辑和错误分类。
    ///
    /// # 参数
    ///
    /// * `request` - 包含 LLM 请求详细信息的 `LlmRequest` 实例。
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `LlmResponse`；否则返回 `LlmError`。
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        let start_time = Instant::now();
        let config = get_config();

        // 1. 获取并渲染提示词
        let prompt_template = config.get_prompt(&request.prompt_name)?;
        let system_prompt = render_template(&prompt_template.system, &request.context);
        let user_message = render_template(&prompt_template.user, &request.context);

        // 2. 构建 API 请求体
        let body = json!({
            "model": &request.parameters.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_message
                }
            ],
            "temperature": request.parameters.temperature,
            "max_tokens": request.parameters.max_tokens,
            "top_p": request.parameters.top_p,
            "stream": request.parameters.stream,
        });

        debug!(
            request_id = %request.id,
            prompt_name = %request.prompt_name,
            body = %serde_json::to_string(&body).unwrap_or_default(),
            "Sending request to DeepSeek API"
        );

        // 3. 执行请求（包含重试逻辑）
        let mut attempt: u32 = 0;
        let max_retries = self.config.max_retries;

        // 构建完整的 API 端点 URL
        let api_url = if self.config.base_url.ends_with("/chat/completions") {
            self.config.base_url.clone()
        } else if self.config.base_url.ends_with('/') {
            format!("{}chat/completions", self.config.base_url)
        } else {
            format!("{}/chat/completions", self.config.base_url)
        };

        let response_value = loop {
            attempt += 1;
            let request_builder = self.http_client.post(&api_url).json(&body);

            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    let text = response.text().await.unwrap_or_default();

                    if status.is_success() {
                        break serde_json::from_str(&text).map_err(LlmError::from);
                    } else {
                        let error_message = serde_json::from_str::<Value>(&text)
                            .ok()
                            .and_then(|v| v["error"]["message"].as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| text.clone());

                        let err = match status {
                            StatusCode::UNAUTHORIZED => LlmError::AuthenticationFailed {
                                reason: error_message,
                            },
                            StatusCode::TOO_MANY_REQUESTS => LlmError::RateLimited {
                                retry_after_seconds: retry_after.unwrap_or(0),
                            },
                            StatusCode::BAD_REQUEST => {
                                if error_message.contains("token limit") {
                                    LlmError::TokenLimitExceeded { limit: 0 } // TODO: Parse actual limit if available
                                } else if error_message.contains("content filtered") {
                                    LlmError::ContentFiltered { reason: error_message }
                                } else {
                                    LlmError::ApiRequestFailed { message: error_message }
                                }
                            }
                            _ => LlmError::ApiRequestFailed { message: error_message },
                        };

                        if err.is_retryable() && attempt <= max_retries {
                            warn!(
                                request_id = %request.id,
                                status = %status,
                                attempt = attempt,
                                error = %err,
                                "Retrying due to API error"
                            );
                            tokio::time::sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
                            continue;
                        } else {
                            error!(
                                request_id = %request.id,
                                status = %status,
                                attempt = attempt,
                                error = %err,
                                "Request failed after max retries or non-retryable error"
                            );
                            return Err(err);
                        }
                    }
                }
                Err(e) => {
                    let err = LlmError::from(e);
                    if err.is_retryable() && attempt <= max_retries {
                        warn!(
                            request_id = %request.id,
                            error = %err,
                            attempt = attempt,
                            "Retrying due to network error"
                        );
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
                        continue;
                    } else {
                        error!(
                            request_id = %request.id,
                            error = %err,
                            attempt = attempt,
                            "Request failed after max retries or non-retryable error"
                        );
                        return Err(err);
                    }
                }
            }
        };

        // 4. 解析响应
        let response: Value = response_value?;
        debug!(request_id = %request.id, response = %response, "Received response from API");

        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::InvalidApiResponse {
                details: format!("Missing 'content' in response: {}", response),
            })?
            .to_string();

        let prompt_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let completion_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(LlmResponse {
            request_id: request.id,
            content,
            token_usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            metadata: ResponseMetadata {
                model: self.config.model.clone(),
                latency_ms: start_time.elapsed().as_millis() as u64,
                retry_count: attempt - 1,
                extra: HashMap::new(),
            },
            created_at: Utc::now(),
        })
    }
}
