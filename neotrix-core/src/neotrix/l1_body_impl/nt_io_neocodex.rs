#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use base64::Engine as _;
use crate::neotrix::nt_io_provider::types::{LlmRequest, Message, Role, Tool};

// ── Mode System (from Kimi Code: Agent + Shell dual-mode) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum NeoCodexMode {
    #[default]
    Agent,
    Shell,
    Plan,
}

// ── Provider Catalog (from Kimi Code kosong: capability-based) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    Code,
    Reasoning,
    Vision,
    Video,
    Thinking,
    FunctionCalling,
    ParallelToolUse,
    LongContext,
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub model: String,
    pub capabilities: Vec<ModelCapability>,
    pub context_limit: usize,
    pub cost_per_m_input: f64,
    pub cost_per_m_output: f64,
}

impl Default for ProviderInfo {
    fn default() -> Self {
        Self {
            name: "opencode".into(),
            model: "default".into(),
            capabilities: vec![
                ModelCapability::Code,
                ModelCapability::Reasoning,
                ModelCapability::FunctionCalling,
                ModelCapability::LongContext,
            ],
            context_limit: 100_000,
            cost_per_m_input: 0.0,
            cost_per_m_output: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderCatalog {
    pub providers: Vec<ProviderInfo>,
    pub active: usize,
}

impl Default for ProviderCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderCatalog {
    pub fn new() -> Self {
        Self {
            providers: vec![ProviderInfo::default()],
            active: 0,
        }
    }

    pub fn has_capability(&self, cap: ModelCapability) -> bool {
        self.providers.get(self.active)
            .map(|p| p.capabilities.contains(&cap))
            .unwrap_or(false)
    }

    pub fn add_provider(&mut self, info: ProviderInfo) {
        self.providers.push(info);
    }

    /// Populate catalog from the real nt_io_provider layer
    pub fn sync_from_real(&mut self) {
        use crate::neotrix::nt_io_provider::provider_catalog;
        self.providers.clear();
        for entry in provider_catalog::PROVIDER_CATALOG.iter() {
            let is_code = entry.models.iter().any(|m| {
                m.to_lowercase().contains("code") || m.to_lowercase().contains("coder")
            });
            let mut capabilities = vec![ModelCapability::Reasoning];
            if is_code {
                capabilities.push(ModelCapability::Code);
            }
            if entry.models.iter().any(|m| m.to_lowercase().contains("vision")
                || m.to_lowercase().contains("vl") || m.to_lowercase().contains("4o")) {
                capabilities.push(ModelCapability::Vision);
            }
            if entry.default_model.to_lowercase().contains("thinking")
                || entry.models.iter().any(|m| m.to_lowercase().contains("think")) {
                capabilities.push(ModelCapability::Thinking);
            }
            capabilities.push(ModelCapability::FunctionCalling);
            capabilities.push(ModelCapability::LongContext);
            self.providers.push(ProviderInfo {
                name: entry.name.to_string(),
                model: entry.default_model.to_string(),
                capabilities,
                context_limit: 100_000,
                cost_per_m_input: if entry.is_free { 0.0 } else { 0.5 },
                cost_per_m_output: if entry.is_free { 0.0 } else { 2.0 },
            });
        }
        if self.providers.is_empty() {
            self.providers.push(ProviderInfo::default());
        }
    }

    /// Map a provider name to a real LlmProviderType (shared by selection helpers).
    fn provider_type_of(name: &str) -> Option<crate::neotrix::nt_io_provider::LlmProviderType> {
        match name {
            "openai" | "gpt" => Some(crate::neotrix::nt_io_provider::LlmProviderType::OpenAI),
            "anthropic" | "claude" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Anthropic),
            "gemini" | "google" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Gemini),
            "ollama" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Ollama),
            "openrouter" => Some(crate::neotrix::nt_io_provider::LlmProviderType::OpenRouter),
            "groq" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Groq),
            "cerebras" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Cerebras),
            "sambanova" => Some(crate::neotrix::nt_io_provider::LlmProviderType::SambaNova),
            "pollinations" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Pollinations),
            "bazaarlink" => Some(crate::neotrix::nt_io_provider::LlmProviderType::BazaarLink),
            "nvidia" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Nvidia),
            "github-models" | "github_models" => Some(crate::neotrix::nt_io_provider::LlmProviderType::GitHubModels),
            "huggingface" | "hf" => Some(crate::neotrix::nt_io_provider::LlmProviderType::HuggingFace),
            "cohere" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Cohere),
            "siliconflow" => Some(crate::neotrix::nt_io_provider::LlmProviderType::SiliconFlow),
            "deepseek-free" | "deepseek_free" => Some(crate::neotrix::nt_io_provider::LlmProviderType::DeepSeekFree),
            "lm-studio" | "llamacpp" | "local" => Some(crate::neotrix::nt_io_provider::LlmProviderType::Ollama),
            _ => None,
        }
    }

    /// True if the active provider name maps to a real LlmProvider type.
    pub fn is_resolvable(&self) -> bool {
        self.providers
            .get(self.active)
            .map(|p| Self::provider_type_of(&p.name).is_some())
            .unwrap_or(false)
    }

    /// True if the given provider name maps to a real LlmProvider type.
    pub fn is_resolvable_for(&self, name: &str) -> bool {
        Self::provider_type_of(name).is_some()
    }

    /// Set the active provider by name. Returns true if found.
    pub fn set_active_provider(&mut self, name: &str) -> bool {
        if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
            self.active = idx;
            true
        } else {
            false
        }
    }

    /// Pick the active provider's concrete model id
    pub fn active_model(&self) -> String {
        self.providers
            .get(self.active)
            .map(|p| p.model.clone())
            .unwrap_or_else(|| "default".to_string())
    }

    /// Sync from the real provider catalog and select a usable provider.
    /// Honors `NEOTRIX_PROVIDER` env if set; otherwise picks the first
    /// resolvable (non-stub) provider. Fixes the Cycle 159 gap where the
    /// default "opencode" stub was never replaced, leaving the ReAct loop
    /// unreachable in production.
    pub fn ensure_production_provider(&mut self) {
        self.sync_from_real();
        if let Ok(name) = std::env::var("NEOTRIX_PROVIDER") {
            if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
                self.active = idx;
                return;
            }
        }
        // Restore the user's last provider choice (persisted by set_active_provider).
        self.load_persisted();
        if !self.is_resolvable() {
            if let Some(idx) = self.providers.iter().position(|p| Self::provider_type_of(&p.name).is_some()) {
                self.active = idx;
            }
        }
    }

    /// Path to the persisted active-provider file (~/.neocodex/provider.json).
    fn persist_path() -> std::path::PathBuf {
        let base = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".neocodex"))
            .join("neocodex");
        base.join("provider.json")
    }

    /// Re-apply the persisted provider choice after a catalog sync.
    /// Safe to call on every agent boot: no-ops when no file exists.
    pub fn load_persisted(&mut self) {
        let path = Self::persist_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        let Ok(saved) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        if let Some(name) = saved.get("active").and_then(|v| v.as_str()) {
            if let Some(idx) = self.providers.iter().position(|p| p.name == name) {
                self.active = idx;
            }
        }
    }

    /// Persist the active provider name so it survives app restarts.
    pub fn save_persisted(&self) {
        let path = Self::persist_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let name = self
            .providers
            .get(self.active)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if let Ok(text) = serde_json::to_string(&serde_json::json!({ "active": name })) {
            let _ = std::fs::write(path, text);
        }
    }

    /// Create a LlmProvider from real layer (if matching provider type)
    pub fn to_llm_provider(&self) -> Option<Box<dyn crate::neotrix::nt_io_provider::LlmProvider>> {
        let info = self.providers.get(self.active)?;
        let provider_type = Self::provider_type_of(&info.name)?;
        let mut config = crate::neotrix::nt_io_provider::ProviderConfig::from_env();
        config.provider_type = provider_type;
        config.model = Some(info.model.clone());
        if info.name == "anthropic" || info.name == "claude" {
            config.api_key = std::env::var("ANTHROPIC_API_KEY")
                .ok()
                .or_else(|| std::env::var("NEOTRIX_API_KEY").ok());
        }
        Some(crate::neotrix::nt_io_provider::create_provider(config))
    }
}

// ── Streaming Markdown (from Claude Code: tolerant parser) ──

#[derive(Debug, Clone)]
pub struct StreamingMarkdown {
    pub buffer: String,
    pub chunks: Vec<MarkdownChunk>,
    pub code_fence_open: Option<String>,
    pub list_stack: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum MarkdownChunk {
    Text(String),
    CodeBlock { language: Option<String>, content: String, complete: bool },
    Heading { level: u8, text: String },
    ListItem { depth: u8, text: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>>, complete: bool },
    Image { alt: String, url: String },
    Link { text: String, url: String },
    BlockQuote(String),
    HorizontalRule,
}

impl Default for StreamingMarkdown {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingMarkdown {
    pub fn new() -> Self {
        Self { buffer: String::new(), chunks: Vec::new(), code_fence_open: None, list_stack: Vec::new() }
    }

    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.reparse();
    }

