//! 类型 & 注册表 — ProviderState, CallEvent, SubGrid, Benchmark

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::time::{Duration, Instant};

use crate::core::nt_core_llm::{LlmError, LlmRequest};
use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::lookup_provider;
use crate::l1_action::nt_io::nt_io_provider::health::circuit_breaker::CircuitBreaker;
use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::{CommunicationProfile, ProviderCategory};
use crate::l1_action::nt_io::nt_io_provider::health::rate_limiter::RateLimiter;
use super::GatewayV2;

// ═══════════════════════════════════════════════════════════════════
// Auto Exacto 周期重估注册表 (R-P79 生产接线)
// ═══════════════════════════════════════════════════════════════════

/// 进程级活跃 GatewayV2 注册表 — Weak 持有, 网关释放后自动剔除。
pub static RE_EVALUATION_GATEWAYS: LazyLock<Mutex<Vec<Weak<GatewayV2>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// 注册一个进程级共享 GatewayV2 参与 Auto Exacto 周期重估。
pub fn register_gateway_for_re_evaluation(gateway: &Arc<GatewayV2>) {
    if let Ok(mut registry) = RE_EVALUATION_GATEWAYS.lock() {
        registry.retain(|w| w.strong_count() > 0);
        registry.push(Arc::downgrade(gateway));
    }
}

/// 周期驱动 Auto Exacto 重估 — 遍历注册的活跃 GatewayV2 调用
/// [`GatewayV2::maybe_re_evaluate`]。返回本次实际触发重估的网关数。
pub fn run_periodic_re_evaluation() -> usize {
    let mut registry = match RE_EVALUATION_GATEWAYS.lock() {
        Ok(reg) => reg,
        Err(e) => {
            log::warn!("[gateway] re-evaluation registry poisoned: {}", e);
            e.into_inner()
        }
    };
    registry.retain(|w| w.strong_count() > 0);
    let mut evaluated = 0usize;
    for weak in registry.iter() {
        if let Some(gw) = weak.upgrade() {
            if gw.maybe_re_evaluate() {
                evaluated += 1;
            }
        }
    }
    evaluated
}

