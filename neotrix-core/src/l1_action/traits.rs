//! L1 能力网统一架构 — Trait 契约 + 类型定义
//!
//! 设计原则:
//! 1. 每个能力类别 = Trait + Registry + Router + Bridge
//! 2. 所有模块必须实现 L1Capability 基座 Trait
//! 3. 类别专用 Trait 继承 L1Capability
//! 4. 零重复: 共享类型只在这里定义

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// 能力类别枚举
// ════════════════════════════════════════════════════════════════

/// 能力类别 — 8 大类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    Communication,  // CAT-1: 通信
    Content,        // CAT-2: 内容
    Data,           // CAT-3: 数据
    Search,         // CAT-4: 搜索
    Execution,      // CAT-5: 执行
    Security,       // CAT-6: 安全
    Coordination,   // CAT-7: 协调
    Cognition,      // CAT-8: 认知
}

/// 成熟度级别 (Constellation C0-C6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConstellationLevel {
    C0Compiled,      // 编译通过
    C1UnitTest,      // 单元测试
    C2Integration,   // 集成测试
    C3Benchmark,     // 基准测试
    C4Production,    // 主流管道
    C5SelfHealing,   // 自愈
    C6Evolution,     // 自进化
}

// ════════════════════════════════════════════════════════════════
// 核心 Trait — 所有能力必须实现
// ════════════════════════════════════════════════════════════════

/// L1 能力基座 Trait — 每个能力模块必须实现
pub trait L1Capability: Send + Sync + 'static {
    /// 能力唯一标识 (如 "messaging.whatsapp", "media.linkedin")
    fn capability_id(&self) -> &str;

    /// 能力类别
    fn category(&self) -> CapabilityCategory;

    /// 当前成熟度
    fn constellation(&self) -> ConstellationLevel;

    /// 健康检查
    fn health_check(&self) -> CapabilityHealth;

    /// 能力描述
    fn description(&self) -> &str;

    /// 依赖的其他能力 (默认无)
    fn dependencies(&self) -> Vec<&str> { Vec::new() }

    /// 初始化
    fn initialize(&mut self) -> Result<(), CapabilityError> { Ok(()) }

    /// 获取统计信息
    fn stats(&self) -> CapabilityStats { CapabilityStats::default() }
}

// ════════════════════════════════════════════════════════════════
// 类别专用 Trait
// ════════════════════════════════════════════════════════════════

// ── CAT-1: 通信 ──
pub trait MessagingProvider: L1Capability {
    fn send(&self, msg: &Message) -> Result<String, CapabilityError>;
    fn receive(&self, since: Option<u64>) -> Result<Vec<Message>, CapabilityError>;
    fn get_status(&self, id: &str) -> Result<MessageStatus, CapabilityError>;
}

// ── CAT-2: 内容 ──
pub trait ContentProvider: L1Capability {
    fn publish(&self, post: &Post) -> Result<String, CapabilityError>;
    fn get_engagement(&self, post_id: &str) -> Result<EngagementMetrics, CapabilityError>;
    fn best_posting_times(&self) -> Vec<(u32, u32)>;
}

// ── CAT-3: 数据 ──
pub trait DataStore: L1Capability {
    fn store(&self, namespace: &str, key: &str, value: &[u8]) -> Result<String, CapabilityError>;
    fn load(&self, id: &str) -> Result<Option<Vec<u8>>, CapabilityError>;
    fn query(&self, query: &str, limit: usize) -> Result<Vec<QueryResult>, CapabilityError>;
}

// ── CAT-4: 搜索 ──
pub trait SearchEngine: L1Capability {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, CapabilityError>;
    fn index(&self, doc: &Document) -> Result<(), CapabilityError>;
}

// ── CAT-5: 执行 ──
pub trait ToolExecutor: L1Capability {
    fn execute(&self, tool: &str, input: &ToolInput) -> Result<ToolOutput, CapabilityError>;
    fn list_tools(&self) -> Vec<ToolDef>;
}

// ── CAT-6: 安全 ──
pub trait SecurityGuard: L1Capability {
    fn check(&self, action: &ActionRequest) -> SecurityVerdict;
    fn audit(&self, entry: &AuditEntry) -> Result<(), CapabilityError>;
}

// ── CAT-7: 协调 ──
pub trait Orchestrator: L1Capability {
    fn plan(&self, goal: &str) -> Result<Plan, CapabilityError>;
    fn execute(&self, plan: &Plan) -> Result<PlanResult, CapabilityError>;
}

// ── CAT-8: 认知 ──
pub trait LlmRouter: L1Capability {
    fn route(&self, request: &LlmRequest) -> Result<LlmRoute, CapabilityError>;
    fn providers(&self) -> Vec<String>;
}

// ════════════════════════════════════════════════════════════════
// 共享类型
// ════════════════════════════════════════════════════════════════

/// 能力健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityHealth {
    pub healthy: bool,
    pub latency_ms: Option<f64>,
    pub error_rate: f64,
    pub last_check: u64,
    pub message: Option<String>,
}

impl Default for CapabilityHealth {
    fn default() -> Self {
        Self {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: 0,
            message: None,
        }
    }
}

/// 能力统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityStats {
    pub total_calls: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_latency_ms: f64,
}

/// 能力错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityError {
    NotAvailable(String),
    ExecutionFailed(String),
    Timeout(String),
    PermissionDenied(String),
    InvalidInput(String),
    Internal(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAvailable(msg) => write!(f, "Not available: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CapabilityError {}

// ── 通信类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: String,
    pub from: String,
    pub to: String,
    pub body: String,
    pub status: MessageStatus,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Draft, Queued, Sending, Sent, Delivered, Read, Failed, Bounced,
}

// ── 内容类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub platform: String,
    pub body: String,
    pub hashtags: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub impressions: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub engagement_rate: f64,
}

// ── 数据类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: String,
    pub score: f64,
    pub data: Vec<u8>,
}

// ── 搜索类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSearchResult {
    pub id: String,
    pub score: f64,
    pub title: String,
    pub snippet: String,
}

pub type SearchResult = ActionSearchResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

// ── 执行类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// ── 安全类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityVerdict {
    Allow,
    Deny(String),
    RequireApproval(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub action: String,
    pub actor: String,
    pub result: String,
}

// ── 协调类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
    pub estimated_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub action: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResult {
    pub success: bool,
    pub steps_completed: usize,
    pub output: Option<serde_json::Value>,
}

// ── 认知类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRoute {
    pub provider: String,
    pub model: String,
    pub estimated_cost: f64,
}
