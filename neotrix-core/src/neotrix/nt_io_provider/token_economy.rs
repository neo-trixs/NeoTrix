//! Token Economy — context budget management for LLM interactions.
//!
//! ## Components
//! - **PrefixCache**: SHA256 fingerprint of stable system prompt → KV-cache stability hint
//! - **CanonicalSort**: deterministic JSON key ordering for repeatable schema serialization
//! - **StormBreakerGuard**: detect N identical consecutive tool calls → suppress + summary
//! - **UsageTracker**: per-session tool invocation counting

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

// ===== PrefixCache =====

/// Tracks the system prompt fingerprint for prefix-stability hints.
///
/// When the fingerprint is stable across consecutive calls, the provider can
/// potentially reuse KV-cache for the prefix (OpenAI `system_fingerprint`,
/// DeepSeek prefix caching, etc.).
#[derive(Debug, Clone)]
pub struct PrefixCache {
    fingerprint: String,
    stable_calls: u64,
}

impl PrefixCache {
    pub fn new() -> Self {
        Self {
            fingerprint: String::new(),
            stable_calls: 0,
        }
    }

    /// Update with a new prefix and return the fingerprint.
    /// Returns `true` if the fingerprint is unchanged (stable prefix).
    pub fn update(&mut self, system_prompt: &str, tools_json: Option<&str>) -> bool {
        let new_fp = compute_prefix_fingerprint(system_prompt, tools_json);
        let stable = self.fingerprint == new_fp;
        self.fingerprint = new_fp;
        if stable {
            self.stable_calls += 1;
        } else {
            self.stable_calls = 0;
        }
        stable
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn stable_calls(&self) -> u64 {
        self.stable_calls
    }
}

/// Compute a deterministic SHA256 fingerprint for a stable prefix.
/// The prefix covers: system prompt text + canonical-sorted tool schema JSON.
pub fn compute_prefix_fingerprint(system_prompt: &str, tools_json: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"neotrix-prefix-v1");
    hasher.update(system_prompt.as_bytes());
    if let Some(json) = tools_json {
        hasher.update(json.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

// ===== Canonical JSON Sort =====

/// Recursively sort all JSON object keys into deterministic order.
///
/// Two semantically identical JSON values (differing only in key ordering)
/// will produce identical strings after canonical sorting, enabling
/// reliable hash comparison for prefix stability detection.
pub fn canonical_sort_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let sorted = Value::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), canonical_sort_json(&map[k])))
                    .collect(),
            );
            sorted
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonical_sort_json).collect()),
        other => other.clone(),
    }
}

// ===== Tool Args Hash =====

/// Compute a stable hash of tool call arguments for equality comparison.
/// Keys are sorted first so identical semantics produce identical hashes.
pub fn compute_args_hash(args: &std::collections::HashMap<String, String>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut keys: Vec<&String> = args.keys().collect();
    keys.sort();
    for k in keys {
        k.hash(&mut hasher);
        args[k].hash(&mut hasher);
    }
    hasher.finish()
}

// ===== StormBreakerGuard =====

/// Detects repeated tool call cycles and breaks them.
///
/// If the same tool with the same arguments appears N times consecutively,
/// the guard signals suppression so the caller can inject a summary instead
/// of sending the same call to the LLM again.
#[derive(Debug, Clone)]
pub struct StormBreakerGuard {
    recent: Vec<(String, u64)>,
    window: usize,
    suppressed_count: u64,
}

impl StormBreakerGuard {
    /// `window` = how many identical consecutive calls trigger suppression (default 3).
    pub fn new(window: usize) -> Self {
        Self {
            recent: Vec::with_capacity(window),
            window,
            suppressed_count: 0,
        }
    }

    /// Check if a tool call should be suppressed.
    /// Returns `true` when `window` consecutive identical calls are detected.
    pub fn check(&mut self, tool_name: &str, args_hash: u64) -> bool {
        let entry = (tool_name.to_string(), args_hash);

        let should_suppress = self.recent.len() >= self.window
            && self
                .recent
                .iter()
                .all(|(n, h)| n == tool_name && *h == args_hash);

        self.recent.push(entry);
        if self.recent.len() > self.window {
            self.recent.remove(0);
        }

        if should_suppress {
            self.suppressed_count += 1;
        }
        should_suppress
    }

    pub fn suppressed_count(&self) -> u64 {
        self.suppressed_count
    }

    pub fn clear(&mut self) {
        self.recent.clear();
    }
}

// ===== UsageTracker =====

/// Per-session statistics for tool call activity.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ToolInvocation {
    pub tool: String,
    pub duration_ms: u64,
    pub success: bool,
    pub iteration: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UsageTracker {
    pub invocations: Vec<ToolInvocation>,
    pub total_calls: u64,
    pub total_duration_ms: u64,
    pub successful: u64,
    pub failed: u64,
    pub suppressed: u64,
}

const MAX_INVOCATIONS: usize = 10000;

impl UsageTracker {
    pub fn record(&mut self, tool: &str, duration_ms: u64, success: bool, iteration: usize) {
        self.invocations.push(ToolInvocation {
            tool: tool.to_string(),
            duration_ms,
            success,
            iteration,
        });
        if self.invocations.len() > MAX_INVOCATIONS {
            let drain_count = self.invocations.len() - MAX_INVOCATIONS;
            self.invocations.drain(0..drain_count);
        }
        self.total_calls += 1;
        self.total_duration_ms += duration_ms;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
    }

    pub fn record_suppressed(&mut self) {
        self.suppressed += 1;
    }

    /// hit/(hit + suppressed) — how often the guard correctly identified cycles.
    pub fn guard_efficiency(&self) -> f64 {
        let total = self.suppressed + 1;
        (total - self.suppressed) as f64 / total as f64
    }

    pub fn clear(&mut self) {
        self.invocations.clear();
        self.total_calls = 0;
        self.total_duration_ms = 0;
        self.successful = 0;
        self.failed = 0;
        self.suppressed = 0;
    }
}

// ===== Concurrent Tool Batcher =====

/// Classify a tool as read-only (safe to execute concurrently) or mutating.
pub fn is_readonly_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "ReadFile" | "Glob" | "Grep" | "ReadDir" | "Browse" | "Extract"
    )
}
