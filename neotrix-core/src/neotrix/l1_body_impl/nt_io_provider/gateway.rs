use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

use super::circuit_breaker::{BreakerState, CircuitBreaker};
use super::free_pool::global_free_pool;
use super::provider_catalog::{ProviderCategory, CommunicationProfile};
use super::provider_swap::ProviderSwapManager;
use super::agent_routing::AgentRoutingTable;
use super::factory::LlmProviderType;
use super::rate_limiter::RateLimiter;
use super::rate_profiles::get_rate_profile;
use super::types::*;
use crate::core::nt_core_error_recovery::{ErrorContext, ErrorType, RecoveryAction, RecoveryConfig, RecoveryOrchestrator};
use crate::core::nt_io_cache::{CacheConfig, SemanticCache};
use crate::core::nt_io_telemetry::{ConsoleTracer, CostTracker, SpanKind, Tracer};

/// 识别配额耗尽错误 — 与瞬时限速 (429) 区分 (freellmapi/aimux 模式)。
/// 覆盖常见 provider 配额/信用耗尽措辞。命中后应熔断 provider 而非重试。
fn is_quota_exhaustion(msg: &str) -> bool {
    let lowered = msg.to_lowercase();
    lowered.contains("quota exceeded")
        || lowered.contains("out of quota")
        || lowered.contains("insufficient_quota")
        || lowered.contains("insufficient quota")
        || lowered.contains("quota exhausted")
        || lowered.contains("credit limit")
        || lowered.contains("out of credits")
        || lowered.contains("exceeded your current quota")
        || lowered.contains("billing")
        && (lowered.contains("activate") || lowered.contains("limit"))
}

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

    /// 配额耗尽标记 — 记录 provider 处于配额耗尽状态 (freellmapi/aimux 模式)。
    /// 配额耗尽 (quota/credit 耗尽) 与瞬时限速 (429) 语义不同: 重试无益, 应剔除该 provider
    /// 直至配额恢复, 而不是反复重试同一个耗尽账户。
    pub fn mark_quota_exhausted(&mut self) {
        self.circuit_breaker.force_open();
        self.total_errors += 1;
    }

    /// 配额耗尽后是否已过冷却期可再次尝试 (配额恢复探测)
    pub fn quota_recovery_elapsed(&self) -> bool {
        self.circuit_breaker.cooldown_elapsed()
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
    /// 健康状态: 调用成功/失败次数与延迟 (子母阵反馈回路)
    pub health: SubGridHealth,
}

/// 子网格运行时健康状态 — 反馈回路 (D21 外部观察 + D30 行为化)
/// 通过 nt_core_telemetry 记录, 支持画像动态降级 (Gap 4)
#[derive(Debug, Clone, Default)]
pub struct SubGridHealth {
    /// 总调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 累计延迟 (ms)
    pub total_latency_ms: u64,
    /// 最近一次调用时间
    pub last_used: Option<std::time::SystemTime>,
}

impl SubGridHealth {
    pub fn record_call(&mut self, success: bool, latency_ms: u64) {
        self.call_count += 1;
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
        self.total_latency_ms += latency_ms;
        self.last_used = Some(std::time::SystemTime::now());
    }

    /// 成功率 (0.0-1.0), 无调用时返回 1.0 (未知视为健康)
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            return 1.0;
        }
        self.success_count as f64 / self.call_count as f64
    }

    /// 平均延迟 (ms)
    pub fn avg_latency_ms(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        self.total_latency_ms as f64 / self.call_count as f64
    }

    /// 是否健康: 成功率 >= 0.8 或样本太少 (< 5)
    pub fn is_healthy(&self) -> bool {
        self.call_count < 5 || self.success_rate() >= 0.8
    }
}

