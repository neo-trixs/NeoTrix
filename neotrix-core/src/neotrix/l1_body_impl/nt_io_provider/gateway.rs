use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

use super::circuit_breaker::{BreakerState, CircuitBreaker};
use super::free_pool::global_free_pool;
use super::provider_catalog::{ProviderCategory, CommunicationProfile};
use super::rate_limiter::RateLimiter;
use super::rate_profiles::get_rate_profile;
use super::types::*;
use crate::core::nt_core_error_recovery::{ErrorContext, ErrorType, RecoveryAction, RecoveryConfig, RecoveryOrchestrator};
use crate::core::nt_io_cache::{CacheConfig, SemanticCache};
use crate::core::nt_io_telemetry::{ConsoleTracer, CostTracker, SpanKind, Tracer};

#[derive(Debug)]
pub struct ProviderState {
    pub circuit_breaker: CircuitBreaker,
    pub rate_limiter: RateLimiter,
    pub success_ema: f64,
    pub latency_window: Vec<f64>,
    pub total_calls: u64,
    pub total_errors: u64,
    pub total_tokens: u64,
    pub cost_per_1k_tokens: f64,
    pub is_free: bool,
    pub category: ProviderCategory,
}

impl ProviderState {
    pub fn new(is_free: bool, category: ProviderCategory) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::default(),
            rate_limiter: RateLimiter::default(),
            success_ema: 0.8,
            latency_window: Vec::with_capacity(100),
            total_calls: 0,
            total_errors: 0,
            total_tokens: 0,
            cost_per_1k_tokens: if is_free { 0.0 } else { 0.01 },
            is_free,
            category,
        }
    }

    pub fn is_available(&self) -> bool {
        self.circuit_breaker.is_available()
    }

    pub fn composite_score(&self) -> f64 {
        let health = self.circuit_breaker.health_penalty();
        if health <= 0.0 { return 0.0; }
        let latency_factor = {
            let mut sorted = self.latency_window.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p95 = sorted.get((sorted.len() as f64 * 0.95) as usize)
                .copied().unwrap_or(1000.0);
            if p95 > 0.0 { 1.0 / (p95 / 1000.0).max(0.1) } else { 0.5 }
        };
        let cost_factor = if self.cost_per_1k_tokens > 0.0 {
            1.0 / (self.cost_per_1k_tokens * 10.0 + 1.0)
        } else {
            1.0
        };
        // 分类优先级: Local (+0.3) > Proxy (+0.15) > Cloud (+0.0)
        let category_boost = match self.category {
            ProviderCategory::Local => 0.3,
            ProviderCategory::Proxy => 0.15,
            ProviderCategory::Cloud => 0.0,
        };
        (self.success_ema.powi(2) * latency_factor * cost_factor * health) + category_boost
    }

    pub fn record_success(&mut self, latency_ms: f64, tokens: u32) {
        self.circuit_breaker.on_success();
        self.total_calls += 1;
        self.total_tokens += tokens as u64;
        self.success_ema = self.success_ema * 0.9 + 1.0 * 0.1;
        self.latency_window.push(latency_ms);
        if self.latency_window.len() > 100 {
            self.latency_window.remove(0);
        }
    }

    pub fn record_failure(&mut self, latency_ms: f64) {
        self.circuit_breaker.on_failure();
        self.total_calls += 1;
        self.total_errors += 1;
        self.success_ema = self.success_ema * 0.9 + 0.0 * 0.1;
        self.latency_window.push(latency_ms);
        if self.latency_window.len() > 100 {
            self.latency_window.remove(0);
        }
    }
}

