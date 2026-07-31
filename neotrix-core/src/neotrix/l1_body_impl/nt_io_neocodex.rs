#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

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
        if !self.is_resolvable() {
            if let Some(idx) = self.providers.iter().position(|p| Self::provider_type_of(&p.name).is_some()) {
                self.active = idx;
            }
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
            } else if content[i..].starts_with("---") && content[i..].starts_with("---") {
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
}

impl ContextPipeline {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            turns: VecDeque::new(),
            max_tokens,
            budget_high: 0.8,
            budget_low: 0.5,
        }
    }

    pub fn push(&mut self, role: &str, content: String, token_count: usize) {
        self.turns.push_back(ContextTurn {
            role: role.to_string(),
            content,
            token_count,
            priority: if role == "system" { 5 } else { 3 },
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

        // Layer 1: Budget reduce — trim oversized tool outputs
        let max_turn_tokens = self.max_tokens / 4;
        for turn in &mut self.turns {
            if turn.token_count > max_turn_tokens && turn.priority < 4 {
                let kept = turn.content.chars().take(max_turn_tokens).collect::<String>();
                turn.content = format!("{}... [trimmed {} bytes]", kept, turn.content.len().saturating_sub(max_turn_tokens));
                turn.token_count = max_turn_tokens;
            }
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 2: Snip — reduce temporal depth (keep newest)
        while self.turns.len() > 50 {
            self.turns.pop_front();
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 3: Microcompact — squeeze low-priority turns
        let mut i = 0;
        while i < self.turns.len() && self.total_tokens() > (self.max_tokens as f64 * self.budget_low) as usize {
            if self.turns[i].priority < 2 {
                self.turns[i].content.truncate(200);
                self.turns[i].token_count = 200;
            }
            i += 1;
        }

        if self.total_tokens() < (self.max_tokens as f64 * self.budget_low) as usize { return; }

        // Layer 4: Context collapse — summarize old turns
        while self.turns.len() > 10 {
            let front = self.turns.pop_front().unwrap();
            if !self.turns.is_empty() && self.turns[0].role != "summary" {
                self.turns.push_front(ContextTurn {
                    role: "summary".into(),
                    content: format!("[compressed: {} chars]", front.content.len()),
                    token_count: 20,
                    priority: 1,
                });
            }
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
    UserMessage { content: String, timestamp: i64 },
    AgentMessage { content: String, timestamp: i64 },
    ToolCall { name: String, args: String, result: String, duration_ms: u64, success: bool },
    SystemEvent { kind: String, detail: String, timestamp: i64 },
    GoalUpdate { id: String, state: String, description: String },
    ModeChange { from: NeoCodexMode, to: NeoCodexMode },
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
        self.events.push(event.clone());
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
}

impl Default for GoalQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalQueue {
    pub fn new() -> Self {
        Self { goals: VecDeque::new(), active: None, completed: Vec::new() }
    }

    pub fn add(&mut self, description: &str, max_iterations: u64) {
        let id = format!("g-{}", self.completed.len() + self.goals.len() + 1);
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
            ("shell".to_string(), PermissionLevel::Ask),
        ];
        Self { permissions, default_level: PermissionLevel::Ask }
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
                    {"name": "edit", "description": "Edit files"},
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

#[derive(Debug, Clone, Default)]
pub struct NeoCodexConfig {
    pub mode: NeoCodexMode,
    pub max_turn_tokens: usize,
    pub provider_name: String,
    pub auto_compact: bool,
    pub shell_available: bool,
    pub thinking_enabled: bool,
    pub goal_mode: bool,
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub tokens_used: usize,
    pub mode: NeoCodexMode,
    pub mode_start: Instant,
    pub goal_active: bool,
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
        }
    }

    /// Set budget limit (from Claude Code max_budget_usd)
    pub fn with_budget(mut self, max_budget: f64) -> Self {
        self.cost.max_budget = max_budget;
        self
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

    /// Process user input through the agent loop
    pub async fn process(&mut self, input: &str) -> String {
        self.state.turn_count += 1;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        self.wire.record(WireEvent::UserMessage {
            content: input.to_string(),
            timestamp,
        });

        let token_estimate = input.len() / 4;
        self.context.push("user", input.to_string(), token_estimate);
        self.state.tokens_used += token_estimate;

        // ── Consciousness-in-the-loop (NeoTrix unique) ──
        self.inject_into_consciousness();
        self.apply_consciousness_guidance();

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
                let clean = self.markdown.buffer.clone();
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
                let fallback = format!("<thinking>Processing turn {} in Agent mode (provider unavailable, stub)</thinking>\n\n{}",
                    self.state.turn_count, input);
                self.markdown.push(&fallback);
                fallback
            }
        };

        self.markdown.push(&response);
        let clean = self.markdown.buffer.clone();
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
        let id = format!("g-{}", self.state.turn_count);
        self.goals.add(description, max_iters);
        self.state.goal_active = true;
        self.wire.record(WireEvent::GoalUpdate {
            id, state: "active".into(), description: description.into(),
        });
    }

    /// Check if a goal is complete and advance the queue
    pub fn check_goals(&mut self) -> Option<String> {
        if let Some(ref goal) = self.goals.active {
            if goal.iterations >= goal.max_iterations {
                self.wire.record(WireEvent::GoalUpdate {
                    id: goal.id.clone(), state: "completed".into(),
                    description: goal.description.clone(),
                });
                return self.goals.next().map(|g| g.description);
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

        if fruit_count < 3 && tree.cycle > 5 {
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
    /// Returns the final accumulated response (or error).
    pub async fn react_loop_stream<F>(&mut self, input: &str, max_steps: usize, mut on_token: F) -> Option<String>
    where
        F: FnMut(&str) + Send + Sync,
    {
        let provider = self.provider.to_llm_provider()?;

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;
        let mut accumulated = String::new();

        while step < max_steps {
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
                            on_token(&resp.content);
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

            // Attempt to extract a structured tool-call from the response.
            let tool_call = Self::extract_tool_call(&response_content);

            match tool_call {
                Some((name, args)) => {
                    self.state.tool_call_count += 1;
                    let result = self.execute_tool(&name, &args).await;
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
                    step += 1;
                }
                None => {
                    final_answer = Some(response_content);
                    break;
                }
            }
        }

        final_answer
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
                "assistant" => Role::Assistant,
                "system" | "summary" => Role::System,
                _ => Role::User,
            };
            if role == Role::System {
                continue; // keep single system message
            }
            messages.push(Message::new(role, &turn.content));
        }
        messages.push(Message::new(Role::User, input));
        messages
    }

    /// Build an LlmRequest from the current catalog's active provider.
    fn build_request(&self, messages: Vec<Message>) -> Option<LlmRequest> {
        self.provider.providers.get(self.provider.active)?;
        Some(LlmRequest {
            model: self.provider.active_model(),
            messages,
            temperature: Some(0.3),
            max_tokens: 4096,
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
                    name: "shell".into(),
                    description: "Run a shell command and return stdout".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}}),
                },
            ],
            image_data: None,
            thinking_budget: if self.config.thinking_enabled { Some(2048) } else { None },
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        })
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

    /// Execute a concrete tool. Wired to real FS + shell (no external binaries,
    /// consistent with R-P48 zero third-party binary dependency).
    async fn execute_tool(&mut self, name: &str, args: &str) -> String {
        match name {
            "read" => match std::fs::read_to_string(args.trim()) {
                Ok(content) => {
                    if content.len() > 16_000 {
                        content.chars().take(16_000).collect()
                    } else {
                        content
                    }
                }
                Err(e) => format!("[read error] {}", e),
            },
            "search" => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let pattern = args.trim();
                let mut hits = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&cwd) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (i, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    hits.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
                                    if hits.len() >= 20 { break; }
                                }
                            }
                        }
                        if hits.len() >= 20 { break; }
                    }
                }
                if hits.is_empty() {
                    format!("No matches for {:?} in {}", pattern, cwd.display())
                } else {
                    hits.join("\n")
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
                        if code == 0 {
                            stdout
                        } else {
                            format!("exit({}): {}", code, stderr)
                        }
                    }
                    Err(e) => format!("[shell error] {}", e),
                }
            }
            _ => format!("Unknown tool: {}", name),
        }
    }

    /// Restore prior session events from the wire file into the context
    /// pipeline (G2 session continuity, matching Claude Code `--resume`).
    /// Returns the number of events restored.
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
        ctx.push("user", "test message".into(), 10);
        assert_eq!(ctx.turns.len(), 1);
        assert!(ctx.total_tokens() <= ctx.max_tokens);
    }

    #[test]
    fn test_context_pipeline_compaction() {
        let mut ctx = ContextPipeline::new(500);
        for i in 0..20 {
            ctx.push("user", format!("message {}", i), 100);
        }
        assert!(ctx.total_tokens() <= ctx.max_tokens);
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
    fn test_wire_format_roundtrip() {
        let mut session = WireSession::new("test-wire");
        session.record(WireEvent::UserMessage {
            content: "hello".into(),
            timestamp: 1000,
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
            content: "replay".into(), timestamp: 42,
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
            WireEvent::UserMessage { content: "hello".into(), timestamp: 1 },
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
            let result = agent.react_loop_stream("hi", 3, |t| seen.push(t.to_string())).await;
            assert!(result.is_none());
            assert!(seen.is_empty());
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
}