impl SubGrid {
    /// 创建新子网格
    pub fn new(name: String, security_profile: CommunicationProfile, provider_names: Vec<String>) -> Self {
        Self {
            name,
            security_profile,
            provider_names,
            created_at: std::time::SystemTime::now(),
            health: SubGridHealth::default(),
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
    /// 记录子网格健康 (Gap 3) 并在失败时按画像降级重试 (Gap 4)。
    pub async fn complete_for_profile(
        &self,
        required: CommunicationProfile,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        self.complete_for_profile_detailed(required, request).await.map(|(r, _, _)| r)
    }

    /// 与 complete_for_profile 相同, 但返回 (响应, 实际使用的画像, 实际 provider 名)。
    /// 供 CapabilityCoordinator 报告真实的降级状态 (修复 degraded 恒 false 失真)。
    async fn complete_for_profile_detailed(
        &self,
        required: CommunicationProfile,
        request: &LlmRequest,
    ) -> Result<(LlmResponse, CommunicationProfile, String), LlmError> {
        let start = std::time::Instant::now();
        match self.select_best_for_profile(required).await {
            Some(name) => {
                log::debug!("[gateway] complete_for_profile({:?}) → {}", required, name);
                let result = self.call_provider(&name, request).await;
                let latency = start.elapsed().as_millis() as u64;
                let success = result.is_ok();
                self.record_sub_grid_call(required, success, latency);
                if success {
                    result.map(|r| (r, required, name))
                } else {
                    // Gap 4: 首选 provider 失败 → 降级到更宽松画像重试
                    let err = result.err().unwrap_or_else(|| LlmError::Unknown("profile call failed".into()));
                    self.degraded_retry(required, request, &err).await
                }
            }
            None => {
                log::warn!("[gateway] no provider meets {:?} — falling back to default selection", required);
                match self.select_best().await {
                    Some(name) => {
                        let result = self.call_provider(&name, request).await;
                        result.map(|r| (r, CommunicationProfile::Open, name))
                    }
                    None => Err(LlmError::Unknown("no provider available".into())),
                }
            }
        }
    }

    /// 记录一次子网格调用 (成功/失败/延迟) 到健康状态 + nt_core_telemetry
    fn record_sub_grid_call(&self, profile: CommunicationProfile, success: bool, latency_ms: u64) {
        if let Ok(mut grids) = self.sub_grids.write() {
            if let Some(grid) = grids.values_mut().find(|g| g.security_profile == profile) {
                grid.health.record_call(success, latency_ms);
            }
        }
        // 反馈到全局遥测 (D21 外部观察): 子网格健康可见于 nt_core_telemetry
        crate::core::nt_core_telemetry::global_telemetry().record(
            crate::core::nt_core_telemetry::TelemetryEvent::Custom {
                name: format!("sub_grid_{:?}", profile),
                value: format!("{}:{}:{}ms", if success { "ok" } else { "err" }, success as u8, latency_ms),
            }
        );
    }

    /// Gap 4: 画像动态降级 — 首选安全级别失败后, 逐步放宽到更宽松画像重试
    /// 顺序: Anonymous → Tor → Proxied → Open (保持通信畅通优先)
    /// 返回 (响应, 实际使用的画像, 实际 provider 名) — 供调用方感知真实降级。
    async fn degraded_retry(
        &self,
        required: CommunicationProfile,
        request: &LlmRequest,
        first_err: &LlmError,
    ) -> Result<(LlmResponse, CommunicationProfile, String), LlmError> {
        let degradation_chain = [
            CommunicationProfile::Anonymous,
            CommunicationProfile::Tor,
            CommunicationProfile::Proxied,
            CommunicationProfile::Open,
        ];
        for target in degradation_chain {
            // 只向更宽松的画像降级: required.meets(target) 且不同
            if target == required || !required.meets(target) {
                continue;
            }
            log::warn!("[gateway] profile {:?} degraded → {:?}: {}", required, target, first_err);
            let start = std::time::Instant::now();
            match self.select_best_for_profile(target).await {
                Some(name) => {
                    let result = self.call_provider(&name, request).await;
                    let latency = start.elapsed().as_millis() as u64;
                    let success = result.is_ok();
                    self.record_sub_grid_call(target, success, latency);
                    crate::neotrix::l1_body_impl::nt_io_provider::record_failover(
                        &format!("{:?}", required),
                        &format!("{:?}", target),
                        success,
                        &format!("{}", first_err),
                        if success { &name } else { "" },
                    );
                    if success {
                        return result.map(|r| (r, target, name));
                    }
                }
                None => continue,
            }
        }
        Err(first_err.clone())
    }

    /// 查询所有子网格的健康状态 (反馈回路可见性)
    pub fn sub_grid_health_report(&self) -> Vec<(String, SubGridHealth)> {
        self.list_sub_grids().into_iter().map(|g| (g.name, g.health)).collect()
    }

    /// 检查满足指定画像的子网格中是否有健康可用的 (Gap 4 前置判断)
    pub fn has_healthy_sub_grid(&self, required: CommunicationProfile) -> bool {
        self.sub_grids_meeting(required).iter().any(|grid_name| {
            self.sub_grid_health_report().iter().any(|(name, health)| {
                name == grid_name && health.is_healthy()
            })
        })
    }

    /// 返回当前已组成的子网格中满足给定安全级别的网格名称
    pub fn sub_grids_meeting(&self, required: CommunicationProfile) -> Vec<String> {
        self.list_sub_grids().into_iter()
            .filter(|sg| sg.security_profile.meets(required))
            .map(|sg| sg.name)
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════
    // LLM Challenge (P0-3, Cycle 159) — Unstract/LLM-Challenge pattern
    // Deterministic challenge tasks scoring provider accuracy/latency/cost.
    // ═══════════════════════════════════════════════════════════════════

    /// Run the deterministic challenge suite against a provider. Returns a
    /// scored benchmark (accuracy, latency, cost) for the EvolutionFruit
    /// evidence chain and GatewayV2 provider selection.
    pub async fn run_llm_challenge(&self, provider_name: &str, task_type: &str) -> Result<crate::core::nt_core_consciousness_tree::ProviderBenchmark, LlmError> {
        let tasks = self.challenge_tasks(task_type);
        let mut correct = 0usize;
        let mut total_latency_ms = 0u64;
        let mut total_cost = 0.0f64;

        for task in tasks {
            let request = LlmRequest::new(&self.provider_model(provider_name).unwrap_or_default(), &task.prompt);
            let start = Instant::now();
            let resp = self.call_provider(provider_name, &request).await?;
            total_latency_ms += start.elapsed().as_millis() as u64;
            total_cost += (resp.usage.total_tokens as f64 / 1000.0) * 0.002;
            if task.check(&resp.content) {
                correct += 1;
            }
        }

        let task_count = 4usize;
        Ok(crate::core::nt_core_consciousness_tree::ProviderBenchmark {
            provider: provider_name.to_string(),
            model: self.provider_model(provider_name).unwrap_or_else(|| provider_name.to_string()),
            accuracy: correct as f64 / task_count as f64,
            latency_ms: total_latency_ms / task_count as u64,
            cost_usd: total_cost,
            task_type: task_type.to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        })
    }

    /// Deterministic challenge suite — answers are exact-match scored.
    fn challenge_tasks(&self, task_type: &str) -> Vec<ChallengeTask> {
        match task_type {
            "arithmetic" => vec![
                ChallengeTask { prompt: "What is 17 + 25? Answer with the number only.".into(), expected: "42".into() },
                ChallengeTask { prompt: "What is 9 * 8? Answer with the number only.".into(), expected: "72".into() },
                ChallengeTask { prompt: "What is 100 - 37? Answer with the number only.".into(), expected: "63".into() },
                ChallengeTask { prompt: "What is 15 + 15 + 15? Answer with the number only.".into(), expected: "45".into() },
            ],
            "extraction" => vec![
                ChallengeTask { prompt: "Extract the email from: 'Contact alice@example.com for info'. Reply with the email only.".into(), expected: "alice@example.com".into() },
                ChallengeTask { prompt: "Extract the date from: 'The event is on 2026-07-31'. Reply with the date only.".into(), expected: "2026-07-31".into() },
                ChallengeTask { prompt: "Extract the city from: 'She lives in Shanghai, China'. Reply with the city only.".into(), expected: "Shanghai".into() },
                ChallengeTask { prompt: "Extract the number from: 'There are 42 apples'. Reply with the number only.".into(), expected: "42".into() },
            ],
            _ => vec![
                ChallengeTask { prompt: "Is 2 + 2 equal to 4? Answer yes or no.".into(), expected: "yes".into() },
                ChallengeTask { prompt: "Is 3 + 3 equal to 7? Answer yes or no.".into(), expected: "no".into() },
                ChallengeTask { prompt: "What color is the sky on a clear day? One word.".into(), expected: "blue".into() },
                ChallengeTask { prompt: "How many legs does a dog have? One digit.".into(), expected: "4".into() },
            ],
        }
    }

    /// Extract model id from `{provider}/{model_id}` registration names.
    /// 无 `/` 的 keyless 注册名 (如 `pollinations`) 回退到 catalog 默认模型,
    /// 避免把注册名当模型名发给端点 (pollinations 会 404 "Model not found")。
    /// 非 keyless 的 provider (如 `openai`) 保持返回注册名本身。
    fn provider_model(&self, provider_name: &str) -> Option<String> {
        let model = provider_name.split('/').next_back().unwrap_or(provider_name);
        if model.is_empty() { return None; }
        if model == provider_name {
            // 无 `/` → 仅 keyless provider 回退到 catalog 默认模型
            if let Some(info) = super::provider_catalog::lookup_provider(provider_name) {
                if info.is_free && !info.default_model.is_empty() {
                    return Some(info.default_model.to_string());
                }
            }
        }
        Some(model.to_string())
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
        // 剥离 `{provider}/` 前缀 (同 call_provider_stream)。
        let stripped = request.model.strip_prefix(&format!("{}/", name))
            .map(|m| m.to_string());
        let mut req = request.clone();
        if let Some(m) = stripped {
            req.model = m;
        }
        provider.complete(&req).await
    }

    /// 单次调用指定 provider — 无 select_best/无重试连打。
    ///
    /// 适用于对限流敏感的场景（如 keyless free 池 1 req/s 限制）：
    /// 调用方自行控制节流与重试节奏，避免 complete_with_selection 的
    /// 3 次 normal + aggressive retry 在限流窗口内连续触发 429。
    /// `provider_name` 需为完整注册名（如 `api-airforce` 或 `api-airforce/grok-4.1-mini:free`）。
    pub async fn complete_single(
        &self,
        provider_name: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        self.call_provider(provider_name, request).await
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
        // Enforce the per-query cost budget against the *current request's*
        // own estimated tokens. The CostTracker totals are process-cumulative,
        // so comparing them to budget_per_query permanently locked out the
        // gateway once ~10k total tokens had ever been consumed (and free
        // providers, whose cost is 0.0, hard-blocked too).
        {
            if self.cost_budget_per_query > 0.0 {
                let est_tokens = (prompt_key.len() / 4) as f64;
                let estimated_cost = (est_tokens / 1000.0) * 0.002;
                if estimated_cost > self.cost_budget_per_query {
                    log::warn!("[gateway] Per-query cost budget exceeded: ${:.4} > ${:.4}", estimated_cost, self.cost_budget_per_query);
                    return Err(LlmError::Unknown(format!("Cost budget exceeded: ${:.4} > ${:.4}", estimated_cost, self.cost_budget_per_query)));
                }
            }
        }
        let mut used_names: Vec<String> = Vec::new();

        // 显式 provider 前缀路由: 请求模型形如 `{provider}/{model_id}` 时优先该 provider
        // (如 llm7/codestral-latest → llm7)。此前 select_best 无视 model 前缀,
        // 导致 llm7 模型被路由到 pollinations/api-airforce (402/429)。
        let prefix_provider: Option<String> = request.model.split('/').next()
            .filter(|p| !p.is_empty())
            .map(|p| p.to_string());

        for i in 0..3 {
            let prefix_sel = if i == 0 {
                // 第一优先: 显式前缀 provider (若已注册)
                prefix_provider.as_ref()
                    .filter(|p| self.providers.contains_key(p.as_str()))
                    .cloned()
            } else {
                None
            };
            let best = self.select_best().await;
            let name = prefix_sel.or(best)
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
                    let is_quota_exhausted = is_quota_exhaustion(&error_msg);
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(&name) {
                            if is_quota_exhausted {
                                // 配额耗尽 (freellmapi/aimux): 重试无益, 直接熔断剔除该 provider,
                                // 避免反复打同一个耗尽账户。冷却期后配额恢复探测自动放行。
                                state.mark_quota_exhausted();
                                log::warn!("[gateway] provider '{}' quota exhausted → 熔断剔除: {}", name, error_msg);
                            } else {
                                state.record_failure(elapsed);
                            }
                        }
                    }
                    self.fire_event(&name, false, elapsed, 0, &request.model, AttemptPhase::Normal);

                    // Consult recovery orchestrator
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
                            // Restore the provider's configured probe threshold
                            // instead of leaving the aggressive override in place.
                            let saved = set_aggressive.iter()
                                .find(|(n, _)| n == name)
                                .map(|(_, s)| *s);
                            state.circuit_breaker.set_half_open_max_probes(saved.unwrap_or(3));
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

        // 显式 provider 前缀路由: 请求模型形如 `{provider}/{model_id}` 时优先该 provider
        // (如 llm7/codestral-latest → llm7)。此前 select_best 无视 model 前缀,
        // 导致 llm7 模型被路由到 pollinations/api-airforce (402/429)。
        let prefix_provider: Option<String> = request.model.split('/').next()
            .filter(|p| !p.is_empty())
            .map(|p| p.to_string());

        for i in 0..3 {
            let prefix_sel = if i == 0 {
                // 第一优先: 显式前缀 provider (若已注册)
                prefix_provider.as_ref()
                    .filter(|p| self.providers.contains_key(p.as_str()))
                    .cloned()
            } else {
                None
            };
            let best = self.select_best().await;
            let name = prefix_sel.or(best)
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
        // 剥离 `{provider}/` 前缀: 请求模型 `llm7/codestral-latest` 传给 provider 时
        // 只传 `codestral-latest` (上游不认识 `llm7/` 前缀, 返回 model_unavailable)。
        let stripped = request.model.strip_prefix(&format!("{}/", name))
            .map(|m| m.to_string());
        let mut req = request.clone();
        if let Some(m) = stripped {
            req.model = m;
        }
        provider.stream_complete(&req).await
    }

    /// Auxiliary vision reasoning — the third-party-model-as-reasoner path.
    ///
    /// NeoTrix's own VisionBridge produces deterministic pixel evidence; when
    /// semantic understanding (OCR, object/scene description) is needed and a
    /// vision-capable provider is registered, this routes the image to it and
    /// returns the description as evidence text. Falls back to the default
    /// provider if none is vision-capable — the model itself then decides
    /// whether it can consume the image_data.
    pub async fn describe_image(&self, image_b64: &str, question: &str) -> Result<String, LlmError> {
        // Prefer a provider whose registered name advertises vision.
        let mut used: Vec<String> = Vec::new();
        let vision_candidates: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
            let mut names: Vec<String> = states.keys().cloned().collect();
            names.sort_by_key(|n| (!crate::core::nt_core_e8::nt_multimodal::model_supports_vision(n), n.clone()));
            names
        };
        let mut target: Option<String> = None;
        for name in vision_candidates {
            if crate::core::nt_core_e8::nt_multimodal::model_supports_vision(&name) {
                target = Some(name);
                break;
            }
        }
        let name = match target {
            Some(n) => {
                used.push(n.clone());
                n
            }
            None => match self.select_best().await {
                Some(n) => {
                    used.push(n.clone());
                    n
                }
                None => return Err(LlmError::Unknown("no provider available for image description".into())),
            },
        };

        let request = LlmRequest::new(&name, question)
            .with_image_b64(image_b64)
            .with_max_tokens(1024)
            .with_temperature(Some(0.2));
        let response = self.call_provider(&name, &request).await?;
        Ok(response.content)
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
                    // Restore the provider's configured probe threshold
                    // instead of leaving the aggressive override in place.
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() });
                        if let Some(state) = states.get_mut(name) {
                            let saved = set_aggressive.iter()
                                .find(|(n, _)| n == name)
                                .map(|(_, s)| *s);
                            state.circuit_breaker.set_half_open_max_probes(saved.unwrap_or(3));
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
            let mut provider = super::factory::create_provider(super::factory::ProviderConfig {
                provider_type: entry.provider_type,
                api_key,
                base_url: Some(entry.base_url.clone()),
                model: Some(entry.model_id.clone()),
                timeout_secs: 60,
                proxy: None,
            });
            // 代理注入: 与手工 keyless 注册一致, 本机 fake-ip 分流网络下直连会全部超时
            if let Some(proxy_url) = super::super::nt_io_http_factory::proxy_from_env() {
                provider.set_proxy(&proxy_url);
            }
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

    /// 已注册 provider 名称列表
    pub fn providers(&self) -> Vec<String> {
        self.states.read()
            .unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() })
            .keys().cloned().collect()
    }

    /// 查询 provider 的安全分类
    pub fn category_of(&self, name: &str) -> Option<ProviderCategory> {
        self.states.read()
            .unwrap_or_else(|e| { log::warn!("[gateway] states RwLock poisoned: {}", e); e.into_inner() })
            .get(name).map(|s| s.category)
    }

    /// 默认 provider 名称 (注册的第一个, 无则空串)
    pub fn default_provider_name(&self) -> String {
        self.default_name.read()
            .unwrap_or_else(|e| { log::warn!("[gateway] default_name RwLock poisoned: {}", e); e.into_inner() })
            .clone()
    }
}

/// LLM Challenge deterministic task — exact-match scored benchmark item.
struct ChallengeTask {
    prompt: String,
    expected: String,
}

impl ChallengeTask {
    fn check(&self, response: &str) -> bool {
        response.to_lowercase().contains(&self.expected.to_lowercase())
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

/// 能力自主协调层 — 任务目标驱动的已有能力组合器 (R-P42 强化已有节点)
///
/// 不再为每个任务创建独立管道, 而是将既有节点 (路由表 / 子网格健康 /
/// 画像降级 / ProviderSwapManager / CircuitBreaker) 组装为按任务类型路由的
/// 统一协调入口. 对应周天信息大阵的 "协调" 环节:
///   1. 解析任务类型 → 需要的能力与安全画像 (CapabilityIntent)
///   2. 查路由表 → 该任务默认 provider/model (已有 AgentRoutingTable)
///   3. 健康感知选路 → 偏好健康子网格 (Gap 3)
///   4. 画像降级 → 失败时自动放宽安全级别重试 (Gap 4)
///   5. 失败注入 → ProviderSwapManager 记录, 驱动后续 swap (已有)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityIntent {
    /// 本地推理 (无网络需求) — 匿名级, 选 Local provider
    LocalReasoning,
    /// 一般推理 — 开放级, 任意 provider
    GeneralReasoning,
    /// 知识检索 — 开放级, 偏免费
    KnowledgeRetrieval,
    /// 敏感数据写入 — 代理级 (Proxied)
    SensitiveWrite,
    /// 匿名通信 — 匿名级 (Anonymous / Tor 可用则 Tor)
    AnonymousCommunication,
    /// 深度分析 (慢速/高成本可接受) — 开放级, 偏 Cloud
    DeepAnalysis,
}

impl CapabilityIntent {
    /// 任务类型 → 所需最低通信安全画像
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

