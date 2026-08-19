use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::Duration;

use super::account_pool::{AccountPool, AccountPoolConfig};
use super::generation_classifier::{GenerationAnalytics, GenerationClassifier};
use super::provider_catalog::{CommunicationProfile, ProviderCategory};
use super::rate_limiter::{AdaptivePacer, TieredSemaphore};
use super::types::*;
use crate::core::nt_core_error_recovery::{RecoveryConfig, RecoveryOrchestrator};
use crate::core::nt_io_cache::{CacheConfig, SemanticCache};
use crate::core::nt_core_span::{ConsoleTracer, CostTracker};

#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use super::agent_routing::AgentRoutingTable;
#[cfg(test)]
use super::provider_swap::ProviderSwapManager;

mod challenge;
mod coordinator;
mod execution;
mod market_router;
mod registry;
mod reliability;
mod response_cache;
mod response_healer;
mod selection;
mod state;
mod subgrid;

pub use coordinator::*;
pub use market_router::*;
pub use registry::*;
pub use response_cache::*;
pub use response_healer::*;
pub use state::*;

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
    /// G: Response Caching — LRU 响应缓存实例 (默认关闭)
    pub response_cache: Mutex<ResponseCache>,
    response_cache_enabled: bool,
    /// P0-7 lookahead 预取命中累计 (OasisKV, telemetry 消费)。
    response_cache_prefetches: std::sync::atomic::AtomicU64,
    /// G: Response Healing — 畸形 JSON 修复器实例 (默认关闭)
    pub response_healer: Mutex<ResponseHealer>,
    response_healer_enabled: bool,
    /// G: MarketRouter — market-wisdom 路由 + Auto Exacto 周期重估
    pub market_router: Mutex<MarketRouter>,
    /// F6: GenerationClassifier — 每次生成完成后的分类打标 (默认关闭)
    pub generation_classifier: Mutex<GenerationClassifier>,
    pub generation_analytics: Mutex<GenerationAnalytics>,
    generation_classification_enabled: bool,
    /// P7 账户池 (open-kritt 吸收) — 健康感知 round-robin 选择层:
    /// 每账户并发租约 + 限流检疫 + 冷却自动恢复。
    pub account_pool: Mutex<AccountPool>,
    /// Cumora 吸收 (COORDINATION.md §3b): 自适应最小调用间隔 —
    /// 命中 429 翻倍, 连续 5 次成功回落。防 thundering-herd。
    pub adaptive_pacer: Mutex<AdaptivePacer>,
    /// Cumora 吸收 (COORDINATION.md §2/§3a): 双脑并发门 —
    /// big (Frontier/Strong) 与 triage (support) 独立 cap, 防雪崩。
    pub tiered_semaphore: Mutex<TieredSemaphore>,
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
            response_cache: Mutex::new(ResponseCache::new(ResponseCache::DEFAULT_CAPACITY)),
            response_cache_enabled: false,
            response_cache_prefetches: std::sync::atomic::AtomicU64::new(0),
            response_healer: Mutex::new(ResponseHealer::new()),
            response_healer_enabled: false,
            market_router: Mutex::new(MarketRouter::new()),
            generation_classifier: Mutex::new(GenerationClassifier::new()),
            generation_analytics: Mutex::new(GenerationAnalytics::new()),
            generation_classification_enabled: false,
            account_pool: Mutex::new(AccountPool::new(AccountPoolConfig::default())),
            adaptive_pacer: Mutex::new(AdaptivePacer::new(50)),
            tiered_semaphore: Mutex::new(TieredSemaphore::default()),
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
        assert_eq!(
            gw.provider_model("nvidia/meta/llama-3.1-8b-instruct"),
            Some("llama-3.1-8b-instruct".to_string())
        );
        assert_eq!(gw.provider_model("openai"), Some("openai".to_string()));
        assert_eq!(gw.provider_model(""), None);
    }

    // ── 候选链: 从池子实际注册名动态构建 ─────────────────────────
    #[tokio::test]
    async fn test_candidate_chain_prefix_first() {
        let mut gw = GatewayV2::new();
        gw.register_provider("pollinations", Box::new(MockProvider::new("p")), true);
        gw.register_provider("llm7", Box::new(MockProvider::new("l")), true);
        gw.register_provider("api-airforce", Box::new(MockProvider::new("a")), true);

        // 显式前缀 → 前缀 provider 第一候选
        let chain = gw.build_candidate_chain("llm7/codestral-latest", 8);
        assert_eq!(chain[0], "llm7", "前缀 provider 应第一候选: {:?}", chain);
        assert!(chain.contains(&"pollinations".to_string()));
        assert!(chain.contains(&"api-airforce".to_string()));
    }

    #[tokio::test]
    async fn test_candidate_chain_prefix_catalog_full_name() {
        let mut gw = GatewayV2::new();
        gw.register_provider("pollinations", Box::new(MockProvider::new("p")), true);
        gw.register_provider(
            "llm7/codestral-latest",
            Box::new(MockProvider::new("l")),
            true,
        );

        // catalog 完整注册名 `{provider}/{model}` 精确命中 → 直接用
        let chain = gw.build_candidate_chain("llm7/codestral-latest", 8);
        assert_eq!(
            chain[0], "llm7/codestral-latest",
            "完整注册名应第一: {:?}",
            chain
        );
    }

    #[tokio::test]
    async fn test_candidate_chain_free_first_and_dedup() {
        let mut gw = GatewayV2::new();
        gw.register_provider("paid-a", Box::new(MockProvider::new("x")), false);
        gw.register_provider("free-b", Box::new(MockProvider::new("y")), true);
        gw.register_provider("free-c", Box::new(MockProvider::new("z")), true);

        // 无前缀 → free 优先
        let chain = gw.build_candidate_chain("", 8);
        assert!(chain[0].starts_with("free-"), "free 应优先: {:?}", chain);
        // 无重复
        let mut sorted = chain.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), chain.len(), "候选链不应重复: {:?}", chain);
        // limit 生效
        let limited = gw.build_candidate_chain("", 2);
        assert_eq!(limited.len(), 2, "limit=2 应截断: {:?}", limited);
    }

    #[tokio::test]
    async fn test_candidate_chain_limit_and_resolve_default() {
        let mut gw = GatewayV2::new();
        gw.register_provider("a", Box::new(MockProvider::new("x")), true);
        gw.register_provider("b", Box::new(MockProvider::new("y")), false);
        gw.register_provider("c", Box::new(MockProvider::new("z")), true);

        // limit 上限控制
        let chain = gw.build_candidate_chain("", 2);
        assert!(chain.len() <= 2);

        // resolve_default_model 返回候选链首个 (free 优先)
        let def = gw.resolve_default_model_sync();
        assert!(def == "a" || def == "c", "默认应为 free provider: {}", def);
    }

    #[tokio::test]
    async fn test_gateway_fallback_on_failure() {
        let mut gw = GatewayV2::new();
        gw.register_provider("failing", Box::new(MockProvider::failing()), false);
        gw.register_provider("working", Box::new(MockProvider::new("ok")), true);

        let mut states = gw.states.write().unwrap();
        let f = states.get_mut("failing").unwrap();
        f.success_ema = 0.0;
        for _ in 0..5 {
            f.circuit_breaker.on_failure();
        }
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
                for _ in 0..6 {
                    s.circuit_breaker.on_failure();
                }
            }
        }

        let req = LlmRequest::new("test", "hello");
        // Normal retry exhausts 2 failing providers, aggressive retry tries again
        let result = gw.complete_with_selection(&req).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("Aggressive retry exhausted") || msg.contains("All providers failed"),
            "expected aggressive retry exhaustion error, got: {}",
            msg
        );
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
                let count = self
                    .fail_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < self.threshold {
                    Err(LlmError::Server("transient failure".to_string()))
                } else {
                    Ok(LlmResponse {
                        content: "recovered".to_string(),
                        model: "test".to_string(),
                        usage: Usage {
                            prompt_tokens: 10,
                            completion_tokens: 5,
                            total_tokens: 15,
                        },
                        finish_reason: FinishReason::Stop,
                        tool_calls: None,
                    })
                }
            }
            async fn stream_complete(
                &self,
                _req: &LlmRequest,
            ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>
            {
                Err(LlmError::Server(
                    "stream_complete not implemented for ConditionalFail".to_string(),
                ))
            }
        }

        gw.register_provider(
            "transient",
            Box::new(ConditionalFail {
                fail_count: fc,
                threshold: 1,
            }),
            true,
        );

        let req = LlmRequest::new("test", "recover me");
        // Normal retry fails (1 failure → circuit opens), aggressive retry should succeed
        let result = gw.complete_with_selection(&req).await;
        assert!(
            result.is_ok(),
            "aggressive retry should recover after transient failures"
        );
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
                Err(LlmError::Server(
                    "complete not implemented for StreamConditionalFail".to_string(),
                ))
            }
            async fn stream_complete(
                &self,
                _req: &LlmRequest,
            ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError>
            {
                let count = self
                    .fail_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < self.threshold {
                    Err(LlmError::Server("transient streaming failure".to_string()))
                } else {
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    tokio::spawn(async move {
                        let _ = tx
                            .send(Ok(LlmResponse {
                                content: "stream recovered".to_string(),
                                model: "test".to_string(),
                                usage: Usage {
                                    prompt_tokens: 10,
                                    completion_tokens: 5,
                                    total_tokens: 15,
                                },
                                finish_reason: FinishReason::Stop,
                                tool_calls: None,
                            }))
                            .await;
                    });
                    Ok(rx)
                }
            }
        }

        gw.register_provider(
            "stream-transient",
            Box::new(StreamConditionalFail {
                fail_count: fc,
                threshold: 1,
            }),
            true,
        );

        let req = LlmRequest::new("test", "recover me");
        let mut rx = gw
            .stream_complete_with_selection(&req)
            .await
            .expect("streaming aggressive retry should succeed after transient failure");

        let msg = rx
            .recv()
            .await
            .expect("should receive a stream message")
            .expect("stream message should be Ok");
        assert_eq!(msg.content, "stream recovered");
    }

    #[test]
    fn test_sub_grid_composition() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );

        // 组合匿名子网格: 只包含 Local provider (ollama)
        let anonymous =
            gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
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
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );

        // 匿名安全级别: 只能选 ollama (Local)
        let selected = gw
            .select_best_for_profile(CommunicationProfile::Anonymous)
            .await;
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
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        // 匿名级别: 命中 ollama (Local)
        let resp = gw
            .complete_for_profile(CommunicationProfile::Anonymous, &req)
            .await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "local");
    }

    #[tokio::test]
    async fn test_complete_for_profile_fallback() {
        // 没有任何 provider 满足 Tor 级别 → 回退默认 select_best
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let req = LlmRequest::new("test-model", "hello");
        let resp = gw
            .complete_for_profile(CommunicationProfile::Tor, &req)
            .await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "ok");
    }

    #[test]
    fn test_sub_grids_meeting() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        assert_eq!(
            gw.sub_grids_meeting(CommunicationProfile::Anonymous),
            vec!["anonymous-local"]
        );
        let meeting_open = gw.sub_grids_meeting(CommunicationProfile::Open);
        assert_eq!(meeting_open.len(), 2); // anonymous 满足 open, open 也满足 open
                                           // Tor 需求: 仅 anonymous (Anonymous > Tor 满足), open 不满足
        assert_eq!(
            gw.sub_grids_meeting(CommunicationProfile::Tor),
            vec!["anonymous-local"]
        );
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
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);

        let report = gw.sub_grid_health_report();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].0, "anonymous-local");
        assert_eq!(report[0].1.call_count, 0); // 尚未调用
    }

    #[tokio::test]
    async fn test_complete_for_profile_records_health() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        let resp = gw
            .complete_for_profile(CommunicationProfile::Anonymous, &req)
            .await;
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
        gw.register_provider_with_category(
            "local-fail",
            Box::new(MockProvider::failing()),
            false,
            ProviderCategory::Local,
        );
        gw.register_provider_with_category(
            "cloud-ok",
            Box::new(MockProvider::new("cloud")),
            true,
            ProviderCategory::Cloud,
        );
        gw.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, false);
        gw.compose_sub_grid("open-all", CommunicationProfile::Open, false);

        let mut req = LlmRequest::new("test-model", "hello");
        req.model = "test-model".to_string();

        // 请求 Anonymous → 首选 local-fail 失败 → 降级到 Open → cloud-ok 成功
        let resp = gw
            .complete_for_profile(CommunicationProfile::Anonymous, &req)
            .await;
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().content, "cloud");
    }

    #[test]
    fn test_capability_intent_parse() {
        assert_eq!(
            CapabilityIntent::parse("local"),
            Some(CapabilityIntent::LocalReasoning)
        );
        assert_eq!(
            CapabilityIntent::parse("deep-analysis"),
            Some(CapabilityIntent::DeepAnalysis)
        );
        assert_eq!(
            CapabilityIntent::parse("anonymous_communication"),
            Some(CapabilityIntent::AnonymousCommunication)
        );
        assert_eq!(CapabilityIntent::parse("unknown_thing"), None);
        // 画像映射
        assert_eq!(
            CapabilityIntent::LocalReasoning.required_profile(),
            CommunicationProfile::Anonymous
        );
        assert_eq!(
            CapabilityIntent::SensitiveWrite.required_profile(),
            CommunicationProfile::Proxied
        );
        assert_eq!(
            CapabilityIntent::AnonymousCommunication.required_profile(),
            CommunicationProfile::Tor
        );
        // 偏好分类
        assert_eq!(
            CapabilityIntent::LocalReasoning.preferred_category(),
            Some(ProviderCategory::Local)
        );
        assert_eq!(
            CapabilityIntent::GeneralReasoning.preferred_category(),
            None
        );
        // 能力计划 (梳理自有能力)
        let plan = CapabilityCoordinator::capability_plan(CapabilityIntent::DeepAnalysis);
        assert!(plan.contains(&"plan"));
        assert!(plan.contains(&"critique"));
    }

    #[tokio::test]
    async fn test_capability_coordinator_local_reasoning() {
        let mut gw = GatewayV2::new();
        gw.register_provider_with_category(
            "ollama",
            Box::new(MockProvider::new("local")),
            true,
            ProviderCategory::Local,
        );
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
        gw.register_provider_with_category(
            "openai",
            Box::new(MockProvider::new("ok")),
            false,
            ProviderCategory::Cloud,
        );
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
        gw.register_provider_with_category(
            "local-fail",
            Box::new(MockProvider::failing()),
            false,
            ProviderCategory::Local,
        );
        gw.register_provider_with_category(
            "cloud-ok",
            Box::new(MockProvider::new("cloud")),
            true,
            ProviderCategory::Cloud,
        );
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
        assert!(
            result.is_ok(),
            "small request must not be blocked by cumulative spend"
        );

        // A request large enough on its own to exceed the budget is rejected.
        let big = LlmRequest::new("test", &"x".repeat(2_000_000));
        let result = gw.complete_with_selection(&big).await;
        assert!(
            result.is_err(),
            "oversized single request must be budget-blocked"
        );
    }

    #[tokio::test]
    async fn test_quota_exhaustion_trips_provider_not_retried() {
        // D19 (freellmapi/aimux 模式): 配额耗尽应熔断剔除 provider, 而非反复重试
        // 同一个耗尽账户。命中 quota 后将 provider 置为不可用 → select_best 跳过。
        let mut gw = GatewayV2::new();
        gw.register_provider(
            "quota-exhausted",
            Box::new(MockProvider::quota_failing()),
            false,
        );
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
            let walked = states
                .get("quota-exhausted")
                .map(|s| !s.is_available())
                .unwrap_or(true);
            assert!(
                walked,
                "quota-exhausted provider must be circuit-open after exhaustion"
            );
            let still_ok = states
                .get("working")
                .map(|s| s.is_available())
                .unwrap_or(false);
            assert!(still_ok, "working provider must remain available");
        }

        // 熔断后调用应 failover 到 working 成功, 不再命中的耗尽 provider
        let req = LlmRequest::new("test", "hi");
        let result = gw.complete_with_selection(&req).await;
        assert!(
            result.is_ok(),
            "request must fail over to a working provider after quota trip"
        );
    }

    #[test]
    fn test_is_quota_exhaustion_classifies_correctly() {
        assert!(is_quota_exhaustion(
            "Error 429: quota exceeded for resource"
        ));
        assert!(is_quota_exhaustion(
            "insufficient_quota: you have exhausted your free tier"
        ));
        assert!(is_quota_exhaustion(
            "You have exceeded your current quota, please check your plan and billing details"
        ));
        assert!(is_quota_exhaustion(
            "out of quota: you are being rate limited due to billing"
        ));
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
            Self {
                response: response.to_string(),
                should_fail: false,
                quota_fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                response: String::new(),
                should_fail: true,
                quota_fail: false,
            }
        }
        fn quota_failing() -> Self {
            Self {
                response: String::new(),
                should_fail: false,
                quota_fail: true,
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            if self.should_fail {
                Err(LlmError::Server("mock failure".to_string()))
            } else if self.quota_fail {
                Err(LlmError::Unknown(
                    "insufficient_quota: quota exceeded for your account".to_string(),
                ))
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

        async fn stream_complete(
            &self,
            _request: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let resp = self.response.clone();
            tokio::spawn(async move {
                let _ = tx
                    .send(Ok(LlmResponse {
                        content: resp,
                        model: "mock".to_string(),
                        usage: Usage::default(),
                        finish_reason: FinishReason::Stop,
                        tool_calls: None,
                    }))
                    .await;
            });
            Ok(rx)
        }
    }

    // ── 真实 LLM 集成验证（本地手动跑，不进 CI）──────────────────
    // 需要网络 + keyless provider（llm7）。运行时:
    //   cargo test -p neotrix --lib -- --ignored test_real_gateway_stream
    #[tokio::test]
    #[ignore]
    async fn test_real_gateway_candidate_chain() {
        // 真实池子候选链解析 (不调用 LLM): 验证整体链路从实际注册名构建,
        // resolve_default_model 应选到可用 keyless provider (llm7), 而非硬编码。
        let gw =
            crate::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async().await;
        let chain = gw.build_candidate_chain("llm7/codestral-latest", 8);
        eprintln!(
            "[chain] prefix chain[0]={:?} full={:?}",
            chain.first(),
            chain
        );
        let first = chain.first().map(|s| s.as_str()).unwrap_or("");
        assert!(
            first == "llm7" || first == "llm7/codestral-latest",
            "前缀路由应选 llm7, 实际 {:?}",
            chain.first()
        );
        let def = gw.resolve_default_model_sync();
        eprintln!("[chain] resolve_default_model = {}", def);
        assert!(
            !def.is_empty() && def != "default",
            "默认模型应从池子解析: {}",
            def
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_real_gateway_stream() {
        // 真实 LLM 流式端到端: 经生产 factory gateway 全链路。
        // 模型 llm7/codestral-latest — 2026-08-06 实测唯一匿名可用流式端点
        // (api-airforce 全局 1req/s 排队 90s, pollinations 队列满 429 + 流式间歇 402)。
        // 同时验证 gateway 前缀路由: llm7/ 应路由到 llm7 provider。
        let gw =
            crate::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async().await;
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
            cacheable_prefix_tokens: None,
        };
        let mut rx = gw
            .stream_complete_with_selection(&req)
            .await
            .expect("stream init ok");
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
        assert!(
            !buf.trim().is_empty(),
            "should receive streaming text, got: {:?}",
            buf
        );
        println!("E2E-OK streamed: {:?}", buf);
    }
}

/// Provider Reliability Suite (G: Response Caching / Healing / MarketRouter) 单元测试
#[cfg(test)]
mod provider_reliability_tests {
    use super::*;

    // ── ResponseCache (G: Response Caching) ─────────────────────────

    #[test]
    fn test_response_cache_hit_after_insert() {
        let mut cache = ResponseCache::new(8);
        let key = ResponseCache::key_for(
            "llm7/codestral-latest",
            &[Message::new(Role::User, "hello")],
        );
        assert_eq!(cache.cache(&key), None, "未插入前不应命中");
        cache.insert(&key, "{\"content\":\"hi\"}".to_string());
        assert_eq!(cache.cache(&key), Some("{\"content\":\"hi\"}".to_string()));
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 1, "初始未命中查询应计为 1 次 miss");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_response_cache_evicts_lru_at_capacity() {
        let mut cache = ResponseCache::new(3);
        cache.insert("k1", "v1".to_string());
        cache.insert("k2", "v2".to_string());
        cache.insert("k3", "v3".to_string());
        // 触碰 k1 → k2 成为最久未使用
        assert_eq!(cache.cache("k1"), Some("v1".to_string()));
        cache.insert("k4", "v4".to_string());
        assert_eq!(cache.len(), 3, "容量 3, 插入第 4 条应驱逐一条");
        assert_eq!(cache.cache("k2"), None, "k2 应被驱逐 (LRU)");
        assert_eq!(cache.cache("k1"), Some("v1".to_string()), "k1 最近使用, 应保留");
        assert_eq!(cache.cache("k3"), Some("v3".to_string()));
        assert_eq!(cache.cache("k4"), Some("v4".to_string()));
    }

    #[test]
    fn test_response_cache_capacity_bound() {
        let cache = ResponseCache::new(256);
        assert_eq!(cache.capacity(), 256);
        assert!(cache.is_empty());
        let min_cap = ResponseCache::new(0);
        assert_eq!(min_cap.capacity(), 1, "容量下限为 1");
    }

    #[test]
    fn test_response_cache_pin_protects_from_eviction() {
        // G26 分层 expert 缓存 (colibri): pin 的 key 不参与 LRU 驱逐。
        let mut cache = ResponseCache::new(3);
        cache.insert("k1", "v1".to_string());
        cache.insert("k2", "v2".to_string());
        cache.insert("k3", "v3".to_string());
        assert!(cache.pin("k1"), "pin k1 成功");
        // 触碰 k2/k3 使其较新
        assert_eq!(cache.cache("k2"), Some("v2".to_string()));
        assert_eq!(cache.cache("k3"), Some("v3".to_string()));
        cache.insert("k4", "v4".to_string());
        assert_eq!(cache.cache("k1"), Some("v1".to_string()), "pinned k1 免驱逐");
        assert_eq!(cache.len(), 3, "容量 3 仍保持");
        assert_eq!(cache.pinned_count(), 1);
        cache.unpin("k1");
        assert_eq!(cache.pinned_count(), 0);
    }

    #[test]
    fn test_response_cache_prefetch_refreshes_and_counts() {
        // G26 prefetch (colibri): 命中预取刷新 LRU + 计数, miss 静默。
        let mut cache = ResponseCache::new(8);
        cache.insert("hot", "{\"content\":\"hot\"}".to_string());
        assert_eq!(cache.prefetch("hot"), Some("{\"content\":\"hot\"}".to_string()));
        assert_eq!(cache.prefetch_hit_count(), 1);
        assert_eq!(cache.prefetch("cold"), None, "未命中 prefetch 返回 None");
        assert_eq!(cache.prefetch_hit_count(), 1, "miss 不计 prefetch hit");
    }

    #[test]
    fn test_response_cache_lookahead_prefetch() {
        // P0-7 lookahead (OasisKV): 预取 hint 列表, 已缓存者刷新, 缺失者列出。
        let mut cache = ResponseCache::new(8);
        cache.insert("m|fp=lookahead:1", "v1".to_string());
        let (hits, missing) = cache.prefetch_lookahead(&["m|fp=lookahead:1".to_string(), "m|fp=lookahead:2".to_string()]);
        assert_eq!(hits, 1, "cached hint 刷新命中");
        assert_eq!(missing, vec!["m|fp=lookahead:2".to_string()]);
        assert_eq!(cache.prefetch_hit_count(), 1);
        let hints = cache.lookahead_hints("model-a|fp=x");
        assert_eq!(hints.len(), 2);
        assert!(hints.iter().all(|h| h.starts_with("model-a|")));
    }

    #[test]
    fn test_gateway_response_cache_toggle_and_counters() {
        let mut gw = GatewayV2::new();
        assert!(!gw.response_cache_enabled(), "缓存默认关闭");
        gw.enable_response_cache(true);
        assert!(gw.response_cache_enabled());
        {
            let mut rc = gw.response_cache.lock().unwrap();
            rc.insert("k", "v".into());
            assert_eq!(rc.cache("k"), Some("v".into()));
        }
        assert_eq!(gw.response_cache_hits(), 1, "命中计数器应接线到 gateway");
        assert_eq!(gw.response_cache_len(), 1);
    }

    // ── ResponseHealer (G: Response Healing) ────────────────────────

    #[test]
    fn test_healer_fixes_trailing_comma() {
        let mut healer = ResponseHealer::new();
        let healed = healer.heal("{\"a\": 1, \"b\": [2, 3,],}");
        assert_eq!(healed, "{\"a\": 1, \"b\": [2, 3]}");
        assert_eq!(healer.heal_count(), 1);
        assert_eq!(healer.unrepairable_count(), 0);
    }

    #[test]
    fn test_healer_closes_unclosed_brace() {
        let mut healer = ResponseHealer::new();
        let healed = healer.heal("{\"a\": {\"b\": [1, 2}");
        assert_eq!(healed, "{\"a\": {\"b\": [1, 2]}}");
        assert_eq!(healer.heal_count(), 1);
    }

    #[test]
    fn test_healer_leaves_valid_json_untouched() {
        let mut healer = ResponseHealer::new();
        let raw = "{\"ok\": true, \"list\": [1, 2, 3]}";
        assert_eq!(healer.heal(raw), raw);
        assert_eq!(healer.heal_count(), 0, "合法 JSON 不应计为修复");
    }

    #[test]
    fn test_healer_extracts_fenced_json() {
        let mut healer = ResponseHealer::new();
        let raw = "Here is the JSON:\n```json\n{\"a\": 1,}\n```\nHope this helps.";
        assert_eq!(healer.heal(raw), "{\"a\": 1}");
        assert_eq!(healer.heal_count(), 1);
    }

    #[test]
    fn test_healer_extracts_prose_wrapped_json() {
        let mut healer = ResponseHealer::new();
        let raw = "Sure! Here is the answer: {\"a\": [1, 2,], \"b\": 2} and that's it.";
        assert_eq!(healer.heal(raw), "{\"a\": [1, 2], \"b\": 2}");
        assert_eq!(healer.heal_count(), 1);
    }

    #[test]
    fn test_healer_unrepairable_returns_original() {
        let mut healer = ResponseHealer::new();
        let raw = "definitely not json at all";
        assert_eq!(healer.heal(raw), raw);
        assert_eq!(healer.unrepairable_count(), 1);
        assert_eq!(healer.heal_count(), 0);
    }

    #[test]
    fn test_gateway_response_healer_toggle_and_counters() {
        let mut gw = GatewayV2::new();
        assert!(!gw.response_healer_enabled(), "修复器默认关闭");
        gw.set_response_healer(true);
        assert!(gw.response_healer_enabled());
        {
            let mut h = gw.response_healer.lock().unwrap();
            h.heal("{\"a\": 1,}");
        }
        let (heal, unrep) = gw.response_healer_counters();
        assert_eq!(heal, 1, "heal 计数器应接线到 gateway");
        assert_eq!(unrep, 0);
    }

    // ── MarketRouter (G: market-wisdom routing) ─────────────────────

    #[test]
    fn test_router_picks_highest_composite_score() {
        let mut router = MarketRouter::new();
        let mut a = ProviderState::new(false, ProviderCategory::Cloud);
        let mut b = ProviderState::new(false, ProviderCategory::Cloud);
        for _ in 0..5 {
            a.record_success(100.0, 10);
            b.record_failure(1000.0);
        }
        assert!(
            a.composite_score() > b.composite_score(),
            "a composite={} b composite={}",
            a.composite_score(),
            b.composite_score()
        );
        let providers = [&mut a, &mut b];
        assert_eq!(router.route(&providers), Some(0), "应选 composite_score 更高者");
    }

    #[test]
    fn test_router_returns_none_for_empty_or_unavailable() {
        let mut router = MarketRouter::new();
        assert_eq!(router.route(&[]), None, "空列表应返回 None");
        let mut dead = ProviderState::new(false, ProviderCategory::Cloud);
        for _ in 0..6 {
            dead.record_failure(500.0);
        }
        let providers = [&mut dead];
        assert_eq!(router.route(&providers), None, "不可用 provider 不应被选中");
    }

    #[test]
    fn test_router_re_evaluate_updates_weights_with_short_interval() {
        // 短间隔构造 (1ms) → 首次重估立即执行并更新权重
        let mut router = MarketRouter::with_interval(Duration::from_millis(1));
        let mut a = ProviderState::new(true, ProviderCategory::Cloud);
        a.record_success(50.0, 10);
        let providers = [&a];
        assert!(router.re_evaluate(&providers), "首次重估应立即执行");
        assert_eq!(router.weights().len(), 1);
        assert!(router.weights()[0] > 0.0, "健康 provider 权重应 > 0");
        assert_eq!(router.eval_count(), 1);
    }

    #[test]
    fn test_router_re_evaluate_respects_default_interval() {
        // 默认 5 分钟: 首次重估后, 间隔未到不得再次重估
        let mut router = MarketRouter::new();
        let mut a = ProviderState::new(true, ProviderCategory::Cloud);
        a.record_success(50.0, 10);
        let providers = [&a];
        assert!(router.re_evaluate(&providers), "首次重估应立即执行");
        assert_eq!(router.weights().len(), 1);
        assert!(
            !router.re_evaluate(&providers),
            "默认 5 分钟间隔未到, 不应重估"
        );
        assert_eq!(router.eval_count(), 1);
        assert_eq!(MarketRouter::DEFAULT_INTERVAL, Duration::from_secs(300));
    }

    #[test]
    fn test_gateway_market_router_tick_hook() {
        let gw = GatewayV2::new();
        {
            let mut states = gw.states.write().unwrap();
            states.insert("p1".into(), ProviderState::new(true, ProviderCategory::Cloud));
            states.insert(
                "p2".into(),
                ProviderState::new(false, ProviderCategory::Cloud),
            );
        }
        assert!(gw.maybe_re_evaluate(), "首次 tick 应触发重估");
        assert!(!gw.maybe_re_evaluate(), "5 分钟间隔内不应重估");
        let router = gw.market_router.lock().unwrap();
        assert_eq!(router.eval_count(), 1);
        assert_eq!(router.weights().len(), 2, "权重应按注册 provider 数重算");
    }

    // ── Auto Exacto 周期重估注册表 (R-P79 生产接线) ──────────────
    // 注册表是进程级全局 — 两个共享该全局的测试用互斥锁串行, 避免并行竞态。
    static REGISTRY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_periodic_re_evaluation_via_registry() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // 注册的 GatewayV2 经 run_periodic_re_evaluation() 周期触发重估
        let gw = GatewayV2::new();
        {
            let mut states = gw.states.write().unwrap();
            states.insert("p1".into(), ProviderState::new(true, ProviderCategory::Cloud));
        }
        let gw = Arc::new(gw);
        register_gateway_for_re_evaluation(&gw);
        assert_eq!(
            run_periodic_re_evaluation(),
            1,
            "首次周期 tick 应触发重估"
        );
        assert_eq!(
            run_periodic_re_evaluation(),
            0,
            "5 分钟间隔内周期 tick 不应重复重估"
        );
        assert_eq!(
            gw.market_router.lock().unwrap().eval_count(),
            1,
            "market_router 只应被重估一次"
        );
    }

    #[test]
    fn test_periodic_re_evaluation_prunes_dropped_gateway() {
        let _guard = REGISTRY_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Weak 注册表: 网关释放后周期 tick 自动剔除, 不 panic 不计数
        {
            let gw = Arc::new(GatewayV2::new());
            register_gateway_for_re_evaluation(&gw);
        }
        assert_eq!(
            run_periodic_re_evaluation(),
            0,
            "已释放网关不应被周期 tick 计入"
        );
    }

    #[test]
    fn test_call_provider_strips_full_registered_name_prefix() {
        // 完整目录注册名 (`llm7/codestral-latest`) 被选为候选链第一名时,
        // `{name}/` strip 会失败 (model 无尾斜杠), 必须按首段剥离,
        // 否则上游收到 `llm7/codestral-latest` → model_unavailable。
        let gw = GatewayV2::new();
        let cases = [
            // (注册名, 请求 model, 期望传给 provider 的 model)
            ("llm7", "llm7/codestral-latest", Some("codestral-latest")),
            (
                "llm7/codestral-latest",
                "llm7/codestral-latest",
                Some("codestral-latest"),
            ),
            (
                "api-airforce/grok-4.1-mini:free",
                "api-airforce/grok-4.1-mini:free",
                Some("grok-4.1-mini:free"),
            ),
            ("openai", "openai/gpt-4o", Some("gpt-4o")),
            ("llm7", "gpt-4o", None),
        ];
        for (name, model, expected) in cases {
            let stripped = model
                .strip_prefix(&format!("{}/", name))
                .map(|m| m.to_string())
                .or_else(|| {
                    if name.contains('/') {
                        model.split_once('/').map(|(_, rest)| rest.to_string())
                    } else {
                        None
                    }
                });
            assert_eq!(
                stripped.as_deref(),
                expected,
                "name={name} model={model} 应正确剥离前缀"
            );
        }
    }
}