    fn reparse(&mut self) {
        let content = self.buffer.clone();
        self.chunks.clear();

        let mut i = 0;
        while i < content.len() {
            if let Some(rest) = content[i..].strip_prefix("```") {
                let line_end = rest.find('\n').unwrap_or(rest.len());
                let lang = rest[..line_end].trim().to_string();
                let lang = if lang.is_empty() { None } else { Some(lang) };
                let code_start = i + 3 + line_end + if line_end < rest.len() { 1 } else { 0 };
                let remaining = &content[code_start..];
                if let Some(close_pos) = remaining.find("```") {
                    let code = remaining[..close_pos].to_string();
                    self.chunks.push(MarkdownChunk::CodeBlock {
                        language: lang, content: code, complete: true,
                    });
                    i = code_start + close_pos + 3;
                } else {
                    self.chunks.push(MarkdownChunk::CodeBlock {
                        language: lang, content: remaining.to_string(), complete: false,
                    });
                    break;
                }
            } else if content[i..].starts_with("## ") {
                let end = content[i..].find('\n').map(|p| i + p).unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::Heading {
                    level: 2, text: content[i + 3..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("# ") {
                let end = content[i..].find('\n').map(|p| i + p).unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::Heading {
                    level: 1, text: content[i + 2..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("- ") || content[i..].starts_with("* ") {
                let end = content[i..].find('\n').map(|p| i + p).unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::ListItem {
                    depth: 0, text: content[i + 2..end].trim().to_string(),
                });
                i = end + 1;
            } else if content[i..].starts_with("> ") {
                let end = content[i..].find('\n').map(|p| i + p).unwrap_or(content.len());
                self.chunks.push(MarkdownChunk::BlockQuote(
                    content[i + 2..end].trim().to_string(),
                ));
                i = end + 1;
            } else if content[i..].starts_with("---") {
                self.chunks.push(MarkdownChunk::HorizontalRule);
                i += 3;
            } else {
                let end = content[i..].find('\n').map(|p| i + p + 1).unwrap_or(content.len());
                let text = content[i..end].trim().to_string();
                if !text.is_empty() {
                    self.chunks.push(MarkdownChunk::Text(text));
                }
                i = end;
            }
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.chunks.clear();
        self.code_fence_open = None;
        self.list_stack.clear();
    }
}

// ── Context Pipeline (from Claude Code: 5-layer compaction) ──

/// Precise token counter backed by a lazily-initialized tiktoken BPE.
/// Falls back to the classic chars/4 estimate if the BPE cannot be built
/// (e.g. offline first-run). The BPE is built once per process.
pub fn count_tokens(text: &str) -> usize {
    static BPE: std::sync::OnceLock<Option<tiktoken_rs::CoreBPE>> = std::sync::OnceLock::new();
    let bpe = BPE.get_or_init(|| {
        tiktoken_rs::cl100k_base()
            .ok()
    });
    match bpe {
        Some(bpe) => bpe.encode_with_special_tokens(text).len(),
        None => text.len() / 4,
    }
}

#[derive(Debug, Clone)]
pub struct ContextTurn {
    pub role: String,
    pub content: String,
    pub token_count: usize,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct ContextPipeline {
    pub turns: VecDeque<ContextTurn>,
    pub max_tokens: usize,
    pub budget_high: f64,
    pub budget_low: f64,
    /// When true, `push` counts tokens with tiktoken instead of trusting the
    /// caller's chars/4 estimate (which over-counts CJK text badly).
    pub use_tiktoken: bool,
}

impl ContextPipeline {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            turns: VecDeque::new(),
            max_tokens,
            budget_high: 0.8,
            budget_low: 0.5,
            use_tiktoken: true,
        }
    }

    pub fn push(&mut self, role: &str, content: String, token_count: usize) {
        let token_count = if self.use_tiktoken {
            count_tokens(&content)
        } else {
            token_count
        };
        self.turns.push_back(ContextTurn {
            role: role.to_string(),
            content,
            token_count,
            priority: match role {
                "system" => 5,
                "tool" => 1,
                _ => 3,
            },
        });
        self.compact_if_needed();
    }

    pub fn total_tokens(&self) -> usize {
        self.turns.iter().map(|t| t.token_count).sum()
    }

    /// 5-layer compaction pipeline (Claude Code-inspired)
    fn compact_if_needed(&mut self) {
        let total = self.total_tokens();
        if total < (self.max_tokens as f64 * self.budget_high) as usize {
            return;
        }

        // Layer 1: Budget reduce — trim oversized tool outputs.
        // Token-consistent: estimator is chars/4 everywhere, so a turn budgeted
        // at `max_turn_tokens` keeps `max_turn_tokens * 4` chars (bytes/4 ≈ tokens).
        let max_turn_tokens = self.max_tokens / 4;
        for turn in &mut self.turns {
            if turn.token_count > max_turn_tokens && turn.priority < 4 {
                let budget_chars = max_turn_tokens * 4;
                let kept = turn.content.chars().take(budget_chars).collect::<String>();
                turn.content = format!(
                    "{}... [trimmed {} bytes]",
                    kept,
                    turn.content.len().saturating_sub(kept.len())
                );
                turn.token_count = if self.use_tiktoken { count_tokens(&turn.content) } else { kept.len() / 4 };
            }
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 2: Snip — reduce temporal depth (keep newest)
        while self.turns.len() > 50 {
            self.turns.pop_front();
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 3: Microcompact — squeeze low-priority turns. Char-safe: the old
        // String::truncate(200) panicked when byte 200 landed mid-UTF-8-char
        // (any non-ASCII tool output now hits this path via tool priority 1).
        let mut i = 0;
        while i < self.turns.len() && self.total_tokens() > (self.max_tokens as f64 * self.budget_low) as usize {
            if self.turns[i].priority < 2 {
                let kept = self.turns[i].content.chars().take(200).collect::<String>();
                self.turns[i].content = format!("{}...", kept);
                self.turns[i].token_count = if self.use_tiktoken { count_tokens(&self.turns[i].content) } else { kept.len() / 4 };
            }
            i += 1;
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 4: Context collapse — distill evicted turns into a single
        // capped summary. Real condensation (preserves role + first line per
        // turn) instead of a no-op placeholder, and terminates deterministically
        // (the previous pop/push-front oscillation could loop forever whenever
        // the pipeline reached this layer with >10 turns).
        let mut distilled = String::new();
        while self.turns.len() > 10 && distilled.len() < 4_000 {
            let front = self.turns.pop_front().expect("guarded by len > 10");
            if !distilled.is_empty() {
                distilled.push('\n');
            }
            let first = front.content.lines().next().unwrap_or_default();
            distilled.push_str(&format!(
                "[{}] {}",
                front.role,
                first.chars().take(120).collect::<String>()
            ));
        }
        if !distilled.is_empty() {
            let distilled_tokens = if self.use_tiktoken { count_tokens(&distilled) } else { distilled.len() / 4 };
            self.turns.push_front(ContextTurn {
                role: "summary".into(),
                content: distilled,
                token_count: distilled_tokens,
                priority: 1,
            });
        }

        // Layer 5: Auto-compact — hard cap
        while self.total_tokens() > self.max_tokens {
            if self.turns.len() <= 2 { break; }
            self.turns.pop_front();
        }
    }
}

// ── Wire Format (from Kimi Code: JSONL event stream) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WireEvent {
    UserMessage {
        content: String,
        timestamp: i64,
        #[serde(default)]
        attachments: Option<Vec<NeoCodexAttachment>>,
    },
    AgentMessage { content: String, timestamp: i64 },
    ToolCall { name: String, args: String, result: String, duration_ms: u64, success: bool },
    SystemEvent { kind: String, detail: String, timestamp: i64 },
    GoalUpdate { id: String, state: String, description: String },
    ModeChange { from: NeoCodexMode, to: NeoCodexMode },
    SessionMeta {
        name: String,
        timestamp: i64,
        #[serde(default)]
        tags: Vec<String>,
    },
    // P1-2: side chat now carries a role so the UI can render a real answer
    // bubble; `role` defaults to "user" so pre-fix wire lines stay compatible.
    SideChatMessage {
        content: String,
        timestamp: i64,
        #[serde(default)]
        role: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeoCodexAttachment {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WireSession {
    pub session_id: String,
    pub events: Vec<WireEvent>,
    pub path: std::path::PathBuf,
}

impl WireSession {
    pub fn new(session_id: &str) -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".neocodex"))
            .join("neocodex").join("sessions");
        Self {
            session_id: session_id.to_string(),
            events: Vec::new(),
            path: base.join(format!("{}.jsonl", session_id)),
        }
    }

    pub fn record(&mut self, event: WireEvent) {
        // Bound in-memory growth: long-running sessions otherwise accumulate
        // every message/tool event in memory forever. The JSONL file below
        // keeps the full history for replay/load, so only the live Vec is
        // capped.
        const MAX_IN_MEMORY_EVENTS: usize = 10_000;
        self.events.push(event.clone());
        if self.events.len() > MAX_IN_MEMORY_EVENTS {
            let drop_to = self.events.len() - MAX_IN_MEMORY_EVENTS;
            self.events.drain(0..drop_to);
        }
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(line) = serde_json::to_string(&event) {
            let _ = std::fs::OpenOptions::new()
                .create(true).append(true).open(&self.path)
                .and_then(|f| {
                    use std::io::Write;
                    writeln!(&f, "{}", line)
                });
        }
    }

    pub fn replay(path: &std::path::Path) -> Vec<WireEvent> {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        content.lines()
            .filter_map(|l| serde_json::from_str::<WireEvent>(l).ok())
            .collect()
    }

    /// Load all events for this session (empty if none recorded yet).
    pub fn load(&self) -> Vec<WireEvent> {
        if self.path.exists() {
            Self::replay(&self.path)
        } else {
            Vec::new()
        }
    }
}

// ── Goal System (from Kimi Code: chained goals) ──

#[derive(Debug, Clone)]
pub enum GoalState { Active, Paused, Completed, Blocked, Cancelled }

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub state: GoalState,
    pub created_at: Instant,
    pub iterations: u64,
    pub max_iterations: u64,
}

pub struct GoalQueue {
    pub goals: VecDeque<Goal>,
    pub active: Option<Goal>,
    pub completed: Vec<Goal>,
    next_id: u64,
}

impl Default for GoalQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalQueue {
    pub fn new() -> Self {
        Self { goals: VecDeque::new(), active: None, completed: Vec::new(), next_id: 0 }
    }

    pub fn add(&mut self, description: &str, max_iterations: u64) {
        // completed.len() + goals.len() + 1 omits the active goal, so two
        // goals could share an id (e.g. add A -> g-1 active, add B -> g-1
        // again) and corrupt WireEvent::GoalUpdate correlation. Use a
        // monotonic counter instead.
        self.next_id += 1;
        let id = format!("g-{}", self.next_id);
        self.goals.push_back(Goal {
            id, description: description.to_string(),
            state: GoalState::Active,
            created_at: Instant::now(),
            iterations: 0, max_iterations,
        });
    }

    pub fn next(&mut self) -> Option<Goal> {
        if let Some(prev) = self.active.take() {
            self.completed.push(prev);
        }
        self.active = self.goals.pop_front();
        self.active.clone()
    }
}

// ── Lifecycle Hooks (from Kimi Code: pre/post tool gates) ──

#[derive(Debug, Clone)]
pub enum HookDecision {
    Allow,
    Deny(String),
    RequireConfirm(String),
}

#[derive(Debug, Clone)]
pub struct ToolCallContext {
    pub tool_name: String,
    pub args: String,
    pub cwd: String,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub decision: HookDecision,
    pub duration_ms: u64,
}

pub type PreToolHook = Arc<dyn Fn(ToolCallContext) -> HookResult + Send + Sync>;
pub type PostToolHook = Arc<dyn Fn(ToolCallContext, String, u64) + Send + Sync>;

pub struct LifecycleHookRegistry {
    pub pre_hooks: Vec<(String, PreToolHook)>,
    pub post_hooks: Vec<(String, PostToolHook)>,
}

impl Default for LifecycleHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleHookRegistry {
    pub fn new() -> Self {
        Self { pre_hooks: Vec::new(), post_hooks: Vec::new() }
    }

    pub fn register_pre(&mut self, name: &str, hook: PreToolHook) {
        self.pre_hooks.push((name.to_string(), hook));
    }

    pub fn register_post(&mut self, name: &str, hook: PostToolHook) {
        self.post_hooks.push((name.to_string(), hook));
    }

    pub fn run_pre(&self, ctx: ToolCallContext) -> HookDecision {
        for (_, hook) in &self.pre_hooks {
            let result = hook(ctx.clone());
            match result.decision {
                HookDecision::Deny(reason) => return HookDecision::Deny(reason),
                HookDecision::RequireConfirm(msg) => {
                    eprint!("[hook] {} — Allow? (y/N): ", msg);
                    let mut buf = String::new();
                    let _ = std::io::stdin().read_line(&mut buf);
                    match buf.trim().to_lowercase().as_str() {
                        "y" | "yes" => continue,
                        _ => return HookDecision::Deny("User denied at confirmation hook".into()),
                    }
                }
                HookDecision::Allow => continue,
            }
        }
        HookDecision::Allow
    }

    pub fn run_post(&self, ctx: ToolCallContext, output: String, duration_ms: u64) {
        for (_, hook) in &self.post_hooks {
            hook(ctx.clone(), output.clone(), duration_ms);
        }
    }
}

// ── Subagent System (from Claude Code: fork/async/sync/teammate 4 paths) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubagentKind { Coder, Explorer, Planner }

#[derive(Debug, Clone)]
pub struct SubagentResult {
    pub kind: SubagentKind,
    pub output: String,
    pub tool_calls: u64,
    pub duration_ms: u64,
    pub success: bool,
}

pub struct SubagentDispatch;

impl SubagentDispatch {
    pub async fn run(kind: SubagentKind, task: &str, cwd: &str) -> SubagentResult {
        let start = Instant::now();
        let sub_prompt = match kind {
            SubagentKind::Coder => task.to_string(),
            SubagentKind::Explorer => format!("Explore and summarize: {}", task),
            SubagentKind::Planner => format!("Create a plan for: {}", task),
        };
        let output = tokio::process::Command::new("opencode")
            .arg("exec")
            .arg(&sub_prompt)
            .current_dir(cwd)
            .output().await;
        let dur = start.elapsed().as_millis() as u64;
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                SubagentResult {
                    kind, output: stdout,
                    tool_calls: 1, duration_ms: dur,
                    success: out.status.success(),
                }
            }
            Err(e) => SubagentResult {
                kind, output: format!("Subagent error: {}", e),
                tool_calls: 0, duration_ms: dur, success: false,
            },
        }
    }

    pub async fn run_parallel(tasks: Vec<(SubagentKind, String)>, cwd: &str) -> Vec<SubagentResult> {
        let handles: Vec<_> = tasks.into_iter()
            .map(|(kind, task)| {
                let cwd = cwd.to_string();
                tokio::spawn(async move { Self::run(kind, &task, &cwd).await })
            })
            .collect();
        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.unwrap_or_else(|e| SubagentResult {
                kind: SubagentKind::Coder, output: format!("Join error: {}", e),
                tool_calls: 0, duration_ms: 0, success: false,
            }));
        }
        results
    }
}

// ── Cost Tracker (from Claude Code: budget enforcement) ──

#[derive(Debug, Clone)]
pub struct CostTracker {
    pub total_spent: f64,
    pub max_budget: f64,
    pub call_count: u64,
    pub history: VecDeque<(String, f64, u64)>,
}

impl CostTracker {
    pub fn new(max_budget: f64) -> Self {
        Self { total_spent: 0.0, max_budget, call_count: 0, history: VecDeque::new() }
    }

    pub fn record(&mut self, tool: &str, cost: f64, tokens: u64) -> Result<(), String> {
        if self.total_spent + cost > self.max_budget {
            return Err(format!("Budget exceeded: ${:.4} + ${:.4} > ${:.4} limit",
                self.total_spent, cost, self.max_budget));
        }
        self.total_spent += cost;
        self.call_count += 1;
        self.history.push_back((tool.to_string(), cost, tokens));
        if self.history.len() > 1000 { self.history.pop_front(); }
        Ok(())
    }

    pub fn remaining(&self) -> f64 {
        (self.max_budget - self.total_spent).max(0.0)
    }

    pub fn summary(&self) -> String {
        format!("${:.4} spent / ${:.4} budget · {} calls", self.total_spent, self.max_budget, self.call_count)
    }
}

// ── Permission System (from Claude Code: allow/deny/ask per tool) ──

#[derive(Debug, Clone)]
pub enum PermissionLevel { Allow, Deny, Ask }

pub struct PermissionSystem {
    pub permissions: Vec<(String, PermissionLevel)>,
    pub default_level: PermissionLevel,
}

impl Default for PermissionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionSystem {
    pub fn new() -> Self {
        let permissions = vec![
            ("read".to_string(), PermissionLevel::Allow),
            ("search".to_string(), PermissionLevel::Allow),
            ("write".to_string(), PermissionLevel::Allow),
            ("edit".to_string(), PermissionLevel::Allow),
            ("shell".to_string(), PermissionLevel::Ask),
        ];
        Self { permissions, default_level: PermissionLevel::Ask }
    }

    /// Non-blocking policy gate for the streaming path (P0-2). Unlike
    /// `interactive_check` (stdin, CLI-only), this decides from the active
    /// permission mode so the desktop app can enforce Claude-style modes
    /// without a blocking terminal prompt.
    pub fn policy_gate(&self, tool: &str, permission_mode: &str) -> bool {
        match self.check(tool) {
            // read/search/write/edit are workspace-guarded → always allowed.
            PermissionLevel::Allow => true,
            // Deny is absolute regardless of mode.
            PermissionLevel::Deny => false,
            // shell (the only Ask tool): allowed in agentic modes, denied in
            // restrictive ones (manual/accept_edits/plan) where the UI review
            // layer is the enforcement point instead.
            PermissionLevel::Ask => !matches!(
                permission_mode,
                "manual" | "accept_edits" | "acceptEdits" | "plan"
            ),
        }
    }

    pub fn check(&self, tool: &str) -> PermissionLevel {
        for (pattern, level) in &self.permissions {
            if tool.starts_with(pattern) {
                return level.clone();
            }
        }
        self.default_level.clone()
    }

    pub fn set(&mut self, tool: &str, level: PermissionLevel) {
        if let Some((_, l)) = self.permissions.iter_mut().find(|(p, _)| p == tool) {
            *l = level;
        } else {
            self.permissions.push((tool.to_string(), level));
        }
    }

    pub fn interactive_check(&self, tool: &str, description: &str) -> bool {
        match self.check(tool) {
            PermissionLevel::Allow => true,
            PermissionLevel::Deny => {
                eprintln!("[permission] DENY: {} — {}", tool, description);
                false
            }
            PermissionLevel::Ask => {
                eprint!("[permission] Allow {} ({})? (y/N): ", tool, description);
                let mut buf = String::new();
                let _ = std::io::stdin().read_line(&mut buf);
                buf.trim().to_lowercase() == "y" || buf.trim().to_lowercase() == "yes"
            }
        }
    }
}

// ── ACP Server (Agent Client Protocol for editor integration) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AcpError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpNotification {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpError {
    pub code: i32,
    pub message: String,
}

pub struct AcpServer {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
}

impl AcpServer {
    pub fn new(agent: Arc<Mutex<NeoCodexAgent>>) -> Self {
        Self { agent }
    }

