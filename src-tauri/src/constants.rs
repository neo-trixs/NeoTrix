//! Centralized constants — no magic strings in business logic.

// ── Provider API URLs ──
pub const OPENAI_API_BASE: &str = "https://api.openai.com";
pub const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
pub const OLLAMA_API_BASE: &str = "http://localhost:11434";
pub const SILICONFLOW_API_BASE: &str = "https://api.siliconflow.cn";

// ── Model Download URLs ──
pub const OPENRESEARCH_DOWNLOAD_URL: &str = "https://openresearch.ai/api/models/{}/download";
pub const LOCALHOST_DOWNLOAD_URL: &str = "http://localhost:1234/api/llm/models/{}/download";
pub const OLMX_DOWNLOAD_URL: &str = "https://olmx.ai/api/models/{}/download";

// ── Market API URLs ──
pub const DSHFIND_API_BASE: &str = "https://dshfind.com/api";
pub const GITHUB_API_BASE: &str = "https://api.github.com";

// ── Cost Tracking ──
pub const DEFAULT_INPUT_COST_PER_TOKEN: f64 = 0.00001;
pub const DEFAULT_OUTPUT_COST_PER_TOKEN: f64 = 0.00003;

// ── Circuit Breaker ──
pub const CIRCUIT_BREAKER_COOLDOWN_SECS: u64 = 300;
pub const CIRCUIT_BREAKER_MAX_COOLDOWN_SECS: u64 = 1800;

// ── Time ──
pub const SECS_PER_DAY: i64 = 86_400;
pub const SECS_PER_HOUR: f64 = 3_600.0;

// ── Model Limits ──
pub const DEFAULT_MAX_MEMORIES: usize = 10_000;
pub const DEFAULT_TOKEN_BUDGET: u32 = 4_096;
pub const DEFAULT_MAX_LLM_TOKENS: u32 = 2_048;

// ── HTTP ──
pub const DEFAULT_LLAMACPP_PORT: u16 = 8080;
pub const LLAMACPP_REQUEST_TIMEOUT_SECS: u64 = 120;

// ── PTY ──
pub const PTY_BUF_SIZE: usize = 8_192;

// ── Health ──
pub const MAX_LATENCY_MS: f64 = 5_000.0;

// ── World ──
pub const MAX_FETCH_TEXT_LEN: usize = 10_000;
pub const SUMMARY_MAX_LEN: usize = 500;
