use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use lru::LruCache;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── LlmProviderType ───

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LlmProviderType {
    OpenAiCompatible,
    AnthropicMessages,
    Ollama,
    LlamaCpp,
    NvidiaNim,
    GoogleAiStudio,
    DeepSeek,
    OpenRouter,
    Groq,
    Cerebras,
}

impl LlmProviderType {
    pub fn is_free_tier(&self) -> bool {
        matches!(self, Self::Ollama | Self::LlamaCpp)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "openai-compatible",
            Self::AnthropicMessages => "anthropic-messages",
            Self::Ollama => "ollama",
            Self::LlamaCpp => "llama.cpp",
            Self::NvidiaNim => "nvidia-nim",
            Self::GoogleAiStudio => "google-ai-studio",
            Self::DeepSeek => "deepseek",
            Self::OpenRouter => "openrouter",
            Self::Groq => "groq",
            Self::Cerebras => "cerebras",
        }
    }
}

// ─── ProviderConfig ───

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: LlmProviderType,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub rate_limit_rpm: u32,
    pub cost_per_1k_tokens: f64,
}

impl ProviderConfig {
    pub fn is_free(&self) -> bool {
        self.api_key.is_empty() || self.provider_type.is_free_tier()
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }
}

// ─── SimpleRateLimiter ───

#[derive(Debug, Clone)]
pub struct SimpleRateLimiter {
    window: VecDeque<Instant>,
    max_requests: u32,
    window_seconds: u64,
}

impl SimpleRateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            window: VecDeque::new(),
            max_requests,
            window_seconds,
        }
    }

    /// Returns true if the request is allowed under the rate limit.
    pub fn check(&mut self) -> bool {
        self.evict_stale();
        (self.window.len() as u32) < self.max_requests
    }

    /// Record a request at the current instant.
    pub fn tick(&mut self) {
        self.evict_stale();
        if (self.window.len() as u32) < self.max_requests {
            self.window.push_back(Instant::now());
        }
    }

    fn evict_stale(&mut self) {
        let cutoff = Instant::now() - std::time::Duration::from_secs(self.window_seconds);
        while let Some(&t) = self.window.front() {
            if t < cutoff {
                self.window.pop_front();
            } else {
                break;
            }
        }
    }
}

// ─── RouterError ───

#[derive(Debug, Clone)]
pub enum RouterError {
    NoProviderAvailable(String),
    RateLimited(String),
    CacheDisabled,
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoProviderAvailable(msg) => write!(f, "no provider available: {}", msg),
            Self::RateLimited(msg) => write!(f, "rate limited: {}", msg),
            Self::CacheDisabled => write!(f, "cache disabled"),
        }
    }
}

impl std::error::Error for RouterError {}

// ─── RouterStats ───

#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    pub total_requests: u64,
    pub cached_responses: u64,
    pub rate_limited: u64,
    pub fallback_used: u64,
    pub provider_breakdown: HashMap<String, u64>,
}

// ─── PerModelRouter ───

#[derive(Debug, Clone)]
pub struct PerModelRouter {
    providers: HashMap<String, ProviderConfig>,
    model_routes: HashMap<String, String>,
    rate_limiters: HashMap<String, SimpleRateLimiter>,
    fallback_order: Vec<String>,
    breakers: HashMap<String, CircuitBreaker>,
    latency_trackers: HashMap<String, LatencyTracker>,
}

impl PerModelRouter {
    pub fn new() -> Self {
        let mut this = Self {
            providers: HashMap::new(),
            model_routes: HashMap::new(),
            rate_limiters: HashMap::new(),
            fallback_order: Vec::new(),
            breakers: HashMap::new(),
            latency_trackers: HashMap::new(),
        };
        this.register_defaults();
        this
    }

    pub fn with_default_free_tier() -> Self {
        Self::new()
    }