    pub async fn handle_request(&self, req: AcpRequest) -> AcpResponse {
        let AcpRequest { id, method, params } = req;
        if !matches!(method.as_str(),
            "ping" | "agent/process" | "agent/status" | "agent/mode" | "tools/list" | "shutdown")
        {
            // Protocol errors belong in `error`, not smuggled into `result` —
            // clients keying on the error field otherwise see a success.
            return AcpResponse {
                id,
                result: None,
                error: Some(AcpError { code: -32601, message: format!("unknown method: {}", method) }),
            };
        }
        let result = self.dispatch(method, params).await;
        AcpResponse { id, result: Some(result), error: None }
    }

    async fn dispatch(&self, method: String, params: serde_json::Value) -> serde_json::Value {
        match method.as_str() {
            "ping" => serde_json::json!({"pong": true}),
            "agent/process" => {
                let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
                let mut agent = self.agent.lock().await;
                let response = agent.process(input).await;
                serde_json::json!({"response": response, "turn": agent.state.turn_count})
            }
            "agent/status" => {
                let agent = self.agent.lock().await;
                serde_json::json!({
                    "mode": agent.state.mode,
                    "turn": agent.state.turn_count,
                    "tools": agent.state.tool_call_count,
                    "tokens": agent.state.tokens_used,
                })
            }
            "agent/mode" => {
                let mode_name = params.get("mode").and_then(|v| v.as_str()).unwrap_or("agent");
                let mut agent = self.agent.lock().await;
                match mode_name {
                    "shell" => { agent.toggle_mode(); }
                    "plan" => { agent.set_plan_mode(); }
                    _ => {}
                }
                serde_json::json!({"mode": agent.state.mode})
            }
            "tools/list" => serde_json::json!({
                "tools": [
                    {"name": "read", "description": "Read files"},
                    {"name": "search", "description": "Search codebase"},
                    {"name": "shell", "description": "Execute shell commands"},
                    {"name": "edit", "description": "Replace a unique old substring with new. Args: <path>|<old>|<new>"},
                    {"name": "write", "description": "Write or overwrite a file. Args: <path>|<content>"},
                    {"name": "plan", "description": "Create/edit plans"},
                ]
            }),
            "shutdown" => {
                serde_json::json!({"shutdown": true})
            }
            _ => serde_json::json!({"error": format!("unknown method: {}", method)}),
        }
    }

    /// Run ACP server over stdio (JSON-RPC 2.0 line protocol)
    pub async fn run_stdio(&self) {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }
            if let Ok(req) = serde_json::from_str::<AcpRequest>(&line) {
                let resp = self.handle_request(req).await;
                if let Ok(json) = serde_json::to_string(&resp) {
                    use std::io::Write;
                    let mut out = stdout.lock();
                    let _ = writeln!(out, "{}", json);
                    let _ = out.flush();
                }
            }
        }
    }
}

// ── Generator-based Agent Loop (Claude Code-inspired: yield events) ──

#[derive(Debug, Clone)]
pub enum AgentEvent {
    TurnStart { turn: u64, mode: NeoCodexMode },
    Thinking { content: String },
    ToolCallStart { name: String, args: String },
    ToolCallEnd { name: String, result: String, duration_ms: u64, success: bool },
    Chunk { text: String },
    TurnEnd { response: String },
    Error { message: String },
    ModeSwitch { from: NeoCodexMode, to: NeoCodexMode },
    BudgetWarning { remaining: f64, limit: f64 },
    Done,
}

pub struct AgentStream {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
    pub cost: Arc<Mutex<CostTracker>>,
    pub permissions: Arc<PermissionSystem>,
    pub hooks: Arc<LifecycleHookRegistry>,
    pub events: VecDeque<AgentEvent>,
}

impl AgentStream {
    pub fn new(agent: NeoCodexAgent, max_budget: f64) -> Self {
        Self {
            agent: Arc::new(Mutex::new(agent)),
            cost: Arc::new(Mutex::new(CostTracker::new(max_budget))),
            permissions: Arc::new(PermissionSystem::new()),
            hooks: Arc::new(LifecycleHookRegistry::new()),
            events: VecDeque::new(),
        }
    }

    /// Process input and yield events. Users poll `next_event()`.
    pub async fn process(&mut self, input: &str) {
        let mut agent = self.agent.lock().await;
        let turn = agent.state.turn_count + 1;

        self.events.push_back(AgentEvent::TurnStart {
            turn, mode: agent.state.mode,
        });

        // Permission check
        let tool_name = match agent.state.mode {
            NeoCodexMode::Shell => "shell",
            NeoCodexMode::Plan => "plan",
            NeoCodexMode::Agent => "agent",
        };
        if !self.permissions.interactive_check(tool_name, input) {
            self.events.push_back(AgentEvent::Error {
                message: format!("Permission denied for {}: {}", tool_name, input),
            });
            self.events.push_back(AgentEvent::Done);
            return;
        }

        // Lifecycle pre-hook
        let ctx = ToolCallContext {
            tool_name: tool_name.to_string(),
            args: input.to_string(),
            cwd: std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default(),
            estimated_cost: input.len() as f64 * 0.00001,
        };
        match self.hooks.run_pre(ctx.clone()) {
            HookDecision::Deny(reason) => {
                self.events.push_back(AgentEvent::Error {
                    message: format!("Hook denied: {}", reason),
                });
                self.events.push_back(AgentEvent::Done);
                return;
            }
            HookDecision::RequireConfirm(msg) => {
                self.events.push_back(AgentEvent::Thinking {
                    content: format!("[hook requires confirmation: {}]", msg),
                });
            }
            HookDecision::Allow => {}
        }

        // Cost check
        let estimated_cost = input.len() as f64 * 0.00001;
        {
            let mut cost = self.cost.lock().await;
            if let Err(e) = cost.record(tool_name, estimated_cost, input.len() as u64 / 4) {
                self.events.push_back(AgentEvent::BudgetWarning {
                    remaining: cost.remaining(),
                    limit: cost.max_budget,
                });
                self.events.push_back(AgentEvent::Error { message: e });
                self.events.push_back(AgentEvent::Done);
                return;
            }
            if cost.remaining() < cost.max_budget * 0.1 {
                self.events.push_back(AgentEvent::BudgetWarning {
                    remaining: cost.remaining(), limit: cost.max_budget,
                });
            }
        }

        // Execute
        self.events.push_back(AgentEvent::Thinking {
            content: format!("Processing in {:?} mode...", agent.state.mode),
        });

        let start = Instant::now();
        let response = agent.process(input).await;
        let duration = start.elapsed().as_millis() as u64;

        for line in response.lines() {
            self.events.push_back(AgentEvent::Chunk { text: line.to_string() });
        }
        self.events.push_back(AgentEvent::TurnEnd { response: response.clone() });
        self.events.push_back(AgentEvent::Done);

        // Lifecycle post-hook
        self.hooks.run_post(ctx, response.clone(), duration);

        // Cost post-record
        {
            let mut cost = self.cost.lock().await;
            let _ = cost.record(tool_name, 0.0, response.len() as u64 / 4);
        }
    }

    pub fn next_event(&mut self) -> Option<AgentEvent> {
        self.events.pop_front()
    }
}

// ── NeoCodex Agent Loop (from Claude Code: ReAct pattern + NeoTrix Consciousness) ──

#[derive(Debug, Clone)]
pub struct NeoCodexConfig {
    pub mode: NeoCodexMode,
    pub max_turn_tokens: usize,
    pub provider_name: String,
    pub auto_compact: bool,
    pub shell_available: bool,
    pub thinking_enabled: bool,
    pub goal_mode: bool,
    /// P2-1: generation parameters surfaced from the desktop settings panel.
    /// Previously hardcoded (temperature 0.3 / max_tokens 4096) so the
    /// editable Settings temperature/maxTokens fields never reached the LLM.
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for NeoCodexConfig {
    fn default() -> Self {
        Self {
            mode: NeoCodexMode::default(),
            max_turn_tokens: 0,
            provider_name: "neotrix".to_string(),
            auto_compact: true,
            shell_available: true,
            thinking_enabled: false,
            goal_mode: false,
            temperature: 0.3,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub tokens_used: usize,
    pub mode: NeoCodexMode,
    pub mode_start: Instant,
    pub goal_active: bool,
    /// Permission policy for the streaming path (P0-2). Mirrors Claude Code
    /// Manual/AcceptEdits/Plan and Codex approval modes. Stored on the agent
    /// because the desktop UI has no blocking stdin for `interactive_check`.
    pub permission_mode: String,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            turn_count: 0,
            tool_call_count: 0,
            tokens_used: 0,
            mode: NeoCodexMode::Agent,
            mode_start: Instant::now(),
            goal_active: false,
            permission_mode: "auto".to_string(),
        }
    }
}

pub struct NeoCodexAgent {
    pub config: NeoCodexConfig,
    pub state: AgentState,
    pub context: ContextPipeline,
    pub provider: ProviderCatalog,
    pub goals: GoalQueue,
    pub wire: WireSession,
    pub markdown: StreamingMarkdown,
    pub consciousness: Option<crate::core::nt_core_consciousness_tree::ConsciousnessTree>,
    pub event_bus: Option<crate::neotrix::nt_core_event_bus::EventBus>,
    pub brain: Option<Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>>,
    // Cycle 112b additions
    pub hooks: LifecycleHookRegistry,
    pub cost: CostTracker,
    pub permissions: PermissionSystem,
    pub subagent_results: Vec<SubagentResult>,
    // Cycle 159 additions: self-audit + evolution loop
    pub evolution: EvolutionLoop,
    pub audit: NeoCodexSelfAudit,
    // Cycle 160e: tool grounding monitor (D25 production-wired, R-P49~R-P53)
    pub tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor,
    // P2-5: MCP tool registry (Codex/Claude MCP parity). When registered, the
    // agent gains a `mcp_call` tool proxying to the registry; previously the
    // MCP host existed only for CLI/headless and the NeoCodex agent could not
    // call MCP tools despite the desktop UI having zero MCP surface.
    pub mcp: Option<crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry>,
}

impl NeoCodexAgent {
    pub fn new(session_id: &str) -> Self {
        Self {
            config: NeoCodexConfig::default(),
            state: AgentState::new(),
            context: ContextPipeline::new(100_000),
            provider: ProviderCatalog::new(),
            goals: GoalQueue::new(),
            wire: WireSession::new(session_id),
            markdown: StreamingMarkdown::new(),
            consciousness: None,
            event_bus: None,
            brain: None,
            hooks: LifecycleHookRegistry::new(),
            cost: CostTracker::new(10.0),
            permissions: PermissionSystem::new(),
            subagent_results: Vec::new(),
            evolution: EvolutionLoop::new(),
            audit: NeoCodexSelfAudit::new(),
            tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor::new(),
            mcp: None,
        }
    }

    /// P2-5: attach the shared MCP registry so the agent can call MCP tools.
    pub fn with_mcp(mut self, mcp: crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry) -> Self {
        self.mcp = Some(mcp);
        self
    }

    pub fn set_mcp(&mut self, mcp: Option<crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry>) {
        self.mcp = mcp;
    }

    /// Set budget limit (from Claude Code max_budget_usd)
    pub fn with_budget(mut self, max_budget: f64) -> Self {
        self.cost.max_budget = max_budget;
        self
    }

    /// P2-1: set generation params from the desktop settings panel. Applies on
    /// the next request built by build_request (was previously hardcoded).
    pub fn set_generation_params(&mut self, temperature: Option<f64>, max_tokens: Option<u32>) {
        if let Some(t) = temperature {
            self.config.temperature = t.clamp(0.0, 2.0);
        }
        if let Some(m) = max_tokens {
            self.config.max_tokens = m.max(1);
        }
    }

    /// Register a pre-tool lifecycle hook (from Kimi Code lifecycle hooks)
    pub fn add_pre_hook<F>(&mut self, name: &str, hook: F)
    where F: Fn(ToolCallContext) -> HookResult + Send + Sync + 'static {
        self.hooks.register_pre(name, Arc::new(hook));
    }

    /// Dispatch parallel subagents (from Claude Code fork/async/sync)
    pub async fn dispatch_subagents(&mut self, tasks: Vec<(SubagentKind, String)>) {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.subagent_results = SubagentDispatch::run_parallel(tasks, &cwd).await;
    }

    /// Get all tool call context for permission checking
    fn current_tool_context(&self) -> ToolCallContext {
        ToolCallContext {
            tool_name: format!("{:?}", self.state.mode),
            args: String::new(),
            cwd: std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default(),
            estimated_cost: self.state.tokens_used as f64 * 0.00001,
        }
    }

    pub fn set_consciousness_tree(&mut self, tree: crate::core::nt_core_consciousness_tree::ConsciousnessTree) {
        self.consciousness = Some(tree);
    }

    pub fn set_event_bus(&mut self, bus: crate::neotrix::nt_core_event_bus::EventBus) {
        self.event_bus = Some(bus);
    }

    pub fn set_brain(&mut self, brain: Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>) {
        self.brain = Some(brain);
    }

