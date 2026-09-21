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
