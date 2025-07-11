use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::config;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    
    #[error("Invalid API response: {0}")]
    InvalidApiResponse(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),
}

pub type LlmResult<T> = Result<T, LlmError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub id: Uuid,
    pub prompt_name: String,
    pub context: HashMap<String, serde_json::Value>,
}

impl LlmRequest {
    pub fn new(prompt_name: String, context: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt_name,
            context,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub request_id: Uuid,
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse>;
}

pub struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_retries: u32,
}

impl DeepSeekClient {
    pub fn new() -> LlmResult<Self> {
        let config = config::get();
        
        if config.llm.api_key.is_empty() {
            tracing::warn!("LLM API key not configured. Set SENTIO_LLM_API_KEY environment variable or add it to sentio.toml");
            return Err(LlmError::ConfigurationError(
                "LLM API key not configured".to_string()
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.llm.timeout))
            .build()
            .map_err(|e| LlmError::ConfigurationError(e.to_string()))?;

        Ok(Self {
            client,
            api_key: config.llm.api_key.clone(),
            base_url: config.llm.base_url.clone(),
            model: config.llm.model.clone(),
            max_retries: config.llm.max_retries,
        })
    }

    async fn call_api(&self, messages: Vec<ChatMessage>) -> LlmResult<DeepSeekResponse> {
        let request_body = DeepSeekRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
            max_tokens: 2000,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let api_response: DeepSeekResponse = response.json().await
                    .map_err(|e| LlmError::InvalidApiResponse(e.to_string()))?;
                Ok(api_response)
            }
            StatusCode::UNAUTHORIZED => {
                Err(LlmError::AuthenticationFailed("Invalid API key".to_string()))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(LlmError::RateLimited(60))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(LlmError::ApiRequestFailed(
                    format!("API returned status {}: {}", status, error_text)
                ))
            }
        }
    }

    fn render_prompt(&self, template: &str, context: &HashMap<String, serde_json::Value>) -> String {
        let mut result = template.to_string();
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        result
    }
}

#[async_trait]
impl LlmClient for DeepSeekClient {
    async fn generate_response(&self, request: &LlmRequest) -> LlmResult<LlmResponse> {
        let config = config::get();
        
        let (system_prompt, user_prompt_template) = match request.prompt_name.as_str() {
            "email_analysis" => config.get_email_analysis_prompt(),
            "email_reply" => config.get_email_reply_prompt(),
            _ => ("You are a helpful assistant.", "{{content}}"),
        };

        let user_prompt = self.render_prompt(user_prompt_template, &request.context);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        let mut retries = 0;
        loop {
            match self.call_api(messages.clone()).await {
                Ok(response) => {
                    return Ok(LlmResponse {
                        request_id: request.id,
                        content: response.choices[0].message.content.clone(),
                        model: response.model,
                        usage: TokenUsage {
                            prompt_tokens: response.usage.prompt_tokens,
                            completion_tokens: response.usage.completion_tokens,
                            total_tokens: response.usage.total_tokens,
                        },
                    });
                }
                Err(e) => {
                    if retries >= self.max_retries {
                        return Err(e);
                    }
                    
                    match &e {
                        LlmError::NetworkError(_) | 
                        LlmError::RateLimited(_) => {
                            retries += 1;
                            tokio::time::sleep(Duration::from_secs(2u64.pow(retries))).await;
                        }
                        _ => return Err(e),
                    }
                }
            }
        }
    }
}

#[derive(Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}