    fn register_defaults(&mut self) {
        // Ollama — fully local, free, the universal fallback
        let ollama = ProviderConfig {
            name: "ollama".into(),
            provider_type: LlmProviderType::Ollama,
            base_url: "http://localhost:11434/v1".into(),
            api_key: String::new(),
            models: vec![
                "llama3.3:70b".into(),
                "qwen2.5:72b".into(),
                "deepseek-r1:70b".into(),
                "mistral:7b".into(),
            ],
            rate_limit_rpm: 9999,
            cost_per_1k_tokens: 0.0,
        };
        self.register_provider(ollama);

        // Nvidia NIM — free 40 req/min
        let nvidia = ProviderConfig {
            name: "nvidia-nim".into(),
            provider_type: LlmProviderType::NvidiaNim,
            base_url: "https://ai.api.nvidia.com/v1".into(),
            api_key: String::new(),
            models: vec![
                "meta/llama3.1-405b-instruct".into(),
                "mistralai/mixtral-8x22b-instruct".into(),
                "google/gemma-2-27b-it".into(),
            ],
            rate_limit_rpm: 40,
            cost_per_1k_tokens: 0.0,
        };
        self.register_provider(nvidia);

        // Google AI Studio (Gemini) — free tier
        let gemini = ProviderConfig {
            name: "google-ai-studio".into(),
            provider_type: LlmProviderType::GoogleAiStudio,
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            api_key: String::new(),
            models: vec!["gemini-2.5-flash".into(), "gemini-2.5-pro".into()],
            rate_limit_rpm: 60,
            cost_per_1k_tokens: 0.0,
        };
        self.register_provider(gemini);

        // Set up default model routes
        self.model_routes.insert("opus".into(), "ollama".into());
        self.model_routes
            .insert("sonnet".into(), "nvidia-nim".into());
        self.model_routes
            .insert("haiku".into(), "google-ai-studio".into());
        self.model_routes.insert("fallback".into(), "ollama".into());

        // Fallback chain: if a provider fails, try these in order
        self.fallback_order = vec![
            "ollama".into(),
            "nvidia-nim".into(),
            "google-ai-studio".into(),
        ];
    }

    pub fn register_provider(&mut self, config: ProviderConfig) {
        let name = config.name.clone();
        self.rate_limiters.insert(
            name.clone(),
            SimpleRateLimiter::new(config.rate_limit_rpm, 60),
        );
        self.breakers.insert(name.clone(), CircuitBreaker::new());
        self.latency_trackers
            .insert(name.clone(), LatencyTracker::default());
        self.providers.insert(name, config);
    }

    pub fn route_model(&mut self, model_role: &str, provider_name: &str) {
        self.model_routes
            .insert(model_role.to_string(), provider_name.to_string());
    }

    /// Resolve a model role to a provider, walking the fallback chain if the
    /// primary provider is rate-limited or unavailable.
    pub fn resolve(&mut self, model_role: &str) -> Result<ProviderConfig, String> {
        let primary = self
            .model_routes
            .get(model_role)
            .cloned()
            .unwrap_or_else(|| "fallback".into());

        let mut tried = Vec::new();

        // Try primary + fallback order
        let chain: Vec<&str> = {
            let mut c = vec![primary.as_str()];
            for fb in &self.fallback_order {
                if fb != &primary && !tried.contains(&fb.as_str()) {
                    c.push(fb);
                }
            }
            c
        };

        for provider_name in chain {
            if tried.contains(&provider_name) {
                continue;
            }
            tried.push(provider_name);

            if let Some(rl) = self.rate_limiters.get_mut(provider_name) {
                if !rl.check() {
                    continue;
                }
            }
            if let Some(cfg) = self.providers.get(provider_name) {
                return Ok(cfg.clone());
            }
        }

        Err(format!(
            "all providers exhausted for role '{}': tried {:?}",
            model_role, tried
        ))
    }