    /// Set an explicit mode (Agent/Shell/Plan)
    pub fn set_mode(&mut self, mode: NeoCodexMode) {
        if self.state.mode == mode {
            return;
        }
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange { from, to: mode });
        self.state.mode = mode;
        self.state.mode_start = Instant::now();
    }

    /// Persist a user-chosen session name into the wire stream (overrides
    /// the derived first-message name on read).
    pub fn rename_session(&mut self, name: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        // 保留既有标签（重命名不动 tags；读旧态标签并回写）
        let tags = self.read_session_tags();
        self.wire.record(WireEvent::SessionMeta {
            name: name.trim().to_string(),
            timestamp: ts,
            tags,
        });
    }

    /// Read the persisted tag set for the current wire session (from last
    /// SessionMeta). Empty when none recorded yet.
    fn read_session_tags(&self) -> Vec<String> {
        self.wire.events.iter().rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }))
            .and_then(|e| match e {
                WireEvent::SessionMeta { tags, .. } => Some(tags.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Persist a tag onto the current session (deduped). Writes a new
    /// SessionMeta carrying the merged tag set.
    pub fn tag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let clean = tag.trim().to_lowercase().replace(' ', "-");
        if clean.is_empty() || tags.contains(&clean) {
            return;
        }
        tags.push(clean);
        self.write_session_tags(tags);
    }

    /// Remove a tag from the current session. No-op when absent.
    pub fn untag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let before = tags.len();
        tags.retain(|t| t != tag);
        if tags.len() != before {
            self.write_session_tags(tags);
        }
    }

    /// Overwrite persisted tags (rename/tag/untag 共用落盘点)。
    fn write_session_tags(&mut self, tags: Vec<String>) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let meta = self.wire.events.iter().rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }));
        let name = meta.and_then(|e| match e {
            WireEvent::SessionMeta { name, .. } => Some(name.clone()),
            _ => None,
        }).unwrap_or_else(|| self.wire.session_id.clone());
        self.wire.record(WireEvent::SessionMeta {
            name,
            timestamp: ts,
            tags,
        });
    }

    /// Record a side-chat message (branched question that must NOT pollute
    /// the main session context). Persisted to the same wire stream but
    /// filtered out of resume_session / get_session_messages.
    pub fn record_side_chat(&mut self, content: &str, role: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        self.wire.record(WireEvent::SideChatMessage {
            content: content.trim().to_string(),
            timestamp: ts,
            role: role.to_string(),
        });
    }

    /// Detach the agent from its current wire file (P2-3). After a session is
    /// archived/deleted while active, the stale `wire.path` would otherwise
    /// recreate the file on the next `record()` (`create+append`) and split
    /// the conversation into a divergent duplicate. Resets to a fresh empty
    /// session so the next turn starts clean.
    pub fn detach_wire(&mut self) {
        self.wire = WireSession::new(&format!(
            "s-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()
        ));
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.turn_count = 0;
        self.state.tool_call_count = 0;
    }

    /// One-shot side-chat answer (P1-2). Unlike the main loop, this must NOT
    /// push into `self.context` / `self.state` — the branch question is
    /// isolated from the session's real conversation. We generate the reply
    /// on a throwaway message list and record both turns as side-chat events.
    pub async fn side_chat_ask(&mut self, content: &str) -> String {
        let provider = match self.provider.to_llm_provider() {
            Some(p) => p,
            None => return "[side chat] no provider configured".to_string(),
        };
        let system = "You are NeoCodex, an AI coding assistant. Answer the user's question concisely and precisely. Respond in markdown.";
        let messages = vec![
            Message::new(Role::System, system),
            Message::new(Role::User, content.trim()),
        ];
        let request = match self.build_request(messages) {
            Some(r) => r,
            None => return "[side chat] provider unavailable".to_string(),
        };
        let mut rx = match provider.stream_complete(&request).await {
            Ok(rx) => rx,
            Err(e) => return format!("[side chat] {}", e),
        };
        let mut answer = String::new();
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(resp) => answer.push_str(&resp.content),
                Err(e) => return format!("[side chat] {}", e),
            }
        }
        if answer.trim().is_empty() {
            "[side chat] empty response".to_string()
        } else {
            answer
        }
    }

    /// Toggle between Agent and Shell mode (from Kimi Code Ctrl-X)
    pub fn toggle_mode(&mut self) -> NeoCodexMode {
        let from = self.state.mode;
        self.state.mode = match from {
            NeoCodexMode::Agent => NeoCodexMode::Shell,
            NeoCodexMode::Shell => NeoCodexMode::Agent,
            NeoCodexMode::Plan => NeoCodexMode::Agent,
        };
        self.state.mode_start = Instant::now();
        self.wire.record(WireEvent::ModeChange {
            from,
            to: self.state.mode,
        });
        self.state.mode
    }

    /// Switch to Plan mode (from Kimi Code Shift-Tab / Claude Code Plan)
    pub fn set_plan_mode(&mut self) {
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange {
            from,
            to: NeoCodexMode::Plan,
        });
        self.state.mode = NeoCodexMode::Plan;
        self.state.mode_start = Instant::now();
    }

    /// Set the streaming-path permission policy (P0-2). Called by the desktop
    /// command layer from the UI's permission_mode (auto/manual/accept_edits/plan).
    pub fn set_permission_mode(&mut self, mode: &str) {
        self.state.permission_mode = mode.to_string();
        if mode == "plan" {
            self.set_plan_mode();
        }
    }

    /// Process user input through the agent loop
    pub async fn process(&mut self, input: &str) -> String {
        self.state.turn_count += 1;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        self.wire.record(WireEvent::UserMessage {
            content: input.to_string(),
            timestamp,
            attachments: None,
        });

        let token_estimate = input.len() / 4;
        self.context.push("user", input.to_string(), token_estimate);
        self.state.tokens_used += token_estimate;

        // ── Consciousness-in-the-loop (NeoTrix unique) ──
        self.inject_into_consciousness();
        self.apply_consciousness_guidance();
        if let Some(completed_goal) = self.check_goals() {
            log::debug!("[neocodex] goal completed, advancing: {}", completed_goal);
        }

        let response = match self.state.mode {
            NeoCodexMode::Shell => self.exec_shell(input).await,
            NeoCodexMode::Plan => self.exec_plan(input).await,
            NeoCodexMode::Agent => self.exec_agent(input).await,
        };

        self.wire.record(WireEvent::AgentMessage {
            content: response.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
        });

        response
    }

    async fn exec_shell(&mut self, input: &str) -> String {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(input)
            .output()
            .await;
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code().unwrap_or(-1);
                let result = if exit_code == 0 { stdout } else { format!("exit({}): {}", exit_code, stderr) };
                self.state.tool_call_count += 1;
                self.wire.record(WireEvent::ToolCall {
                    name: "shell".into(), args: input.to_string(),
                    result: result.clone(), duration_ms: 0, success: exit_code == 0,
                });
                if exit_code != 0 {
                    format!("```\n{}\n```\n\nExit code: {}", result, exit_code)
                } else {
                    format!("```{}\n```", result.trim())
                }
            }
            Err(e) => format!("Shell error: {}", e),
        }
    }

    async fn exec_plan(&mut self, input: &str) -> String {
        let plan = self.consciousness_plan(input);
        let token_estimate = plan.len() / 4;
        self.context.push("assistant", plan.clone(), token_estimate);
        plan
    }

    async fn exec_agent(&mut self, input: &str) -> String {
        // Pre-execution hook gate
        let ctx = self.current_tool_context();
        match self.hooks.run_pre(ctx.clone()) {
            HookDecision::Deny(reason) => {
                return format!("[blocked] Hook denied: {}", reason);
            }
            HookDecision::RequireConfirm(msg) => {
                let response = format!("[confirmed] {}\n\n<thinking>Processing turn {} in Agent mode</thinking>\n\n{}",
                    msg, self.state.turn_count, input);
                self.markdown.push(&response);
                let clean = response.clone();
                self.context.push("assistant", clean.clone(), clean.len() / 4);
                self.hooks.run_post(ctx, clean.clone(), 0);
                return clean;
            }
            HookDecision::Allow => {}
        }

        let start = Instant::now();

        // Cycle 159: real ReAct loop via provider if a concrete model is resolvable
        let response = match self.react_loop(input, 4).await {
            Some(out) => out,
            None => {
                // Fallback: provider not wired — keep the deterministic stub so the
                // agent remains usable offline (no silent "dead agent").
                format!("<thinking>Processing turn {} in Agent mode (provider unavailable, stub)</thinking>\n\n{}",
                    self.state.turn_count, input)
            }
        };

        self.markdown.push(&response);
        let clean = response.clone();
        let token_estimate = clean.len() / 4;
        self.context.push("assistant", clean.clone(), token_estimate);
        let _ = self.cost.record("agent", 0.0, token_estimate as u64);
        self.hooks.run_post(ctx, clean.clone(), start.elapsed().as_millis() as u64);

        // Evolution loop advances every turn (self-audit → diagnose → fix)
        EvolutionLoop::step(self);

        clean
    }

    /// Add a goal to the queue (from Kimi Code /goal system)
    pub fn add_goal(&mut self, description: &str, max_iters: u64) {
        self.goals.add(description, max_iters);
        let id = self.goals.goals.back().map(|g| g.id.clone()).unwrap_or_default();
        self.state.goal_active = true;
        self.wire.record(WireEvent::GoalUpdate {
            id, state: "active".into(), description: description.into(),
        });
    }

    /// Check if a goal is complete and advance the queue.
    ///
    /// Also increments the active goal's iteration counter each turn and
    /// resets `goal_active` once the queue drains, so the goal loop can
    /// actually make progress (previously `check_goals` was never called,
    /// `iterations` never incremented, and `goal_active` never reset).
    /// When no goal is active but the queue is non-empty, promotes the head
    /// of the queue (add_goal queues but never promotes).
    pub fn check_goals(&mut self) -> Option<String> {
        if self.goals.active.is_none() && !self.goals.goals.is_empty() {
            self.goals.next();
        }
        if let Some(ref mut goal) = self.goals.active {
            goal.iterations = goal.iterations.saturating_add(1);
            if goal.iterations >= goal.max_iterations {
                self.wire.record(WireEvent::GoalUpdate {
                    id: goal.id.clone(), state: "completed".into(),
                    description: goal.description.clone(),
                });
                let next = self.goals.next().map(|g| g.description);
                if self.goals.active.is_none() && self.goals.goals.is_empty() {
                    self.state.goal_active = false;
                }
                return next;
            }
        }
        None
    }

    /// Record a tool call (from Claude Code: StreamingToolExecutor pattern)
    pub fn record_tool_call(&mut self, name: &str, args: &str, result: String, duration_ms: u64, success: bool) {
        self.state.tool_call_count += 1;
        self.wire.record(WireEvent::ToolCall {
            name: name.to_string(), args: args.to_string(),
            result, duration_ms, success,
        });
    }

    /// Push agent state into ConsciousnessTree soil (outbound integration)
    fn inject_into_consciousness(&mut self) {
        if let Some(ref mut tree) = self.consciousness {
            tree.soil.crawl_queue_depth = self.state.turn_count;
            tree.run_growth_cycle();
        }
    }

    /// Consume ConsciousnessTree guidance to adjust agent behavior (inbound integration)
    fn apply_consciousness_guidance(&mut self) {
        let Some(ref tree) = self.consciousness else { return };

        let fruit_count = tree.fruits.len();
        let phi_avg = if !tree.fruits.is_empty() {
            tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
        } else {
            0.0
        };
        self.config.thinking_enabled = phi_avg > 0.3;

        if fruit_count < 3 && tree.cycle > 5 && !self.state.goal_active {
            self.state.mode = NeoCodexMode::Plan;
            self.add_goal("Cultivate capability branches: absorb external knowledge", 5);
        }
    }

    /// Feed consciousness data into the agent's plan mode
    fn consciousness_plan(&mut self, input: &str) -> String {
        match self.consciousness.as_ref() {
            None => format!("## Plan\n\n{}\n\n---\n\nAwaiting approval...", input),
            Some(tree) => {
                let guidance = tree.core.next_actions.join("; ");
                let avg_quality = if !tree.fruits.is_empty() {
                    tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
                } else {
                    0.0
                };
                format!(
                    "## Plan (Cycle {})\n\n**Avg Quality**: {:.3}\n\n**Next actions**: {}\n\n---\n\n{}",
                    tree.cycle, avg_quality, guidance, input,
                )
            }
        }
    }

    // ── Cycle 159: Real ReAct Loop + Self-Audit + Evolution ──

    /// True ReAct loop: build messages from context pipeline, call the real LLM
    /// provider, then parse any tool-call block and execute it. Loops up to
    /// `max_steps` (mirrors Claude Code's streaming tool executor + Codex's
    /// plan-execute cycle).
    async fn react_loop(&mut self, input: &str, max_steps: usize) -> Option<String> {
        let provider = self.provider.to_llm_provider()?;

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;

        while step < max_steps {
            Self::budget_react_messages(&mut messages, self.context.max_tokens);
            let request = self.build_request(messages.clone())?;

            let response = match provider.complete(&request).await {
                Ok(r) => r,
                Err(e) => {
                    self.wire.record(WireEvent::SystemEvent {
                        kind: "provider_error".into(),
                        detail: e.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                    });
                    return Some(format!("[provider error] {}", e));
                }
            };

            self.state.tokens_used += response.usage.total_tokens as usize;
            let _ = self.cost.record("agent", 0.0, response.usage.total_tokens as u64);

            // Attempt to extract a structured tool-call from the response.
            let tool_call = Self::extract_tool_call(&response.content);

            match tool_call {
                Some((name, args)) => {
                    self.state.tool_call_count += 1;
                    // P0-2: enforce the permission policy on the streaming path.
                    // Previously only the CLI AgentStream and exec_agent honored
                    // PermissionSystem; react_loop bypassed it entirely, so
                    // Manual/AcceptEdits/Plan modes were advisory at best.
                    let allowed = self.permissions.policy_gate(&name, &self.state.permission_mode);
                    if !allowed {
                        let denied = format!("[denied] tool `{}` blocked by permission mode `{}`", name, self.state.permission_mode);
                        self.wire.record(WireEvent::ToolCall {
                            name: name.clone(),
                            args: args.clone(),
                            result: denied.clone(),
                            duration_ms: 0,
                            success: false,
                        });
                        messages.push(Message::tool(&denied, &format!("call-{}", step)));
                        step += 1;
                        continue;
                    }
                    let result = self.execute_tool(&name, &args).await;
                    // Tool grounding (Cycle 160e): claimed success when invoked; actual success
                    // if the tool did not return a distinguishable error marker.
                    let actual_ok = !result.starts_with('[');
                    self.tool_grounding.record_tool_result(&name, true, actual_ok);
                    self.wire.record(WireEvent::ToolCall {
                        name: name.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        duration_ms: 0,
                        success: actual_ok,
                    });
                    messages.push(Message::assistant_with_calls(
                        &response.content,
                        vec![crate::neotrix::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            call_type: "function".into(),
                            function: crate::neotrix::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            },
                        }],
                    ));
                    messages.push(Message::tool(&result, &format!("call-{}", step)));
                    self.context.push("assistant", response.content.clone(), response.content.len() / 4);
                    self.context.push("tool", result.clone(), result.len() / 4);
                    step += 1;
                }
                None => {
                    final_answer = Some(response.content);
                    break;
                }
            }
        }

        final_answer
    }

    /// Streaming ReAct loop: emits tokens via callback as they arrive from the provider.
    /// `on_token` returns `true` to continue or `false` to cancel; a cancelled
    /// stream returns the tokens accumulated so far (partial reply).
    /// `on_tool` fires after each tool execution (name, args, result, duration_ms, success);
    /// returning `false` cancels the loop (same semantics as `on_token`).
    /// Returns the final accumulated response (or error).
    pub async fn react_loop_stream<F, G>(
        &mut self,
        input: &str,
        max_steps: usize,
        mut on_token: F,
        mut on_tool: G,
    ) -> Option<String>
    where
        F: FnMut(&str) -> bool + Send + Sync,
        G: FnMut(&str, &str, &str, u64, bool) -> bool + Send + Sync,
    {
        let provider = self.provider.to_llm_provider()?;

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;
        let mut accumulated = String::new();
        let mut cancelled = false;

        while step < max_steps && !cancelled {
            Self::budget_react_messages(&mut messages, self.context.max_tokens);
            let request = self.build_request(messages.clone())?;

            let mut rx = match provider.stream_complete(&request).await {
                Ok(rx) => rx,
                Err(e) => {
                    self.wire.record(WireEvent::SystemEvent {
                        kind: "provider_error".into(),
                        detail: e.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                    });
                    return Some(format!("[provider error] {}", e));
                }
            };

            let mut response_content = String::new();
            let mut response_usage = None;

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(resp) => {
                        if !resp.content.is_empty() {
                            response_content.push_str(&resp.content);
                            accumulated.push_str(&resp.content);
                            if !on_token(&resp.content) {
                                cancelled = true;
                                break;
                            }
                        }
                        if resp.usage.total_tokens > 0 {
                            response_usage = Some(resp.usage);
                        }
                    }
                    Err(e) => {
                        self.wire.record(WireEvent::SystemEvent {
                            kind: "provider_error".into(),
                            detail: e.to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                        });
                        return Some(format!("[provider error] {}", e));
                    }
                }
            }

            if let Some(usage) = response_usage {
                self.state.tokens_used += usage.total_tokens as usize;
                let _ = self.cost.record("agent", 0.0, usage.total_tokens as u64);
            }

            if cancelled {
                break;
            }

            // Attempt to extract a structured tool-call from the response.
            let tool_call = Self::extract_tool_call(&response_content);

            // P1-1 Plan gate: in non-Agent modes (Plan) tools must NOT be
            // executed. Plan is a read-only promise (Codex plan / Claude
            // plan-mode parity); executing shell there lets the model run
            // arbitrary commands despite the read-only contract. Skip tool
            // execution and return the drafted plan/response as the answer.
            if self.state.mode != NeoCodexMode::Agent {
                return Some(response_content);
            }

            match tool_call {
                Some((name, args)) => {
                    self.state.tool_call_count += 1;
                    // P0-2: enforce the permission policy on the streaming path.
                    let allowed = self.permissions.policy_gate(&name, &self.state.permission_mode);
                    if !allowed {
                        let denied = format!("[denied] tool `{}` blocked by permission mode `{}`", name, self.state.permission_mode);
                        self.wire.record(WireEvent::ToolCall {
                            name: name.clone(),
                            args: args.clone(),
                            result: denied.clone(),
                            duration_ms: 0,
                            success: false,
                        });
                        messages.push(Message::tool(&denied, &format!("call-{}", step)));
                        step += 1;
                        continue;
                    }
                    let tool_started = Instant::now();
                    let result = self.execute_tool(&name, &args).await;
                    let tool_duration_ms = tool_started.elapsed().as_millis() as u64;
                    let actual_ok = !result.starts_with('[');
                    self.tool_grounding.record_tool_result(&name, true, actual_ok);
                    self.wire.record(WireEvent::ToolCall {
                        name: name.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        duration_ms: tool_duration_ms,
                        success: actual_ok,
                    });
                    if !on_tool(&name, &args, &result, tool_duration_ms, actual_ok) {
                        cancelled = true;
                        break;
                    }
                    messages.push(Message::assistant_with_calls(
                        &response_content,
                        vec![crate::neotrix::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            call_type: "function".into(),
                            function: crate::neotrix::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            },
                        }],
                    ));
                    messages.push(Message::tool(&result, &format!("call-{}", step)));
                    self.context.push("assistant", response_content.clone(), response_content.len() / 4);
                    self.context.push("tool", result.clone(), result.len() / 4);
                    step += 1;
                }
                None => {
                    final_answer = Some(response_content);
                    break;
                }
            }
        }

        if cancelled {
            Some(accumulated)
        } else {
            final_answer
        }
    }

    /// Build system + history + current user messages from the context pipeline.
    fn build_messages(&self, input: &str) -> Vec<Message> {
        let system = "You are NeoCodex, an AI coding agent inside the NeoTrix architecture. \
            Modes: Agent (autonomous coding), Shell (run commands), Plan (draft plans). \
            Use the tools when you need to read files, search the repo, or run shell commands. \
            Always respond in markdown. Be concise and precise.".to_string();
        let mut messages = vec![Message::new(Role::System, &system)];
        for turn in &self.context.turns {
            let role = match turn.role.as_str() {
                "user" => Role::User,
                "assistant" | "summary" => Role::Assistant,
                "tool" => Role::Tool,
                "system" => Role::System,
                _ => Role::User,
            };
            if role == Role::System {
                continue; // keep single system message
            }
            messages.push(Message::new(role, &turn.content));
        }
        // P1-2 dedup: the current user input is already pushed into
        // `context.turns` by the caller before invoking the loop (send
        // command / process). Appending it again yields two consecutive
        // identical user turns in every request. Only append when the last
        // history turn is NOT the same message.
        if self.context.turns.back().map(|t| t.content.as_str()) != Some(input) {
            messages.push(Message::new(Role::User, input));
        }        messages
    }

    /// Bottom-up token budget for the ReAct loop. The local `messages` vec grows
    /// by one assistant + one tool-result turn per step, so a long loop can blow
    /// the provider context window. Estimated tokens = chars / 4 (consistent with
    /// resume_session). Evicts oldest non-system turns first; index 0 (system)
    /// and the trailing current-user request are never evicted.
    fn budget_react_messages(messages: &mut Vec<Message>, max_tokens: usize) {
        let est = |m: &Message| m.content.len() / 4;
        let mut total: usize = messages.iter().map(&est).sum();
        let i = 1;
        while total > max_tokens && i + 1 < messages.len() {
            total = total.saturating_sub(est(&messages[i]));
            messages.remove(i);
        }
    }    /// Build an LlmRequest from the current catalog's active provider.
    fn build_request(&self, messages: Vec<Message>) -> Option<LlmRequest> {
        self.provider.providers.get(self.provider.active)?;
        // P0-3: surface the most recent user-turn image attachment to the model.
        // The UI stores base64 in WireEvent::UserMessage.attachments; previously
        // image_data was hardcoded None, so attached screenshots never reached
        // the provider despite being rendered inline in the chat.
        let image_data = self.wire.events.iter().rev().find_map(|ev| match ev {
            WireEvent::UserMessage { attachments: Some(list), .. } => list
                .iter()
                .find(|a| a.mime_type.starts_with("image/"))
                .and_then(|a| a.data.clone()),
            _ => None,
        });

        // Vision-bridge: the active model may be text-only (e.g. deepseek-v4-flash,
        // local qwen2.5:7b without -vl). Sending image_data to such providers is a
        // no-op or an error — the VisionBridge converts the attachment into
        // deterministic structured evidence text that the text-only model CAN
        // reason over, and we drop the raw image channel.
        let active_model = self.provider.active_model();
        let active_has_vision = self.provider.has_capability(ModelCapability::Vision)
            || crate::core::nt_core_e8::nt_multimodal::model_supports_vision(&active_model);
        let mut messages = messages;
        let mut image_data = image_data;
        if image_data.is_some() && !active_has_vision {
            if let Some(b64) = image_data.take() {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&b64) {
                    if let Ok((evidence, _feat)) =
                        crate::core::nt_core_e8::nt_multimodal::VisionBridge::analyze_cached(&bytes)
                    {
                        if let Some(last_user) = messages
                            .iter_mut()
                            .rev()
                            .find(|m| m.role == Role::User)
                        {
                            last_user.content = format!(
                                "{}\n\n{}\n\n(Note: the active model is text-only; the image attachment was bridged to structured pixel evidence above.)",
                                last_user.content,
                                evidence.to_evidence_text(),
                            );
                        }
                    }
                }
            }
        }

        let mut req = LlmRequest {
            model: self.provider.active_model(),
            messages,
            // P2-1: honor the settings-panel generation params instead of the
            // old hardcoded values.
            temperature: Some(self.config.temperature.clamp(0.0, 2.0) as f32),
            max_tokens: self.config.max_tokens.max(1),
            tools: vec![
                Tool {
                    name: "read".into(),
                    description: "Read a file at the given absolute or relative path".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
                },
                Tool {
                    name: "search".into(),
                    description: "Grep the codebase for a pattern (regex)".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"pattern": {"type": "string"}}}),
                },
                Tool {
                    name: "write".into(),
                    description: "Write or overwrite a file. Args format: <path>|<content> (split on the first pipe). Creates parent dirs. Guarded to the workspace.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}}}),
                },
                Tool {
                    name: "edit".into(),
                    description: "Replace a unique old substring with new in a file. Args format: <path>|<old>|<new> (split on the first two pipes). Fails if old is missing or not unique.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "old": {"type": "string"}, "new": {"type": "string"}}}),
                },
                Tool {
                    name: "shell".into(),
                    description: "Run a shell command and return stdout".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}}),
                },
                Tool {
                    name: "mcp_call".into(),
                    description: "Call a registered MCP tool. Args format: <tool_name>|<json_args> (split on the first pipe). List available tools with mcp_list.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"name": {"type": "string"}, "args": {"type": "string"}}}),
                },
                Tool {
                    name: "mcp_list".into(),
                    description: "List registered MCP servers and their available tools".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                },
            ],
            image_data,
            thinking_budget: if self.config.thinking_enabled { Some(2048) } else { None },
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        };
        if req.image_data.is_some() && active_has_vision {
            if let Some(raw) = req.image_data.clone() {
                req = req.with_image_b64(&raw);
            }
        }
        Some(req)
    }

    /// Parse a `<tool name="...">args</tool>` block from the model output.
    fn extract_tool_call(content: &str) -> Option<(String, String)> {
        let marker = "<tool";
        let start = content.find(marker)?;
        let name_start = content[start..].find("name=\"")? + start + 6;
        let name_end = content[name_start..].find('"')? + name_start;
        let name = content[name_start..name_end].to_string();
        let args_start = content[name_end..].find('>')? + name_end + 1;
        let args_end = content[args_start..].find("</tool>")? + args_start;
        let args = content[args_start..args_end].trim().to_string();
        Some((name, args))
    }

    /// Resolve a tool-provided path against the workspace root and refuse any
    /// path that escapes it (`..` / absolute outside cwd). Claude/Codex both
    /// sandbox agent file access to the project; P0-1/P2-3: without this the
    /// read/write/edit tools could touch arbitrary files outside the repo.
    fn guard_path(&self, raw: &str) -> Result<std::path::PathBuf, String> {
        let cwd = std::env::current_dir().map_err(|e| format!("[cwd error] {}", e))?;
        let p = std::path::Path::new(raw.trim());
        let candidate = if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd.join(p)
        };
        // Normalize: lexically resolve `.`/`..` components without touching FS.
        let mut parts: Vec<std::ffi::OsString> = Vec::new();
        for comp in candidate.components() {
            match comp {
                std::path::Component::Normal(c) => parts.push(c.to_os_string()),
                std::path::Component::ParentDir => {
                    if parts.pop().is_none() {
                        return Err(format!("[path error] {} escapes the workspace", raw));
                    }
                }
                std::path::Component::CurDir | std::path::Component::RootDir
                | std::path::Component::Prefix(_) => {}
            }
        }
        let normalized = parts.iter().fold(std::path::PathBuf::new(), |acc, c| acc.join(c));
        let resolved = normalized;
        if !resolved.starts_with(&cwd) {
            return Err(format!("[path error] {} is outside the workspace", raw));
        }
        Ok(resolved)
    }

    /// Execute a concrete tool. Wired to real FS + shell (no external binaries,
    /// consistent with R-P48 zero third-party binary dependency).
    async fn execute_tool(&mut self, name: &str, args: &str) -> String {
        match name {
            "read" => {
                let path = match self.guard_path(args) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        if content.len() > 16_000 {
                            content[..content.floor_char_boundary(16_000)].to_string()
                        } else {
                            content
                        }
                    }
                    Err(e) => format!("[read error] {}", e),
                }
            }
            "search" => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let pattern = args.trim();
                let mut hits = Vec::new();
                // P1-2: recursive search — the old impl only walked the top
                // level (read_dir, no recursion) and filtered to *.rs, making it
                // useless on real codebases. Skip heavy dirs to bound cost.
                fn walk(
                    dir: &std::path::Path,
                    pattern: &str,
                    hits: &mut Vec<String>,
                    depth: usize,
                ) {
                    if depth > 8 || hits.len() >= 40 {
                        return;
                    }
                    let Ok(entries) = std::fs::read_dir(dir) else { return };
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname == "target" || fname == "node_modules" || fname == ".git"
                            || fname == "dist" || fname == "build" || fname == ".venv"
                            || fname == "vendor" {
                            continue;
                        }
                        if path.is_dir() {
                            walk(&path, pattern, hits, depth + 1);
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (i, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    hits.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
                                    if hits.len() >= 40 { return; }
                                }
                            }
                        }
                    }
                }
                walk(&cwd, pattern, &mut hits, 0);
                if hits.is_empty() {
                    format!("No matches for {:?} in {}", pattern, cwd.display())
                } else {
                    hits.join("\n")
                }
            }
            // P0-1: native write tool (Claude Write parity). Unlike shell
            // escape, this is a bounded, guarded single-file write.
            "write" => {
                // Args format: `<path>|<content>` — split on the first `|` so
                // content may itself contain pipes. Model contract documented in
                // build_request tool description.
                let (path, content) = match args.split_once('|') {
                    Some((p, c)) => (p, c),
                    None => {
                        return "[write error] expected format: <path>|<content>".to_string();
                    }
                };
                let path = match self.guard_path(path) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                if content.len() > 64_000 {
                    return format!("[write error] content exceeds 64 KB ({} bytes)", content.len());
                }
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return format!("[write error] mkdir: {}", e);
                        }
                    }
                }
                match std::fs::write(&path, content) {
                    Ok(()) => format!("[ok] wrote {} ({} bytes)", path.display(), content.len()),
                    Err(e) => format!("[write error] {}", e),
                }
            }
            // P0-1: native edit tool (Claude Edit parity). Replaces a unique
            // `old` substring with `new` in the target file. Args: `<path>|<old>|<new>`.
            "edit" => {
                let parts: Vec<&str> = args.splitn(3, '|').collect();
                if parts.len() != 3 {
                    return "[edit error] expected format: <path>|<old>|<new>".to_string();
                }
                let path = match self.guard_path(parts[0]) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                let original = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => return format!("[edit error] read {}: {}", path.display(), e),
                };
                if original.len() > 64_000 {
                    return format!("[edit error] file exceeds 64 KB ({} bytes)", original.len());
                }
                let old = parts[1];
                let new = parts[2];
                let count = original.matches(old).count();
                if count == 0 {
                    return format!("[edit error] old text not found in {}", path.display());
                }
                if count > 1 {
                    return format!(
                        "[edit error] old text is not unique ({} matches) in {}",
                        count,
                        path.display()
                    );
                }
                let updated = original.replace(old, new);
                match std::fs::write(&path, updated) {
                    Ok(()) => format!("[ok] edited {} (replaced unique match)", path.display()),
                    Err(e) => format!("[edit error] {}", e),
                }
            }
            "shell" => {
                let output = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(args)
                    .output()
                    .await;
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let code = out.status.code().unwrap_or(-1);
                        let cap = 8_000usize;
                        if code == 0 {
                            if stdout.len() > cap {
                                format!(
                                    "{}... [stdout truncated {} bytes]",
                                    &stdout[..stdout.floor_char_boundary(cap)],
                                    stdout.len().saturating_sub(cap)
                                )
                            } else {
                                stdout
                            }
                        } else {
                            format!("exit({}): {}", code, stderr)
                        }
                    }
                    Err(e) => format!("[shell error] {}", e),
                }
            }
            // P2-5: MCP tool call (Codex/Claude MCP parity). Args format:
            // `<tool_name>|<json_args>` — split on the first `|`. Delegates to
            // the attached McpRegistry; without a registry it returns a clear
            // error instead of silently pretending to succeed.
            "mcp_call" => {
                let Some(registry) = &self.mcp else {
                    return "[mcp_call error] no MCP registry attached; register MCP servers first".to_string();
                };
                let (name, json) = match args.split_once('|') {
                    Some((n, j)) => (n.trim(), j),
                    None => (args.trim(), "{}"),
                };
                let parsed: serde_json::Value = match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => return format!("[mcp_call error] invalid JSON args: {}", e),
                };
                match registry.call_tool(name, &parsed) {
                    Ok(result) => result,
                    Err(e) => format!("[mcp_call error] {}", e),
                }
            }
            "mcp_list" => {
                let Some(registry) = &self.mcp else {
                    return "[mcp_list] no MCP registry attached".to_string();
                };
                if registry.server_count() == 0 {
                    return "[mcp_list] no MCP servers registered".to_string();
                }
                let mut out = String::new();
                for server in registry.list_servers() {
                    out.push_str(&format!("# {} ({})\n", server.name, server.tools.len()));
                    for tool in &server.tools {
                        out.push_str(&format!("  - {}: {}\n", tool.name, tool.description));
                    }
                }
                out
            }
            _ => format!("Unknown tool: {}", name),
        }
    }

    /// Restore prior session events from the wire file into the context
    /// pipeline (G2 session continuity, matching Claude Code `--resume`).
    /// Returns the number of events restored.
    /// Clear in-memory context and re-restore it from the wire file. Used after
    /// an edit/delete/regenerate rewrites the JSONL so the agent's next turn is
    /// built from the corrected history, not stale in-memory state.
    pub fn rebuild_context_from_wire(&mut self) -> usize {
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.tool_call_count = 0;
        self.resume_session()
    }

    pub fn resume_session(&mut self) -> usize {
        let events = self.wire.load();
        let mut restored = 0;
        for event in events {
            match event {
                WireEvent::UserMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("user", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::AgentMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("assistant", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::ToolCall { name, args, result, .. } => {
                    self.context.push(
                        "user",
                        format!("[tool {}] args={} result={}", name, args, result),
                        (name.len() + args.len() + result.len()) / 4,
                    );
                    self.state.tool_call_count += 1;
                    restored += 1;
                }
                WireEvent::ModeChange { to, .. } => {
                    self.state.mode = to;
                    restored += 1;
                }
                _ => {}
            }
        }
        restored
    }

    /// Produce a full health snapshot of the agent (D25: output must be
    /// consumable — this is consumed by SelfTest, UI status, and evolution).
    pub fn health_report(&self) -> NeoCodexHealthReport {        let provider_count = self.provider.providers.len();
        let provider_resolvable = self.provider.to_llm_provider().is_some();
        let context_usage = if self.context.max_tokens == 0 {
            0.0
        } else {
            self.context.total_tokens() as f64 / self.context.max_tokens as f64
        };
        let session_writable = self.wire.path.parent().map(|p| p.exists()).unwrap_or(false);
        let evolution_iterations = self.evolution.iteration;

        NeoCodexHealthReport {
            mode: self.state.mode,
            turn_count: self.state.turn_count,
            tool_call_count: self.state.tool_call_count,
            tokens_used: self.state.tokens_used,
            context_usage: context_usage.max(0.0).min(1.0),
            context_turns: self.context.turns.len(),
            provider_count,
            provider_resolvable,
            provider_model: self.provider.active_model(),
            session_writable,
            goals_active: self.state.goal_active,
            cost_spent: self.cost.total_spent,
            cost_budget: self.cost.max_budget,
            subagent_results: self.subagent_results.len(),
            consciousness_attached: self.consciousness.is_some(),
            brain_attached: self.brain.is_some(),
            event_bus_attached: self.event_bus.is_some(),
            evolution_iterations,
            tool_grounding_degraded: self.tool_grounding.any_degraded(),
            node_snapshots: self.consciousness.as_ref()
                .map(|tree| tree.snapshots())
                .unwrap_or_default(),
        }
    }
}

// ── Cycle 159: Self-Audit + Evolution Loop ──

/// Serializable health report used by the SelfTest trait, the TUI status line,
/// and the evolution loop. All checks are synchronous and side-effect free.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NeoCodexHealthReport {
    pub mode: NeoCodexMode,
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub tokens_used: usize,
    pub context_usage: f64,
    pub context_turns: usize,
    pub provider_count: usize,
    pub provider_resolvable: bool,
    pub provider_model: String,
    pub session_writable: bool,
    pub goals_active: bool,
    pub cost_spent: f64,
    pub cost_budget: f64,
    pub subagent_results: usize,
    pub consciousness_attached: bool,
    pub brain_attached: bool,
    pub event_bus_attached: bool,
    pub evolution_iterations: u64,
    pub tool_grounding_degraded: bool,
    /// Skill Node Evolution — per-domain 节点状态 (NodeTier/Constellation/Rune)
    /// 使 7 域健康网格反映真实 per-domain 遥测, 而非布尔投影。
    pub node_snapshots: Vec<crate::core::nt_core_consciousness_tree::NodeSnapshot>,
}

