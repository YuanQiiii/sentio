//! # LLM 客户端实现
//!
//! 提供与 LLM API 交互的核心功能，遵循"健壮性是底线"的原则。
//! 实现重试逻辑、超时控制和错误处理。

use crate::error::{LlmError, LlmResult};
use crate::types::*;
use chrono::Utc;
use reqwest::{header::HeaderMap, Client};
use serde_json::{json, Value};
use shared_logic::config::{get_config, LlmConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// 简单的模板渲染函数
///
/// 替换字符串中 `{{key}}` 格式的占位符。
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
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// 生成响应
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
}

/// DeepSeek API 客户端实现
#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    /// HTTP 客户端
    http_client: Client,
    /// 配置信息
    config: Arc<LlmConfig>,
}

impl DeepSeekClient {
    /// 创建新的 DeepSeek 客户端
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

    /// 验证配置有效性
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
    /// 使用配置好的提示词和上下文生成响应
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
        let mut attempt = 0;
        let max_retries = self.config.max_retries;
        let response_value = loop {
            attempt += 1;
            let request_builder = self.http_client.post(&self.config.base_url).json(&body);

            match request_builder.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        break response.json::<Value>().await.map_err(LlmError::from);
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        let err = LlmError::ApiRequestFailed {
                            message: format!(
                                "API returned non-success status: {} - {}",
                                status, text
                            ),
                        };
                        if status.is_client_error() || attempt > max_retries {
                            error!(
                                request_id = %request.id,
                                status = %status,
                                attempt = attempt,
                                "Request failed with client error or max retries reached"
                            );
                            return Err(err);
                        }
                        warn!(
                            request_id = %request.id,
                            status = %status,
                            attempt = attempt,
                            "Retrying due to server error"
                        );
                    }
                }
                Err(e) => {
                    let err = LlmError::from(e);
                    if attempt > max_retries {
                        error!(
                            request_id = %request.id,
                            error = %err,
                            attempt = attempt,
                            "Request failed after max retries"
                        );
                        return Err(err);
                    }
                    warn!(
                        request_id = %request.id,
                        error = %err,
                        attempt = attempt,
                        "Retrying due to request error"
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(2u64.pow(attempt as u32 - 1))).await;
        };

        // 4. 解析响应
        let response = response_value?;
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