/// An event fired after each provider call attempt.
#[derive(Debug, Clone)]
pub struct CallEvent {
    pub provider_name: String,
    pub success: bool,
    pub latency_ms: f64,
    pub tokens: u32,
    pub model: String,
    pub attempt_phase: AttemptPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptPhase {
    Normal,
    AggressiveRetry,
}

pub type CallObserver = std::sync::Arc<dyn Fn(CallEvent) + Send + Sync>;

/// 子网格: 由同一安全画像的 provider 组成的小循环通信单元 (子母阵基本单元)
/// 每个子网格包含满足特定 CommunicationProfile 的 provider 组合
#[derive(Debug, Clone)]
pub struct SubGrid {
    /// 子网格名称 (如 "anonymous-local", "proxied-cloud", "tor-anonymous")
    pub name: String,
    /// 安全画像: 该子网格满足的通信安全级别
    pub security_profile: CommunicationProfile,
    /// 包含的 provider 名称列表
    pub provider_names: Vec<String>,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

impl SubGrid {
    /// 创建新子网格
    pub fn new(name: String, security_profile: CommunicationProfile, provider_names: Vec<String>) -> Self {
        Self {
            name,
            security_profile,
            provider_names,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// 检查子网格是否满足要求的安全级别
    pub fn meets_profile(&self, required: CommunicationProfile) -> bool {
        self.security_profile.meets(required)
    }
}

pub struct GatewayV2 {
    providers: HashMap<String, Box<dyn LlmProvider>>,
    states: RwLock<HashMap<String, ProviderState>>,
    default_name: RwLock<String>,
    prefer_free: bool,
    observer: RwLock<Option<CallObserver>>,
    pub tracer: RwLock<Option<ConsoleTracer>>,
    pub cost_tracker: RwLock<Option<CostTracker>>,
    pub recovery: RwLock<RecoveryOrchestrator>,
    pub cache: Mutex<SemanticCache>,
    pub cost_budget_per_query: f64,
    /// 组合的子网格映射: 子网格名称 -> SubGrid
    /// 支持动态组合已有节点能力构建安全隐匿的通信小循环
    sub_grids: RwLock<HashMap<String, SubGrid>>,
}

impl GatewayV2 {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            states: RwLock::new(HashMap::new()),
            default_name: RwLock::new(String::new()),
            prefer_free: false,
            observer: RwLock::new(None),
            tracer: RwLock::new(None),
            cost_tracker: RwLock::new(None),
            recovery: RwLock::new(RecoveryOrchestrator::new(RecoveryConfig::default())),
            cache: Mutex::new(SemanticCache::new(CacheConfig::default())),
            cost_budget_per_query: 0.02,
            sub_grids: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_observer(&self, observer: CallObserver) {
        if let Ok(mut guard) = self.observer.write() {
            *guard = Some(observer);
        }
    }

    pub fn set_cost_tracker(&self, tracker: CostTracker) {
        if let Ok(mut guard) = self.cost_tracker.write() {
            *guard = Some(tracker);
        }
    }

    pub fn set_cost_budget(&mut self, budget: f64) {
        self.cost_budget_per_query = budget;
    }

    // ═══════════════════════════════════════════════════════════════════
    // SubGrid Composition (子母阵动态组合)
    // 通过组合已有 provider 节点，构建满足指定通信安全级别的小循环子网格
    // ═══════════════════════════════════════════════════════════════════

    /// 动态组合一个子网格: 从已注册 providers 中选出满足安全画像的子集
    /// `security_profile` 指定目标安全级别; `include_free_only` 限制只组合免费 provider
    pub fn compose_sub_grid(&self, name: &str, security_profile: CommunicationProfile, include_free_only: bool) -> SubGrid {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let mut provider_names: Vec<String> = states.iter()
            .filter(|(_, s)| {
                let profile_ok = s.category.default_security_profile().meets(security_profile);
                let free_ok = !include_free_only || s.is_free;
                profile_ok && free_ok
            })
            .map(|(name, _)| name.clone())
            .collect();
        provider_names.sort();
        let grid = SubGrid::new(name.to_string(), security_profile, provider_names);
        if let Ok(mut grids) = self.sub_grids.write() {
            grids.insert(name.to_string(), grid.clone());
        }
        grid
    }

    /// 列出所有已组合的子网格
    pub fn list_sub_grids(&self) -> Vec<SubGrid> {
        match self.sub_grids.read() {
            Ok(guard) => guard.values().cloned().collect(),
            Err(e) => {
                log::warn!("[gateway] sub_grids RwLock poisoned: {}", e);
                e.into_inner().values().cloned().collect()
            }
        }
    }

    /// 根据安全画像选择最佳 provider:
    /// 优先从满足安全级别的子网格中选取可用 provider
    pub async fn select_best_for_profile(&self, required: CommunicationProfile) -> Option<String> {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        // Tier 1: 满足安全级别的免费 provider
        let best_free = states.iter()
            .filter(|(_, s)| s.is_available() && s.is_free && s.category.default_security_profile().meets(required))
            .max_by(|(_, a), (_, b)| {
                a.composite_score().partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone());
        if best_free.is_some() {
            return best_free;
        }
        // Tier 2: 满足安全级别的任意 provider
        states.iter()
            .filter(|(_, s)| s.is_available() && s.category.default_security_profile().meets(required))
            .max_by(|(_, a), (_, b)| {
                a.composite_score().partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone())
    }

    /// 完整请求路由入口 — 通过满足安全画像的子网格完成一次 LLM 调用。
    /// 子母阵动态增幅的公共消费点 (R-P79 接线): 调用方声明所需安全级别，
    /// 网关自动从匹配子网格中选择 provider 并执行 complete()。
    /// 无匹配 provider 时回退到默认 select_best()，保证通信始终畅通。
    pub async fn complete_for_profile(
        &self,
        required: CommunicationProfile,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        match self.select_best_for_profile(required).await {
            Some(name) => {
                log::debug!("[gateway] complete_for_profile({:?}) → {}", required, name);
                self.call_provider(&name, request).await
            }
            None => {
                log::warn!("[gateway] no provider meets {:?} — falling back to default selection", required);
                self.complete_with_selection(request).await
            }
        }
    }

    /// 返回当前已组成的子网格中满足给定安全级别的网格名称
    pub fn sub_grids_meeting(&self, required: CommunicationProfile) -> Vec<String> {
        self.list_sub_grids().into_iter()
            .filter(|sg| sg.security_profile.meets(required))
            .map(|sg| sg.name)
            .collect()
    }

    fn fire_event(&self, provider_name: &str, success: bool, latency_ms: f64, tokens: u32, model: &str, phase: AttemptPhase) {
        if let Ok(guard) = self.observer.read() {
            if let Some(ref obs) = *guard {
                obs(CallEvent {
                    provider_name: provider_name.to_string(),
                    success,
                    latency_ms,
                    tokens,
                    model: model.to_string(),
                    attempt_phase: phase,
                });
            }
        }
    }

    // ── Safe RwLock helpers (poison-resistant) ──
    fn states_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<String, ProviderState>) -> R,
    {
        match self.states.write() {
            Ok(mut guard) => f(&mut guard),
            Err(e) => {
                log::warn!("[gateway] states RwLock poisoned (write): {}", e);
                let mut recovered = e.into_inner();
                f(&mut recovered)
            }
        }
    }

    fn default_name_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut String) -> R,
    {
        match self.default_name.write() {
            Ok(mut guard) => f(&mut guard),
            Err(e) => {
                log::warn!("[gateway] default_name RwLock poisoned: {}", e);
                let mut recovered = e.into_inner();
                f(&mut recovered)
            }
        }
    }

    pub fn register_provider(&mut self, name: &str, provider: Box<dyn LlmProvider>, is_free: bool) {
        self.register_provider_with_category(name, provider, is_free, ProviderCategory::Cloud)
    }

    pub fn register_provider_with_category(&mut self, name: &str, provider: Box<dyn LlmProvider>, is_free: bool, category: ProviderCategory) {
        self.providers.insert(name.to_string(), provider);
        self.states_write(|states| {
            let mut state = ProviderState::new(is_free, category);
            // Apply provider-specific rate limits
            let profile = get_rate_profile(name);
            state.rate_limiter = RateLimiter::new(profile.rpm, profile.tpm, 3);
            states.insert(name.to_string(), state);
            if states.len() == 1 {
                self.default_name_write(|n| *n = name.to_string());
            }
        });
    }

    pub async fn select_best(&self) -> Option<String> {
        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });

        // Tier 1: Available free providers (preferred)
        let free_best = states.iter()
            .filter(|(_, s)| s.is_available() && s.is_free)
            .max_by(|(_, a), (_, b)| {
                a.composite_score().partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone());

        if free_best.is_some() {
            return free_best;
        }

        // Tier 2: Available paid providers if free-first is off OR all free exhausted
        if !self.prefer_free {
            return states.iter()
                .filter(|(_, s)| s.is_available())
                .max_by(|(_, a), (_, b)| {
                    a.composite_score().partial_cmp(&b.composite_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(name, _)| name.clone());
        }

        // Tier 3: Any available provider (free-first exhausted all free, allow paid)
        states.iter()
            .filter(|(_, s)| s.is_available())
            .max_by(|(_, a), (_, b)| {
                a.composite_score().partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone())
    }

    async fn call_provider(&self, name: &str, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let provider = self.providers.get(name)
            .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;
        provider.complete(request).await
    }

    pub async fn complete_with_selection(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // Build prompt key for cache lookup
        let prompt_key: String = request.messages.iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>().join("\n");

        // Check semantic cache (Layer 1: exact match)
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_exact(&request.model, &prompt_key) {
                if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                    return Ok(response);
                }
            }
        }

        // Create telemetry span for this multi-provider attempt
        let mut telemetry_span = match self.tracer.read() {
            Ok(guard) => guard.as_ref().map(|t| {
                let span = t.start_span("llm.complete", SpanKind::Llm);
                span.set_gen_ai_request_model(&request.model);
                span.set_gen_ai_system("neotrix-gateway");
                span
            }),
            _ => None,
        };

        // Phase 1: Normal retry loop (up to 3 providers, best-first)
        // Check if we've exceeded the per-query cost budget
        {
            if let Ok(guard) = self.cost_tracker.read() {
                if let Some(ref ct) = *guard {
                    let total_tokens = ct.total_prompt_tokens.load(std::sync::atomic::Ordering::Relaxed)
                        + ct.total_completion_tokens.load(std::sync::atomic::Ordering::Relaxed);
                    let estimated_cost = (total_tokens as f64 / 1000.0) * 0.002;
                    if estimated_cost > self.cost_budget_per_query && self.cost_budget_per_query > 0.0 {
                        log::warn!("[gateway] Per-query cost budget exceeded: ${:.4} > ${:.4}", estimated_cost, self.cost_budget_per_query);
                        return Err(LlmError::Unknown(format!("Cost budget exceeded: ${:.4} > ${:.4}", estimated_cost, self.cost_budget_per_query)));
                    }
                }
            }
        }
        let mut used_names: Vec<String> = Vec::new();

        for _ in 0..3 {
            let name = self.select_best().await
                .or_else(|| {
                    let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                    states.keys().next().cloned()
                });

            let name = match name {
                Some(n) if !used_names.contains(&n) => n,
                _ => break,
            };
            used_names.push(name.clone());

            let start = Instant::now();

            // Check rate limit
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(&name) {
                    if !state.rate_limiter.allow_request(20.0) {
                        continue;
                    }
                }
            }

            // Call provider
            let result = self.call_provider(&name, request).await;
            let elapsed = start.elapsed().as_millis() as f64;

            match result {
                Ok(response) => {
                    let token_count = response.usage.total_tokens;
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_success(elapsed, token_count);
                            state.rate_limiter.record_usage(token_count as f64);
                        }
                    }
                    self.fire_event(&name, true, elapsed, token_count, &request.model, AttemptPhase::Normal);
                    // End telemetry span on success
                    if let Ok(guard) = self.tracer.read() {
                        if let Some(tracer) = guard.as_ref() {
                            if let Some(span) = telemetry_span.take() {
                                tracer.end_span(span);
                            }
                        }
                    }
                    // Track usage in global free pool
                    {
                        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get(&name) {
                            if state.is_free {
                                global_free_pool().record_usage(&name, token_count as u64);
                            }
                        }
                    }
                    if let Ok(mut ct) = self.cost_tracker.write() {
                        if let Some(ref mut tracker) = *ct {
                            tracker.record(&request.model, response.usage.prompt_tokens.into(), response.usage.completion_tokens.into());
                        }
                    }
                    // Store in semantic cache
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            cache.set_exact(&request.model, &prompt_key, serialized);
                        }
                    }
                    return Ok(response);
                }
                Err(err) => {
                    let error_msg = err.to_string();
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_failure(elapsed);
                        }
                    }
                    self.fire_event(&name, false, elapsed, 0, &request.model, AttemptPhase::Normal);