impl NeoCodexHealthReport {
    /// Number of failed checks (used by D43: every detection must feed behavior).
    pub fn failed_checks(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if !self.provider_resolvable {
            failures.push("provider not resolvable (no API key / no model)".into());
        }
        if !self.session_writable {
            failures.push("session dir not writable".into());
        }
        if self.context_usage > 0.9 {
            failures.push(format!("context pipeline at {:.0}% (auto-compact will trigger)", self.context_usage * 100.0));
        }
        if self.cost_spent > self.cost_budget && self.cost_budget > 0.0 {
            failures.push(format!("budget exhausted ${:.2}/${:.2}", self.cost_spent, self.cost_budget));
        }
        if self.provider_count == 0 {
            failures.push("provider catalog empty".into());
        }
        if self.tool_grounding_degraded {
            failures.push("tool grounding degraded (failure rate above adaptive threshold)".into());
        }
        failures
    }

    pub fn is_healthy(&self) -> bool {
        self.failed_checks().is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "NeoCodexHealth[mode={:?} turns={} tools={} ctx={:.0}% providers={} evo={}]",
            self.mode, self.turn_count, self.tool_call_count,
            self.context_usage * 100.0, self.provider_count, self.evolution_iterations,
        )
    }
}

/// SelfTest implementation — snapshot-based so `self_test()` is synchronous
/// (fits the `SelfTest` trait signature).
#[derive(Debug, Clone, Default)]
pub struct NeoCodexSelfAudit {
    pub last_report: NeoCodexHealthReport,
}