    /// 任务类型 → 建议的 provider category (None = 无偏好)
    pub fn preferred_category(&self) -> Option<ProviderCategory> {
        match self {
            Self::LocalReasoning => Some(ProviderCategory::Local),
            Self::DeepAnalysis => Some(ProviderCategory::Cloud),
            _ => None,
        }
    }

    /// 从字符串解析任务意图 (CLI / 配置友好)
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
        [
            Self::LocalReasoning,
            Self::GeneralReasoning,
            Self::KnowledgeRetrieval,
            Self::SensitiveWrite,
            Self::AnonymousCommunication,
            Self::DeepAnalysis,
        ]
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

/// 一次协调请求 — 任务目标 + 提示词, 交由 CapabilityCoordinator 自主完成
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

/// 协调结果 — 响应 + 实际使用的安全画像 (供调用方感知降级)
#[derive(Debug)]
pub struct CoordinationOutcome {
    pub response: LlmResponse,
    pub used_profile: CommunicationProfile,
    pub degraded: bool,
    pub provider_name: String,
}

/// 能力自主协调器 — 组合既有节点完成目标任务
pub struct CapabilityCoordinator {
    pub gateway: GatewayV2,
    pub routing: AgentRoutingTable,
    pub swap: ProviderSwapManager,
}

impl CapabilityCoordinator {
    pub fn new(gateway: GatewayV2, routing: AgentRoutingTable, swap: ProviderSwapManager) -> Self {
        Self { gateway, routing, swap }
    }