                    // Consult recovery orchestrator
                    let error_type = if error_msg.contains("rate limit") || error_msg.contains("429") {
                        ErrorType::RateLimit { retry_after: None }
                    } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                        ErrorType::Timeout { elapsed_ms: elapsed as u64 }
                    } else if error_msg.contains("50") || error_msg.contains("server error") {
                        ErrorType::ServerError { code: 500 }
                    } else {
                        ErrorType::Unknown(error_msg.clone())
                    };
                    let ctx = ErrorContext {
                        error_type,
                        attempt: used_names.len(),
                        max_retries: 3,
                        model: request.model.clone(),
                        available_models: self.providers.keys().cloned().collect(),
                        prompt: request.messages.iter()
                            .map(|m| m.content.clone())
                            .collect::<Vec<_>>().join("\n"),
                        prompt_variants: Vec::new(),
                        state_snapshot: None,
                        token_budget_remaining: 0,
                        elapsed_ms: elapsed as u64,
                        metadata: HashMap::new(),
                    };
                    let action = self.recovery.write().unwrap_or_else(|e| { log::warn!("[gateway] recovery RwLock poisoned: {}", e); e.into_inner() }).handle(&ctx);
                    if let RecoveryAction::Retry { delay_ms, .. } = action {
                        if delay_ms > 0 {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        }
                        continue;
                    }
                }
            }
        }

        // Phase 2: Aggressive retry — all normal providers exhausted.
        // Temporarily override breaker states to HalfOpen for all Open-circuit providers,
        // reset failure thresholds, and retry every provider once more.
        let result = self.attempt_aggressive_retry(request).await;
        // End telemetry span
        if let Ok(guard) = self.tracer.read() {
            if let Some(tracer) = guard.as_ref() {
                if let Some(span) = telemetry_span.take() {
                    tracer.end_span(span);
                }
            }
        }
        // Record cost on success
        if let Ok(ref response) = result {
            if let Ok(mut ct) = self.cost_tracker.write() {
                if let Some(ref mut tracker) = *ct {
                    tracker.record(&request.model, response.usage.prompt_tokens.into(), response.usage.completion_tokens.into());
                }
            }
        }
        result
    }

    /// Aggressive fallback: when all providers have failed, temporarily
    /// override circuit breaker states (Open → HalfOpen, reduced cooldown)
    /// and retry every registered provider once more.
    async fn attempt_aggressive_retry(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let provider_names: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            states.keys().cloned().collect()
        };

        if provider_names.is_empty() {
            return Err(LlmError::Unknown("No providers available for aggressive retry".to_string()));
        }

        // Save original breaker states before override
        let set_aggressive: Vec<(String, u64)> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            provider_names.iter().filter_map(|name| {
                states.get(name).and_then(|s| {
                    if s.circuit_breaker.state() == BreakerState::Open {
                        let saved = s.circuit_breaker.half_open_max_probes();
                        Some((name.clone(), saved))
                    } else {
                        None
                    }
                })
            }).collect()
        };

        // Apply aggressive overrides
        {
            let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            for (name, _) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(5);
                    state.circuit_breaker.cooldown_reset();
                }
            }
        }

        // Try each provider once aggressively
        for name in &provider_names {
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(name) {
                    if !state.rate_limiter.allow_request(10.0) {
                        continue;
                    }
                }
            }

            let start = Instant::now();
            let result = self.call_provider(name, request).await;
            let elapsed = start.elapsed().as_millis() as f64;

            match result {
                Ok(response) => {
                    let token_count = response.usage.total_tokens;
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(name) {
                            state.record_success(elapsed, token_count);
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(name, true, elapsed, token_count, &request.model, AttemptPhase::AggressiveRetry);
                    // Track usage in global free pool
                    {
                        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get(name) {
                            if state.is_free {
                                global_free_pool().record_usage(name, token_count as u64);
                            }
                        }
                    }
                    // Store in semantic cache
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            let aggressive_key: String = request.messages.iter()
                                .map(|m| m.content.clone())
                                .collect::<Vec<_>>().join("\n");
                            cache.set_exact(&request.model, &aggressive_key, serialized);
                        }
                    }
                    return Ok(response);
                }
                Err(err) => {
                    log::warn!("[gateway] Aggressive retry failed for '{}': {}", name, err);
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(name) {
                            state.record_failure(elapsed);
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(name, false, elapsed, 0, &request.model, AttemptPhase::AggressiveRetry);
                }
            }
        }

        // All aggressive retries failed — restore saved probes
        {
            let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            for (name, saved) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(*saved);
                }
            }
        }

        Err(LlmError::Unknown("Aggressive retry exhausted — all providers failed".to_string()))
    }

    /// Stream completion with 2-phase retry (normal → aggressive), matching
    /// `complete_with_selection`'s fallback strategy but for streaming.
    pub async fn stream_complete_with_selection(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        // Phase 1: Normal retry loop (up to 3 providers, best-first)
        let mut used_names: Vec<String> = Vec::new();

        for _ in 0..3 {
            let name = self.select_best().await
                .or_else(|| {
                    self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }).keys().next().cloned()
                });

            let name = match name {
                Some(n) if !used_names.contains(&n) => n,
                _ => break,
            };
            used_names.push(name.clone());

            // Check rate limit
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(&name) {
                    if !state.rate_limiter.allow_request(20.0) {
                        continue;
                    }
                }
            }

            // Try to stream from this provider
            match self.call_provider_stream(&name, request).await {
                Ok(result) => {
                    self.fire_event(&name, true, 0.0, 0, &request.model, AttemptPhase::Normal);
                    return Ok(result);
                }
                Err(LlmError::RateLimit(_)) => {
                    // Rate limited by upstream — track failure and try next
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_failure(0.0);
                        }
                    }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    continue;
                }
                Err(_) => {
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_failure(0.0);
                        }
                    }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    if used_names.len() >= 3 {
                        break;
                    }
                }
            }
        }

        // Phase 2: Aggressive retry — temporarily override Open → HalfOpen
        self.attempt_aggressive_retry_stream(request).await
    }

    async fn call_provider_stream(
        &self,
        name: &str,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let provider = self.providers.get(name)
            .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;
        provider.stream_complete(request).await
    }

    /// Aggressive streaming fallback: temporarily override circuit breaker states
    /// and retry every registered provider once.
    async fn attempt_aggressive_retry_stream(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let provider_names: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            states.keys().cloned().collect()
        };

        if provider_names.is_empty() {
            return Err(LlmError::Unknown("No providers available for aggressive retry".to_string()));
        }

        // Save original breaker states before override
        let set_aggressive: Vec<(String, u64)> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            provider_names.iter().filter_map(|name| {
                states.get(name).and_then(|s| {
                    if s.circuit_breaker.state() == BreakerState::Open {
                        let saved = s.circuit_breaker.half_open_max_probes();
                        Some((name.clone(), saved))
                    } else {
                        None
                    }
                })
            }).collect()
        };

        // Apply aggressive overrides
        {
            let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            for (name, _) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(5);
                    state.circuit_breaker.cooldown_reset();
                }
            }
        }

        // Try each provider once aggressively
        for name in &provider_names {
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(name) {
                    if !state.rate_limiter.allow_request(10.0) {
                        continue;
                    }
                }
            }

            match self.call_provider_stream(name, request).await {
                Ok(result) => {
                    // Reset probes on success
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(name) {
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(name, true, 0.0, 0, &request.model, AttemptPhase::AggressiveRetry);
                    return Ok(result);
                }
                Err(_) => {
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(name) {
                            state.record_failure(0.0);
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(name, false, 0.0, 0, &request.model, AttemptPhase::AggressiveRetry);
                }
            }
        }

        // All aggressive retries failed — restore saved probes
        {
            let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            for (name, saved) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(*saved);
                }
            }
        }

        Err(LlmError::Unknown("Aggressive streaming retry exhausted — all providers failed".to_string()))
    }

    /// Register providers from FreeModelCatalog discovered entries.
    /// For each entry where the required API key env var is set (or keyless),
    /// create a provider and register it.
    pub fn register_from_catalog(&mut self, entries: &[super::free_catalog::FreeModelEntry]) {
        for entry in entries {
            let name = format!("{}/{}", entry.provider, entry.model_id);
            if self.providers.contains_key(&name) {
                continue; // already registered
            }
            // Check if we have the required API key
            let api_key = if entry.requires_api_key {
                if let Some(ref env_var) = entry.api_key_env {
                    match std::env::var(env_var) {
                        Ok(key) if !key.is_empty() => Some(key),
                        _ => continue, // skip — no key for this entry
                    }
                } else {
                    continue; // requires key but no env var specified
                }
            } else {
                None
            };
            let provider = super::factory::create_provider(super::factory::ProviderConfig {
                provider_type: entry.provider_type,
                api_key,
                base_url: Some(entry.base_url.clone()),
                model: Some(entry.model_id.clone()),
                timeout_secs: 60,
            });
            self.register_provider_with_category(&name, provider, entry.is_free, ProviderCategory::Cloud);
            log::info!("[gateway] Registered from catalog: {} ({})", name, entry.display_name);
        }
    }

    pub fn provider_status(&self) -> Vec<serde_json::Value> {
        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
        states.iter().map(|(name, state)| {
            serde_json::json!({
                "name": name,
                "available": state.is_available(),
                "circuit_state": format!("{:?}", state.circuit_breaker.state()),
                "success_rate": format!("{:.2}", state.success_ema),
                "total_calls": state.total_calls,
                "total_errors": state.total_errors,
                "is_free": state.is_free,
                "composite_score": format!("{:.4}", state.composite_score()),
            })
        }).collect()
    }
}

