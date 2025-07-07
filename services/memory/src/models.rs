use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 主记忆体结构 - 每个用户的完整记忆数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCorpus {
    /// 用户ID (通常是邮箱地址)
    pub user_id: String,
    /// 记忆体数据格式版本
    pub version: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 核心档案信息
    pub core_profile: CoreProfile,
    /// 情节记忆 (具体交互记录)
    pub episodic_memory: EpisodicMemory,
    /// 语义记忆 (概念和知识)
    pub semantic_memory: SemanticMemory,
    /// 行动状态记忆 (待办和计划)
    pub action_state_memory: ActionStateMemory,
    /// 战略推断记忆 (假设和策略)
    pub strategic_inferential_memory: StrategicInferentialMemory,
}

/// 核心档案信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreProfile {
    /// 用户姓名或称呼
    pub name: Option<String>,
    /// 年龄
    pub age: Option<u32>,
    /// 性别
    pub gender: Option<String>,
    /// 居住城市
    pub city: Option<String>,
    /// 职业
    pub occupation: Option<String>,
    /// 重要的个人关系
    pub relationships: Vec<Relationship>,
    /// 基本个性特征
    pub personality_traits: Vec<String>,
    /// 当前生活状态摘要
    pub current_life_summary: Option<String>,
}

/// 人际关系信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// 关系类型 (家庭、朋友、同事等)
    pub relationship_type: String,
    /// 对方姓名
    pub name: String,
    /// 关系描述
    pub description: Option<String>,
    /// 重要程度 (1-5)
    pub importance_level: u8,
}

/// 情节记忆 - 具体的交互历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// 交互日志列表
    pub interaction_log: Vec<InteractionLog>,
}

/// 单次交互记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionLog {
    /// 日志唯一ID
    pub log_id: String,
    /// 用户ID（交互对象标识）
    pub user_id: String,
    /// 邮件ID (来自邮件头)
    pub email_id: Option<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 消息方向
    pub direction: MessageDirection,
    /// 交互内容摘要
    pub summary: String,
    /// 情感色调
    pub emotional_tone: Vec<String>,
    /// 关键话题
    pub key_topics: Vec<String>,
    /// 使用的LLM模型版本
    pub llm_model_version: String,
    /// 思考链快照
    pub reasoning_chain_snapshot: Option<String>,
    /// 本次交互成本 (USD)
    pub cost_usd: Option<f64>,
}

impl InteractionLog {
    /// 创建新的交互记录
    pub fn new(user_id: String, direction: MessageDirection, summary: String) -> Self {
        Self {
            log_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            email_id: None,
            timestamp: chrono::Utc::now(),
            direction,
            summary,
            emotional_tone: Vec::new(),
            key_topics: Vec::new(),
            llm_model_version: "demo".to_string(),
            reasoning_chain_snapshot: None,
            cost_usd: None,
        }
    }
}

/// 消息方向枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    /// 用户发来的消息
    Inbound,
    /// AI发出的回复
    Outbound,
}

/// 语义记忆 - 抽象的概念和知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// 用户的喜好和厌恶
    pub preferences_and_dislikes: PreferencesAndDislikes,
    /// 习惯和行为模式
    pub habits_and_patterns: Vec<HabitPattern>,
    /// 重要的生活事件
    pub significant_events: Vec<SignificantEvent>,
    /// 技能和专长
    pub skills_and_expertise: Vec<SkillExpertise>,
    /// 价值观和信念
    pub values_and_beliefs: Vec<String>,
}

/// 喜好和厌恶
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesAndDislikes {
    /// 喜欢的事物
    pub likes: Vec<String>,
    /// 不喜欢的事物
    pub dislikes: Vec<String>,
    /// 兴趣爱好
    pub hobbies: Vec<String>,
    /// 食物偏好
    pub food_preferences: Vec<String>,
    /// 娱乐偏好
    pub entertainment_preferences: Vec<String>,
}

/// 习惯和行为模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitPattern {
    /// 习惯描述
    pub description: String,
    /// 频率 (daily, weekly, monthly, etc.)
    pub frequency: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 首次观察到的时间
    pub first_observed: chrono::DateTime<chrono::Utc>,
    /// 最后确认时间
    pub last_confirmed: chrono::DateTime<chrono::Utc>,
}

/// 重要生活事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantEvent {
    /// 事件描述
    pub description: String,
    /// 事件日期 (可能是大概的)
    pub date: Option<chrono::NaiveDate>,
    /// 情感影响 (positive, negative, neutral)
    pub emotional_impact: String,
    /// 重要程度 (1-5)
    pub importance_level: u8,
    /// 相关话题标签
    pub related_topics: Vec<String>,
}

/// 技能和专长
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExpertise {
    /// 技能名称
    pub skill_name: String,
    /// 熟练程度 (beginner, intermediate, advanced, expert)
    pub proficiency_level: String,
    /// 相关经验描述
    pub experience_description: Option<String>,
}

/// 行动状态记忆 - 待办事项和计划
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionStateMemory {
    /// 当前的待办事项
    pub current_tasks: Vec<Task>,
    /// 未来计划
    pub future_plans: Vec<Plan>,
    /// 需要跟进的事项
    pub follow_ups: Vec<FollowUp>,
}

