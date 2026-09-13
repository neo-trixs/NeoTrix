//! 执行 — 协调器、请求执行、Keyless、统一推理、通用适配器

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::l0_substrate::nt_core_error::recovery::{ErrorContext, ErrorType, RecoveryAction};
use crate::core::nt_core_cache::text_to_embedding;
use crate::core::nt_core_llm::{LlmError, LlmRequest, LlmResponse};
use crate::core::nt_core_span::{SpanKind, Tracer};
use crate::l1_action::nt_io::nt_io_provider::pool::account_pool::{AccountPoolError};
use crate::l1_action::nt_io::nt_io_provider::gateway::routing::agent_routing::ModelTier;
use crate::l1_action::nt_io::nt_io_provider::health::circuit_breaker::BreakerState;
use crate::l1_action::nt_io::nt_io_provider::health::context_budget::estimate_tokens;
use crate::l1_action::nt_io::nt_io_provider::pool::free_pool::global_free_pool;
use crate::l1_action::nt_io::nt_io_provider::health::rate_limiter::BrainTier;
use crate::l1_action::nt_io::nt_io_provider::common::privacy_guard::{egress_privacy_guard, trust_from_name};
use crate::l1_action::nt_io::nt_io_provider::gateway::types::registry_core::{AttemptPhase, ProviderState};
use crate::l1_action::nt_io::nt_io_provider::gateway::resilience::ResponseCache;
use crate::l1_action::nt_io::nt_io_provider::gateway::CallEvent;
use crate::l1_action::nt_io::nt_io_provider::common::factory::LlmProviderType;
use crate::l1_action::nt_io::nt_io_provider::routing::provider_swap::ProviderSwapManager;
use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::{CommunicationProfile, ProviderCategory};
use super::routing::agent_routing::AgentRoutingTable;
use super::GatewayV2;

// ═══════════════════════════════════════════════════════════════════
// Capability Coordinator — 能力自主协调层
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityIntent {
    LocalReasoning,
    GeneralReasoning,
    KnowledgeRetrieval,
    SensitiveWrite,
    AnonymousCommunication,
    DeepAnalysis,
}

impl CapabilityIntent {
    pub fn required_profile(&self) -> CommunicationProfile {
        match self {
            Self::LocalReasoning => CommunicationProfile::Anonymous,
            Self::GeneralReasoning => CommunicationProfile::Open,
            Self::KnowledgeRetrieval => CommunicationProfile::Open,
            Self::SensitiveWrite => CommunicationProfile::Proxied,
            Self::AnonymousCommunication => CommunicationProfile::Tor,
            Self::DeepAnalysis => CommunicationProfile::Open,
        }
    }

    pub fn preferred_category(&self) -> Option<ProviderCategory> {
        match self {
            Self::LocalReasoning => Some(ProviderCategory::Local),
            Self::DeepAnalysis => Some(ProviderCategory::Cloud),
            _ => None,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "local" | "local_reasoning" | "local-reasoning" => Self::LocalReasoning,
            "general" | "general_reasoning" | "general-reasoning" | "reason" => Self::GeneralReasoning,
            "retrieve" | "retrieval" | "knowledge" | "knowledge_retrieval" | "knowledge-retrieval" => Self::KnowledgeRetrieval,
            "write" | "sensitive" | "sensitive_write" | "sensitive-write" => Self::SensitiveWrite,
            "anonymous" | "anon" | "anonymous_communication" | "anonymous-communication" | "tor" => Self::AnonymousCommunication,
            "deep" | "deep_analysis" | "deep-analysis" | "analysis" => Self::DeepAnalysis,
            _ => return None,
        })
    }

    pub fn all() -> [CapabilityIntent; 6] {
        [Self::LocalReasoning, Self::GeneralReasoning, Self::KnowledgeRetrieval, Self::SensitiveWrite, Self::AnonymousCommunication, Self::DeepAnalysis]
    }
}

impl std::fmt::Display for CapabilityIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LocalReasoning => "local_reasoning",
            Self::GeneralReasoning => "general_reasoning",
            Self::KnowledgeRetrieval => "knowledge_retrieval",
            Self::SensitiveWrite => "sensitive_write",
            Self::AnonymousCommunication => "anonymous_communication",
            Self::DeepAnalysis => "deep_analysis",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct CoordinationRequest {
    pub intent: CapabilityIntent,
    pub prompt: String,
    pub model: Option<String>,
}

impl CoordinationRequest {
    pub fn new(intent: CapabilityIntent, prompt: &str) -> Self {
        Self { intent, prompt: prompt.to_string(), model: None }
    }
}

#[derive(Debug)]
pub struct CoordinationOutcome {
    pub response: LlmResponse,
    pub used_profile: CommunicationProfile,
    pub degraded: bool,
    pub provider_name: String,
}

pub struct CapabilityCoordinator {
    pub gateway: GatewayV2,
    pub routing: AgentRoutingTable,
    pub swap: ProviderSwapManager,
}

impl CapabilityCoordinator {
    pub fn new(gateway: GatewayV2, routing: AgentRoutingTable, swap: ProviderSwapManager) -> Self {
        Self { gateway, routing, swap }
    }

    pub fn ensure_default_sub_grids(&self) {
        let grids = self.gateway.list_sub_grids();
        if grids.is_empty() {
            self.gateway.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
            self.gateway.compose_sub_grid("proxied-cloud", CommunicationProfile::Proxied, false);
            self.gateway.compose_sub_grid("open-all", CommunicationProfile::Open, false);
        }
    }