impl Default for GatewayV2 {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmProvider for GatewayV2 {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_with_selection(request).await
    }

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.stream_complete_with_selection(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_selects_free_provider() {
        let mut gw = GatewayV2::new();
        gw.register_provider("paid", Box::new(MockProvider::new("paid response")), false);
        gw.register_provider("free", Box::new(MockProvider::new("free response")), true);

        let selected = gw.select_best().await;
        assert_eq!(selected, Some("free".to_string()));
    }

    #[tokio::test]
    async fn test_gateway_fallback_on_failure() {
        let mut gw = GatewayV2::new();
        gw.register_provider("failing", Box::new(MockProvider::failing()), false);
        gw.register_provider("working", Box::new(MockProvider::new("ok")), true);

        let mut states = gw.states.write().unwrap();
        let f = states.get_mut("failing").unwrap();
        f.success_ema = 0.0;
        for _ in 0..5 { f.circuit_breaker.on_failure(); }
        drop(states);

        let selected = gw.select_best().await;
        assert_eq!(selected, Some("working".to_string()));
    }

    #[tokio::test]
    async fn test_gateway_rate_limit() {
        let mut gw = GatewayV2::new();
        gw.register_provider("limited", Box::new(MockProvider::new("ok")), true);

        let req = LlmRequest::new("test", "hello");
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_aggressive_retry_recovers_after_all_fail() {
        let mut gw = GatewayV2::new();
        // Register 2 failing providers — normal retry will exhaust both
        gw.register_provider("fail1", Box::new(MockProvider::failing()), false);
        gw.register_provider("fail2", Box::new(MockProvider::failing()), false);

        // Drive both to Open state
        {
            let mut states = gw.states.write().unwrap();
            for name in ["fail1", "fail2"] {
                let s = states.get_mut(name).unwrap();
                for _ in 0..6 { s.circuit_breaker.on_failure(); }
            }
        }

        let req = LlmRequest::new("test", "hello");
        // Normal retry exhausts 2 failing providers, aggressive retry tries again
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("Aggressive retry exhausted") || msg.contains("All providers failed"),
            "expected aggressive retry exhaustion error, got: {}", msg);
    }

    #[tokio::test]
    async fn test_aggressive_retry_succeeds_on_second_wave() {
        let mut gw = GatewayV2::new();
        // fail1 fails always; fail2 fails first 3 times, succeeds on 4th
        let fail_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let fc = fail_count.clone();

        struct ConditionalFail {
            fail_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
            threshold: u64,
        }
        #[async_trait::async_trait]
        impl LlmProvider for ConditionalFail {
            async fn complete(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
                let count = self.fail_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < self.threshold {
                    Err(LlmError::Server("transient failure".to_string()))
                } else {
                    Ok(LlmResponse {
                        content: "recovered".to_string(),
                        model: "test".to_string(),
                        usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
                        finish_reason: FinishReason::Stop,
                    })
                }
            }
            async fn stream_complete(&self, _req: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
                Err(LlmError::Server("stream_complete not implemented for ConditionalFail".to_string()))
            }
        }

        gw.register_provider("transient", Box::new(ConditionalFail { fail_count: fc, threshold: 1 }), true);

        let req = LlmRequest::new("test", "recover me");
        // Normal retry fails (1 failure → circuit opens), aggressive retry should succeed
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_ok(), "aggressive retry should recover after transient failures");
        let resp = result.unwrap();
        assert_eq!(resp.content, "recovered");
    }

