//! # LLM 服务类型定义
//!
//! 定义了 LLM 服务的核心数据结构，遵循类型安全和清晰命名的原则。
//! 所有结构都支持序列化，便于日志记录和调试。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// LLM 请求 ID，用于追踪和关联
pub type RequestId = Uuid;

/// LLM 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// 请求唯一标识符
    pub id: RequestId,
    /// 系统提示词
    pub system_prompt: String,
    /// 用户输入消息
    pub user_message: String,
    /// 请求参数
    pub parameters: LlmParameters,
    /// 请求创建时间
    pub created_at: DateTime<Utc>,
}

/// LLM 请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmParameters {
    /// 使用的模型名称
    pub model: String,
    /// 生成温度 (0.0-2.0)
    pub temperature: f32,
    /// 最大生成令牌数
    pub max_tokens: u32,
    /// Top-p 采样参数
    pub top_p: f32,
    /// 是否流式返回
    pub stream: bool,
}

/// LLM 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// 对应的请求 ID
    pub request_id: RequestId,
    /// 生成的文本内容
    pub content: String,
    /// 使用的令牌统计
    pub token_usage: TokenUsage,
    /// 响应元数据
    pub metadata: ResponseMetadata,
    /// 响应创建时间
    pub created_at: DateTime<Utc>,
}

/// 令牌使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 提示词令牌数
    pub prompt_tokens: u32,
    /// 完成文本令牌数
    pub completion_tokens: u32,
    /// 总令牌数
    pub total_tokens: u32,
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// 实际使用的模型
    pub model: String,
    /// 处理延迟（毫秒）
    pub latency_ms: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 附加字段
    pub extra: HashMap<String, serde_json::Value>,
}

/// 邮件分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAnalysisRequest {
    /// 邮件唯一标识
    pub email_id: String,
    /// 邮件发送者
    pub sender: String,
    /// 邮件主题
    pub subject: String,
    /// 邮件正文
    pub body: String,
    /// 邮件接收时间
    pub received_at: DateTime<Utc>,
}

/// 邮件分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAnalysis {
    /// 情感分析结果
    pub sentiment: SentimentAnalysis,
    /// 意图识别结果
    pub intent: IntentAnalysis,
    /// 关键信息提取
    pub key_information: Vec<KeyInformation>,
    /// 紧急程度评估
    pub urgency_level: UrgencyLevel,
    /// 建议的回复策略
    pub suggested_strategy: ReplyStrategy,
}

/// 情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    /// 整体情感倾向
    pub overall_sentiment: Sentiment,
    /// 情感强度 (0.0-1.0)
    pub intensity: f32,
    /// 具体情感标签
    pub emotion_tags: Vec<String>,
}

/// 情感类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// 意图分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    /// 主要意图
    pub primary_intent: Intent,
    /// 意图置信度 (0.0-1.0)
    pub confidence: f32,
    /// 次要意图（如果存在）
    pub secondary_intents: Vec<Intent>,
}

/// 意图类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// 询问信息
    InformationRequest,
    /// 请求帮助
    HelpRequest,
    /// 预约安排
    SchedulingRequest,
    /// 投诉或问题
    ComplaintOrIssue,
    /// 社交寒暄
    SocialGreeting,
    /// 商务洽谈
    BusinessProposal,
    /// 其他
    Other(String),
}

/// 关键信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInformation {
    /// 信息类型
    pub info_type: String,
    /// 提取的值
    pub value: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
}

/// 紧急程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 回复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyStrategy {
    /// 建议的语调
    pub tone: CommunicationTone,
    /// 建议的回复长度
    pub suggested_length: ReplyLength,
    /// 需要包含的关键点
    pub key_points: Vec<String>,
    /// 建议的回复时间框架
    pub time_frame: ResponseTimeFrame,
}

/// 沟通语调
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationTone {
    Formal,
    Casual,
    Empathetic,
    Professional,
    Friendly,
}

/// 回复长度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplyLength {
    Brief,
    Medium,
    Detailed,
}

/// 响应时间框架
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseTimeFrame {
    Immediate,
    WithinHours,
    WithinDay,
    NoRush,
}

impl Default for LlmParameters {
    fn default() -> Self {
        Self {
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            stream: false,
        }
    }
}

impl LlmRequest {
    /// 创建新的 LLM 请求
    pub fn new(system_prompt: String, user_message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            system_prompt,
            user_message,
            parameters: LlmParameters::default(),
            created_at: Utc::now(),
        }
    }

    /// 设置请求参数
    pub fn with_parameters(mut self, parameters: LlmParameters) -> Self {
        self.parameters = parameters;
        self
    }
}