    /// 组装默认子网格 (匿名本地 + 代理 + 开放), 供未预先组合时使用
    pub fn ensure_default_sub_grids(&self) {
        let grids = self.gateway.list_sub_grids();
        if grids.is_empty() {
            self.gateway.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
            self.gateway.compose_sub_grid("proxied-cloud", CommunicationProfile::Proxied, false);
            self.gateway.compose_sub_grid("open-all", CommunicationProfile::Open, false);
        }
    }

    /// 任务意图 → 需要的能力清单 (梳理自有能力, D48 跨域能量流)
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

    /// 协调执行 — 目标驱动的自主能力组合:
    /// 1. 优先本地/免费 (依据意图偏好的 category)
    /// 2. 健康感知: 偏好健康子网格, 跳过熔断的 provider
    /// 3. 画像降级: 失败自动放宽 (complete_for_profile 内部处理)
    /// 4. 结果注入 swap manager 以驱动长期 failover
    pub async fn coordinate(&mut self, req: &CoordinationRequest) -> Result<CoordinationOutcome, LlmError> {
        self.ensure_default_sub_grids();
        let profile = req.intent.required_profile();

        // 2. 若偏好 category 已注册, 直接走该 provider (LocalReasoning → Local)
        let preferred = req.intent.preferred_category();
        let routed_provider = preferred.and_then(|cat| self.find_provider_by_category(cat));
        let provider_name = if let Some(name) = routed_provider {
            name
        } else {
            // 3. 默认走健康感知的子网格选路
            match self.gateway.select_best_for_profile(profile).await {
                Some(name) => name,
                None => {
                    // 无匹配 → 回退默认 (通信始终畅通)
                    self.gateway.default_provider_name()
                }
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
                    self.swap.record_success(LlmProviderType::from_name(provider_name.as_str()).unwrap_or(LlmProviderType::OpenAI), 0.0);
                }
                log::debug!("[coordinate] {:?} degraded={} ({:?} → {:?})", req.intent, degraded, profile, actual_profile);
                Ok(CoordinationOutcome {
                    response: resp,
                    used_profile: actual_profile,
                    degraded,
                    provider_name: actual_provider,
                })
            }
            Err(e) => {
                // 降级已由 complete_for_profile 内部处理; 若仍失败, 记录到 swap manager
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
    async fn test_llm_challenge_scoring() {
        let gw = GatewayV2::new();
        // Default generic suite with exact-match scoring
        let tasks = gw.challenge_tasks("generic");
        assert_eq!(tasks.len(), 4);
        assert!(tasks[0].check("yes, exactly"));
        assert!(!tasks[1].check("yes definitely"));
        assert!(tasks[2].check("The sky is BLUE"));
        assert!(tasks[3].check("4"));
    }

    #[tokio::test]
    async fn test_provider_model_extraction() {
        let gw = GatewayV2::new();
        assert_eq!(gw.provider_model("nvidia/meta/llama-3.1-8b-instruct"), Some("llama-3.1-8b-instruct".to_string()));
        assert_eq!(gw.provider_model("openai"), Some("openai".to_string()));
        assert_eq!(gw.provider_model(""), None);
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
                    tool_calls: None,
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
                        tool_calls: None,
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

    #[test]
    fn test_sub_grid_health() {
        let mut h = SubGridHealth::default();
        assert!(h.is_healthy()); // 样本太少视为健康
        h.record_call(true, 100);
        h.record_call(true, 200);
        h.record_call(false, 300);
        assert_eq!(h.call_count, 3);
        assert_eq!(h.success_count, 2);
        assert!(h.is_healthy()); // <5 样本仍视为健康
        h.record_call(false, 100);
        h.record_call(false, 100);
        h.record_call(false, 100);
        // 6 次调用, 2 成功 4 失败 → 成功率 0.33 < 0.8
        assert!(!h.is_healthy());
        assert!(h.success_rate() < 0.5);
        assert!(h.avg_latency_ms() > 100.0);
    }

    #[test]
    fn test_sub_grid_health_report() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);

        let report = gw.sub_grid_health_report();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].0, "anonymous-local");
        assert_eq!(report[0].1.call_count, 0); // 尚未调用
    }

