//! # LLM 客户端实现
//!
//! 提供与 LLM API 交互的核心功能，遵循"健壮性是底线"的原则。
//! 实现重试逻辑、超时控制和错误处理。

use crate::error::{LlmError, LlmResult};
use crate::types::*;
use chrono::Utc;
use reqwest::{Client, header::HeaderMap};
use serde_json::{json, Value};
use shared_logic::config::{get_config, LlmConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// LLM 客户端接口 trait
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// 生成响应
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
    
    /// 分析邮件内容
    async fn analyze_email(&self, request: &EmailAnalysisRequest) -> LlmResult<EmailAnalysis>;
    
    /// 生成邮件回复
    async fn generate_reply(&self, analysis: &EmailAnalysis, context: &str) -> LlmResult<String>;
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
            format!("Bearer {}", config.api_key)
                .parse()
                .map_err(|_| LlmError::ConfigurationError {
                    field: "api_key".to_string(),
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
    
    /// 执行带重试的 API 请求
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> LlmResult<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = LlmResult<T>> + Send,
        T: Send,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(
                            attempt = attempt,
                            "API request succeeded after retry"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);
                    
                    if attempt < self.config.max_retries && last_error.as_ref().unwrap().is_retryable() {
                        let delay = Duration::from_millis(1000 * (attempt + 1) as u64);
                        warn!(
                            attempt = attempt,
                            delay_ms = delay.as_millis(),
                            error = %last_error.as_ref().unwrap(),
                            "API request failed, retrying"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }
        
        error!(
            max_retries = self.config.max_retries,
            error = %last_error.as_ref().unwrap(),
            "API request failed after all retries"
        );
        
        Err(last_error.unwrap_or_else(|| LlmError::MaxRetriesExceeded {
            max_retries: self.config.max_retries,
        }))
    }
    
    /// 调用 DeepSeek Chat API
    async fn call_chat_api(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        let start_time = Instant::now();
        
        // 构建 API 请求体
        let request_body = json!({
            "model": request.parameters.model,
            "messages": [
                {
                    "role": "system",
                    "content": request.system_prompt
                },
                {
                    "role": "user", 
                    "content": request.user_message
                }
            ],
            "temperature": request.parameters.temperature,
            "max_tokens": request.parameters.max_tokens,
            "top_p": request.parameters.top_p,
            "stream": request.parameters.stream
        });
        
        debug!(
            request_id = %request.id,
            model = %request.parameters.model,
            "Sending request to DeepSeek API"
        );
        
        // 发送请求
        let url = format!("{}/chat/completions", self.config.base_url);
        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError { source: e })?;
            
        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
                
            return Err(LlmError::ApiRequestFailed {
                message: format!("HTTP {}: {}", status, error_text),
            });
        }
        
        // 解析响应
        let response_text = response
            .text()
            .await
            .map_err(|e| LlmError::NetworkError { source: e })?;
            
        let response_json: Value = serde_json::from_str(&response_text)
            .map_err(|e| LlmError::InvalidApiResponse {
                details: format!("JSON parse error: {}", e),
            })?;
            
        // 验证响应结构
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| LlmError::InvalidApiResponse {
                details: "Missing content in API response".to_string(),
            })?;
            
        // 提取令牌使用信息
        let usage = response_json.get("usage");
        let token_usage = TokenUsage {
            prompt_tokens: usage
                .and_then(|u| u.get("prompt_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32,
            completion_tokens: usage
                .and_then(|u| u.get("completion_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32,
            total_tokens: usage
                .and_then(|u| u.get("total_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32,
        };
        
        let latency = start_time.elapsed();
        
        debug!(
            request_id = %request.id,
            latency_ms = latency.as_millis(),
            prompt_tokens = token_usage.prompt_tokens,
            completion_tokens = token_usage.completion_tokens,
            "DeepSeek API request completed"
        );
        
        Ok(LlmResponse {
            request_id: request.id,
            content: content.to_string(),
            token_usage,
            metadata: ResponseMetadata {
                model: request.parameters.model.clone(),
                latency_ms: latency.as_millis() as u64,
                retry_count: 0, // 在重试逻辑中更新
                extra: std::collections::HashMap::new(),
            },
            created_at: Utc::now(),
        })
    }
}

#[async_trait::async_trait]
impl LlmClient for DeepSeekClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        self.execute_with_retry(|| {
            self.call_chat_api(request)
        })
        .await
    }
    
    async fn analyze_email(&self, email_request: &EmailAnalysisRequest) -> LlmResult<EmailAnalysis> {
        let system_prompt = r#"
你是一个专业的邮件分析助手。请仔细分析以下邮件内容，并提供结构化的分析结果。

分析维度包括：
1. 情感分析：判断邮件的整体情感倾向和强度
2. 意图识别：识别发送者的主要意图
3. 关键信息提取：提取重要的人名、时间、地点、事件等信息
4. 紧急程度评估：评估邮件的紧急程度
5. 回复策略建议：建议合适的回复语调和策略

请以 JSON 格式返回分析结果。
"#;
        
        let user_message = format!(
            "邮件信息：\n发送者：{}\n主题：{}\n正文：{}\n接收时间：{}",
            email_request.sender,
            email_request.subject,
            email_request.body,
            email_request.received_at.format("%Y-%m-%d %H:%M:%S")
        );
        
        let llm_request = LlmRequest::new(system_prompt.to_string(), user_message);
        let response = self.generate_response(&llm_request).await?;
        
        // 解析 JSON 响应为 EmailAnalysis
        // 这里简化处理，实际应该有更robust的JSON解析和验证
        let analysis: EmailAnalysis = serde_json::from_str(&response.content)
            .map_err(|e| LlmError::InvalidApiResponse {
                details: format!("Failed to parse email analysis: {}", e),
            })?;
            
        info!(
            email_id = %email_request.email_id,
            urgency = ?analysis.urgency_level,
            primary_intent = ?analysis.intent.primary_intent,
            "Email analysis completed"
        );
        
        Ok(analysis)
    }
    
    async fn generate_reply(&self, analysis: &EmailAnalysis, context: &str) -> LlmResult<String> {
        let system_prompt = format!(
            r#"
你是一个专业的邮件回复助手。基于以下邮件分析结果和上下文信息，生成一个合适的回复。

邮件分析结果：
- 情感倾向：{:?}
- 主要意图：{:?}
- 紧急程度：{:?}
- 建议语调：{:?}

请生成一个：
1. 符合建议语调的回复
2. 适当回应原邮件的意图
3. 体现专业性和同理心
4. 长度适中，信息完整

只返回邮件正文内容，不要包含主题行或签名。
"#,
            analysis.sentiment.overall_sentiment,
            analysis.intent.primary_intent,
            analysis.urgency_level,
            analysis.suggested_strategy.tone
        );
        
        let llm_request = LlmRequest::new(system_prompt, context.to_string());
        let response = self.generate_response(&llm_request).await?;
        
        info!(
            urgency = ?analysis.urgency_level,
            tone = ?analysis.suggested_strategy.tone,
            "Email reply generated"
        );
        
        Ok(response.content)
    }
}