    pub fn providers_summary(&self) -> Vec<(String, usize, String)> {
        let mut out: Vec<_> = self
            .providers
            .values()
            .map(|p| {
                let tier = if p.is_free() {
                    "free".to_string()
                } else {
                    format!("paid (${:.3}/1k tok)", p.cost_per_1k_tokens)
                };
                (p.name.clone(), p.models.len(), tier)
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Resolve a model role to a provider, using health state to skip dead
    /// providers. Returns `None` if no healthy or degraded provider is found.
    pub fn resolve_with_breaker(&mut self, model_role: &str) -> Result<ProviderConfig, String> {
        let primary = self
            .model_routes
            .get(model_role)
            .cloned()
            .unwrap_or_else(|| "fallback".into());

        let chain: Vec<&str> = {
            let mut c = vec![primary.as_str()];
            for fb in &self.fallback_order {
                if fb != &primary {
                    c.push(fb);
                }
            }
            c
        };

        // First pass: skip dead providers entirely.
        for provider_name in &chain {
            if let Some(b) = self.breakers.get(*provider_name) {
                if b.health == ProviderHealth::Dead && !b.should_probe() {
                    continue;
                }
            }
            if let Some(rl) = self.rate_limiters.get_mut(*provider_name) {
                if !rl.check() {
                    continue;
                }
            }
            if let Some(cfg) = self.providers.get(*provider_name) {
                return Ok(cfg.clone());
            }
        }

        // Second pass: probe dead providers that are eligible.
        for provider_name in &chain {
            if let Some(b) = self.breakers.get_mut(*provider_name) {
                if b.health == ProviderHealth::Dead && b.should_probe() {
                    b.probe();
                    if let Some(rl) = self.rate_limiters.get_mut(*provider_name) {
                        if !rl.check() {
                            continue;
                        }
                    }
                    if let Some(cfg) = self.providers.get(*provider_name) {
                        return Ok(cfg.clone());
                    }
                }
            }
        }

        Err(format!(
            "all providers exhausted for role '{}' (breaker-gated)",
            model_role
        ))
    }

    /// Build an EpsilonGreedyRouter from current provider state and select
    /// an endpoint, recording the result. Returns the selected provider config.
    pub fn route_with_exploration(
        &mut self,
        model_role: &str,
        epsilon: f64,
    ) -> Result<ProviderConfig, String> {
        // Build endpoints from providers that are not Dead (unless probeable).
        let mut greedy = EpsilonGreedyRouter::new(epsilon);
        for (name, cfg) in &self.providers {
            let breaker = self.breakers.get(name).cloned().unwrap_or_default();
            let latency = self.latency_trackers.get(name).cloned().unwrap_or_default();
            greedy.add_endpoint(ProviderEndpoint {
                config: cfg.clone(),
                breaker,
                latency,
            });
        }

        // If the primary provider is in the greedy set, prefer it unless
        // exploration dictates otherwise.
        if let Some(endpoint) = greedy.select() {
            let pname = endpoint.config.name.clone();
            if let Some(rl) = self.rate_limiters.get_mut(&pname) {
                if rl.check() {
                    rl.tick();
                } else {
                    // Rate-limited — fall through to fallback chain
                    return self.resolve_with_breaker(model_role);
                }
            }
            if let Some(cfg) = self.providers.get(&pname) {
                return Ok(cfg.clone());
            }
        }

        self.resolve_with_breaker(model_role)
    }
}

impl Default for PerModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── LlmRouter ───

pub struct LlmRouter {
    router: PerModelRouter,
    request_cache: LruCache<String, String>,
    stats: RouterStats,
}

impl LlmRouter {
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            router: PerModelRouter::new(),
            request_cache: LruCache::new(
                std::num::NonZeroUsize::new(cache_capacity)
                    .unwrap_or(std::num::NonZeroUsize::new(1).unwrap()),
            ),
            stats: RouterStats::default(),
        }
    }

    pub fn with_free_tier_defaults() -> Self {
        Self::new(128)
    }

    /// Main entry point: resolve a provider for the given model role, then
    /// return the provider config so the caller can dispatch the actual HTTP
    /// call.  The `request_body` is used for rate-limit bookkeeping.
    pub fn route_request(
        &mut self,
        model_role: &str,
        _request_body: &str,
    ) -> Result<ProviderConfig, RouterError> {
        self.stats.total_requests += 1;

        match self.router.resolve(model_role) {
            Ok(cfg) => {
                let pname = cfg.name.clone();
                if let Some(rl) = self.router.rate_limiters.get_mut(&pname) {
                    if rl.check() {
                        rl.tick();
                    } else {
                        self.stats.rate_limited += 1;
                        // Try fallback chain by resolving "fallback"
                        return self.fallback(cfg, model_role);
                    }
                }
                *self.stats.provider_breakdown.entry(pname).or_insert(0) += 1;
                Ok(cfg)
            }
            Err(e) => Err(RouterError::NoProviderAvailable(e)),
        }
    }

    fn fallback(
        &mut self,
        _failed: ProviderConfig,
        _original_role: &str,
    ) -> Result<ProviderConfig, RouterError> {
        self.stats.fallback_used += 1;
        // Resolve the explicit "fallback" route
        match self.router.resolve("fallback") {
            Ok(cfg) => {
                let pname = cfg.name.clone();
                if let Some(rl) = self.router.rate_limiters.get_mut(&pname) {
                    if rl.check() {
                        rl.tick();
                    }
                }
                *self.stats.provider_breakdown.entry(pname).or_insert(0) += 1;
                Ok(cfg)
            }
            Err(e) => Err(RouterError::NoProviderAvailable(format!(
                "fallback also failed: {}",
                e
            ))),
        }
    }

    pub fn check_cache(&mut self, request_hash: &str) -> Option<String> {
        self.request_cache.get(request_hash).cloned()
    }

    pub fn cache_response(&mut self, request_hash: String, response: String) {
        self.request_cache.put(request_hash, response);
    }

    pub fn stats_report(&self) -> RouterStats {
        self.stats.clone()
    }

    pub fn router_mut(&mut self) -> &mut PerModelRouter {
        &mut self.router
    }

    /// Dispatch an LLM request to the resolved provider.
    /// Returns the response text on success.
    pub async fn call_llm(
        &mut self,
        model_role: &str,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String, RouterError> {
        // Check semantic cache first
        let cache_key = Self::compute_cache_key(model_role, &messages);
        if let Some(cached) = self.check_cache(&cache_key) {
            self.stats.cached_responses += 1;
            return Ok(cached);
        }

        // Resolve provider
        let cfg = self.route_request(model_role, "")?;

        // Build request body
        let request_body = ChatCompletionRequest {
            model: cfg.models.first().cloned().unwrap_or_else(|| "default".into()),
            messages,
            temperature,
            max_tokens,
            stream: Some(false),
        };

        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let client = crate::neotrix::nt_io_http_factory::global_client();

        let mut req = client.post(&url).json(&request_body);
        if !cfg.api_key.is_empty() {
            req = req.bearer_auth(&cfg.api_key);
        }

        let start = Instant::now();
        let resp = req.send().await.map_err(|e| {
            self.record_provider_failure(&cfg.name);
            RouterError::NoProviderAvailable(format!("HTTP error: {}", e))
        })?;

        let latency_ms = start.elapsed().as_millis() as f64;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            self.record_provider_failure(&cfg.name);
            return Err(RouterError::NoProviderAvailable(format!(
                "Provider {} returned {}: {}",
                cfg.name, resp.status(), err_text
            )));
        }

        let data: ChatCompletionResponse = resp.json().await.map_err(|e| {
            self.record_provider_failure(&cfg.name);
            RouterError::NoProviderAvailable(format!("JSON parse error: {}", e))
        })?;

        self.record_provider_success(&cfg.name, latency_ms);

        let response_text = data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Cache the response
        self.cache_response(cache_key, response_text.clone());

        Ok(response_text)
    }

    fn compute_cache_key(model_role: &str, messages: &[ChatMessage]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        model_role.hash(&mut hasher);
        for m in messages {
            m.role.hash(&mut hasher);
            m.content.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    fn record_provider_success(&mut self, provider_name: &str, latency_ms: f64) {
        if let Some(rl) = self.router.rate_limiters.get_mut(provider_name) {
            // success doesn't affect rate limiter
        }
        if let Some(b) = self.router.breakers.get_mut(provider_name) {
            b.record_success();
        }
        if let Some(lt) = self.router.latency_trackers.get_mut(provider_name) {
            lt.record(latency_ms);
        }
    }

    fn record_provider_failure(&mut self, provider_name: &str) {
        if let Some(b) = self.router.breakers.get_mut(provider_name) {
            b.record_failure();
        }
    }
}

// ─── OpenAI-compatible Chat Types ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// Re-export for external use
pub use self::{ChatMessage, ChatCompletionRequest, ChatCompletionResponse, ChatChoice, Usage};

// ─── ProviderHealth State Machine ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Dead,
}

/// Circuit breaker with half-open probes for provider health tracking.
/// Trips to Degraded after `failure_threshold` consecutive failures,
/// to Dead after `failure_threshold * 2`. Probes after
/// `half_open_probe_interval` seconds in Dead state.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub health: ProviderHealth,
    pub consecutive_failures: u32,
    pub failure_threshold: u32,
    pub half_open_probe_interval: u64,
    pub last_failure: Instant,
    pub last_probe: Instant,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            health: ProviderHealth::Healthy,
            consecutive_failures: 0,
            failure_threshold: 3,
            half_open_probe_interval: 30,
            last_failure: Instant::now(),
            last_probe: Instant::now(),
        }
    }

    pub fn with_threshold(failure_threshold: u32, probe_interval_secs: u64) -> Self {
        Self {
            health: ProviderHealth::Healthy,
            consecutive_failures: 0,
            failure_threshold,
            half_open_probe_interval: probe_interval_secs,
            last_failure: Instant::now(),
            last_probe: Instant::now(),
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.health = ProviderHealth::Healthy;
    }

    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure = Instant::now();
        if self.consecutive_failures >= self.failure_threshold * 2 {
            self.health = ProviderHealth::Dead;
        } else if self.consecutive_failures >= self.failure_threshold {
            self.health = ProviderHealth::Degraded;
        }
    }

    pub fn should_probe(&self) -> bool {
        if self.health != ProviderHealth::Dead {
            return false;
        }
        self.last_probe.elapsed() >= std::time::Duration::from_secs(self.half_open_probe_interval)
    }

    pub fn probe(&mut self) {
        self.health = ProviderHealth::Degraded;
        self.last_probe = Instant::now();
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Latency Tracker ───

#[derive(Debug, Clone)]
pub struct LatencyTracker {
    pub latencies: VecDeque<f64>,
    pub max_samples: usize,
}

impl LatencyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            latencies: VecDeque::with_capacity(max_samples.min(1)),
            max_samples,
        }
    }

    pub fn record(&mut self, latency_ms: f64) {
        if self.latencies.len() >= self.max_samples {
            self.latencies.pop_front();
        }
        self.latencies.push_back(latency_ms);
    }

    /// Median of the rolling window.
    pub fn p50(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.latencies.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// 99th percentile of the rolling window.
    pub fn p99(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.latencies.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((sorted.len() as f64) * 0.99).ceil() as usize - 1;
        let idx = idx.min(sorted.len() - 1);
        sorted[idx]
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

// ─── ProviderEndpoint — merged health + latency + config ───

#[derive(Debug, Clone)]
pub struct ProviderEndpoint {
    pub config: ProviderConfig,
    pub breaker: CircuitBreaker,
    pub latency: LatencyTracker,
}

// ─── EpsilonGreedy Router ───

#[derive(Debug, Clone)]
pub struct EpsilonGreedyRouter {
    pub epsilon: f64,
    pub rng: Cell<u64>,
    pub provider_endpoints: Vec<ProviderEndpoint>,
}

impl EpsilonGreedyRouter {
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon,
            rng: Cell::new(42),
            provider_endpoints: Vec::new(),
        }
    }

    pub fn with_default_exploration() -> Self {
        Self::new(0.1)
    }

    fn next_f64(&self) -> f64 {
        let mut state = self.rng.get();
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        self.rng.set(state);
        (state as f64) / (u64::MAX as f64)
    }

    pub fn add_endpoint(&mut self, endpoint: ProviderEndpoint) {
        self.provider_endpoints.push(endpoint);
    }

    pub fn provider_count(&self) -> usize {
        self.provider_endpoints.len()
    }

    /// 90% exploit (lowest P50), 10% explore (random healthy).
    pub fn select(&self) -> Option<&ProviderEndpoint> {
        if self.provider_endpoints.is_empty() {
            return None;
        }

        if self.next_f64() < self.epsilon {
            let healthy: Vec<usize> = self
                .provider_endpoints
                .iter()
                .enumerate()
                .filter(|(_, ep)| ep.breaker.health != ProviderHealth::Dead)
                .map(|(i, _)| i)
                .collect();
            return if healthy.is_empty() {
                Some(&self.provider_endpoints[0])
            } else {
                let idx =
                    healthy[(self.next_f64() * healthy.len() as f64) as usize % healthy.len()];
                Some(&self.provider_endpoints[idx])
            };
        }

        let mut best_idx = None;
        let mut best_latency = f64::MAX;
        for (i, ep) in self.provider_endpoints.iter().enumerate() {
            if ep.breaker.health == ProviderHealth::Dead {
                continue;
            }
            let p50 = ep.latency.p50();
            if p50 < best_latency {
                best_latency = p50;
                best_idx = Some(i);
            }
        }

        match best_idx {
            Some(i) => Some(&self.provider_endpoints[i]),
            None if !self.provider_endpoints.is_empty() => Some(&self.provider_endpoints[0]),
            None => None,
        }
    }

    pub fn record_result(&mut self, provider_name: &str, latency_ms: f64, success: bool) {
        for ep in &mut self.provider_endpoints {
            if ep.config.name == provider_name {
                ep.latency.record(latency_ms);
                if success {
                    ep.breaker.record_success();
                } else {
                    ep.breaker.record_failure();
                }
                return;
            }
        }
    }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_free_tier_has_ollama_and_nvidia() {
        let router = PerModelRouter::with_default_free_tier();
        let summary = router.providers_summary();
        let names: Vec<&str> = summary.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"ollama"), "ollama should be registered");
        assert!(
            names.contains(&"nvidia-nim"),
            "nvidia-nim should be registered"
        );
        assert!(
            names.contains(&"google-ai-studio"),
            "google-ai-studio should be registered"
        );
    }

    #[test]
    fn test_route_model_and_resolve() {
        let mut router = PerModelRouter::new();

        // The default route (sonnet) should resolve to nvidia-nim.
        let cfg = router.resolve("sonnet").expect("sonnet should resolve");
        assert_eq!(cfg.name, "nvidia-nim");

        // Custom route: map "opus" to "google-ai-studio".
        router.route_model("opus", "google-ai-studio");
        let cfg = router.resolve("opus").expect("opus should resolve");
        assert_eq!(cfg.name, "google-ai-studio");
    }

    #[test]
    fn test_rate_limiter_blocks_after_limit() {
        let mut rl = SimpleRateLimiter::new(3, 60);
        assert!(rl.check(), "should allow first request");
        rl.tick();
        assert!(rl.check(), "should allow second request");
        rl.tick();
        assert!(rl.check(), "should allow third request");
        rl.tick();
        // Fourth should be blocked
        assert!(!rl.check(), "should block fourth request");
    }

    #[test]
    fn test_fallback_chain_when_provider_unavailable() {
        // Remove nvidia-nim from defaults so sonnet falls back.
        let mut router = PerModelRouter::new();
        // Remove and re-add with a very strict rate limiter.
        let nvidia = ProviderConfig {
            name: "nvidia-nim".into(),
            provider_type: LlmProviderType::NvidiaNim,
            base_url: "https://ai.api.nvidia.com/v1".into(),
            api_key: String::new(),
            models: vec!["meta/llama3.1-405b-instruct".into()],
            rate_limit_rpm: 0, // zero — always rate-limited
            cost_per_1k_tokens: 0.0,
        };
        router.register_provider(nvidia);
        router.route_model("sonnet", "nvidia-nim");

        // sonnet should skip nvidia-nim (rate-limited) and fall through to
        // the next provider in the fallback order.
        let cfg = router.resolve("sonnet").expect("should fallback");
        // The fallback chain starts with ollama, so we should get ollama.
        assert_eq!(cfg.name, "ollama", "fallback should give ollama");
    }

    #[test]
    fn test_cache_hit_and_miss() {
        let mut router = LlmRouter::with_free_tier_defaults();

        // Miss
        assert!(router.check_cache("hash-1").is_none());

        // Store
        router.cache_response("hash-1".into(), "cached response".into());

        // Hit
        let cached = router.check_cache("hash-1");
        assert_eq!(cached, Some("cached response".into()));
    }

    #[test]
    fn test_providers_summary_format() {
        let router = PerModelRouter::new();
        let summary = router.providers_summary();
        for (name, count, tier) in &summary {
            assert!(!name.is_empty(), "provider name should not be empty");
            assert!(*count > 0, "{} should have at least one model", name);
            assert!(
                tier == "free" || tier.starts_with("paid"),
                "tier should be 'free' or 'paid'"
            );
        }
    }

    #[test]
    fn test_llm_router_stats_tracking() {
        let mut router = LlmRouter::with_free_tier_defaults();
        // Resolve sonnet (should succeed)
        let _ = router.route_request("sonnet", "hello");
        let stats = router.stats_report();
        assert_eq!(stats.total_requests, 1);
        assert!(!stats.provider_breakdown.is_empty());
    }

    #[test]
    fn test_rate_limiter_evicts_stale() {
        let mut rl = SimpleRateLimiter::new(2, 0); // window of 0 seconds
        rl.tick();
        rl.tick();
        // With 0-second window, stale entries should be immediately evictable.
        assert!(rl.check(), "should allow after eviction");
    }

    #[test]
    fn test_circuit_breaker_tripping() {
        let mut cb = CircuitBreaker::with_threshold(2, 30);
        assert_eq!(cb.health, ProviderHealth::Healthy);

        cb.record_failure();
        assert_eq!(
            cb.health,
            ProviderHealth::Healthy,
            "one failure should not trip"
        );

        cb.record_failure();
        assert_eq!(
            cb.health,
            ProviderHealth::Degraded,
            "two failures → Degraded"
        );

        cb.record_failure();
        assert_eq!(
            cb.health,
            ProviderHealth::Degraded,
            "third failure still Degraded (threshold*2=4)"
        );

        cb.record_failure();
        assert_eq!(cb.health, ProviderHealth::Dead, "fourth failure → Dead");

        // Success resets to healthy
        cb.record_success();
        assert_eq!(cb.health, ProviderHealth::Healthy);
        assert_eq!(cb.consecutive_failures, 0);
    }

    #[test]
    fn test_latency_p50() {
        let mut lt = LatencyTracker::new(10);
        assert_eq!(lt.p50(), 0.0, "empty tracker → 0.0");

        lt.record(10.0);
        lt.record(20.0);
        lt.record(30.0);
        assert!((lt.p50() - 20.0).abs() < 1e-9, "median of [10,20,30] = 20");

        lt.record(40.0);
        // Sorted: [10, 20, 30, 40] → median = (20+30)/2 = 25
        assert!(
            (lt.p50() - 25.0).abs() < 1e-9,
            "median of [10,20,30,40] = 25"
        );

        // P99 of 4 elements: ceil(4*0.99)=4 → idx=3 → 40
        assert!((lt.p99() - 40.0).abs() < 1e-9, "p99 should be 40");
    }

    #[test]
    fn test_epsilon_greedy_explore_rate() {
        let mut greedy = EpsilonGreedyRouter::new(1.0); // 100% explore
        let cfg1 = ProviderConfig {
            name: "fast".into(),
            provider_type: LlmProviderType::Ollama,
            base_url: "http://localhost:11434/v1".into(),
            api_key: String::new(),
            models: vec!["m1".into()],
            rate_limit_rpm: 100,
            cost_per_1k_tokens: 0.0,
        };
        let cfg2 = ProviderConfig {
            name: "slow".into(),
            provider_type: LlmProviderType::NvidiaNim,
            base_url: "http://localhost:9999/v1".into(),
            api_key: String::new(),
            models: vec!["m2".into()],
            rate_limit_rpm: 100,
            cost_per_1k_tokens: 0.0,
        };

        greedy.add_endpoint(ProviderEndpoint {
            config: cfg1,
            breaker: CircuitBreaker::new(),
            latency: LatencyTracker::new(10),
        });
        greedy.add_endpoint(ProviderEndpoint {
            config: cfg2,
            breaker: CircuitBreaker::new(),
            latency: LatencyTracker::new(10),
        });

        // With 100% exploration, run 20 selections and confirm at least one of each
        let mut seen_fast = false;
        let mut seen_slow = false;
        for _ in 0..20 {
            if let Some(ep) = greedy.select() {
                if ep.config.name == "fast" {
                    seen_fast = true;
                }
                if ep.config.name == "slow" {
                    seen_slow = true;
                }
            }
        }
        assert!(seen_fast, "fast should be selected at least once");
        assert!(seen_slow, "slow should be selected at least once");
    }

    #[test]
    fn test_provider_health_state_machine() {
        let mut cb = CircuitBreaker::with_threshold(3, 30);
        assert_eq!(cb.health, ProviderHealth::Healthy);

        // 3 failures → Degraded
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.health, ProviderHealth::Degraded);

        // 3 more → Dead (3*2=6)
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.health, ProviderHealth::Dead);

        // should_probe = false (interval not elapsed)
        assert!(!cb.should_probe(), "should not probe immediately");
    }

    #[test]
    fn test_resolve_with_breaker_healthy_only() {
        let mut router = PerModelRouter::new();

        // Normal resolve works
        let cfg = router.resolve_with_breaker("sonnet");
        assert!(cfg.is_ok(), "sonnet should resolve with healthy breaker");

        // Mark nvidia-nim as dead
        if let Some(b) = router.breakers.get_mut("nvidia-nim") {
            b.health = ProviderHealth::Dead;
            b.consecutive_failures = 99;
        }

        // Resolve should skip dead nvidia-nim and fall through to ollama
        let cfg = router
            .resolve_with_breaker("sonnet")
            .expect("should fallback past dead");
        assert_eq!(
            cfg.name, "ollama",
            "should skip dead nvidia-nim and pick ollama"
        );
    }
}