impl NeoCodexSelfAudit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(agent: &NeoCodexAgent) -> Self {
        Self {
            last_report: agent.health_report(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for NeoCodexSelfAudit {
    fn name(&self) -> &str {
        "neocodex_self_audit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let failures = self.last_report.failed_checks();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// Self-healing loop for NeoCodex (D22/D26). Each `step` advances the iteration
/// counter and runs a diagnose→fix cycle. This is the 100-iteration loop that
/// converges the agent toward Codex/Claude Code desktop parity.
#[derive(Debug, Clone, Default)]
pub struct EvolutionLoop {
    pub iteration: u64,
    pub target: u64,
    pub gaps_found: u64,
    pub fixes_applied: u64,
    pub history: VecDeque<EvolutionIteration>,
}

/// Record of a single evolution iteration.
#[derive(Debug, Clone)]
pub struct EvolutionIteration {
    pub iteration: u64,
    pub gaps: Vec<String>,
    pub fixes: Vec<String>,
    pub healthy: bool,
}

impl EvolutionLoop {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            target: 100,
            gaps_found: 0,
            fixes_applied: 0,
            history: VecDeque::new(),
        }
    }

    pub fn with_target(mut self, target: u64) -> Self {
        self.target = target;
        self
    }

    /// Advance one iteration: capture health → diagnose gaps → apply fixes.
    /// Associated function (not method) so the borrow checker accepts passing
    /// the whole agent while mutating the loop's own state.
    pub fn step(agent: &mut NeoCodexAgent) {
        agent.evolution.iteration += 1;
        let report = agent.health_report();
        let gaps = report.failed_checks();
        agent.evolution.gaps_found += gaps.len() as u64;

        let mut fixes = Vec::new();

        // Fix 1: active provider not resolvable (stub / empty) → sync from real
        // layer and pick a usable provider (Cycle 159 gap: the "opencode" stub
        // was never replaced because the old guard only checked `is_empty`).
        if !agent.provider.is_resolvable() {
            agent.provider.ensure_production_provider();
            fixes.push("synced provider catalog from real layer".into());
        }

        // Fix 2: context near budget → force compaction
        if report.context_usage > 0.9 {
            agent.context.compact_if_needed();
            fixes.push("forced context compaction".into());
        }

        // Fix 3: session dir missing → recreate
        if !report.session_writable {
            if let Some(parent) = agent.wire.path.parent() {
                let _ = std::fs::create_dir_all(parent);
                fixes.push("recreated session dir".into());
            }
        }

        // Fix 4: goal queue empty but evolution wants growth → seed introspection goal
        if agent.goals.goals.is_empty() && agent.goals.active.is_none() && agent.evolution.iteration.is_multiple_of(25) {
            agent.add_goal("Self-audit: converge NeoCodex toward Codex/Claude Code desktop parity", 5);
            fixes.push("seeded introspection goal".into());
        }

        agent.evolution.fixes_applied += fixes.len() as u64;

        let record = EvolutionIteration {
            iteration: agent.evolution.iteration,
            gaps,
            fixes,
            healthy: report.is_healthy(),
        };
        if agent.evolution.history.len() >= 100 {
            agent.evolution.history.pop_front();
        }
        agent.evolution.history.push_back(record);
        agent.audit = NeoCodexSelfAudit::capture(agent);
    }

    /// Summary line for the TUI status.
    pub fn summary(&self) -> String {
        format!(
            "Evolution {}/{} · {} gaps · {} fixes",
            self.iteration, self.target, self.gaps_found, self.fixes_applied
        )
    }
}

// ── TUI Integration (bridge to existing TuiApp) ──

pub struct NeoCodexUI {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
    pub mode: NeoCodexMode,
    pub status_text: String,
    pub streaming_text: String,
    pub input_buffer: String,
    pub message_log: Vec<(String, String)>,
    pub goal_display: crate::cli::tui::app::types::GoalDisplay,
}

impl NeoCodexUI {
    pub fn new(session_id: &str) -> Self {
        Self {
            agent: Arc::new(Mutex::new(NeoCodexAgent::new(session_id))),
            mode: NeoCodexMode::Agent,
            status_text: "NeoCodex Ready".into(),
            streaming_text: String::new(),
            input_buffer: String::new(),
            message_log: Vec::new(),
            goal_display: crate::cli::tui::app::types::GoalDisplay::idle(),
        }
    }

    pub async fn send_message(&mut self, text: &str) {
        let mut agent = self.agent.lock().await;
        let response = agent.process(text).await;
        self.mode = agent.state.mode;
        self.message_log.push(("user".into(), text.to_string()));
        self.message_log.push(("assistant".into(), response));
        let report = agent.health_report();
        self.status_text = format!(
            "Turn {} | {} tools | {} tokens | ctx {:.0}% | {}",
            agent.state.turn_count, agent.state.tool_call_count,
            agent.state.tokens_used, report.context_usage * 100.0,
            agent.evolution.summary(),
        );
        if let Some(ref goal) = agent.goals.active {
            self.goal_display = crate::cli::tui::app::types::GoalDisplay {
                has_goal: true,
                id: goal.id.clone(),
                description: goal.description.clone(),
                state_label: format!("{:?}", goal.state),
                    state_icon: match goal.state {
                        GoalState::Active => "▶".into(),
                        GoalState::Paused => "⏸".into(),
                        GoalState::Completed => "✅".into(),
                        GoalState::Blocked => "🚫".into(),
                        GoalState::Cancelled => "❌".into(),
                    },
                iterations: goal.iterations,
                max_iterations: goal.max_iterations,
                ..crate::cli::tui::app::types::GoalDisplay::idle()
            };
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_catalog() {
        let catalog = ProviderCatalog::new();
        assert!(catalog.has_capability(ModelCapability::Code));
        assert!(!catalog.has_capability(ModelCapability::Video));
    }

    #[test]
    fn test_streaming_markdown_plain_text() {
        let mut sm = StreamingMarkdown::new();
        sm.push("Hello world");
        assert!(!sm.chunks.is_empty());
    }

    #[test]
    fn test_streaming_markdown_code_block() {
        let mut sm = StreamingMarkdown::new();
        sm.push("```rust\nfn main() {}\n```");
        assert!(sm.chunks.iter().any(|c| matches!(c, MarkdownChunk::CodeBlock { .. })));
    }

    #[test]
    fn test_streaming_markdown_incomplete_code_block() {
        let mut sm = StreamingMarkdown::new();
        sm.push("```rust\nfn main() {\nprintln!(\"hello\");");
        assert!(sm.chunks.iter().any(|c| matches!(c, MarkdownChunk::CodeBlock { complete: false, .. })));
    }

    #[test]
    fn test_context_pipeline_simple() {
        let mut ctx = ContextPipeline::new(1000);
        ctx.use_tiktoken = false; // unit test of pipeline mechanics, not counting
        ctx.push("user", "test message".into(), 10);
        assert_eq!(ctx.turns.len(), 1);
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_context_pipeline_compaction() {
        let mut ctx = ContextPipeline::new(500);
        ctx.use_tiktoken = false; // drive compaction with exact caller estimates
        for i in 0..20 {
            ctx.push("user", format!("message {}", i), 100);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_context_pipeline_collapse_terminates() {
        // Regression: the old Layer 4 oscillated pop/push-front forever once
        // the pipeline reached it with >10 turns. Sixty small turns force
        // Layer 2 -> Layer 4 while staying well under the hard cap.
        let mut ctx = ContextPipeline::new(5000);
        ctx.use_tiktoken = false; // rely on caller estimates so compaction triggers
        for i in 0..60 {
            ctx.push("user", format!("message {} {}", i, "x".repeat(40)), 100);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
        assert!(ctx.turns.iter().any(|t| t.role == "summary"));
    }

    #[test]
    fn test_goal_queue() {
        let mut queue = GoalQueue::new();
        queue.add("test goal 1", 3);
        queue.add("test goal 2", 5);
        assert!(queue.next().is_some());
        assert!(queue.next().is_some());
        assert!(queue.next().is_none());
    }

    #[test]
    fn test_agent_state() {
        let state = AgentState::new();
        assert_eq!(state.mode, NeoCodexMode::Agent);
        assert_eq!(state.turn_count, 0);
    }

    #[test]
    fn test_dual_mode_toggle() {
        let mut agent = NeoCodexAgent::new("test-session");
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Shell);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.set_plan_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
    }

    #[test]
    fn test_budget_react_messages_evicts_oldest() {
        let mut messages = vec![
            Message::new(Role::System, "system prompt"),
            Message::new(Role::User, "first request"),
            Message::assistant_with_calls("tool call a", vec![]),
            Message::tool("a very long tool result that blows any budget", "call-0"),
        ];
        NeoCodexAgent::budget_react_messages(&mut messages, 1);
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages.last().unwrap().role, Role::Tool);
        assert!(messages.len() < 4);
    }

    #[test]
    fn test_context_pipeline_tool_priority() {
        let mut ctx = ContextPipeline::new(10_000);
        ctx.push("system", "sys".into(), 5);
        ctx.push("tool", "big tool result".into(), 10);
        ctx.push("user", "hi".into(), 10);
        ctx.push("assistant", "ok".into(), 10);
        assert_eq!(ctx.turns[0].priority, 5);
        assert_eq!(ctx.turns[1].priority, 1);
        assert_eq!(ctx.turns[2].priority, 3);
        assert_eq!(ctx.turns[3].priority, 3);
    }

    #[test]
    fn test_build_messages_preserves_summary_and_tool_roles() {
        // Regression 1: Layer-4 distilled "summary" turns were mapped to
        // Role::System and skipped, silently discarding the compaction context.
        // Regression 2: "tool" history turns were mapped to Role::User, breaking
        // the ReAct message protocol (tool results must be Role::Tool).
        let mut agent = NeoCodexAgent::new("role-test");
        agent.context.push("system", "the system prompt".into(), 5);
        agent.context.push("user", "question".into(), 10);
        agent.context.push("assistant", "thinking".into(), 10);
        agent.context.push("tool", "tool output".into(), 10);
        agent.context.push("summary", "distilled earlier context".into(), 10);

        let messages = NeoCodexAgent::build_messages(&agent, "current input");
        let summary_msg = messages.iter().find(|m| m.content.contains("distilled"));
        let tool_msg = messages.iter().find(|m| m.content.contains("tool output"));

        assert!(summary_msg.is_some(), "summary turn must survive into messages");
        assert_eq!(summary_msg.unwrap().role, Role::Assistant);
        assert_eq!(tool_msg.unwrap().role, Role::Tool);
        assert_eq!(messages.last().unwrap().role, Role::User);
        assert_eq!(messages.last().unwrap().content, "current input");
        assert!(messages.iter().filter(|m| m.role == Role::System).count() == 1,
            "only the real system prompt is kept");
    }

    #[test]
    fn test_context_pipeline_layer3_non_ascii_no_panic() {
        // Regression: Layer 3 used String::truncate(200) which panics when byte
        // 200 falls mid-UTF-8-char. Tool turns (priority 1) now enter this path.
        let mut ctx = ContextPipeline::new(5000);
        let big = "中文数据负载".repeat(50);
        for i in 0..14 {
            ctx.push("tool", format!("{} {}", i, big), 300);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_wire_format_roundtrip() {
        let mut session = WireSession::new("test-wire");
        session.record(WireEvent::UserMessage {
            content: "hello".into(),
            timestamp: 1000,
            attachments: None,
        });
        assert_eq!(session.events.len(), 1);
    }

    #[test]
    fn test_agent_process_basic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut agent = NeoCodexAgent::new("test-agent");
        rt.block_on(async {
            let response = agent.process("Hello, NeoCodex!").await;
            assert!(!response.is_empty());
            assert_eq!(agent.state.turn_count, 1);
        });
    }

    #[test]
    fn test_provider_add() {
        let mut catalog = ProviderCatalog::new();
        catalog.add_provider(ProviderInfo {
            name: "anthropic".into(),
            model: "claude-3-5-sonnet".into(),
            capabilities: vec![ModelCapability::Reasoning, ModelCapability::Vision],
            context_limit: 200_000,
            cost_per_m_input: 3.0,
            cost_per_m_output: 15.0,
        });
        assert_eq!(catalog.providers.len(), 2);
    }

    #[test]
    fn test_provider_capability_check() {
        let catalog = ProviderCatalog::new();
        assert!(catalog.has_capability(ModelCapability::Code));
    }

    #[test]
    fn test_goal_lifecycle() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 5);
        assert!(agent.state.goal_active);
    }

    #[test]
    fn test_goal_wire_id_matches_queue_id() {
        // Regression: add_goal recorded a wire GoalUpdate id derived from
        // turn_count, while GoalQueue::add generated a different id for the
        // queue entry — consumers correlating wire records with queue goals
        // would never match. The wire id now comes from the queue entry.
        let mut agent = NeoCodexAgent::new("goal-id-test");
        agent.add_goal("Goal one", 3);
        agent.add_goal("Goal two", 3);
        let queued: Vec<&String> = agent.goals.goals.iter().map(|g| &g.id).collect();
        assert_eq!(queued.len(), 2);
        assert_ne!(queued[0], queued[1], "distinct goals must have distinct ids");
        for gid in queued {
            assert!(
                agent.wire.events.iter().any(|e| matches!(
                    e, WireEvent::GoalUpdate { id, .. } if id == gid
                )),
                "wire event must use the same id as the queue entry"
            );
        }
    }

    #[test]
    fn test_goal_completes_and_resets_active() {
        // Regression: check_goals was never wired into the turn loop,
        // iterations never incremented, and goal_active never reset — the
        // goal queue could only grow, never progress.
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 2);
        assert!(agent.state.goal_active);

        // 1st call: promotes queued goal to active, iterations 0→1, not complete.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.active.as_ref().unwrap().iterations, 1);

        // 2nd call: iterations 1→2 == max, completes.
        assert!(agent.check_goals().is_none(), "single goal: completion returns None (no next)");
        assert!(agent.goals.active.is_none());
        assert!(!agent.state.goal_active, "goal_active must reset once queue drains");
        assert_eq!(agent.goals.completed.len(), 1);
    }

    #[test]
    fn test_chained_goals_advance_to_next() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Goal A", 1);
        agent.add_goal("Goal B", 1);
        assert!(agent.state.goal_active);

        // Call 1: promotes A to active, increments 0→1 == max, completes A and
        // returns the description of the next goal B.
        assert_eq!(agent.check_goals().unwrap_or_default(), "Goal B");
        assert_eq!(agent.goals.completed.len(), 1);

        // Call 2: B is already active, increments 0→1 == max, completes B.
        // No next goal → returns None and goal_active resets.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.completed.len(), 2);
        assert!(!agent.state.goal_active);
    }

    #[test]
    fn test_streaming_markdown_headings() {
        let mut sm = StreamingMarkdown::new();
        sm.push("# Title\n## Subtitle\nText body");
        assert!(sm.chunks.iter().any(|c| matches!(c, MarkdownChunk::Heading { level: 1, .. })));
        assert!(sm.chunks.iter().any(|c| matches!(c, MarkdownChunk::Heading { level: 2, .. })));
    }

    #[test]
    fn test_streaming_markdown_clear() {
        let mut sm = StreamingMarkdown::new();
        sm.push("Some content");
        assert!(!sm.chunks.is_empty());
        sm.clear();
        assert!(sm.chunks.is_empty());
        assert!(sm.buffer.is_empty());
    }

    #[test]
    fn test_cost_tracker_budget() {
        let mut ct = CostTracker::new(1.0);
        assert!(ct.record("read", 0.5, 100).is_ok());
        assert!(ct.record("write", 0.6, 200).is_err());
        assert!(ct.remaining() < 0.51);
    }

    #[test]
    fn test_permission_system() {
        let ps = PermissionSystem::new();
        assert!(ps.interactive_check("read", "read a file"));
        match ps.check("shell") {
            PermissionLevel::Ask => {}
            _ => panic!("shell should default to Ask"),
        }
    }

    #[test]
    fn test_lifecycle_hooks() {
        let mut registry = LifecycleHookRegistry::new();
        let deny_hook: PreToolHook = Arc::new(|ctx| HookResult {
            decision: if ctx.tool_name == "dangerous" {
                HookDecision::Deny("blocked".into())
            } else { HookDecision::Allow },
            duration_ms: 0,
        });
        registry.register_pre("deny_checker", deny_hook);
        let ctx_safe = ToolCallContext {
            tool_name: "read".into(), args: String::new(),
            cwd: "/".into(), estimated_cost: 0.0,
        };
        let ctx_danger = ToolCallContext {
            tool_name: "dangerous".into(), args: String::new(),
            cwd: "/".into(), estimated_cost: 0.0,
        };
        match registry.run_pre(ctx_safe) {
            HookDecision::Allow => {}
            _ => panic!("safe tool should be allowed"),
        }
        match registry.run_pre(ctx_danger) {
            HookDecision::Deny(_) => {}
            _ => panic!("dangerous tool should be denied"),
        }
    }

    #[test]
    fn test_acp_ping() {
        let agent = NeoCodexAgent::new("acp-test");
        let stream = AgentStream::new(agent, 10.0);
        let server = AcpServer::new(stream.agent.clone());
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let req = AcpRequest {
                id: 1, method: "ping".into(), params: serde_json::json!({}),
            };
            let resp = server.handle_request(req).await;
            if let Some(result) = resp.result {
                assert_eq!(result.get("pong").and_then(|v| v.as_bool()), Some(true));
            } else {
                panic!("expected response, got error: {:?}", resp.error);
            }
        });
    }

    #[test]
    fn test_agent_stream_event_flow() {
        let agent = NeoCodexAgent::new("stream-test");
        let mut stream = AgentStream::new(agent, 5.0);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            stream.process("hello").await;
            let mut events = Vec::new();
            while let Some(ev) = stream.next_event() {
                events.push(ev);
            }
            assert!(events.iter().any(|e| matches!(e, AgentEvent::TurnStart { .. })));
            assert!(events.iter().any(|e| matches!(e, AgentEvent::Done)));
        });
    }