    #[tokio::test]
    async fn test_stream_aggressive_retry_succeeds_on_second_wave() {
        let mut gw = GatewayV2::new();
        let fail_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let fc = fail_count.clone();

        struct StreamConditionalFail {
            fail_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
            threshold: u64,
        }
        #[async_trait::async_trait]
        impl LlmProvider for StreamConditionalFail {
            async fn complete(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
                Err(LlmError::Server("complete not implemented for StreamConditionalFail".to_string()))
            }
            async fn stream_complete(&self, _req: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
                let count = self.fail_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < self.threshold {
                    Err(LlmError::Server("transient streaming failure".to_string()))
                } else {
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(LlmResponse {
                            content: "stream recovered".to_string(),
                            model: "test".to_string(),
                            usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
                            finish_reason: FinishReason::Stop,
                        })).await;
                    });
                    Ok(rx)
                }
            }
        }

        gw.register_provider("stream-transient", Box::new(StreamConditionalFail { fail_count: fc, threshold: 1 }), true);

        let req = LlmRequest::new("test", "recover me");
        let mut rx = gw.stream_complete_with_selection(&req).await
            .expect("streaming aggressive retry should succeed after transient failure");

        let msg = rx.recv().await
            .expect("should receive a stream message")
            .expect("stream message should be Ok");
        assert_eq!(msg.content, "stream recovered");
    }

    #[test]
    fn test_sub_grid_composition() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);

        // 组合匿名子网格: 只包含 Local provider (ollama)
        let anonymous = gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        assert_eq!(anonymous.provider_names, vec!["ollama"]);
        assert!(anonymous.meets_profile(CommunicationProfile::Anonymous));
        assert!(anonymous.meets_profile(CommunicationProfile::Open));

        // 组合开放子网格: 包含所有 provider
        let open = gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);
        assert_eq!(open.provider_names.len(), 2);

        // 列表验证
        let grids = gw.list_sub_grids();
        assert_eq!(grids.len(), 2);
    }

    #[tokio::test]
    async fn test_select_best_for_profile() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);

        // 匿名安全级别: 只能选 ollama (Local)
        let selected = gw.select_best_for_profile(CommunicationProfile::Anonymous).await;
        assert_eq!(selected, Some("ollama".to_string()));

        // 开放安全级别: 可以选 openai 或 ollama (免费优先)
        let selected = gw.select_best_for_profile(CommunicationProfile::Open).await;
        assert_eq!(selected, Some("ollama".to_string()));
    }

    #[test]
    fn test_communication_profile_meets() {
        assert!(CommunicationProfile::Anonymous.meets(CommunicationProfile::Anonymous));
        assert!(CommunicationProfile::Anonymous.meets(CommunicationProfile::Tor));
        assert!(CommunicationProfile::Anonymous.meets(CommunicationProfile::Open));
        assert!(CommunicationProfile::Open.meets(CommunicationProfile::Open));
        assert!(!CommunicationProfile::Open.meets(CommunicationProfile::Tor));
        assert!(CommunicationProfile::Proxied.meets(CommunicationProfile::Open));
        assert!(!CommunicationProfile::Proxied.meets(CommunicationProfile::Tor));
    }

    #[tokio::test]
    async fn test_complete_for_profile() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        // 匿名级别: 命中 ollama (Local)
        let resp = gw.complete_for_profile(CommunicationProfile::Anonymous, &req).await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "local");
    }

    #[tokio::test]
    async fn test_complete_for_profile_fallback() {
        // 没有任何 provider 满足 Tor 级别 → 回退默认 select_best
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let req = LlmRequest::new("test-model", "hello");
        let resp = gw.complete_for_profile(CommunicationProfile::Tor, &req).await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "ok");
    }

    #[test]
    fn test_sub_grids_meeting() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        assert_eq!(gw.sub_grids_meeting(CommunicationProfile::Anonymous), vec!["anonymous-local"]);
        let meeting_open = gw.sub_grids_meeting(CommunicationProfile::Open);
        assert_eq!(meeting_open.len(), 2); // anonymous 满足 open, open 也满足 open
        // Tor 需求: 仅 anonymous (Anonymous > Tor 满足), open 不满足
        assert_eq!(gw.sub_grids_meeting(CommunicationProfile::Tor), vec!["anonymous-local"]);
    }

    struct MockProvider {
        response: String,
        should_fail: bool,
    }

    impl MockProvider {
        fn new(response: &str) -> Self {
            Self { response: response.to_string(), should_fail: false }
        }
        fn failing() -> Self {
            Self { response: String::new(), should_fail: true }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            if self.should_fail {
                Err(LlmError::Server("mock failure".to_string()))
            } else {
                Ok(LlmResponse {
                    content: self.response.clone(),
                    model: "mock".to_string(),
                    usage: Usage::default(),
                    finish_reason: FinishReason::Stop,
                })
            }
        }

        async fn stream_complete(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let resp = self.response.clone();
            tokio::spawn(async move {
                let _ = tx.send(Ok(LlmResponse {
                    content: resp,
                    model: "mock".to_string(),
                    usage: Usage::default(),
                    finish_reason: FinishReason::Stop,
                })).await;
            });
            Ok(rx)
        }
    }
}