/// 任务/待办事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub task_id: String,
    /// 任务描述
    pub description: String,
    /// 优先级 (low, medium, high, urgent)
    pub priority: String,
    /// 状态 (pending, in_progress, completed, cancelled)
    pub status: String,
    /// 截止日期
    pub due_date: Option<chrono::NaiveDate>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 未来计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// 计划描述
    pub description: String,
    /// 计划时间范围
    pub timeframe: String,
    /// 相关目标
    pub related_goals: Vec<String>,
    /// 置信度 (用户提及此计划的可能性)
    pub confidence: f64,
}

/// 跟进事项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUp {
    /// 跟进内容
    pub content: String,
    /// 建议跟进时间
    pub suggested_time: chrono::DateTime<chrono::Utc>,
    /// 重要程度
    pub importance: u8,
    /// 是否已处理
    pub resolved: bool,
}

/// 战略推断记忆 - AI的假设和策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInferentialMemory {
    /// 用户模型假设
    pub user_model_hypotheses: Vec<UserModelHypothesis>,
    /// 关系目标
    pub relational_goals: RelationalGoals,
    /// 沟通策略
    pub communication_strategy: CommunicationStrategy,
    /// 自我反思日志
    pub self_reflection_log: Vec<SelfReflectionEntry>,
}

/// 用户模型假设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModelHypothesis {
    /// 假设ID
    pub hypothesis_id: String,
    /// 假设内容
    pub hypothesis: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 状态 (active, refuted, confirmed)
    pub status: String,
    /// 支持证据 (引用交互日志ID)
    pub evidence: Vec<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 关系目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationalGoals {
    /// 短期目标 (1-4周)
    pub short_term: Vec<String>,
    /// 中期目标 (1-6个月)
    pub medium_term: Vec<String>,
    /// 长期目标 (6个月以上)
    pub long_term: Vec<String>,
}

/// 沟通策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStrategy {
    /// 当前使用的语气风格
    pub current_tone_style: String,
    /// 适合的话题
    pub suitable_topics: Vec<String>,
    /// 应该避免的话题
    pub topics_to_avoid: Vec<String>,
    /// 用户的沟通偏好
    pub user_communication_preferences: HashMap<String, String>,
}

/// 自我反思条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReflectionEntry {
    /// 反思内容
    pub content: String,
    /// 反思类型 (strategy_adjustment, user_insight, communication_improvement, etc.)
    pub reflection_type: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 相关的交互日志ID
    pub related_interaction: Option<String>,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self {
            interaction_log: Vec::new(),
        }
    }
}

impl Default for MemoryCorpus {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            user_id: String::new(),
            version: "2.1".to_string(),
            created_at: now,
            updated_at: now,
            core_profile: CoreProfile::default(),
            episodic_memory: EpisodicMemory {
                interaction_log: Vec::new(),
            },
            semantic_memory: SemanticMemory::default(),
            action_state_memory: ActionStateMemory::default(),
            strategic_inferential_memory: StrategicInferentialMemory::default(),
        }
    }
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self {
            preferences_and_dislikes: PreferencesAndDislikes {
                likes: Vec::new(),
                dislikes: Vec::new(),
                hobbies: Vec::new(),
                food_preferences: Vec::new(),
                entertainment_preferences: Vec::new(),
            },
            habits_and_patterns: Vec::new(),
            significant_events: Vec::new(),
            skills_and_expertise: Vec::new(),
            values_and_beliefs: Vec::new(),
        }
    }
}

impl Default for StrategicInferentialMemory {
    fn default() -> Self {
        Self {
            user_model_hypotheses: Vec::new(),
            relational_goals: RelationalGoals {
                short_term: Vec::new(),
                medium_term: Vec::new(),
                long_term: Vec::new(),
            },
            communication_strategy: CommunicationStrategy {
                current_tone_style: "friendly_and_supportive".to_string(),
                suitable_topics: Vec::new(),
                topics_to_avoid: Vec::new(),
                user_communication_preferences: HashMap::new(),
            },
            self_reflection_log: Vec::new(),
        }
    }
}

impl MemoryCorpus {
    /// 创建新的用户记忆体
    pub fn new(user_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            user_id,
            version: "1.0".to_string(),
            created_at: now,
            updated_at: now,
            core_profile: CoreProfile::default(),
            episodic_memory: EpisodicMemory::default(),
            semantic_memory: SemanticMemory::default(),
            action_state_memory: ActionStateMemory::default(),
            strategic_inferential_memory: StrategicInferentialMemory::default(),
        }
    }
}


/// 记忆查询结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// 用户ID
    pub user_id: Option<String>,
    /// 查询文本
    pub query_text: String,
    /// 过滤条件
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// 记忆片段 - 用于搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// 片段ID
    pub fragment_id: String,
    /// 用户ID
    pub user_id: String,
    /// 内容
    pub content: String,
    /// 来源 (e.g., "episodic", "semantic")
    pub source: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 相关性得分
    pub relevance_score: Option<f64>,
}

/// 用户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    /// 用户ID
    pub user_id: String,
    /// 账户创建时间
    pub account_created: chrono::DateTime<chrono::Utc>,
    /// 总交互次数
    pub total_interactions: u64,
    /// 首次交互时间
    pub first_interaction: chrono::DateTime<chrono::Utc>,
    /// 末次交互时间
    pub last_interaction: chrono::DateTime<chrono::Utc>,
    /// 总记忆条目数
    pub total_memories: u64,
    /// 记忆类型分布
    pub memory_type_distribution: HashMap<String, u64>,
}