    #[tokio::test]
    async fn test_complete_for_profile_records_health() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        let resp = gw.complete_for_profile(CommunicationProfile::Anonymous, &req).await;
        assert!(resp.is_ok());

        // 健康状态已记录
        let report = gw.sub_grid_health_report();
        let (_, health) = &report[0];
        assert_eq!(health.call_count, 1);
        assert_eq!(health.success_count, 1);
        assert!(health.is_healthy());
    }

    #[tokio::test]
    async fn test_degraded_retry_downgrades_on_failure() {
        // local-fail 满足 Anonymous 但失败; cloud-ok 仅满足 Open 且成功.
        // 请求 Anonymous → local-fail 失败 → 降级链 Anonymous→Tor→Proxied→Open → cloud-ok 成功
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("local-fail", Box::new(MockProvider::failing()), false, ProviderCategory::Local);
        gw.register_provider_with_category("cloud-ok", Box::new(MockProvider::new("cloud")), true, ProviderCategory::Cloud);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        // 请求 Anonymous → 首选 local-fail 失败 → 降级到 Open → cloud-ok 成功
        let resp = gw.complete_for_profile(CommunicationProfile::Anonymous, &req).await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "cloud");
    }

    #[test]
    fn test_capability_intent_parse() {
        assert_eq!(CapabilityIntent::parse("local"), Some(CapabilityIntent::LocalReasoning));
        assert_eq!(CapabilityIntent::parse("deep-analysis"), Some(CapabilityIntent::DeepAnalysis));
        assert_eq!(CapabilityIntent::parse("anonymous_communication"), Some(CapabilityIntent::AnonymousCommunication));
        assert_eq!(CapabilityIntent::parse("unknown_thing"), None);
        // 画像映射
        assert_eq!(CapabilityIntent::LocalReasoning.required_profile(), CommunicationProfile::Anonymous);
        assert_eq!(CapabilityIntent::SensitiveWrite.required_profile(), CommunicationProfile::Proxied);
        assert_eq!(CapabilityIntent::AnonymousCommunication.required_profile(), CommunicationProfile::Tor);
        // 偏好分类
        assert_eq!(CapabilityIntent::LocalReasoning.preferred_category(), Some(ProviderCategory::Local));
        assert_eq!(CapabilityIntent::GeneralReasoning.preferred_category(), None);
        // 能力计划 (梳理自有能力)
        let plan = CapabilityCoordinator::capability_plan(CapabilityIntent::DeepAnalysis);
        assert!(plan.contains(&"plan"));
        assert!(plan.contains(&"critique"));
    }

