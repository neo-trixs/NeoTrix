//! Common types for NT-ACT

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest;

/// HTTP Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub query: Option<HashMap<String, String>>,
}

/// HTTP Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// Delivery outcome for side-effectful operations (dsh-im absorption).
///
/// Three-state semantics: `Unknown` means the peer may or may not have
/// processed the request (timeout after send, 5xx, response read failure).
/// Retrying an `Unknown` outcome risks duplicate execution of non-idempotent
/// requests, so callers must stop retrying and surface the ambiguity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryOutcome {
    /// Peer confirmed receipt (success response).
    Delivered,
    /// Request may have been delivered; result is unverifiable. Never safe to auto-retry.
    Unknown,
    /// Peer definitively did not process the request (connect failure, 4xx rejection). Safe to retry.
    Failed,
}

impl DeliveryOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryOutcome::Delivered => "delivered",
            DeliveryOutcome::Unknown => "unknown",
            DeliveryOutcome::Failed => "failed",
        }
    }
}

/// Transport cause for failure classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportCause {
    /// Deadline hit; request may or may not have been sent.
    pub timeout: bool,
    /// Connection/DNS/TLS failure; nothing was sent.
    pub connect: bool,
    /// Response-phase failure (body read/decode); peer already processed the request.
    pub response_phase: bool,
}

/// Classify a transport failure into a delivery outcome.
pub fn classify_failure(cause: TransportCause) -> DeliveryOutcome {
    if cause.response_phase || cause.timeout {
        DeliveryOutcome::Unknown
    } else {
        DeliveryOutcome::Failed
    }
}

/// Classify an HTTP status into a delivery outcome.
/// 5xx = peer may have processed side effects (`Unknown`); 4xx = definitive rejection.
pub fn classify_status(status: u16) -> DeliveryOutcome {
    if status >= 500 {
        DeliveryOutcome::Unknown
    } else if status >= 400 {
        DeliveryOutcome::Failed
    } else {
        DeliveryOutcome::Delivered
    }
}

/// Extract transport cause from a reqwest error.
pub fn transport_cause(err: &reqwest::Error) -> TransportCause {
    TransportCause {
        timeout: err.is_timeout(),
        connect: err.is_connect(),
        response_phase: err.is_body() || err.is_decode(),
    }
}

/// MCP Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP Resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP Prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// MCP Prompt Argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// MCP Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResult {
    pub content: serde_json::Value,
    pub is_error: bool,
}

/// ACP Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub from: String,
    pub to: String,
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// ACP Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Search Options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchOptions {
    pub max_results: Option<usize>,
    pub engine: Option<SearchEngine>,
    pub safe_search: Option<bool>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub time_range: Option<String>,
}

/// Search Result
#[derive(Debug, Clone, PartialSerialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: f64,
    pub engine: SearchEngine,
}

/// Tool Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
    pub tags: Vec<String>,
    pub version: String,
    pub author: String,
}

/// Tool Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Communication Intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationIntent {
    pub target: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
    pub priority: Priority,
    pub timeout_secs: Option<u64>,
}

/// Priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Communication Method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommunicationMethod {
    Http,
    Mcp,
    Acp,
    Search,
    Tool,
}

/// Communication Decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationDecision {
    pub method: CommunicationMethod,
    pub confidence: f64,
    pub reasoning: String,
    pub http_request: Option<HttpRequest>,
    pub mcp_tool: Option<String>,
    pub mcp_args: Option<serde_json::Value>,
    pub acp_message: Option<AcpMessage>,
    pub search_query: Option<String>,
    pub search_options: Option<SearchOptions>,
    pub tool_name: Option<String>,
    pub tool_args: Option<serde_json::Value>,
}

/// Communication Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationResult {
    Http(HttpResponse),
    Mcp(McpResult),
    Acp(AcpResponse),
    Search(Vec<SearchResult>),
    Tool(ToolResult),
}

/// Communication Capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationCapabilities {
    pub http: bool,
    pub mcp: bool,
    pub acp: bool,
    pub search: bool,
    pub tools: Vec<String>,
}

/// Communication Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    pub http: HttpClientConfig,
    pub mcp: McpConfig,
    pub acp: AcpConfig,
    pub search: SearchConfig,
    pub tools: ToolsConfig,
    pub decision: DecisionConfig,
    pub autonomous_mode: bool,
    pub max_concurrent: usize,
    pub default_timeout_secs: u64,
    pub circuit_breaker_threshold: u32,
}

/// HTTP Client Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    pub base_url: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub headers: std::collections::HashMap<String, String>,
    pub proxy: Option<String>,
    pub tls_verify: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            headers: std::collections::HashMap::new(),
            proxy: None,
            tls_verify: true,
        }
    }
}

/// MCP Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub server_url: Option<String>,
    pub transport: McpTransport,
    pub auth_token: Option<String>,
    pub timeout_secs: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server_url: None,
            transport: McpTransport::Stdio,
            auth_token: None,
            timeout_secs: 60,
        }
    }
}

/// MCP Transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio,
    Http,
    WebSocket,
    Sse,
}

/// ACP Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpConfig {
    pub server_url: Option<String>,
    pub auth_token: Option<String>,
    pub timeout_secs: u64,
}

impl Default for AcpConfig {
    fn default() -> Self {
        Self {
            server_url: None,
            auth_token: None,
            timeout_secs: 60,
        }
    }
}

/// Search Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_engine: SearchEngine,
    pub max_results: usize,
    pub timeout_secs: u64,
    pub safe_search: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_engine: SearchEngine::Unified,
            max_results: 10,
            timeout_secs: 30,
            safe_search: true,
        }
    }
}

/// Search Engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchEngine {
    Unified,
    Google,
    Bing,
    DuckDuckGo,
    BingNews,
    Arxiv,
    Wikipedia,
    GitHub,
    ArXiv,
    SemanticScholar,
}

/// Tools Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub builtin_tools: bool,
    pub custom_tools_path: Option<String>,
    pub max_execution_time_secs: u64,
    pub sandbox: bool,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            builtin_tools: true,
            custom_tools_path: None,
            max_execution_time_secs: 300,
            sandbox: true,
        }
    }
}

/// Decision Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConfig {
    pub min_confidence: f64,
    pub prefer_mcp: bool,
    pub prefer_search: bool,
    pub enable_learning: bool,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            prefer_mcp: true,
            prefer_search: true,
            enable_learning: true,
        }
    }
}

/// Communication Config
impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            http: HttpClientConfig::default(),
            mcp: McpConfig::default(),
            acp: AcpConfig::default(),
            search: SearchConfig::default(),
            tools: ToolsConfig::default(),
            decision: DecisionConfig::default(),
            autonomous_mode: true,
            max_concurrent: 10,
            default_timeout_secs: 30,
            circuit_breaker_threshold: 5,
        }
    }
}

/// Tool Executor
pub struct ToolExecutor;

impl ToolExecutor {
    pub fn new() -> Self { Self }
}