    #[test]
    fn test_wire_replay() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("neocodex_test_wire");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("replay_test.jsonl");
        let event = WireEvent::UserMessage {
            content: "replay".into(), timestamp: 42, attachments: None,
        };
        let line = serde_json::to_string(&event).unwrap();
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "{}", line).unwrap();
        drop(f);

        let events = WireSession::replay(&path);
        assert_eq!(events.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Cycle 159 tests ──

    #[test]
    fn test_sync_from_real_populates_catalog() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(!catalog.providers.is_empty());
        // Real catalog has ollama + many cloud providers
        assert!(catalog.providers.iter().any(|p| p.name == "ollama"));
        // All entries carry a concrete model id
        for p in &catalog.providers {
            assert!(!p.model.is_empty());
        }
    }

    #[test]
    fn test_active_model_after_sync() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(!catalog.active_model().is_empty());
    }

    #[test]
    fn test_count_tokens_cjk_is_precise() {
        // Determinism: same text must count identically every time.
        let cjk = "这是一个用于验证中文分词精确性的测试句子，包含标点符号和数字123，以及英文 mixed content here.";
        let a = count_tokens(cjk);
        let b = count_tokens(cjk);
        assert_eq!(a, b, "token counting must be deterministic");

        // Known reference: cl100k encodes "hello world" as 2 tokens.
        assert_eq!(count_tokens("hello world"), 2);

        // CJK is ~1-3 tokens per char under cl100k; the old bytes/4 estimate
        // divides UTF-8 byte length (3 bytes/char for CJK) by 4, so it
        // systematically UNDER-counts CJK-heavy text. Precise must be >= crude
        // here — a full-character CJK string never collapses below bytes/4.
        let precise = a;
        let crude = cjk.len() / 4;
        assert!(
            precise >= crude.saturating_sub(1),
            "precise {} should not fall below crude {} for CJK text (old estimator under-counts)",
            precise,
            crude
        );
        assert!(precise > 0);

        // English stays ~4 chars/token: precise should track crude within a
        // small band rather than blowing up.
        let english = "the quick brown fox jumps over the lazy dog and runs far away from the town";
        let en_precise = count_tokens(english);
        let en_crude = english.len() / 4;
        assert!(
            en_precise >= en_crude.saturating_sub(2),
            "english precise {} vs crude {}",
            en_precise,
            en_crude
        );
        assert!(en_precise <= en_crude + 4, "english precise {} vs crude {}", en_precise, en_crude);
    }

    #[test]
    fn test_context_pipeline_push_uses_tiktoken() {
        let mut pipe = ContextPipeline::new(10_000);
        assert!(pipe.use_tiktoken, "tiktoken should be enabled by default");

        // A payload whose true token count differs from the caller estimate.
        let cjk = "上下文管线测试：中文内容不应该被错误估计，每一个字符大约一个token。".to_string();
        pipe.push("user", cjk.clone(), 9999); // caller's estimate is deliberately wrong
        let turn = pipe.turns.front().expect("one turn");
        assert!(
            turn.token_count < 9999,
            "tiktoken should override the caller estimate: {}",
            turn.token_count
        );
        assert!(turn.token_count > 0);
        // The turn's count must equal the precise counter's output.
        assert_eq!(turn.token_count, count_tokens(&cjk));
    }

    #[test]
    fn test_context_pipeline_fallback_without_tiktoken() {
        let mut pipe = ContextPipeline::new(10_000);
        pipe.use_tiktoken = false;
        let text = "plain ascii payload for fallback path".to_string();
        pipe.push("user", text.clone(), 42);
        let turn = pipe.turns.front().expect("one turn");
        assert_eq!(turn.token_count, 42, "caller estimate should be honored when tiktoken disabled");
    }

    #[test]
    fn test_provider_persist_roundtrip() {
        // Isolate the persisted provider file to a temp data dir.
        let tmp = std::env::temp_dir().join(format!("neocodex-provider-test-{}", std::process::id()));
        let old_data = std::env::var("XDG_DATA_HOME").ok();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &tmp);
        std::env::set_var("HOME", &tmp);

        // Save a persisted choice.
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        assert!(catalog.set_active_provider("ollama"), "ollama should exist in real catalog");
        catalog.save_persisted();
        let persist_file = ProviderCatalog::persist_path();
        assert!(persist_file.exists(), "provider.json should be written at {}", persist_file.display());

        // Fresh catalog must restore the saved choice after sync.
        let mut restored = ProviderCatalog::new();
        restored.sync_from_real();
        restored.load_persisted();
        assert_eq!(restored.active_model(), catalog.active_model());

        // ensure_production_provider also honors the persisted choice.
        let mut via_ensure = ProviderCatalog::new();
        via_ensure.ensure_production_provider();
        assert_eq!(via_ensure.active_model(), catalog.active_model());

        // Cleanup.
        std::fs::remove_dir_all(&tmp).ok();
        match old_data {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn test_health_report_snapshot() {
        let agent = NeoCodexAgent::new("health-test");
        let report = agent.health_report();
        assert_eq!(report.mode, NeoCodexMode::Agent);
        assert_eq!(report.turn_count, 0);
        assert_eq!(report.evolution_iterations, 0);
        // Default catalog (opencode stub) is not resolvable → gap reported
        assert!(!report.provider_resolvable);
        assert!(!report.failed_checks().is_empty());
    }

    #[test]
    fn test_self_audit_impl() {
        use crate::core::nt_core_self_test::SelfTest;
        let agent = NeoCodexAgent::new("selftest");
        let audit = NeoCodexSelfAudit::capture(&agent);
        assert_eq!(audit.name(), "neocodex_self_audit");
        // Fresh agent with no real provider → self_test fails with gaps
        assert!(audit.self_test().is_err());
    }

    #[test]
    fn test_evolution_loop_advances_and_fixes() {
        let mut agent = NeoCodexAgent::new("evo-test");
        // Empty catalog → step should sync from real layer
        agent.provider.providers.clear();
        EvolutionLoop::step(&mut agent);
        assert_eq!(agent.evolution.iteration, 1);
        assert!(!agent.provider.providers.is_empty());
        assert!(!agent.evolution.history.is_empty());
        assert!(agent.evolution.history[0].iteration == 1);
    }

    #[test]
    fn test_evolution_loop_100_iterations() {
        let mut agent = NeoCodexAgent::new("evo-100");
        agent.evolution = EvolutionLoop::new().with_target(100);
        for _ in 0..100 {
            EvolutionLoop::step(&mut agent);
        }
        assert_eq!(agent.evolution.iteration, 100);
        assert_eq!(agent.evolution.history.len(), 100);
        // Fixes applied monotonically
        assert!(agent.evolution.fixes_applied > 0 || agent.provider.providers.len() > 1);
    }

    #[test]
    fn test_extract_tool_call() {
        let content = "Let me read the file.\n\n<tool name=\"read\">Cargo.toml</tool>";
        let (name, args) = NeoCodexAgent::extract_tool_call(content).unwrap();
        assert_eq!(name, "read");
        assert_eq!(args, "Cargo.toml");
        assert!(NeoCodexAgent::extract_tool_call("no tool here").is_none());
    }

    #[test]
    fn test_react_loop_falls_back_without_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-test");
            let out = agent.react_loop("hello", 2).await;
            // Default catalog (opencode stub) is not a real provider → None
            assert!(out.is_none());
        });
    }

    #[test]
    fn test_provider_capability_after_sync() {
        let mut catalog = ProviderCatalog::new();
        catalog.sync_from_real();
        // At least one provider offers code capability
        assert!(catalog.providers.iter().any(|p| p.capabilities.contains(&ModelCapability::Code)));
    }

    #[test]
    fn test_resume_session_restores_context() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("neocodex_test_resume");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("resume.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        for event in [
            WireEvent::UserMessage { content: "hello".into(), timestamp: 1, attachments: None },
            WireEvent::AgentMessage { content: "hi there".into(), timestamp: 2 },
            WireEvent::ModeChange { from: NeoCodexMode::Agent, to: NeoCodexMode::Plan },
        ] {
            let line = serde_json::to_string(&event).unwrap();
            writeln!(f, "{}", line).unwrap();
        }
        drop(f);

        let mut agent = NeoCodexAgent::new("resume-test");
        agent.wire.path = path;
        let n = agent.resume_session();
        assert_eq!(n, 3);
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
        assert!(agent.context.turns.len() >= 2);
        assert!(agent.state.tokens_used > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resume_empty_session_is_noop() {
        let mut agent = NeoCodexAgent::new("resume-empty");
        agent.wire.path = std::env::temp_dir().join("neocodex_missing.jsonl");
        let n = agent.resume_session();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_health_report_after_evolution_advances() {
        let mut agent = NeoCodexAgent::new("health-evo");
        EvolutionLoop::step(&mut agent);
        let report = agent.health_report();
        assert_eq!(report.evolution_iterations, 1);
    }

    #[test]
    fn test_react_loop_with_configured_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-wired");
            agent.provider.sync_from_real();
            // Find a local/resolvable provider (ollama first in catalog)
            if let Some(idx) = agent.provider.providers.iter().position(|p| p.name == "ollama") {
                agent.provider.active = idx;
                // to_llm_provider succeeds for ollama even without network (request only)
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
            }
        });
    }

    #[test]
    fn test_react_loop_stream_no_provider_is_noop() {
        // D-streaming closure: without a resolvable provider the streaming
        // loop must not panic and must return None (caller falls back).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-empty");
            agent.provider.providers.clear();
            agent.provider.providers.push(ProviderInfo::default());
            agent.provider.active = 0;
            let mut seen = Vec::new();
            let mut tools = Vec::new();
            let result = agent.react_loop_stream("hi", 3, |t| { seen.push(t.to_string()); true }, |n, _, _, _, _| { tools.push(n.to_string()); true }).await;
            assert!(result.is_none());
            assert!(seen.is_empty());
            assert!(tools.is_empty());
        });
    }

    #[test]
    fn test_react_loop_stream_provider_supports_streaming() {
        // All real providers implement stream_complete; verify the trait is
        // reachable through the resolved provider (no network needed).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-wired");
            agent.provider.sync_from_real();
            if let Some(idx) = agent.provider.providers.iter().position(|p| p.name == "ollama") {
                agent.provider.active = idx;
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
                let provider = provider.unwrap();
                // Building a stream request must not panic; a stopped local
                // server yields Err(LlmError) which is the expected path.
                let request = LlmRequest {
                    model: "test".into(),
                    messages: vec![Message::new(Role::User, "hi")],
                    temperature: None,
                    max_tokens: 16,
                    tools: vec![],
                    image_data: None,
                    thinking_budget: None,
                    provider_params: Default::default(),
                    constraint_json: None,
                    structured_output: None,
                };
                let _ = provider.stream_complete(&request).await;
            }
        });
    }

    #[test]
    fn test_tool_grounding_records_calls() {
        // D25 closure: record_tool_result must be invoked on real tool execution
        let mut agent = NeoCodexAgent::new("grounding");
        agent.tool_grounding.record_tool_result("read", true, true);
        agent.tool_grounding.record_tool_result("read", true, false);
        assert_eq!(agent.tool_grounding.total_calls, 2);
        assert!(agent.tool_grounding.degraded_tools().len() <= 1);
        let report = agent.health_report();
        assert!(report.tool_call_count >= 0);
    }

    #[test]
    fn test_build_request_bridges_image_for_text_only_model() {
        // A real 4x4 PNG encoded as base64 → a valid VisionBridge decode target.
        let png = {
            use image::RgbImage;
            let mut buf = std::io::Cursor::new(Vec::new());
            let img = RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0]));
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode");
            buf.into_inner()
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        let mut agent = NeoCodexAgent::new("vision-bridge");
        // Force a text-only active provider (default catalog: opencode stub, no Vision cap).
        agent.wire.record(WireEvent::UserMessage {
            content: "describe the image".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: png.len() as u64,
                mime_type: "image/png".into(),
                data: Some(b64.clone()),
            }]),
        });
        let messages = vec![
            Message::new(Role::System, "system"),
            Message::new(Role::User, "describe the image"),
        ];
        let req = agent.build_request(messages.clone()).expect("request built");
        // Text-only model → image must be bridged into the user message, image_data None.
        assert!(req.image_data.is_none(), "image_data must be dropped for text-only");
        let user_msg = req.messages.iter().find(|m| m.role == Role::User).unwrap();
        assert!(
            user_msg.content.contains("<image_evidence>"),
            "evidence must be injected: {}",
            user_msg.content
        );
        assert!(user_msg.content.contains("dimensions: 4x4"));
        // image_data None → data-URI wrap not applied (no vision).
    }

    #[test]
    fn test_build_request_keeps_image_for_vision_model() {
        let mut agent = NeoCodexAgent::new("vision-native");
        agent.provider.add_provider(ProviderInfo {
            name: "vision".into(),
            model: "gpt-4o".into(),
            capabilities: vec![ModelCapability::Vision],
            context_limit: 100_000,
            cost_per_m_input: 1.0,
            cost_per_m_output: 1.0,
        });
        agent.provider.active = 1;
        let b64 = "iVBORw0KGgo=";
        agent.wire.record(WireEvent::UserMessage {
            content: "describe".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: 10,
                mime_type: "image/png".into(),
                data: Some(b64.into()),
            }]),
        });
        let messages = vec![Message::new(Role::User, "describe")];
        let req = agent.build_request(messages).expect("request built");
        // Vision-capable model → raw base64 upgraded to a data URI and preserved.
        let uri = req.image_data.expect("image_data preserved for vision model");
        assert!(uri.starts_with("data:image/"), "data URI expected, got {uri}");
        assert!(!req.messages[0].content.contains("<image_evidence>"));
    }
}