/// 当前注册表中仍存活 (`strong_count > 0`) 的网关注册数。
pub fn registered_gateway_count() -> usize {
    RE_EVALUATION_GATEWAYS
        .lock()
        .map(|reg| reg.iter().filter(|w| w.strong_count() > 0).count())
        .unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════════
// Provider State — provider 运行时状态
// ═══════════════════════════════════════════════════════════════════

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
    /// 模型级锁 (L3 韧性, 对齐 OmniRoute model lockout)
    pub model_locks: HashMap<String, Instant>,
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
            model_locks: HashMap::new(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.circuit_breaker.is_available()
    }

    /// 配额耗尽标记 — 记录 provider 处于配额耗尽状态 (freellmapi/aimux 模式)。
    pub fn mark_quota_exhausted(&mut self) {
        self.circuit_breaker.force_open();
        self.total_errors += 1;
    }

    /// 配额耗尽后是否已过冷却期可再次尝试 (配额恢复探测)
    pub(crate) fn _quota_recovery_elapsed(&self) -> bool {
        self.circuit_breaker.cooldown_elapsed()
    }

    /// 锁定某模型 (cooldown_secs 后自动解锁), 不影响该 provider 的其它模型。
    pub fn lock_model(&mut self, model: &str, cooldown_secs: u64) {
        self.model_locks.insert(
            model.to_string(),
            Instant::now() + Duration::from_secs(cooldown_secs),
        );
    }

    /// 该模型当前是否被锁定 (过期视为未锁, 但仍保留在 map 中, 由 prune 清理)。
    pub fn is_model_locked(&self, model: &str) -> bool {
        match self.model_locks.get(model) {
            Some(&until) => Instant::now() < until,
            None => false,
        }
    }

    /// 清理已过期的模型锁, 防止 map 无限增长
    pub fn prune_expired_model_locks(&mut self) {
        let now = Instant::now();
        self.model_locks.retain(|_, &mut until| now < until);
    }

    pub fn composite_score(&self) -> f64 {
        let health = self.circuit_breaker.health_penalty();
        if health <= 0.0 {
            return 0.0;
        }
        let latency_factor = {
            let mut sorted = self.latency_window.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p95 = sorted
                .get((sorted.len() as f64 * 0.95) as usize)
                .copied()
                .unwrap_or(1000.0);
            if p95 > 0.0 {
                1.0 / (p95 / 1000.0).max(0.1)
            } else {
                0.5
            }
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
        self.prune_expired_model_locks();
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
        self.prune_expired_model_locks();
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

/// 子网格: 由同一安全画像的 provider 组成的小循环通信单元
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
    pub fn new(
        name: String,
        security_profile: CommunicationProfile,
        provider_names: Vec<String>,
    ) -> Self {
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

// ═══════════════════════════════════════════════════════════════════
// Benchmark Types — L1-local benchmark types for challenge evaluation
// ═══════════════════════════════════════════════════════════════════

/// An evaluation case for LLM challenge testing.
#[derive(Debug, Clone)]
pub struct OriEvalCase {
    pub id: String,
    pub prompt: String,
    /// Expected tool name (correctness check: pass if any match)
    pub expected_tool: Option<String>,
    /// Keywords expected in response (rubric scoring)
    pub rubric_keywords: Vec<String>,
    /// Whether tool call is required (necessity check)
    pub requires_tool: bool,
}

impl OriEvalCase {
    pub fn new(
        id: &str,
        prompt: &str,
        expected_tool: Option<&str>,
        rubric_keywords: &[&str],
        requires_tool: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            expected_tool: expected_tool.map(|s| s.to_string()),
            rubric_keywords: rubric_keywords.iter().map(|s| s.to_string()).collect(),
            requires_tool,
        }
    }
}

/// Score for a single evaluation case.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OriCaseScore {
    pub case_id: String,
    pub answer_grade: f64,         // rubric keyword hit rate 0.0-1.0
    pub tool_call_legit: bool,     // was tool call in expected set?
    pub tool_call_necessary: bool, // was tool call needed?
}

/// Model-level aggregate score.
#[derive(Debug, Clone)]
pub struct OriModelScore {
    pub model: String,
    pub accuracy: f64,
    pub tool_accuracy: f64,
    pub tool_necessity: f64,
    pub composite: f64,
}

/// Evaluation report with per-model scores and ranking.
#[derive(Debug, Clone)]
pub struct OriEvalReport {
    pub per_model: Vec<OriModelScore>,
    /// Models ranked by composite score (index 0 = best)
    pub ranking: Vec<String>,
    pub timestamp: String,
}

impl OriEvalReport {
    /// Best model (rank 0).
    pub fn best_model(&self) -> Option<&str> {
        self.ranking.first().map(|s| s.as_str())
    }
}

/// Evaluation suite that runs cases across models.
#[derive(Debug, Default)]
pub struct OriEvalSuite {
    pub cases: Vec<OriEvalCase>,
}

impl OriEvalSuite {
    pub fn new(cases: Vec<OriEvalCase>) -> Self {
        Self { cases }
    }

    /// Run evaluation cases through a provider and score them.
    pub async fn score_with_provider(
        &self,
        model_name: &str,
        provider: &(dyn crate::core::nt_core_llm::LlmProvider + Send + Sync),
    ) -> Result<OriModelScore, crate::core::nt_core_llm::LlmError> {
        use crate::l1_action::nt_io::nt_io_provider::common::types::LlmRequest;

        let mut correct = 0usize;
        let mut tool_correct = 0usize;
        let mut tool_needed = 0usize;

        for case in &self.cases {
            let request = LlmRequest::new(model_name, &case.prompt);
            let resp = provider.complete(&request).await?;
            let content = resp.content.to_lowercase();

            // Rubric keyword scoring
            let keyword_hits = case.rubric_keywords.iter()
                .filter(|kw| content.contains(&kw.to_lowercase()))
                .count();
            let answer_grade = if case.rubric_keywords.is_empty() {
                1.0
            } else {
                keyword_hits as f64 / case.rubric_keywords.len() as f64
            };
            if answer_grade >= 0.5 {
                correct += 1;
            }

            // Tool call scoring (simplified: check if tool call info exists in response)
            let has_tool = resp.tool_calls.as_ref().map_or(false, |tc| !tc.is_empty());
            if let Some(ref expected) = case.expected_tool {
                if has_tool && content.contains(&expected.to_lowercase()) {
                    tool_correct += 1;
                }
            }
            if case.requires_tool && has_tool {
                tool_needed += 1;
            }
        }

        let total = self.cases.len() as f64;
        let accuracy = correct as f64 / total;
        let tool_accuracy = if self.cases.iter().any(|c| c.expected_tool.is_some()) {
            tool_correct as f64 / self.cases.iter().filter(|c| c.expected_tool.is_some()).count() as f64
        } else {
            1.0
        };
        let tool_necessity = if self.cases.iter().any(|c| c.requires_tool) {
            tool_needed as f64 / self.cases.iter().filter(|c| c.requires_tool).count() as f64
        } else {
            1.0
        };
        let composite = accuracy * 0.5 + tool_accuracy * 0.25 + tool_necessity * 0.25;

        Ok(OriModelScore {
            model: model_name.to_string(),
            accuracy,
            tool_accuracy,
            tool_necessity,
            composite,
        })
    }

    /// Finalize scores into a report.
    pub fn finalize_report(scores: Vec<OriModelScore>) -> OriEvalReport {
        let mut ranking: Vec<String> = scores.iter().map(|s| s.model.clone()).collect();
        ranking.sort_by(|a, b| {
            let score_a = scores.iter().find(|s| s.model == *a).map(|s| s.composite).unwrap_or(0.0);
            let score_b = scores.iter().find(|s| s.model == *b).map(|s| s.composite).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        OriEvalReport {
            per_model: scores,
            ranking,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// LLM Challenge — Deterministic challenge tasks scoring provider accuracy/latency/cost
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
    /// Run the deterministic challenge suite against a provider. Returns a
    /// scored benchmark (accuracy, latency, cost) for the EvolutionFruit
    /// evidence chain and GatewayV2 provider selection.
    pub async fn run_llm_challenge(
        &self,
        provider_name: &str,
        task_type: &str,
    ) -> Result<crate::core::nt_core_consciousness_tree::ProviderBenchmark, LlmError> {
        let tasks = self.challenge_tasks(task_type);
        let mut correct = 0usize;
        let mut total_latency_ms = 0u64;
        let mut total_cost = 0.0f64;

        for task in tasks {
            let request = LlmRequest::new(
                &self.provider_model(provider_name).unwrap_or_default(),
                &task.prompt,
            );
            let start = Instant::now();
            let resp = self.call_provider_backoff(provider_name, &request).await?;
            total_latency_ms += start.elapsed().as_millis() as u64;
            total_cost += (resp.usage.total_tokens as f64 / 1000.0) * 0.002;
            if task.check(&resp.content) {
                correct += 1;
            }
        }

        let task_count = 4usize;
        Ok(crate::core::nt_core_consciousness_tree::ProviderBenchmark {
            provider: provider_name.to_string(),
            model: self
                .provider_model(provider_name)
                .unwrap_or_else(|| provider_name.to_string()),
            accuracy: correct as f64 / task_count as f64,
            latency_ms: total_latency_ms / task_count as u64,
            cost_usd: total_cost,
            task_type: task_type.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Deterministic challenge suite — answers are exact-match scored.
    pub(super) fn challenge_tasks(&self, task_type: &str) -> Vec<ChallengeTask> {
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
    pub(super) fn provider_model(&self, provider_name: &str) -> Option<String> {
        let parts: Vec<&str> = provider_name.split('/').collect();
        if parts.len() <= 1 {
            let name = parts.first().copied().unwrap_or(provider_name);
            if name.is_empty() {
                return None;
            }
            if let Some(info) = lookup_provider(name) {
                if info.is_free && !info.default_model.is_empty() {
                    return Some(info.default_model.to_string());
                }
            }
            return Some(name.to_string());
        }
        Some(parts[1..].join("/"))
    }

    /// F7: Ori-Eval 生产接线 (R-P79)
    pub async fn run_ori_eval_self(
        &self,
        cases: Vec<OriEvalCase>,
        model_names: &[&str],
    ) -> Result<OriEvalReport, LlmError> {
        let suite = OriEvalSuite::new(cases);
        let mut scores = Vec::new();
        for name in model_names {
            let score = suite.score_with_provider(name, self).await?;
            scores.push(score);
        }
        Ok(OriEvalSuite::finalize_report(scores))
    }
}

/// LLM Challenge deterministic task — exact-match scored benchmark item.
pub(super) struct ChallengeTask {
    prompt: String,
    expected: String,
}

impl ChallengeTask {
    pub(super) fn check(&self, response: &str) -> bool {
        response
            .to_lowercase()
            .contains(&self.expected.to_lowercase())
    }
}