    pub fn capability_plan(intent: CapabilityIntent) -> Vec<&'static str> {
        match intent {
            CapabilityIntent::LocalReasoning => vec!["local_reasoning", "reason", "generate"],
            CapabilityIntent::GeneralReasoning => vec!["reason", "generate", "coordinate"],
            CapabilityIntent::KnowledgeRetrieval => vec!["retrieve", "search", "reason"],
            CapabilityIntent::SensitiveWrite => vec!["mutate", "send", "verify"],
            CapabilityIntent::AnonymousCommunication => vec!["send", "communicate", "shield"],
            CapabilityIntent::DeepAnalysis => vec!["plan", "decompose", "critique", "simulate"],
        }
    }

    pub async fn coordinate(&mut self, req: &CoordinationRequest) -> Result<CoordinationOutcome, LlmError> {
        self.ensure_default_sub_grids();
        let profile = req.intent.required_profile();
        let preferred = req.intent.preferred_category();
        let routed_provider = preferred.and_then(|cat| self.find_provider_by_category(cat));
        let provider_name = if let Some(name) = routed_provider {
            name
        } else {
            match self.gateway.select_best_for_profile(profile).await {
                Some(name) => name,
                None => self.gateway.default_provider_name(),
            }
        };
        let llm_req = LlmRequest {
            model: req.model.clone().unwrap_or_else(|| {
                let (_, model) = self.routing.route_for(provider_name.as_str());
                model.clone()
            }),
            ..LlmRequest::new(provider_name.as_str(), &req.prompt)
        };
        let result = self.gateway.complete_for_profile_detailed(profile, &llm_req).await;
        match result {
            Ok((resp, actual_profile, actual_provider)) => {
                let degraded = actual_profile != profile;
                if !degraded {
                    self.swap.record_success(
                        LlmProviderType::from_name(provider_name.as_str()).unwrap_or(LlmProviderType::OpenAI),
                        0.0,
                    );
                }
                Ok(CoordinationOutcome { response: resp, used_profile: actual_profile, degraded, provider_name: actual_provider })
            }
            Err(e) => {
                self.swap.record_error(
                    LlmProviderType::from_name(provider_name.as_str()).unwrap_or(LlmProviderType::OpenAI),
                    &e,
                );
                Err(e)
            }
        }
    }

    fn find_provider_by_category(&self, category: ProviderCategory) -> Option<String> {
        self.gateway.providers().into_iter().find(|name| {
            self.gateway.category_of(name).map(|c| c == category).unwrap_or(false)
        })
    }
}

