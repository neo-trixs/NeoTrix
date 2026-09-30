//! 策略组装 — 响应缓存 / 质量评估 / 响应修复 / 池健康 + GatewayV2 韧性接线。
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::l1_action::nt_core_llm::{LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::common::generation_classifier::{GenerationRecord, LlmPurpose, TaskType};
use crate::l1_action::nt_core_llm::Message;
use super::super::types::ProviderState;
use super::super::GatewayV2;
use super::nt_resilience_types::{PoolHealthReport, ResponseQualityScore};

// ═══════════════════════════════════════════════════════════════════
// Response Cache — LRU 响应缓存
// ═══════════════════════════════════════════════════════════════════
#[derive(Debug)]
pub struct ResponseCache {
    entries: HashMap<u64, (String, u64)>,
    capacity: usize,
    tick: u64,
    hit_count: u64,
    miss_count: u64,
    pinned: HashSet<u64>,
    prefetch_hits: u64,
}
impl ResponseCache {
    pub const DEFAULT_CAPACITY: usize = 256;
    pub const MAX_PINNED: usize = 32;

    /// Create a ResponseCache with the given capacity.
    ///
    /// Note: Real implementation needs — hash-based keying uses DefaultHasher which
    /// is not cryptographically stable across platforms. Consider: using a stable
    /// hash (e.g., xxHash) for cross-session cache persistence.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity: capacity.max(1),
            tick: 0,
            hit_count: 0,
            miss_count: 0,
            pinned: HashSet::new(),
            prefetch_hits: 0,
        }
    }

    /// Build a cache key from model ID and message content.
    ///
    /// Note: Real implementation needs — concatenates all message content which may
    /// produce long keys. Consider: using a content hash for shorter, fixed-size keys.
    pub fn key_for(model_id: &str, messages: &[Message]) -> String {
        let body = messages
            .iter()
            .map(|m| format!("{:?}:{}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}|{}", model_id, body)
    }

    /// Build a cache key from model ID and a pre-computed fingerprint.
    ///
    /// Note: Real implementation needs — fingerprint should be deterministic for
    /// the same request. Consider: adding cache versioning to invalidate stale entries.
    pub fn key_for_request(model_id: &str, fingerprint: &str) -> String {
        format!("{}|fp={}", model_id, fingerprint)
    }

    fn hash_key(key: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Look up a cached response by key, updating LRU timestamp on hit.
    ///
    /// Note: Real implementation needs — O(1) HashMap lookup. Consider: TTL-based
    /// expiration for time-sensitive responses and size-based eviction pressure.
    pub fn cache(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        if let Some((resp, last_used)) = self.entries.get_mut(&hash) {
            self.tick += 1;
            *last_used = self.tick;
            self.hit_count += 1;
            return Some(resp.clone());
        }
        self.miss_count += 1;
        None
    }

    /// Insert a response into the cache, evicting LRU entry if at capacity.
    ///
    /// Note: Real implementation needed — pinned entries are never evicted.
    /// Consider: adding compression for large responses and tracking per-key
    /// access patterns for smarter eviction.
    pub fn insert(&mut self, key: &str, response: String) {
        let hash = Self::hash_key(key);
        self.tick += 1;
        if let Some(entry) = self.entries.get_mut(&hash) {
            *entry = (response, self.tick);
            return;
        }
        if self.entries.len() >= self.capacity {
            let lru_key = self
                .entries
                .iter()
                .filter(|(k, _)| !self.pinned.contains(k))
                .min_by_key(|(_, (_, t))| *t)
                .map(|(k, _)| *k);
            if let Some(k) = lru_key {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(hash, (response, self.tick));
    }

    /// Pin a cache entry to prevent eviction. Returns false if at MAX_PINNED limit.
    ///
    /// Note: Real implementation needed — pinned entries consume capacity but are
    /// never evicted. Consider: adding TTL to pins and auto-unpin on staleness.
    pub fn pin(&mut self, key: &str) -> bool {
        if self.pinned.len() >= Self::MAX_PINNED {
            return false;
        }
        self.pinned.insert(Self::hash_key(key))
    }

    /// Unpin a previously pinned cache entry, making it eligible for eviction.
    ///
    /// Note: Real implementation needed — no-op if key is not pinned. Consider:
    /// logging unpin events for cache behavior analysis.
    pub fn unpin(&mut self, key: &str) {
        self.pinned.remove(&Self::hash_key(key));
    }

    /// Return the number of currently pinned cache entries.
    ///
    /// Note: Real implementation needed — useful for cache health monitoring.
    /// Consider: adding pinned percentage as a cache health metric.
    pub fn pinned_count(&self) -> usize {
        self.pinned.len()
    }

    /// Prefetch a cache entry, updating its LRU timestamp without returning it.
    ///
    /// Note: Real implementation needed — useful for warming cache before expected
    /// access. Consider: batch prefetch with parallel lookup for multi-key warming.
    pub fn prefetch(&mut self, key: &str) -> Option<String> {
        let hash = Self::hash_key(key);
        let exists = self.entries.contains_key(&hash);
        if !exists {
            return None;
        }
        self.tick += 1;
        if let Some((_, t)) = self.entries.get_mut(&hash) {
            *t = self.tick;
        }
        self.prefetch_hits += 1;
        self.entries.get(&hash).map(|(resp, _)| resp.clone())
    }

    pub fn prefetch_hit_count(&self) -> u64 {
        self.prefetch_hits
    }

    /// Batch prefetch: look up multiple hint keys, returning hit count and missing keys.
    ///
    /// Note: Real implementation needed — sequential lookup. Consider: parallel lookup
    /// for large hint sets and selective prefetch based on access pattern prediction.
    pub fn prefetch_lookahead(&mut self, hints: &[String]) -> (usize, Vec<String>) {
        let mut hits = 0;
        let mut missing = Vec::new();
        for hint in hints {
            let hash = Self::hash_key(hint);
            if self.entries.contains_key(&hash) {
                self.tick += 1;
                if let Some((_, t)) = self.entries.get_mut(&hash) {
                    *t = self.tick;
                }
                self.prefetch_hits += 1;
                hits += 1;
            } else {
                missing.push(hint.clone());
            }
        }
        (hits, missing)
    }

    /// Generate lookahead hint keys based on a base cache key.
    ///
    /// Note: Real implementation needed — generates fixed pattern hints
    /// (`fp=lookahead:1`, `fp=lookahead:2`). Consider: learning actual access
    /// patterns from history to generate smarter hints.
    pub fn lookahead_hints(&self, key: &str) -> Vec<String> {
        let mut hints = Vec::new();
        if let Some((model, _)) = key.split_once('|') {
            hints.push(format!("{}|fp=lookahead:1", model));
            hints.push(format!("{}|fp=lookahead:2", model));
        }
        hints
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Non-mutating membership test.
    ///
    /// Deliberately does **not** refresh the LRU timestamp, unlike [`Self::cache`]:
    /// asking "is it cached?" must not change what gets evicted next. Added for the
    /// behavioural parity harness (`.neotrix/parity/response-cache.vectors.json`),
    /// whose `contains` step needs to observe presence without perturbing order —
    /// the reference implementation's `contains` is likewise non-promoting.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(&Self::hash_key(key))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn hit_count(&self) -> u64 {
        self.hit_count
    }

    pub fn miss_count(&self) -> u64 {
        self.miss_count
    }
}
// ═══════════════════════════════════════════════════════════════════
// Response Quality — 响应质量评估
// ═══════════════════════════════════════════════════════════════════
impl ResponseQualityScore {
    pub fn new(coherence: f64, relevance: f64, completeness: f64) -> Self {
        Self { coherence, relevance, completeness }
    }

    pub fn composite(&self) -> f64 {
        (self.coherence + self.relevance + self.completeness) / 3.0
    }
}
pub fn evaluate_response_quality(content: &str) -> ResponseQualityScore {
    let coherence = if content.is_empty() {
        0.0
    } else {
        let len = content.len() as f64;
        (1.0 - (len / 10000.0).min(1.0)) * 0.8
    };
    let relevance = 0.7;
    let completeness = if content.contains('.') || content.contains('。') {
        0.9
    } else {
        0.5
    };
    ResponseQualityScore::new(coherence, relevance, completeness)
}
// ═══════════════════════════════════════════════════════════════════
// Response Healer — 畸形 JSON 修复
// ═══════════════════════════════════════════════════════════════════
#[derive(Debug, Default)]
pub struct ResponseHealer {
    heal_count: u64,
    unrepairable_count: u64,
}
impl ResponseHealer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Heal malformed JSON response (extract, trim trailing commas, close unclosed brackets).
    ///
    /// Note: Real implementation needs — the healing pipeline is extract → trim → close.
    /// Consider adding: Unicode normalization, escape sequence repair, and schema
    /// validation after healing. Also track which providers commonly return malformed
    /// JSON for targeted improvement.
    pub fn heal(&mut self, raw: &str) -> String {
        if serde_json::from_str::<serde_json::Value>(raw).is_ok() {
            return raw.to_string();
        }
        let extracted = self.extract_json(raw);
        let trimmed = self.trim_trailing_commas(&extracted);
        let closed = self.close_unclosed(&trimmed);
        if serde_json::from_str::<serde_json::Value>(&closed).is_ok() {
            self.heal_count += 1;
            return closed;
        }
        self.unrepairable_count += 1;
        raw.to_string()
    }

    fn extract_json(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(fence) = trimmed.find("```json") {
            let after = &trimmed[fence + "```json".len()..];
            let content = match after.find("```") {
                Some(end) => &after[..end],
                None => after,
            };
            let c = content.trim();
            return if c.is_empty() {
                raw.to_string()
            } else {
                c.to_string()
            };
        }
        let chars: Vec<char> = trimmed.chars().collect();
        let mut start = None;
        for (i, c) in chars.iter().enumerate() {
            if *c == '{' || *c == '[' {
                start = Some(i);
                break;
            }
        }
        let start = match start {
            Some(s) => s,
            None => return raw.to_string(),
        };
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        let mut end = chars.len();
        for (i, c) in chars.iter().enumerate().skip(start) {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *c == '\\' {
                    escaped = true;
                } else if *c == '"' {
                    in_string = false;
                }
                continue;
            }
            match *c {
                '"' => in_string = true,
                '{' | '[' => depth += 1,
                '}' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        chars[start..end].iter().collect::<String>().trim().to_string()
    }

    fn trim_trailing_commas(&self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut in_string = false;
        let mut escaped = false;
        while i < n {
            let c = chars[i];
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if c == '"' {
                in_string = true;
                out.push('"');
                i += 1;
                continue;
            }
            if c == ',' {
                let mut j = i + 1;
                while j < n && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < n && (chars[j] == '}' || chars[j] == ']') {
                    i += 1;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        }
        out
    }

    fn close_unclosed(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 4);
        let mut stack: Vec<char> = Vec::new();
        let mut in_string = false;
        let mut escaped = false;
        for c in s.chars() {
            if in_string {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => {
                    in_string = true;
                    out.push('"');
                }
                '{' => {
                    stack.push('{');
                    out.push('{');
                }
                '[' => {
                    stack.push('[');
                    out.push('[');
                }
                '}' => {
                    if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                    } else if stack.last() == Some(&'[') {
                        stack.pop();
                        out.push(']');
                        stack.pop();
                        out.push('}');
                    }
                }
                ']' => {
                    if stack.last() == Some(&'[') {
                        stack.pop();
                        out.push(']');
                    } else if stack.last() == Some(&'{') {
                        stack.pop();
                        out.push('}');
                        stack.pop();
                        out.push(']');
                    }
                }
                _ => out.push(c),
            }
        }
        while let Some(open) = stack.pop() {
            out.push(match open {
                '{' => '}',
                _ => ']',
            });
        }
        out
    }

    pub fn heal_count(&self) -> u64 {
        self.heal_count
    }

    pub fn unrepairable_count(&self) -> u64 {
        self.unrepairable_count
    }
}
// ═══════════════════════════════════════════════════════════════════
// Pool Health — LLM 池健康探测器
// ═══════════════════════════════════════════════════════════════════
pub struct LlmPoolHealth;
impl LlmPoolHealth {
    /// Evaluate the health of the LLM provider pool.
    ///
    /// Note: Real implementation needs — returns a snapshot of pool metrics.
    /// Consider: adding time-windowed health tracking, provider-specific
    /// health scores, and trend analysis for pool degradation detection.
    pub fn evaluate(gw: &GatewayV2, min_free: usize) -> PoolHealthReport {
        let providers = gw.providers();
        let total = providers.len();
        let free = gw.available_free_providers().len();
        let model_locked = gw.model_locked_count();
        let sufficient = gw.is_pool_sufficient(min_free);
        let local = providers
            .iter()
            .filter(|n| n.contains("ollama") || n.contains("vllm") || n.contains("llama"))
            .count();
        PoolHealthReport {
            total,
            free,
            paid: total.saturating_sub(free),
            local,
            model_locked,
            sufficient,
            min_free,
        }
    }

    /// Generate a human-readable summary of pool health.
    ///
    /// Note: Real implementation needs — simple format string. Consider:
    /// structured logging integration and telemetry export.
    pub fn summarize(gw: &GatewayV2, min_free: usize) -> String {
        let r = Self::evaluate(gw, min_free);
        format!(
            "LLM pool health: total={} free={} paid={} local={} model_locked={} sufficient={} (min_free={})",
            r.total, r.free, r.paid, r.local, r.model_locked, r.sufficient, r.min_free
        )
    }
}
// ═══════════════════════════════════════════════════════════════════
// GatewayV2 Resilience Extensions — 韧性接线
// ═══════════════════════════════════════════════════════════════════
impl GatewayV2 {
    /// Enable or disable the response cache.
    ///
    /// Note: Real implementation needs — simple toggle. Consider: adding
    /// cache warming on enable, and cache flush on disable.
    pub fn enable_response_cache(&mut self, enabled: bool) {
        self.response_cache_enabled = enabled;
    }

    /// Check if response cache is enabled.
    ///
    /// Note: Real implementation needs — returns current toggle state.
    pub fn response_cache_enabled(&self) -> bool {
        self.response_cache_enabled
    }

    /// Get the total number of response cache hits.
    ///
    /// Note: Real implementation needs — returns counter from cache.
    /// Consider: adding hit rate calculation and time-windowed stats.
    pub fn response_cache_hits(&self) -> u64 {
        self.response_cache.lock().map(|c| c.hit_count()).unwrap_or(0)
    }

    /// Get the current number of entries in the response cache.
    ///
    /// Note: Real implementation needs — returns cache entry count.
    pub fn response_cache_len(&self) -> usize {
        self.response_cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// Get the total number of response cache prefetch operations.
    ///
    /// Note: Real implementation needs — returns atomic counter.
    pub fn response_cache_prefetches(&self) -> u64 {
        self.response_cache_prefetches
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Enable or disable the response healer (malformed JSON repair).
    ///
    /// Note: Real implementation needs — simple toggle. Consider: adding
    /// healer statistics reset on disable.
    pub fn set_response_healer(&mut self, enabled: bool) {
        self.response_healer_enabled = enabled;
    }

    /// Check if response healer is enabled.
    ///
    /// Note: Real implementation needs — returns current toggle state.
    pub fn response_healer_enabled(&self) -> bool {
        self.response_healer_enabled
    }

    /// Get the (healed, unrepairable) counters from the response healer.
    ///
    /// Note: Real implementation needs — returns tuple of heal counts.
    /// Consider: adding per-provider breakdown and time-windowed stats.
    pub fn response_healer_counters(&self) -> (u64, u64) {
        match self.response_healer.lock() {
            Ok(h) => (h.heal_count(), h.unrepairable_count()),
            Err(_) => (0, 0),
        }
    }

    /// Enable or disable generation classification for analytics.
    ///
    /// Note: Real implementation needs — simple toggle. Consider: adding
    /// classification model hot-swap and statistics reset on toggle.
    pub(crate) fn _set_generation_classification(&mut self, enabled: bool) {
        self.generation_classification_enabled = enabled;
    }

    /// Check if generation classification is enabled.
    ///
    /// Note: Real implementation needs — returns current toggle state.
    pub fn generation_classification_enabled(&self) -> bool {
        self.generation_classification_enabled
    }

    /// Classify and record a generation event for analytics.
    ///
    /// Note: Real implementation needs — classifies prompt+response into task type,
    /// complexity, and domain. Records to GenerationAnalytics. Consider: adding
    /// sampling rate control, async batch recording, and classification model
    /// hot-swap without downtime.
    pub(crate) fn tag_generation(
        &self,
        request: &LlmRequest,
        response: &LlmResponse,
        provider_name: &str,
        latency_ms: f64,
        tokens: u32,
        success: bool,
    ) {
        if !self.generation_classification_enabled {
            return;
        }
        let prompt = request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let classification = match self.generation_classifier.lock() {
            Ok(c) => c.classify(&prompt, &response.content),
            Err(e) => {
                log::warn!("[gateway] generation_classifier poisoned: {}", e);
                return;
            }
        };
        let purpose = match classification.task_type {
            TaskType::ToolUse => LlmPurpose::ToolUse,
            TaskType::Summarization => LlmPurpose::Summarization,
            TaskType::Extraction => LlmPurpose::ToolUse,
            _ => LlmPurpose::AgentTurn,
        };
        let record = GenerationRecord {
            model: format!("{}/{}", provider_name, request.model),
            classification,
            prompt_len: prompt.len(),
            response_len: response.content.len(),
            latency_ms: latency_ms as u64,
            tokens,
            success,
            purpose,
        };
        if let Ok(mut analytics) = self.generation_analytics.lock() {
            analytics.record(&record);
        }
    }

    /// Take a snapshot of generation analytics: total count and distributions.
    ///
    /// Note: Real implementation needs — returns (total, task_type_dist, complexity_dist, domain_dist).
    /// Consider: adding time-windowed snapshots and per-provider breakdowns.
    pub(crate) fn _generation_analytics_snapshot(&self) -> (u64, HashMap<String, u64>, HashMap<String, u64>, HashMap<String, u64>) {
        match self.generation_analytics.lock() {
            Ok(a) => (
                a.total,
                a.distribution("task_type"),
                a.distribution("complexity"),
                a.distribution("domain"),
            ),
            Err(_) => (0, HashMap::new(), HashMap::new(), HashMap::new()),
        }
    }

    /// Trigger market router re-evaluation if the re-evaluation interval has elapsed.
    ///
    /// Note: Real implementation needs — delegates to MarketRouter::re_evaluate.
    /// Consider: adding event notification on weight changes and configurable
    /// re-evaluation intervals.
    pub fn maybe_re_evaluate(&self) -> bool {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let refs: Vec<&ProviderState> = states.values().collect();
        match self.market_router.lock() {
            Ok(mut router) => router.re_evaluate(&refs),
            Err(e) => {
                log::warn!("[gateway] market_router Mutex poisoned: {}", e);
                false
            }
        }
    }

    /// Heal malformed JSON in response and cache the result.
    ///
    /// Note: Real implementation needs — applies ResponseHealer if enabled, then
    /// caches in ResponseCache if enabled. Consider: adding response validation
    /// after healing, cache key deduplication, and heal statistics reporting.
    pub(crate) fn heal_and_cache_response(&self, request: &LlmRequest, response: LlmResponse) -> LlmResponse {
        let mut response = response;
        if self.response_healer_enabled {
            if let Ok(mut healer) = self.response_healer.lock() {
                if response.content.contains('{') || response.content.contains('[') {
                    response.content = healer.heal(&response.content);
                }
            }
        }
        if self.response_cache_enabled {
            let rc_key = ResponseCache::key_for_request(&request.model, &self.prompt_cache_key(request));
            if let Ok(mut rc) = self.response_cache.lock() {
                match serde_json::to_string(&response) {
                    Ok(serialized) => rc.insert(&rc_key, serialized),
                    Err(_) => rc.insert(&rc_key, response.content.clone()),
                }
                let hints = rc.lookahead_hints(&rc_key);
                if !hints.is_empty() {
                    let (_, _) = rc.prefetch_lookahead(&hints);
                    self.response_cache_prefetches.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        response
    }

    /// Extract concatenated text content from all messages in a request.
    ///
    /// Note: Real implementation needs — joins all message content with newlines.
    /// Consider: adding role-aware extraction, tool call content handling, and
    /// content truncation for very long requests.
    pub(crate) fn prompt_text(&self, request: &LlmRequest) -> String {
        request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build a deterministic cache key from request parameters.
    ///
    /// Note: Real implementation needs — includes content, max_tokens, thinking_budget,
    /// tools, structured_output, and cacheable_prefix_tokens. Consider: using a
    /// content hash for shorter keys, and cache versioning for invalidation.
    pub(crate) fn prompt_cache_key(&self, request: &LlmRequest) -> String {
        let content = self.prompt_text(request);
        let mut tools: Vec<&str> = request
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>();
        tools.sort_unstable();
        let tools_fp = tools.join(",");
        let structured_fp = match &request.structured_output {
            Some(s) => serde_json::to_string(s).unwrap_or_else(|e| {
                log::warn!("[gateway] structured_output serialization failed for cache key: {}", e);
                String::new()
            }),
            None => String::new(),
        };
        format!(
            "{}|max={}|think={:?}|tools=[{}]|struct={}|prefix={:?}",
            content,
            request.max_tokens,
            request.thinking_budget,
            tools_fp,
            structured_fp,
            request.cacheable_prefix_tokens
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_health_empty() {
        let gw = GatewayV2::new();
        let r = LlmPoolHealth::evaluate(&gw, 3);
        assert_eq!(r.total, 0);
        assert_eq!(r.free, 0);
        assert!(!r.sufficient);
        assert_eq!(r.min_free, 3);
    }

    #[test]
    fn test_pool_health_summary() {
        let gw = GatewayV2::new();
        let s = LlmPoolHealth::summarize(&gw, 3);
        assert!(s.contains("LLM pool health"));
        assert!(s.contains("total=0"));
    }
}