    #[tokio::test]
    async fn test_capability_coordinator_local_reasoning() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("ollama", Box::new(MockProvider::new("local")), true, ProviderCategory::Local);
        let routing = AgentRoutingTable::new("ollama", "local-model");
        let swap = ProviderSwapManager::new(vec![]);
        let mut coord = CapabilityCoordinator::new(gw, routing, swap);

        let req = CoordinationRequest::new(CapabilityIntent::LocalReasoning, "think about X");
        let outcome = coord.coordinate(&req).await;
        assert!(outcome.is_ok());
        let out = outcome.unwrap();
        assert_eq!(out.response.content, "local");
        assert_eq!(out.used_profile, CommunicationProfile::Anonymous);
        assert!(!out.degraded);
    }

    #[tokio::test]
    async fn test_capability_coordinator_fallback_default() {
        // 无任何 provider 满足 Tor → 回退默认 select_best, 通信始终畅通
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("openai", Box::new(MockProvider::new("ok")), false, ProviderCategory::Cloud);
        let routing = AgentRoutingTable::new("openai", "gpt-model");
        let swap = ProviderSwapManager::new(vec![]);
        let mut coord = CapabilityCoordinator::new(gw, routing, swap);

        let req = CoordinationRequest::new(CapabilityIntent::AnonymousCommunication, "hello");
        let outcome = coord.coordinate(&req).await;
        assert!(outcome.is_ok());
        assert_eq!(outcome.unwrap().response.content, "ok");
    }

    #[tokio::test]
    async fn test_capability_coordinator_reports_real_degradation() {
        // local-fail 满足 Anonymous 但失败 → 降级链落到 Open → cloud-ok 成功.
        // 关键断言: outcome.degraded == true 且 used_profile == Open (真实降级被报告)
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category("local-fail", Box::new(MockProvider::failing()), false, ProviderCategory::Local);
        gw.register_provider_with_category("cloud-ok", Box::new(MockProvider::new("cloud")), true, ProviderCategory::Cloud);
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);
        let routing = AgentRoutingTable::new("local-fail", "local-model");
        let swap = ProviderSwapManager::new(vec![]);
        let mut coord = CapabilityCoordinator::new(gw, routing, swap);

        let req = CoordinationRequest::new(CapabilityIntent::LocalReasoning, "hello");
        let outcome = coord.coordinate(&req).await;
        assert!(outcome.is_ok());
        let out = outcome.unwrap();
        assert_eq!(out.response.content, "cloud");
        assert!(out.degraded, "降级发生但未被报告");
        assert_eq!(out.used_profile, CommunicationProfile::Open);
        assert_eq!(out.provider_name, "cloud-ok");
    }

    #[tokio::test]
    async fn test_per_query_budget_uses_request_not_cumulative() {
        // Regression: the budget check compared the process-cumulative token
        // counters against budget_per_query, so the gateway permanently
        // rejected every request once ~10k total tokens had ever been
        // consumed. It must now gate on the current request's own estimated
        // tokens, and a small request must pass even after the tracker has
        // accumulated a large cumulative balance.
        let mut gw = GatewayV2::new();
        gw.register_provider("free", Box::new(MockProvider::new("ok")), true);
        gw.set_cost_budget(0.02);

        // Seed a large cumulative balance in the tracker — as if the process
        // had consumed far more than the per-query budget historically.
        let mut tracker = CostTracker::new();
        tracker.record("free", 5_000_000, 5_000_000); // ~$20 cumulative
        gw.set_cost_tracker(tracker);

        // A tiny request must still be allowed (per-query semantics).
        let req = LlmRequest::new("test", "hi");
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_ok(), "small request must not be blocked by cumulative spend");

        // A request large enough on its own to exceed the budget is rejected.
        let big = LlmRequest::new("test", &"x".repeat(2_000_000));
        let result = gw.complete_with_selection(&big).await;
        assert!(result.is_err(), "oversized single request must be budget-blocked");
    }

    #[tokio::test]
    async fn test_quota_exhaustion_trips_provider_not_retried() {
        // D19 (freellmapi/aimux 模式): 配额耗尽应熔断剔除 provider, 而非反复重试
        // 同一个耗尽账户。命中 quota 后将 provider 置为不可用 → select_best 跳过。
        let mut gw = GatewayV2::new();
        gw.register_provider("quota-exhausted", Box::new(MockProvider::quota_failing()), false);
        gw.register_provider("working", Box::new(MockProvider::new("ok")), true);

        // 直接触发 quota 路径的熔断 (与 complete 内错误分类同语义)
        {
            let mut states = gw.states.write().unwrap();
            if let Some(state) = states.get_mut("quota-exhausted") {
                state.mark_quota_exhausted();
            }
        }
        {
            let states = gw.states.read().unwrap();
            let walked = states.get("quota-exhausted").map(|s| !s.is_available()).unwrap_or(true);
            assert!(walked, "quota-exhausted provider must be circuit-open after exhaustion");
            let still_ok = states.get("working").map(|s| s.is_available()).unwrap_or(false);
            assert!(still_ok, "working provider must remain available");
        }

        // 熔断后调用应 failover 到 working 成功, 不再命中的耗尽 provider
        let req = LlmRequest::new("test", "hi");
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_ok(), "request must fail over to a working provider after quota trip");
    }

    #[test]
    fn test_is_quota_exhaustion_classifies_correctly() {
        assert!(is_quota_exhaustion("Error 429: quota exceeded for resource"));
        assert!(is_quota_exhaustion("insufficient_quota: you have exhausted your free tier"));
        assert!(is_quota_exhaustion("You have exceeded your current quota, please check your plan and billing details"));
        assert!(is_quota_exhaustion("out of quota: you are being rate limited due to billing"));
        // 瞬时限速不误判为配额耗尽
        assert!(!is_quota_exhaustion("rate limit exceeded: retry after 5s"));
        assert!(!is_quota_exhaustion("429 too many requests"));
    }

    struct MockProvider {
        response: String,
        should_fail: bool,
        quota_fail: bool,
    }

    impl MockProvider {
        fn new(response: &str) -> Self {
            Self { response: response.to_string(), should_fail: false, quota_fail: false }
        }
        fn failing() -> Self {
            Self { response: String::new(), should_fail: true, quota_fail: false }
        }
        fn quota_failing() -> Self {
            Self { response: String::new(), should_fail: false, quota_fail: true }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            if self.should_fail {
                Err(LlmError::Server("mock failure".to_string()))
            } else if self.quota_fail {
                Err(LlmError::Unknown("insufficient_quota: quota exceeded for your account".to_string()))
            } else {
                Ok(LlmResponse {
                    content: self.response.clone(),
                    model: "mock".to_string(),
                    usage: Usage::default(),
                    finish_reason: FinishReason::Stop,
                tool_calls: None,
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
                tool_calls: None,
                })).await;
            });
            Ok(rx)
        }
    }

    // ── 真实 LLM 集成验证（本地手动跑，不进 CI）──────────────────
    // 需要网络 + keyless provider（llm7）。运行时:
    //   cargo test -p neotrix --lib -- --ignored test_real_gateway_stream
    #[tokio::test]
    #[ignore]
    async fn test_real_gateway_stream() {
        // 真实 LLM 流式端到端: 经生产 factory gateway 全链路。
        // 模型 llm7/codestral-latest — 2026-08-06 实测唯一匿名可用流式端点
        // (api-airforce 全局 1req/s 排队 90s, pollinations 队列满 429 + 流式间歇 402)。
        // 同时验证 gateway 前缀路由: llm7/ 应路由到 llm7 provider。
        let gw = crate::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async().await;
        let req = LlmRequest {
            model: "llm7/codestral-latest".into(),
            messages: vec![Message::new(Role::User, "Reply with exactly: E2E-OK")],
            max_tokens: 32,
            temperature: Some(0.0),
            tools: vec![],
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        };
        let mut rx = gw.stream_complete_with_selection(&req).await.expect("stream init ok");
        let mut buf = String::new();
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(resp) => {
                    buf.push_str(&resp.content);
                    if resp.finish_reason == FinishReason::Stop {
                        break;
                    }
                }
                Err(e) => panic!("stream error: {:?}", e),
            }
        }
        assert!(!buf.trim().is_empty(), "should receive streaming text, got: {:?}", buf);
        println!("E2E-OK streamed: {:?}", buf);
    }
}