impl Default for CapabilityCoordinator {
    fn default() -> Self {
        Self::new(GatewayV2::new(), AgentRoutingTable::new("default", "default"), ProviderSwapManager::new(vec![]))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Execution — 请求执行与重试
// ═══════════════════════════════════════════════════════════════════

fn is_maintenance_window(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("switching to new models")
        || m.contains("retrying in")
        || (m.contains("maintenance") && m.contains("window"))
        || m.contains("temporarily unavailable for maintenance")
}

fn exponential_backoff(attempt: u32, base_ms: u64, cap_ms: u64) -> u64 {
    let exp = 2u64.saturating_pow(attempt);
    let delay = base_ms.saturating_mul(exp).min(cap_ms);
    let jitter = (rand::random::<f64>() * delay as f64) as u64;
    delay.saturating_add(jitter)
}

fn is_model_unavailable(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("model not found")
        || m.contains("model does not exist")
        || m.contains("model_unavailable")
        || m.contains("model unavailable")
        || m.contains("unknown model")
        || m.contains("model is not available")
        || m.contains("no model named")
        || m.contains("model not supported")
        || m.contains("model '") && (m.contains("not found") || m.contains("does not exist"))
        || (m.contains("the model") && (m.contains("does not exist") || m.contains("not exist")))
        || (m.contains("currently not available") && m.contains("model"))
        || m.contains("decommissioned")
        || m.contains("no longer available")
        || m.contains("invalid model")
        || (m.contains("404") && m.contains("model"))
}

impl GatewayV2 {
    pub(crate) async fn call_provider(&self, name: &str, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let provider = {
            let guard = self.providers.read().unwrap_or_else(|e| e.into_inner());
            guard.get(name).cloned().or_else(|| {
                name.split('/').next().and_then(|p| guard.get(p).cloned())
            })
        }
        .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;

        let stripped = request.model.strip_prefix(&format!("{}/", name)).map(|m| m.to_string()).or_else(|| {
            if name.contains('/') && request.model.starts_with(&format!("{}/", name)) {
                request.model.strip_prefix(&format!("{}/", name)).map(|m| m.to_string())
            } else {
                None
            }
        });
        let mut req = request.clone();
        if req.model == name {
            req.model.clear();
        }
        let final_model = if let Some(m) = stripped {
            if m.is_empty() {
                crate::l1_action::nt_io::nt_io_provider::provider_catalog::lookup_provider(name.split('/').next().unwrap_or(name))
                    .map(|info| info.default_model.to_string())
                    .unwrap_or_else(|| req.model.clone())
            } else {
                m
            }
        } else if req.model.is_empty() {
            if name.contains('/') {
                name.split('/').skip(1).collect::<Vec<_>>().join("/")
            } else {
                crate::l1_action::nt_io::nt_io_provider::provider_catalog::lookup_provider(name)
                    .map(|info| info.default_model.to_string())
                    .unwrap_or_else(|| "auto".to_string())
            }
        } else {
            req.model.clone()
        };
        req.model = final_model;

        let tier = if ModelTier::parse(&req.model) >= ModelTier::Capable {
            BrainTier::Big
        } else {
            BrainTier::Triage
        };
        let gate_wait = {
            loop {
                let acquired = self.tiered_semaphore.lock().unwrap_or_else(|e| e.into_inner()).try_acquire(tier);
                if acquired { break; }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            self.adaptive_pacer.lock().unwrap_or_else(|e| e.into_inner()).gate()
        };
        if gate_wait > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(gate_wait)).await;
        }
        if let Err(reason) = egress_privacy_guard(&mut req, trust_from_name(name), name) {
            return Err(LlmError::InvalidRequest(reason));
        }
        if let Ok(plugins) = self.plugin_manager.read() {
            let mut ctx = super::observability::RequestContext {
                provider: name.to_string(),
                model: req.model.clone(),
                headers: std::collections::HashMap::new(),
                body: serde_json::to_vec(&req).unwrap_or_default(),
                timestamp: std::time::Instant::now(),
            };
            if let Err(e) = plugins._run_pre_request(&mut ctx) {
                return Err(LlmError::Unknown(format!("Plugin pre_request aborted: {:?}", e)));
            }
        }
        let result = provider.complete(&req).await;
        match &result {
            Ok(response) => {
                if let Ok(plugins) = self.plugin_manager.read() {
                    let mut ctx = super::observability::ResponseContext {
                        provider: name.to_string(),
                        status: 200,
                        headers: std::collections::HashMap::new(),
                        body: serde_json::to_vec(response).unwrap_or_default(),
                        latency: std::time::Duration::from_millis(0),
                    };
                    let _ = plugins._run_post_response(&mut ctx);
                }
            }
            Err(e) => {
                if let Ok(plugins) = self.plugin_manager.read() {
                    let mut ctx = super::observability::ErrorContext {
                        provider: name.to_string(),
                        error: e.to_string(),
                        retry_count: 0,
                        timestamp: std::time::Instant::now(),
                    };
                    let _ = plugins._run_on_error(&mut ctx);
                }
            }
        }
        if let Ok(response) = result.as_ref() {
            crate::core::nt_core_telemetry::global_provider_usage_ledger()
                .record_provider_usage(name, &response.usage);
        }
        {
            self.tiered_semaphore.lock().unwrap_or_else(|e| e.into_inner()).release(tier);
            let mut pacer = self.adaptive_pacer.lock().unwrap_or_else(|e| e.into_inner());
            match &result {
                Ok(_) => pacer.on_ok(),
                Err(e) => {
                    let msg = e.to_string();
                    if crate::is_quota_exhaustion(&msg) || msg.contains("rate limit") || msg.contains("429") {
                        pacer.on_rate_limited();
                    } else {
                        pacer.on_ok();
                    }
                }
            }
        }
        let _ = gate_wait;
        result
    }

    pub async fn complete_single(&self, provider_name: &str, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.call_provider(provider_name, request).await
    }

    pub async fn complete_with_account_pool(&self, provider: &str, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let scope = if provider.is_empty() {
            request.model.split('/').next().map(|s| s.to_string()).unwrap_or_default()
        } else {
            provider.to_string()
        };
        if scope.is_empty() {
            return Err(LlmError::Unknown("account_pool: cannot infer provider scope from empty model".to_string()));
        }
        let lease = {
            let pool = self.account_pool.lock().unwrap_or_else(|e| { log::warn!("[gateway] account_pool Mutex poisoned: {}", e); e.into_inner() });
            match pool.select(&scope) {
                Ok(lease) => lease,
                Err(AccountPoolError::NoAccounts(_)) => return Err(LlmError::Unknown(format!("account_pool: no accounts registered for provider '{scope}'"))),
                Err(AccountPoolError::NoHealthyAccount(_)) => return Err(LlmError::RateLimit(format!("account_pool: all accounts for '{scope}' are quarantined/unavailable"))),
                Err(AccountPoolError::Saturated(_)) => return Err(LlmError::RateLimit(format!("account_pool: account concurrency cap reached for '{scope}'"))),
            }
        };
        let account_name = lease.account_name().to_string();
        let result = self.call_provider(&account_name, request).await;
        let pool = self.account_pool.lock().unwrap_or_else(|e| { log::warn!("[gateway] account_pool Mutex poisoned: {}", e); e.into_inner() });
        match &result {
            Ok(_) => { pool.record_success(&account_name); }
            Err(LlmError::RateLimit(_)) => {
                log::warn!("[gateway] account '{}' rate-limited → quarantine ({}s cooldown)", account_name, pool.config().quarantine_cooldown.as_secs());
                pool.quarantine(&account_name);
            }
            Err(_) => { pool.record_failure(&account_name); }
        }
        let _ = lease;
        result
    }

    pub async fn complete_with_selection(&self, request: &LlmRequest) -> Result<SelectionResult, LlmError> {
        self.ensure_pool_sufficient(3, 60).await;
        let prompt_key: String = self.prompt_cache_key(request);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_exact(&request.model, &prompt_key) {
                if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                    return Ok(SelectionResult { response, provider: "cache".to_string() });
                }
            }
        }
        {
            let embedding = text_to_embedding(&self.prompt_text(request));
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(cached) = cache.get_semantic(&embedding) {
                    if let Ok(response) = serde_json::from_str::<LlmResponse>(cached) {
                        return Ok(SelectionResult { response, provider: "cache".to_string() });
                    }
                }
            }
        }
        if self.response_cache_enabled {
            let rc_key = ResponseCache::key_for_request(&request.model, &self.prompt_cache_key(request));
            if let Ok(mut rc) = self.response_cache.lock() {
                if let Some(cached) = rc.cache(&rc_key) {
                    rc.pin(&rc_key);
                    if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                        return Ok(SelectionResult { response, provider: "cache".to_string() });
                    }
                }
            }
        }
        let mut telemetry_span = match self.tracer.read() {
            Ok(guard) => guard.as_ref().map(|t| {
                let span = t.start_span("llm.complete", SpanKind::Llm);
                span.set_gen_ai_request_model(&request.model);
                span.set_gen_ai_system("neotrix-gateway");
                span
            }),
            _ => None,
        };
        {
            if self.cost_budget_per_query > 0.0 {
                let est_tokens = estimate_tokens(&self.prompt_text(request)) as f64;
                let provider_name = request.model.split('/').next().unwrap_or("");
                let cost_per_1k = crate::l1_action::nt_io::nt_io_provider::provider_catalog::lookup_provider_cost(provider_name).unwrap_or(0.002);
                let estimated_cost = (est_tokens / 1000.0) * cost_per_1k;
                if estimated_cost > self.cost_budget_per_query {
                    return Err(LlmError::Unknown(format!("Cost budget exceeded: ${:.4} > ${:.4}", estimated_cost, self.cost_budget_per_query)));
                }
            }
        }
        let mut used_names: Vec<String> = Vec::new();
        let chain = self.quota_aware_fallback_chain(&request.model, &used_names, 3);
        let mut fatal_error: Option<LlmError> = None;

        for name in chain {
            if used_names.contains(&name) { continue; }
            used_names.push(name.clone());
            let start = Instant::now();
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(&name) {
                    if !state.rate_limiter.allow_request(20.0) { continue; }
                }
            }
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
                    if let Ok(guard) = self.tracer.read() {
                        if let Some(tracer) = guard.as_ref() {
                            if let Some(span) = telemetry_span.take() { tracer.end_span(span); }
                        }
                    }
                    {
                        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get(&name) {
                            if state.is_free { global_free_pool().record_usage(&name, token_count as u64); }
                        }
                    }
                    if let Ok(mut ct) = self.cost_tracker.write() {
                        if let Some(ref mut tracker) = *ct { tracker.record(&request.model, response.usage.prompt_tokens.into(), response.usage.completion_tokens.into()); }
                    }
                    let response = self.heal_and_cache_response(request, response);
                    self.tag_generation(request, &response, &name, elapsed, token_count, true);
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            cache.set_with_embedding(&request.model, &prompt_key, serialized, text_to_embedding(&self.prompt_text(request)));
                        }
                    }
                    return Ok(SelectionResult { response, provider: name });
                }
                Err(err) => {
                    let error_msg = err.to_string();
                    let is_quota_exhausted = crate::is_quota_exhaustion(&error_msg);
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            if is_quota_exhausted {
                                state.mark_quota_exhausted();
                            } else if is_maintenance_window(&error_msg) {
                                state.circuit_breaker.force_open_secs(1800);
                            } else if is_model_unavailable(&error_msg) {
                                state.lock_model(&request.model, 1800);
                                state.record_failure(elapsed);
                            } else {
                                state.record_failure(elapsed);
                            }
                        }
                    }
                    self.fire_event(&name, false, elapsed, 0, &request.model, AttemptPhase::Normal);
                    if matches!(err, LlmError::Authentication(_) | LlmError::InvalidRequest(_)) {
                        fatal_error = Some(err);
                        break;
                    }
                    let error_type = if is_quota_exhausted {
                        ErrorType::Unknown("quota exhausted — provider tripped, skip retry".to_string())
                    } else if error_msg.contains("rate limit") || error_msg.contains("429") {
                        ErrorType::RateLimit { retry_after: None }
                    } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                        ErrorType::Timeout { elapsed_ms: elapsed as u64 }
                    } else if error_msg.contains("50") || error_msg.contains("server error") {
                        ErrorType::ServerError { code: 500 }
                    } else {
                        ErrorType::Unknown(error_msg.clone())
                    };
                    let ctx = ErrorContext {
                        error_type, attempt: used_names.len(), max_retries: 3, model: request.model.clone(),
                        available_models: self.providers.read().unwrap_or_else(|e| e.into_inner()).keys().cloned().collect(),
                        prompt: request.messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n"),
                        prompt_variants: Vec::new(), state_snapshot: None, token_budget_remaining: 0,
                        elapsed_ms: elapsed as u64, metadata: HashMap::new(),
                    };
                    let action = self.recovery.write().unwrap_or_else(|e| { log::warn!("[gateway] recovery RwLock poisoned: {}", e); e.into_inner() }).handle(&ctx);
                    if let RecoveryAction::Retry { delay_ms, .. } = action {
                        if delay_ms > 0 { tokio::time::sleep(Duration::from_millis(delay_ms)).await; }
                        continue;
                    }
                }
            }
        }

        let result = match fatal_error {
            Some(f) => Err(f),
            None => self.attempt_aggressive_retry(request).await,
        };
        if let Ok(guard) = self.tracer.read() {
            if let Some(tracer) = guard.as_ref() {
                if let Some(span) = telemetry_span.take() { tracer.end_span(span); }
            }
        }
        if let Ok(ref selection) = result {
            if let Ok(mut ct) = self.cost_tracker.write() {
                if let Some(ref mut tracker) = *ct { tracker.record(&request.model, selection.response.usage.prompt_tokens.into(), selection.response.usage.completion_tokens.into()); }
            }
        }
        result
    }

    async fn attempt_aggressive_retry(&self, request: &LlmRequest) -> Result<SelectionResult, LlmError> {
        let provider_names: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            states.keys().cloned().collect()
        };
        if provider_names.is_empty() {
            return Err(LlmError::Unknown("No providers available for aggressive retry".to_string()));
        }
        let set_aggressive: Vec<(String, u64)> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            provider_names.iter().filter_map(|name| {
                states.get(name).and_then(|s| {
                    if matches!(s.circuit_breaker.state(), BreakerState::Open { .. }) {
                        let saved = s.circuit_breaker.half_open_max_probes();
                        Some((name.clone(), saved))
                    } else { None }
                })
            }).collect()
        };
        {
            let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            for (name, _) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(5);
                    state.circuit_breaker.cooldown_reset();
                }
            }
        }
        for name in &provider_names {
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(name) {
                    if !state.rate_limiter.allow_request(10.0) { continue; }
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
                            let saved = set_aggressive.iter().find(|(n, _)| n == name).map(|(_, s)| *s);
                            state.circuit_breaker.set_half_open_max_probes(saved.unwrap_or(3));
                        }
                    }
                    self.fire_event(name, true, elapsed, token_count, &request.model, AttemptPhase::AggressiveRetry);
                    {
                        let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get(name) {
                            if state.is_free { global_free_pool().record_usage(name, token_count as u64); }
                        }
                    }
                    let response = self.heal_and_cache_response(request, response);
                    self.tag_generation(request, &response, name, elapsed, token_count, true);
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            cache.set_with_embedding(&request.model, &self.prompt_cache_key(request), serialized, text_to_embedding(&self.prompt_text(request)));
                        }
                    }
                    return Ok(SelectionResult { response, provider: name.clone() });
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

    pub async fn stream_complete_with_selection(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        self.ensure_pool_sufficient(3, 60).await;
        if self.cost_budget_per_query > 0.0 {
            let prompt_tokens = estimate_tokens(&self.prompt_text(request));
            let cost_estimate = (prompt_tokens as f64 / 1000.0) * 0.002;
            if cost_estimate > self.cost_budget_per_query {
                log::warn!("[gateway] stream cost estimate ${:.4} exceeds budget ${:.4}", cost_estimate, self.cost_budget_per_query);
            }
        }
        let mut used_names: Vec<String> = Vec::new();
        let chain = self.quota_aware_fallback_chain(&request.model, &used_names, 3);
        for name in chain {
            if used_names.contains(&name) { continue; }
            used_names.push(name.clone());
            {
                let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                if let Some(state) = states.get_mut(&name) {
                    if !state.rate_limiter.allow_request(20.0) { continue; }
                }
            }
            match self.call_provider_stream(&name, request).await {
                Ok(result) => {
                    self.fire_event(&name, true, 0.0, 0, &request.model, AttemptPhase::Normal);
                    return Ok(result);
                }
                Err(LlmError::RateLimit(_)) => {
                    { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(&name) { state.record_failure(0.0); } }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    continue;
                }
                Err(err @ (LlmError::Authentication(_) | LlmError::InvalidRequest(_))) => {
                    { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(&name) { state.record_failure(0.0); } }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    return Err(err);
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    if err.is_quota_exhaustion() {
                        self.states_write(|states: &mut HashMap<String, ProviderState>| {
                            if let Some(state) = states.get_mut(&name) { state.circuit_breaker.force_open(); }
                        });
                    }
                    { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(&name) { if is_model_unavailable(&err_msg) { state.lock_model(&request.model, 1800); } state.record_failure(0.0); } }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    if used_names.len() >= 3 { break; }
                }
            }
        }
        self.attempt_aggressive_retry_stream(request).await
    }

    async fn call_provider_stream(&self, name: &str, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let provider = self.providers.read().unwrap_or_else(|e| e.into_inner()).get(name).cloned()
            .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;
        let stripped = request.model.strip_prefix(&format!("{}/", name)).map(|m| m.to_string()).or_else(|| {
            if name.contains('/') { request.model.split_once('/').map(|(_, rest)| rest.to_string()) } else { None }
        });
        let mut req = request.clone();
        if req.model == name {
            req.model = crate::l1_action::nt_io::nt_io_provider::provider_catalog::lookup_provider(name.split('/').next().unwrap_or(name))
                .map(|info| info.default_model.to_string()).unwrap_or_default();
        } else if let Some(m) = stripped { req.model = m; }
        if let Err(reason) = egress_privacy_guard(&mut req, trust_from_name(name), name) {
            return Err(LlmError::InvalidRequest(reason));
        }
        provider.stream_complete(&req).await
    }

    pub async fn describe_image(&self, image_b64: &str, question: &str) -> Result<String, LlmError> {
        let mut used: Vec<String> = Vec::new();
        let vision_candidates: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            let mut names: Vec<String> = states.keys().cloned().collect();
            names.sort_by_key(|n| (!crate::core::nt_core_e8::nt_multimodal::model_supports_vision(n), n.clone()));
            names
        };
        let mut target: Option<String> = None;
        for name in vision_candidates {
            if crate::core::nt_core_e8::nt_multimodal::model_supports_vision(&name) { target = Some(name); break; }
        }
        let name = match target {
            Some(n) => { used.push(n.clone()); n }
            None => match self.select_best().await {
                Some(n) => { used.push(n.clone()); n }
                None => return Err(LlmError::Unknown("no provider available for image description".into())),
            },
        };
        let request = LlmRequest::new(&name, question).with_image_b64(image_b64).with_max_tokens(1024).with_temperature(Some(0.2));
        let response = self.call_provider(&name, &request).await?;
        Ok(response.content)
    }

    async fn attempt_aggressive_retry_stream(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let provider_names: Vec<String> = { let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); states.keys().cloned().collect() };
        if provider_names.is_empty() { return Err(LlmError::Unknown("No providers available for aggressive retry".to_string())); }
        let set_aggressive: Vec<(String, u64)> = { let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); provider_names.iter().filter_map(|name| { states.get(name).and_then(|s| { if matches!(s.circuit_breaker.state(), BreakerState::Open { .. }) { let saved = s.circuit_breaker.half_open_max_probes(); Some((name.clone(), saved)) } else { None } }) }).collect() };
        { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); for (name, _) in &set_aggressive { if let Some(state) = states.get_mut(name) { state.circuit_breaker.set_half_open_max_probes(5); state.circuit_breaker.cooldown_reset(); } } }
        for name in &provider_names {
            { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(name) { if !state.rate_limiter.allow_request(10.0) { continue; } } }
            match self.call_provider_stream(name, request).await {
                Ok(result) => {
                    { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(name) { let saved = set_aggressive.iter().find(|(n, _)| n == name).map(|(_, s)| *s); state.circuit_breaker.set_half_open_max_probes(saved.unwrap_or(3)); } }
                    self.fire_event(name, true, 0.0, 0, &request.model, AttemptPhase::AggressiveRetry);
                    return Ok(result);
                }
                Err(_) => { { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); if let Some(state) = states.get_mut(name) { state.record_failure(0.0); state.circuit_breaker.set_half_open_max_probes(3); } } self.fire_event(name, false, 0.0, 0, &request.model, AttemptPhase::AggressiveRetry); }
            }
        }
        { let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() }); for (name, saved) in &set_aggressive { if let Some(state) = states.get_mut(name) { state.circuit_breaker.set_half_open_max_probes(*saved); } } }
        Err(LlmError::Unknown("Aggressive streaming retry exhausted — all providers failed".to_string()))
    }

    fn fire_event(&self, provider_name: &str, success: bool, latency_ms: f64, tokens: u32, model: &str, phase: AttemptPhase) {
        if let Ok(guard) = self.observer.read() {
            if let Some(ref obs) = *guard {
                obs(CallEvent { provider_name: provider_name.to_string(), success, latency_ms, tokens, model: model.to_string(), attempt_phase: phase });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Keyless — 匿名免费端点路由
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
    pub fn keyless_candidates(&self) -> Vec<String> {
        if let Ok(guard) = self.providers.read() {
            let mut v: Vec<String> = guard.keys().filter(|k| k.starts_with("opencode-zen/") || k.starts_with("llm7/")).cloned().collect();
            if !v.is_empty() { v.sort(); return v; }
        }
        vec!["opencode-zen/big-pickle".into(), "opencode-zen/mimo-v2.5-free".into(), "llm7/codestral-latest".into()]
    }

    pub(crate) async fn call_provider_backoff(&self, name: &str, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        const MAX_ATTEMPTS: u32 = 5;
        let mut attempt = 0u32;
        loop {
            match self.call_provider(name, request).await {
                Ok(resp) => return Ok(resp),
                Err(LlmError::RateLimit(msg)) => {
                    if attempt >= MAX_ATTEMPTS { return Err(LlmError::RateLimit(msg)); }
                    let backoff = parse_retry_after(&msg).unwrap_or(1.0);
                    tokio::time::sleep(Duration::from_secs_f32(backoff)).await;
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub async fn route_keyless(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let candidates = self.keyless_candidates();
        let mut last_err = LlmError::Unknown("no keyless candidates configured".into());
        for cand in candidates {
            match self.call_provider_backoff(&cand, request).await {
                Ok(resp) => return Ok(resp),
                Err(e) => { last_err = e; continue; }
            }
        }
        Err(last_err)
    }
}

fn parse_retry_after(msg: &str) -> Option<f32> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    v.get("error")?.get("retry_after")?.as_f64().map(|x| x as f32)
}

// ═══════════════════════════════════════════════════════════════════
// Unified Inference Layer — 所有 LLM 调用的统一入口
// ═══════════════════════════════════════════════════════════════════

use async_trait::async_trait;
use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::{ProviderCapabilities as CatalogCapabilities};
use crate::l1_action::nt_io::nt_io_provider::common::types::*;

#[derive(Debug, Clone, Default)]
pub struct InferenceRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Vec<Tool>,
    pub image_data: Option<String>,
    pub thinking_budget: Option<u32>,
    pub structured_output: Option<StructuredOutputConfig>,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct RequestMetadata {
    pub task_type: Option<String>,
    pub priority: Priority,
    pub cost_budget: Option<f64>,
    pub latency_budget: Option<u64>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct InferenceResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub usage: Usage,
    pub finish_reason: FinishReason,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    pub reasoning: Option<String>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct ResponseMetadata {
    pub provider_selected: String,
    pub fallback_count: u32,
    pub latency_ms: u64,
    pub cost_estimate: CostEstimate,
    pub from_cache: bool,
}

#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub response: LlmResponse,
    pub provider: String,
}

#[derive(Debug, Clone, Default)]
pub struct InferenceCapabilities {
    pub providers: Vec<InferenceProviderInfo>,
    pub total_free: usize,
    pub total_paid: usize,
    pub total_local: usize,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
}

#[derive(Debug, Clone)]
pub struct InferenceProviderInfo {
    pub name: String,
    pub category: ProviderCategory,
    pub is_free: bool,
    pub health: InferenceHealthStatus,
    pub capabilities: CatalogCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InferenceHealthStatus {
    Healthy,
    Degraded,
    CircuitOpen,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RouterHealth {
    pub total_providers: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub circuit_open: usize,
    pub pool_sufficient: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CostEstimate {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub estimated_cost_usd: f64,
    pub provider_name: String,
}

#[derive(Debug, Clone)]
pub enum InferenceError {
    ProviderError { provider: String, message: String },
    RateLimitError { provider: String, retry_after: Option<u64> },
    AuthenticationError { provider: String, message: String },
    ValidationError(String),
    NetworkError { provider: String, message: String },
    TimeoutError { provider: String, elapsed_ms: u64 },
    BudgetExceeded { estimated: f64, budget: f64 },
    AllProvidersFailed { errors: Vec<InferenceError> },
    NoProvidersAvailable,
}

impl InferenceError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, InferenceError::RateLimitError { .. } | InferenceError::NetworkError { .. } | InferenceError::TimeoutError { .. })
    }
    pub fn should_fallback(&self) -> bool {
        !matches!(self, InferenceError::AuthenticationError { .. } | InferenceError::ValidationError(_) | InferenceError::BudgetExceeded { .. })
    }
    pub fn provider_name(&self) -> Option<&str> {
        match self {
            InferenceError::ProviderError { provider, .. } | InferenceError::RateLimitError { provider, .. }
            | InferenceError::AuthenticationError { provider, .. } | InferenceError::NetworkError { provider, .. }
            | InferenceError::TimeoutError { provider, .. } => Some(provider),
            _ => None,
        }
    }
    pub fn is_quota_exhaustion(&self) -> bool {
        match self {
            InferenceError::ProviderError { message, .. } => {
                let m = message.to_lowercase();
                m.contains("quota exceeded") || m.contains("out of quota") || m.contains("insufficient quota") || m.contains("credit limit") || m.contains("out of credits") || m.contains("billing")
            }
            InferenceError::AuthenticationError { message, .. } => { let m = message.to_lowercase(); m.contains("quota") || m.contains("billing") }
            _ => false,
        }
    }
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceError::ProviderError { provider, message } => write!(f, "[{provider}] Provider error: {message}"),
            InferenceError::RateLimitError { provider, retry_after } => write!(f, "[{provider}] Rate limited, retry after {retry_after:?}s"),
            InferenceError::AuthenticationError { provider, message } => write!(f, "[{provider}] Auth error: {message}"),
            InferenceError::ValidationError(msg) => write!(f, "Validation: {msg}"),
            InferenceError::NetworkError { provider, message } => write!(f, "[{provider}] Network: {message}"),
            InferenceError::TimeoutError { provider, elapsed_ms } => write!(f, "[{provider}] Timeout after {elapsed_ms}ms"),
            InferenceError::BudgetExceeded { estimated, budget } => write!(f, "Budget exceeded: ${estimated:.4} > ${budget:.4}"),
            InferenceError::AllProvidersFailed { errors } => write!(f, "All {} providers failed: {}", errors.len(), errors.first().map(|e| e.to_string()).unwrap_or_default()),
            InferenceError::NoProvidersAvailable => write!(f, "No providers available"),
        }
    }
}

impl std::error::Error for InferenceError {}

impl From<LlmError> for InferenceError {
    fn from(e: LlmError) -> Self {
        match e {
            LlmError::Network(s) => InferenceError::NetworkError { provider: String::new(), message: s },
            LlmError::Authentication(s) => InferenceError::AuthenticationError { provider: String::new(), message: s },
            LlmError::RateLimit(_s) => InferenceError::RateLimitError { provider: String::new(), retry_after: None },
            LlmError::InvalidRequest(s) => InferenceError::ValidationError(s),
            LlmError::Server(s) => InferenceError::ProviderError { provider: String::new(), message: s },
            LlmError::Unknown(s) => InferenceError::ProviderError { provider: String::new(), message: s },
        }
    }
}

pub struct StreamHandle {
    pub rx: tokio::sync::mpsc::Receiver<Result<InferenceResponse, InferenceError>>,
    pub provider: String,
}

#[async_trait]
pub trait UnifiedInference: Send + Sync {
    async fn complete(&self, request: &InferenceRequest) -> Result<InferenceResponse, InferenceError>;
    async fn stream(&self, request: &InferenceRequest) -> Result<StreamHandle, InferenceError>;
    fn capabilities(&self) -> InferenceCapabilities;
    async fn health(&self) -> RouterHealth;
    fn estimate_cost(&self, request: &InferenceRequest) -> CostEstimate;
}

impl InferenceRequest {
    pub fn to_llm_request(&self) -> LlmRequest {
        LlmRequest {
            model: self.model.clone().unwrap_or_default(),
            messages: self.messages.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens.unwrap_or(4096),
            tools: self.tools.clone(),
            image_data: self.image_data.clone(),
            thinking_budget: self.thinking_budget,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: self.structured_output.clone(),
            cacheable_prefix_tokens: None,
        }
    }
}

impl InferenceResponse {
    pub fn from_llm_response(resp: LlmResponse, provider: &str, latency_ms: u64, fallback_count: u32) -> Self {
        Self {
            content: resp.content, model: resp.model, provider: provider.to_string(),
            usage: resp.usage, finish_reason: resp.finish_reason, tool_calls: resp.tool_calls,
            reasoning: resp.reasoning, metadata: ResponseMetadata { provider_selected: provider.to_string(), fallback_count, latency_ms, cost_estimate: CostEstimate::default(), from_cache: false },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Universal Adapter — 通用模型适配器
// ═══════════════════════════════════════════════════════════════════

pub struct ModelCapabilities {
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_structured_output: bool,
    pub max_context_tokens: u32,
    pub supports_images: bool,
    pub supports_audio: bool,
    pub cost_per_1k_tokens: f64,
}

pub struct ModelConfig {
    pub model_id: String,
    pub provider: String,
    pub capabilities: ModelCapabilities,
    pub endpoint: String,
    pub api_key_env: String,
}

pub struct UnifiedRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<Tool>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

pub struct Message2 {
    pub role: String,
    pub content: String,
}

pub struct Tool2 {
    pub name: String,
    pub description: String,
    pub parameters: String,
}

pub struct UnifiedResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall2>,
    pub tokens_used: u32,
    pub model: String,
}

pub struct ToolCall2 {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub trait FormatConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String;
    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse>;
}

pub struct OpenAiConverter;
impl FormatConverter for OpenAiConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String {
        let messages: Vec<String> = request.messages.iter().map(|m| format!("{{\"role\":\"{}\",\"content\":\"{}\"}}", escape_json(&m.role), escape_json(&m.content))).collect();
        let tools: Vec<String> = request.tools.iter().map(|t| format!("{{\"type\":\"function\",\"function\":{{\"name\":\"{}\",\"description\":\"{}\",\"parameters\":{}}}}}", escape_json(&t.name), escape_json(&t.description), t.parameters)).collect();
        format!("{{\"model\":\"{}\",\"messages\":[{}],\"tools\":[{}],\"temperature\":{},\"max_tokens\":{},\"stream\":{}}}", escape_json(&model.model_id), messages.join(","), tools.join(","), request.temperature, request.max_tokens, request.stream)
    }
    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "content").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "total_tokens").unwrap_or(0.0) as u32;
        Some(UnifiedResponse { content, tool_calls: Vec::new(), tokens_used, model: model.model_id.clone() })
    }
}

pub struct AnthropicConverter;
impl FormatConverter for AnthropicConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> String {
        let messages: Vec<String> = request.messages.iter().filter(|m| m.role != "system").map(|m| format!("{{\"role\":\"{}\",\"content\":\"{}\"}}", escape_json(&m.role), escape_json(&m.content))).collect();
        let system_msg = request.messages.iter().find(|m| m.role == "system").map(|m| format!(",\"system\":\"{}\"", escape_json(&m.content))).unwrap_or_default();
        format!("{{\"model\":\"{}\",\"messages\":[{}]{},\"max_tokens\":{}}}", escape_json(&model.model_id), messages.join(","), system_msg, request.max_tokens)
    }
    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "text").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "input_tokens").zip(extract_json_number(raw, "output_tokens")).map(|(i, o)| (i + o) as u32).unwrap_or(0);
        Some(UnifiedResponse { content, tool_calls: Vec::new(), tokens_used, model: model.model_id.clone() })
    }
}

pub struct GeminiConverter;
impl FormatConverter for GeminiConverter {
    fn to_provider_format(&self, request: &UnifiedRequest, _model: &ModelConfig) -> String {
        let contents: Vec<String> = request.messages.iter().map(|m| format!("{{\"role\":\"{}\",\"parts\":[{{\"text\":\"{}\"}}]}}", if m.role == "assistant" { "model" } else { "user" }, escape_json(&m.content))).collect();
        format!("{{\"contents\":[{}],\"generationConfig\":{{\"temperature\":{},\"maxOutputTokens\":{}}}}}", contents.join(","), request.temperature, request.max_tokens)
    }
    fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> {
        let content = extract_json_string(raw, "text").unwrap_or_default();
        let tokens_used = extract_json_number(raw, "totalTokenCount").unwrap_or(0.0) as u32;
        Some(UnifiedResponse { content, tool_calls: Vec::new(), tokens_used, model: model.model_id.clone() })
    }
}

pub struct UniversalAdapter {
    models: Vec<ModelConfig>,
    active_model: Option<String>,
    converters: HashMap<String, Box<dyn FormatConverter>>,
}

impl UniversalAdapter {
    pub fn new() -> Self {
        let mut converters: HashMap<String, Box<dyn FormatConverter>> = HashMap::new();
        converters.insert("openai".to_string(), Box::new(OpenAiConverter));
        converters.insert("anthropic".to_string(), Box::new(AnthropicConverter));
        converters.insert("gemini".to_string(), Box::new(GeminiConverter));
        Self { models: Vec::new(), active_model: None, converters }
    }
    pub fn register_model(&mut self, config: ModelConfig) { self.models.push(config); }
    pub fn set_active(&mut self, model_id: &str) -> bool {
        if self.models.iter().any(|m| m.model_id == model_id) { self.active_model = Some(model_id.to_string()); true } else { false }
    }
    pub fn active_model(&self) -> Option<&ModelConfig> { self.active_model.as_ref().and_then(|id| self.models.iter().find(|m| m.model_id == *id)) }
    pub fn find_model(&self, model_id: &str) -> Option<&ModelConfig> { self.models.iter().find(|m| m.model_id == model_id) }
    pub fn select_best(&self, needs_tools: bool, needs_streaming: bool, needs_images: bool, min_context_tokens: u32) -> Option<&ModelConfig> {
        let mut candidates: Vec<&ModelConfig> = self.models.iter().filter(|m| (!needs_tools || m.capabilities.supports_tools) && (!needs_streaming || m.capabilities.supports_streaming) && (!needs_images || m.capabilities.supports_images) && m.capabilities.max_context_tokens >= min_context_tokens).collect();
        candidates.sort_by(|a, b| a.capabilities.cost_per_1k_tokens.partial_cmp(&b.capabilities.cost_per_1k_tokens).unwrap_or(std::cmp::Ordering::Equal));
        candidates.into_iter().next()
    }
    pub fn models_by_provider(&self, provider: &str) -> Vec<&ModelConfig> { self.models.iter().filter(|m| m.provider == provider).collect() }
    pub fn converter(&self, provider: &str) -> Option<&dyn FormatConverter> { self.converters.get(provider).map(|c| c.as_ref()) }
    pub fn to_provider_format(&self, request: &UnifiedRequest, model: &ModelConfig) -> Option<String> { self.converter(&model.provider).map(|c| c.to_provider_format(request, model)) }
    pub fn from_provider_response(&self, raw: &str, model: &ModelConfig) -> Option<UnifiedResponse> { self.converter(&model.provider).and_then(|c| c.from_provider_response(raw, model)) }
    pub fn models(&self) -> &[ModelConfig] { &self.models }
    pub fn stats(&self) -> (usize, usize, usize) {
        let with_tools = self.models.iter().filter(|m| m.capabilities.supports_tools).count();
        let with_streaming = self.models.iter().filter(|m| m.capabilities.supports_streaming).count();
        (self.models.len(), with_tools, with_streaming)
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    if rest.starts_with('"') { let end = rest[1..].find('"')? + 1; Some(rest[1..end].to_string()) } else { None }
}

fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(rest.len());
    rest[..end].parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_maintenance_window_detects_empero() {
        assert!(is_maintenance_window("We are switching the free endpoint to new models. retrying in 29m 57s"));
        assert!(!is_maintenance_window("503 Service Unavailable"));
        assert!(!is_maintenance_window("rate limit exceeded"));
    }
}
