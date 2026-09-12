use std::time::{Duration, Instant};

use crate::core::l0_substrate::nt_core_error::recovery::{ErrorContext, ErrorType, RecoveryAction};
use crate::core::nt_core_cache::text_to_embedding;
use crate::core::nt_core_span::{SpanKind, Tracer};

use crate::l1_action::nt_io::nt_io_provider::pool::account_pool::AccountPoolError;
use crate::l1_action::nt_io::nt_io_provider::gateway::routing::agent_routing::ModelTier;
use crate::l1_action::nt_io::nt_io_provider::health::circuit_breaker::BreakerState;
use crate::l1_action::nt_io::nt_io_provider::health::context_budget::estimate_tokens;
use crate::l1_action::nt_io::nt_io_provider::pool::free_pool::global_free_pool;
use crate::l1_action::nt_io::nt_io_provider::health::rate_limiter::BrainTier;
use crate::l1_action::nt_io::nt_io_provider::common::privacy_guard::{egress_privacy_guard, trust_from_name};
use crate::l1_action::nt_io::nt_io_provider::gateway::types::registry_core::AttemptPhase;
use super::*;

/// 检测 provider 返回的维护窗提示 (如 empero "switching to new models" / "retrying in"),
/// 用于触发长冷却熔断, 避免在其维护期间反复重试浪费输入 token。
fn is_maintenance_window(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("switching to new models")
        || m.contains("retrying in")
        || (m.contains("maintenance") && m.contains("window"))
        || m.contains("temporarily unavailable for maintenance")
}

/// 指数退避重试延迟计算 (带 full jitter)
/// 参考: AWS SDK 标准重试模式
fn exponential_backoff(attempt: u32, base_ms: u64, cap_ms: u64) -> u64 {
    let exp = 2u64.saturating_pow(attempt);
    let delay = base_ms.saturating_mul(exp).min(cap_ms);
    // Full jitter: random(0, delay)
    let jitter = (rand::random::<f64>() * delay as f64) as u64;
    delay.saturating_add(jitter)
}

/// 检测"模型不可用"类错误 (404 / model not found / does not exist / not supported 等),
/// 对应 L3 模型级锁: 该模型下线/改名时只锁定模型, 不熔断整个 provider,
/// 使 provider 对其它模型仍可用 → 池子不因单模型故障而缩水 (对齐 OmniRoute model lockout)。
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
    pub(super) async fn call_provider(
        &self,
        name: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        let provider = {
            let guard = self
                .providers
                .read()
                .unwrap_or_else(|e| e.into_inner());
            guard
                .get(name)
                .cloned()
                .or_else(|| {
                    // "provider/model" 格式: 用 provider 前缀查找注册实例
                    name.split('/')
                        .next()
                        .and_then(|p| guard.get(p).cloned())
                })
        }
        .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;
        // 剥离 `{provider}/` 前缀 (同 call_provider_stream)。
        // 兼容两种注册名: 裸 provider 名 (`llm7`) 与完整目录名 (`llm7/codestral-latest`)。
        // 完整目录名场景下请求 model 恰等于注册名, `{name}/` strip 会失败,
        // 需按 provider 前缀剥离, 否则上游收到 `llm7/codestral-latest` → model_unavailable。
        // 2026-08-22: 请求 model 恰等于裸 keyless 注册名 (RouterConfig preferred_model
        // = "llm7") 时同样视为"未指定模型" → 回退 catalog default_model (codestral-latest),
        // 防止字面 "llm7" 作为模型名发给上游 → model_unavailable。
        // 修复 (2026-08-28): 仅当 request.model 以 `name/` 开头时才剥离 provider 前缀;
        // 纯 model 名含内部 '/' (如 `meta/llama-3.2-11b-vision-instruct`) 原样保留,
        // 防误把模型内的 '/' 当 provider 前缀截断 → 404 model not found。
        let stripped = request
            .model
            .strip_prefix(&format!("{}/", name))
            .map(|m| m.to_string())
            .or_else(|| {
                // 仅当 request.model 以 `name/` 开头时才剥离 provider 前缀;
                // 否则 (纯 model 名含内部 '/', 如 `meta/llama-3.2-11b-vision-instruct`)
                // 不剥离, 保留完整 model 名
                if name.contains('/') && request.model.starts_with(&format!("{}/", name)) {
                    request
                        .model
                        .strip_prefix(&format!("{}/", name))
                        .map(|m| m.to_string())
                } else {
                    None
                }
            });
        let mut req = request.clone();
        // 裸注册名等值 (model == name) 视作未指定
        if req.model == name {
            req.model.clear();
        }
        let final_model = if let Some(m) = stripped {
            if m.is_empty() {
                // 模型被剥离为空 (如请求 model="llm7/codestral-latest" 且 provider="llm7/codestral-latest")
                // 回退到该 provider 在 catalog 中的 default_model
                super::super::provider_catalog::lookup_provider(name.split('/').next().unwrap_or(name))
                    .map(|info| info.default_model.to_string())
                    .unwrap_or_else(|| req.model.clone())
            } else {
                m
            }
        } else if req.model.is_empty() {
            // 请求未指定模型: "provider/model" 格式优先从 name 提取模型部分,
            // 否则回退 provider 的 default_model
            if name.contains('/') {
                name.split('/').skip(1).collect::<Vec<_>>().join("/")
            } else {
                super::super::provider_catalog::lookup_provider(name)
                    .map(|info| info.default_model.to_string())
                    .unwrap_or_else(|| "auto".to_string())
            }
        } else {
            req.model.clone()
        };
        req.model = final_model;
        // Cumora 吸收: 双脑并发门 — 按模型 tier 选 big/triage 槽位, 防雪崩。
        let tier = if ModelTier::parse(&req.model) >= ModelTier::Capable {
            BrainTier::Big
        } else {
            BrainTier::Triage
        };
        let gate_wait = {
            // 自旋等待槽位: guard 在 .await 前即释放, 避免 std MutexGuard 跨 await (future 非 Send)。
            loop {
                let acquired = self
                    .tiered_semaphore
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .try_acquire(tier);
                if acquired {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            self.adaptive_pacer
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .gate()
        };
        // 自适应间隔: 等待 pacer 允许的下一次调用 (429 后全局放慢)。
        if gate_wait > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(gate_wait)).await;
        }
        // 出站隐私守卫: 剥离密钥 + 阻断/脱敏 NeoTrix 内部源码与对话指纹,
        // 防止免费/代理 provider 记录并用于训练 (fail-closed for untrusted)。
        if let Err(reason) = egress_privacy_guard(&mut req, trust_from_name(name), name) {
            return Err(LlmError::InvalidRequest(reason));
        }
        let result = provider.complete(&req).await;
        // Per-provider 用量账本接线 (R-P79): 仅在真实请求完成后记录 provider
        // 自报 usage。活动记录，非权威账单数据 (诚实标注见 ProviderUsageLedger)。
        if let Ok(response) = result.as_ref() {
            crate::core::nt_core_telemetry::global_provider_usage_ledger()
                .record_provider_usage(name, &response.usage);
        }
        {
            self.tiered_semaphore
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .release(tier);
            let mut pacer = self.adaptive_pacer.lock().unwrap_or_else(|e| e.into_inner());
            match &result {
                Ok(_) => pacer.on_ok(),
                Err(e) => {
                    let msg = e.to_string();
                    if is_quota_exhaustion(&msg) || msg.contains("rate limit") || msg.contains("429") {
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

    /// P7 账户池选择层: 经 AccountPool 按健康状态 round-robin 选择账户并获取并发租约,
    /// 再走既有 `call_provider` 调用路径。RateLimit 错误 → 检疫该账户 (冷却后自动恢复)。
    ///
    /// R-P42: 强化 GatewayV2 既有选择路径, 非平行 provider 系统 — 池只决定"用哪个账户"。
    ///
    /// `provider` 用于限定账户范围 (同 provider 多账户); 传空字符串则从请求 model 前缀推断。
    pub async fn complete_with_account_pool(
        &self,
        provider: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        // 推断 provider 范围: 显式参数优先, 否则取 model 前缀 (如 `openai/gpt-4o` → openai)
        let scope = if provider.is_empty() {
            request
                .model
                .split('/')
                .next()
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            provider.to_string()
        };
        if scope.is_empty() {
            return Err(LlmError::Unknown(
                "account_pool: cannot infer provider scope from empty model".to_string(),
            ));
        }
        let lease = {
            let pool = self.account_pool.lock().unwrap_or_else(|e| {
                log::warn!("[gateway] account_pool Mutex poisoned: {}", e);
                e.into_inner()
            });
            match pool.select(&scope) {
                Ok(lease) => lease,
                Err(AccountPoolError::NoAccounts(_)) => {
                    return Err(LlmError::Unknown(format!(
                        "account_pool: no accounts registered for provider '{scope}'"
                    )));
                }
                Err(AccountPoolError::NoHealthyAccount(_)) => {
                    return Err(LlmError::RateLimit(format!(
                        "account_pool: all accounts for '{scope}' are quarantined/unavailable"
                    )));
                }
                Err(AccountPoolError::Saturated(_)) => {
                    return Err(LlmError::RateLimit(format!(
                        "account_pool: account concurrency cap reached for '{scope}'"
                    )));
                }
            }
        };
        let account_name = lease.account_name().to_string();
        let result = self.call_provider(&account_name, request).await;

        let pool = self.account_pool.lock().unwrap_or_else(|e| {
            log::warn!("[gateway] account_pool Mutex poisoned: {}", e);
            e.into_inner()
        });
        match &result {
            Ok(_) => {
                pool.record_success(&account_name);
            }
            Err(LlmError::RateLimit(_)) => {
                log::warn!(
                    "[gateway] account '{account_name}' rate-limited → quarantine ({}s cooldown)",
                    pool.config().quarantine_cooldown.as_secs()
                );
                pool.quarantine(&account_name);
            }
            Err(_) => {
                pool.record_failure(&account_name);
            }
        }
        let _ = lease; // 释放并发租约 (AccountLease::drop)
        result
    }

    pub async fn complete_with_selection(
        &self,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        // 需求驱动自愈 (T3): 每次请求入口检查自有 LLM 池是否仍有充足免费模型,
        // 偏薄时按需从 FreeModelCatalog 补充 (reconcile 内置 cooldown 节流)。
        // 对齐 OmniRoute Radar 刷新 + NeoTrix NT-REPAIR 自愈节点 — 池子始终"活着"。
        self.ensure_pool_sufficient(3, 60).await;

        // Build hardened prompt key for cache lookup (含请求指纹, 见 prompt_cache_key)
        let prompt_key: String = self.prompt_cache_key(request);

        // Check semantic cache (Layer 1: exact match)
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_exact(&request.model, &prompt_key) {
                    if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                        return Ok(response);
                    }
            }
        }

        // Layer 1.25: semantic fallback — exact miss 后按 embedding 余弦相似兜底
        // (语义阈值保守 0.98, 避免不同语义误命中; 命中即复用同模型的历史响应)。
        {
            let embedding = text_to_embedding(&self.prompt_text(request));
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(cached) = cache.get_semantic(&embedding) {
                    if let Ok(response) = serde_json::from_str::<LlmResponse>(cached) {
                        return Ok(response);
                    }
                }
            }
        }

        // Layer 1.5: LRU response cache (G: Response Caching) — 命中直接返回
        if self.response_cache_enabled {
            let rc_key = ResponseCache::key_for_request(&request.model, &self.prompt_cache_key(request));
            if let Ok(mut rc) = self.response_cache.lock() {
                if let Some(cached) = rc.cache(&rc_key) {
                    // G26 分层 expert 缓存 (colibri): 高频命中 key 自动 pin 热集,
                    // 免于 LRU 驱逐 (限 MAX_PINNED 防挤占)。
                    rc.pin(&rc_key);
                if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                        return Ok(response);
                    }
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
                let est_tokens = estimate_tokens(&self.prompt_text(request)) as f64;
                // 动态查询 provider 成本: 从 model 名提取 provider 前缀
                let provider_name = request.model.split('/').next().unwrap_or("");
                let cost_per_1k = super::super::catalog::provider_catalog::lookup_provider_cost(provider_name)
                    .unwrap_or(0.002);
                let estimated_cost = (est_tokens / 1000.0) * cost_per_1k;
                if estimated_cost > self.cost_budget_per_query {
                    log::warn!(
                        "[gateway] Per-query cost budget exceeded: ${:.4} > ${:.4}",
                        estimated_cost,
                        self.cost_budget_per_query
                    );
                    return Err(LlmError::Unknown(format!(
                        "Cost budget exceeded: ${:.4} > ${:.4}",
                        estimated_cost, self.cost_budget_per_query
                    )));
                }
            }
        }
        let mut used_names: Vec<String> = Vec::new();

        // 候选链: 从池子实际注册名动态构建 (前缀优先 + free/available/score 排序)
        // P2-A4: 8→3 — 失败放大收敛 (每次重试同量输入 token 全付, 3 次封顶)。
        // quota-aware 回退 (吸收 OmniRoute): 排除已耗尽 (used_names) provider, 自动跳到次优可用。
        let chain = self.quota_aware_fallback_chain(&request.model, &used_names, 3);
        // P2-A4: 4xx (认证/非法请求) 非瞬时错误 — 同凭据换 provider 亦失败, 直接熔断整链。
        let mut fatal_error: Option<LlmError> = None;

        for name in chain {
            if used_names.contains(&name) {
                continue;
            }
            used_names.push(name.clone());

            let start = Instant::now();

            // Check rate limit
            {
                let mut states = self.states.write().unwrap_or_else(|e| {
                    log::warn!("[gateway] states RwLock poisoned: {}", e);
                    e.into_inner()
                });
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
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_success(elapsed, token_count);
                            state.rate_limiter.record_usage(token_count as f64);
                        }
                    }
                    self.fire_event(
                        &name,
                        true,
                        elapsed,
                        token_count,
                        &request.model,
                        AttemptPhase::Normal,
                    );
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
                        let states = self.states.read().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get(&name) {
                            if state.is_free {
                                global_free_pool().record_usage(&name, token_count as u64);
                            }
                        }
                    }
                    if let Ok(mut ct) = self.cost_tracker.write() {
                        if let Some(ref mut tracker) = *ct {
                            tracker.record(
                                &request.model,
                                response.usage.prompt_tokens.into(),
                                response.usage.completion_tokens.into(),
                            );
                        }
                    }
                    // G: Response Healing + Caching — 修复畸形 JSON, 回写 LRU 缓存
                    let response = self.heal_and_cache_response(request, response);
                    // F6: Generation Classification — 打标本次生成供 analytics
                    self.tag_generation(request, &response, &name, elapsed, token_count, true);
                    // Store in semantic cache
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            cache.set_with_embedding(
                                &request.model,
                                &prompt_key,
                                serialized,
                                text_to_embedding(&self.prompt_text(request)),
                            );
                        }
                    }
                    return Ok(response);
                }
                Err(err) => {
                    let error_msg = err.to_string();
                    let is_quota_exhausted = is_quota_exhaustion(&error_msg);
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(&name) {
                            if is_quota_exhausted {
                                // 配额耗尽 (freellmapi/aimux): 重试无益, 直接熔断剔除该 provider,
                                // 避免反复打同一个耗尽账户。冷却期后配额恢复探测自动放行。
                                state.mark_quota_exhausted();
                                log::warn!(
                                    "[gateway] provider '{}' quota exhausted → 熔断剔除: {}",
                                    name,
                                    error_msg
                                );
                            } else if is_maintenance_window(&error_msg) {
                                // 维护窗 (如 empero "switching to new models"): 非瞬时错误,
                                // 强制熔断并设置 30min 冷却, selector 在冷却期内跳过该 provider,
                                // 透明 failover 到备用源 — 无需手动等其恢复 (解决 29m 等待)。
                                state.circuit_breaker.force_open_secs(1800);
                                log::warn!(
                                    "[gateway] provider '{}' maintenance window → circuit opened (30min), failover: {}",
                                    name,
                                    error_msg
                                );
                            } else if is_model_unavailable(&error_msg) {
                                // L3 模型级锁 (对齐 OmniRoute): 单模型 404/model_unavailable
                                // 只锁定该模型, 不熔断整个 provider → 池子不因单模型故障缩水。
                                state.lock_model(&request.model, 1800);
                                state.record_failure(elapsed);
                                log::warn!(
                                    "[gateway] provider '{}' model '{}' unavailable → model-locked (30min), routing next: {}",
                                    name,
                                    request.model,
                                    error_msg
                                );
                            } else {
                                state.record_failure(elapsed);
                            }
                        }
                    }
                    self.fire_event(
                        &name,
                        false,
                        elapsed,
                        0,
                        &request.model,
                        AttemptPhase::Normal,
                    );

                    // P2-A4: 4xx 非瞬时错误 (认证失败/非法请求) — 同凭据换 provider
                    // 亦失败, 跳过 recovery 重试与 Phase-2 aggressive retry, 熔断整链。
                    if matches!(err, LlmError::Authentication(_) | LlmError::InvalidRequest(_)) {
                        fatal_error = Some(err);
                        break;
                    }

                    // Consult recovery orchestrator
                    let error_type = if is_quota_exhausted {
                        ErrorType::Unknown(
                            "quota exhausted — provider tripped, skip retry".to_string(),
                        )
                    } else if error_msg.contains("rate limit") || error_msg.contains("429") {
                        ErrorType::RateLimit { retry_after: None }
                    } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                        ErrorType::Timeout {
                            elapsed_ms: elapsed as u64,
                        }
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
                        available_models: self
                            .providers
                            .read()
                            .unwrap_or_else(|e| e.into_inner())
                            .keys()
                            .cloned()
                            .collect(),
                        prompt: request
                            .messages
                            .iter()
                            .map(|m| m.content.clone())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        prompt_variants: Vec::new(),
                        state_snapshot: None,
                        token_budget_remaining: 0,
                        elapsed_ms: elapsed as u64,
                        metadata: HashMap::new(),
                    };
                    let action = self
                        .recovery
                        .write()
                        .unwrap_or_else(|e| {
                            log::warn!("[gateway] recovery RwLock poisoned: {}", e);
                            e.into_inner()
                        })
                        .handle(&ctx);
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
        // P2-A4: 若链上出现 4xx 致命错误 (认证/非法请求), 跳过 aggressive retry —
        // 该重试只会再次全付输入 token。
        let result = match fatal_error {
            Some(f) => Err(f),
            None => self.attempt_aggressive_retry(request).await,
        };
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
                    tracker.record(
                        &request.model,
                        response.usage.prompt_tokens.into(),
                        response.usage.completion_tokens.into(),
                    );
                }
            }
        }
        result
    }

    /// Aggressive fallback: when all providers have failed, temporarily
    /// override circuit breaker states (Open → HalfOpen, reduced cooldown)
    /// and retry every registered provider once more.
    async fn attempt_aggressive_retry(
        &self,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        let provider_names: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            states.keys().cloned().collect()
        };

        if provider_names.is_empty() {
            return Err(LlmError::Unknown(
                "No providers available for aggressive retry".to_string(),
            ));
        }

        // Save original breaker states before override
        let set_aggressive: Vec<(String, u64)> = {
            let states = self.states.read().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            provider_names
                .iter()
                .filter_map(|name| {
                    states.get(name).and_then(|s| {
                        if matches!(s.circuit_breaker.state(), BreakerState::Open { .. }) {
                            let saved = s.circuit_breaker.half_open_max_probes();
                            Some((name.clone(), saved))
                        } else {
                            None
                        }
                    })
                })
                .collect()
        };

        // Apply aggressive overrides
        {
            let mut states = self.states.write().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
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
                let mut states = self.states.write().unwrap_or_else(|e| {
                    log::warn!("[gateway] states RwLock poisoned: {}", e);
                    e.into_inner()
                });
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
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(name) {
                            state.record_success(elapsed, token_count);
                            // Restore the provider's configured probe threshold
                            // instead of leaving the aggressive override in place.
                            let saved = set_aggressive
                                .iter()
                                .find(|(n, _)| n == name)
                                .map(|(_, s)| *s);
                            state
                                .circuit_breaker
                                .set_half_open_max_probes(saved.unwrap_or(3));
                        }
                    }
                    self.fire_event(
                        name,
                        true,
                        elapsed,
                        token_count,
                        &request.model,
                        AttemptPhase::AggressiveRetry,
                    );
                    // Track usage in global free pool
                    {
                        let states = self.states.read().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get(name) {
                            if state.is_free {
                                global_free_pool().record_usage(name, token_count as u64);
                            }
                        }
                    }
                    // G: Response Healing + Caching — 修复畸形 JSON, 回写 LRU 缓存
                    let response = self.heal_and_cache_response(request, response);
                    // F6: Generation Classification — 打标本次生成供 analytics
                    self.tag_generation(request, &response, name, elapsed, token_count, true);
                    // Store in semantic cache
                    if let Ok(mut cache) = self.cache.lock() {
                        if let Ok(serialized) = serde_json::to_string(&response) {
                            cache.set_with_embedding(
                                &request.model,
                                &self.prompt_cache_key(request),
                                serialized,
                                text_to_embedding(&self.prompt_text(request)),
                            );
                        }
                    }
                    return Ok(response);
                }
                Err(err) => {
                    log::warn!("[gateway] Aggressive retry failed for '{}': {}", name, err);
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(name) {
                            state.record_failure(elapsed);
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(
                        name,
                        false,
                        elapsed,
                        0,
                        &request.model,
                        AttemptPhase::AggressiveRetry,
                    );
                }
            }
        }

        // All aggressive retries failed — restore saved probes
        {
            let mut states = self.states.write().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            for (name, saved) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(*saved);
                }
            }
        }

        Err(LlmError::Unknown(
            "Aggressive retry exhausted — all providers failed".to_string(),
        ))
    }

    /// Stream completion with 2-phase retry (normal → aggressive), matching
    /// `complete_with_selection`'s fallback strategy but for streaming.
    pub async fn stream_complete_with_selection(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        // 需求驱动自愈 (T3): 同 complete_with_selection, 偏薄时按需补充自有 LLM 池。
        self.ensure_pool_sufficient(3, 60).await;

        // Cost budget check (与 complete_with_selection 对齐)
        if self.cost_budget_per_query > 0.0 {
            let prompt_tokens = estimate_tokens(&self.prompt_text(request));
            let cost_estimate = (prompt_tokens as f64 / 1000.0) * 0.002;
            if cost_estimate > self.cost_budget_per_query {
                log::warn!(
                    "[gateway] stream cost estimate ${:.4} exceeds budget ${:.4}",
                    cost_estimate,
                    self.cost_budget_per_query
                );
            }
        }

        // Phase 1: Normal retry loop (up to 3 providers, best-first)
        let mut used_names: Vec<String> = Vec::new();

        // 候选链: 从池子实际注册名动态构建 (前缀优先 + free/available/score 排序)
        // P2-A4: 8→3 — 失败放大收敛。
        // quota-aware 回退 (吸收 OmniRoute): 排除已耗尽 (used_names) provider, 自动跳到次优可用。
        let chain = self.quota_aware_fallback_chain(&request.model, &used_names, 3);

        for name in chain {
            if used_names.contains(&name) {
                continue;
            }
            used_names.push(name.clone());

            // Check rate limit
            {
                let mut states = self.states.write().unwrap_or_else(|e| {
                    log::warn!("[gateway] states RwLock poisoned: {}", e);
                    e.into_inner()
                });
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
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_failure(0.0);
                        }
                    }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    continue;
                }
                // P2-A4: 4xx 非瞬时错误 — 同凭据换 provider 亦失败, 立即终止整链。
                Err(err @ (LlmError::Authentication(_) | LlmError::InvalidRequest(_))) => {
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(&name) {
                            state.record_failure(0.0);
                        }
                    }
                    self.fire_event(&name, false, 0.0, 0, &request.model, AttemptPhase::Normal);
                    return Err(err);
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    // Quota exhaustion detection
                    if err.is_quota_exhaustion() {
                        log::warn!(
                            "[gateway] stream provider {} quota exhausted, marking",
                            name
                        );
                        self.states_write(|states| {
                            if let Some(state) = states.get_mut(&name) {
                                state.circuit_breaker.force_open();
                            }
                        });
                    }
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(&name) {
                            if is_model_unavailable(&err_msg) {
                                // L3 模型级锁: 流式单模型不可用只锁模型, 不拖垮 provider。
                                state.lock_model(&request.model, 1800);
                            }
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
        let provider = self
            .providers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(name)
            .cloned()
            .ok_or_else(|| LlmError::Unknown(format!("Provider '{}' not found", name)))?;
        // 剥离 `{provider}/` 前缀: 请求模型 `llm7/codestral-latest` 传给 provider 时
        // 只传 `codestral-latest` (上游不认识 `llm7/` 前缀, 返回 model_unavailable)。
        // 兼容裸 provider 名与完整目录注册名 (见 call_provider 注释)。
        let stripped = request
            .model
            .strip_prefix(&format!("{}/", name))
            .map(|m| m.to_string())
            .or_else(|| {
                if name.contains('/') {
                    request
                        .model
                        .split_once('/')
                        .map(|(_, rest)| rest.to_string())
                } else {
                    None
                }
            });
        let mut req = request.clone();
        // 裸注册名等值 (model == name) 视作未指定 → 回退 catalog default_model (同 call_provider)
        if req.model == name {
            req.model = super::super::provider_catalog::lookup_provider(name.split('/').next().unwrap_or(name))
                .map(|info| info.default_model.to_string())
                .unwrap_or_default();
        } else if let Some(m) = stripped {
            req.model = m;
        }
        // 出站隐私守卫: 剥离密钥 + 阻断/脱敏 NeoTrix 内部源码与对话指纹。
        if let Err(reason) = egress_privacy_guard(&mut req, trust_from_name(name), name) {
            return Err(LlmError::InvalidRequest(reason));
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
    pub async fn describe_image(
        &self,
        image_b64: &str,
        question: &str,
    ) -> Result<String, LlmError> {
        // Prefer a provider whose registered name advertises vision.
        let mut used: Vec<String> = Vec::new();
        let vision_candidates: Vec<String> = {
            let states = self.states.read().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            let mut names: Vec<String> = states.keys().cloned().collect();
            names.sort_by_key(|n| {
                (
                    !crate::core::nt_core_e8::nt_multimodal::model_supports_vision(n),
                    n.clone(),
                )
            });
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
                None => {
                    return Err(LlmError::Unknown(
                        "no provider available for image description".into(),
                    ))
                }
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
            let states = self.states.read().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            states.keys().cloned().collect()
        };

        if provider_names.is_empty() {
            return Err(LlmError::Unknown(
                "No providers available for aggressive retry".to_string(),
            ));
        }

        // Save original breaker states before override
        let set_aggressive: Vec<(String, u64)> = {
            let states = self.states.read().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            provider_names
                .iter()
                .filter_map(|name| {
                    states.get(name).and_then(|s| {
                        if matches!(s.circuit_breaker.state(), BreakerState::Open { .. }) {
                            let saved = s.circuit_breaker.half_open_max_probes();
                            Some((name.clone(), saved))
                        } else {
                            None
                        }
                    })
                })
                .collect()
        };

        // Apply aggressive overrides
        {
            let mut states = self.states.write().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
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
                let mut states = self.states.write().unwrap_or_else(|e| {
                    log::warn!("[gateway] states RwLock poisoned: {}", e);
                    e.into_inner()
                });
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
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(name) {
                            let saved = set_aggressive
                                .iter()
                                .find(|(n, _)| n == name)
                                .map(|(_, s)| *s);
                            state
                                .circuit_breaker
                                .set_half_open_max_probes(saved.unwrap_or(3));
                        }
                    }
                    self.fire_event(
                        name,
                        true,
                        0.0,
                        0,
                        &request.model,
                        AttemptPhase::AggressiveRetry,
                    );
                    return Ok(result);
                }
                Err(_) => {
                    {
                        let mut states = self.states.write().unwrap_or_else(|e| {
                            log::warn!("[gateway] states RwLock poisoned: {}", e);
                            e.into_inner()
                        });
                        if let Some(state) = states.get_mut(name) {
                            state.record_failure(0.0);
                            state.circuit_breaker.set_half_open_max_probes(3);
                        }
                    }
                    self.fire_event(
                        name,
                        false,
                        0.0,
                        0,
                        &request.model,
                        AttemptPhase::AggressiveRetry,
                    );
                }
            }
        }

        // All aggressive retries failed — restore saved probes
        {
            let mut states = self.states.write().unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            });
            for (name, saved) in &set_aggressive {
                if let Some(state) = states.get_mut(name) {
                    state.circuit_breaker.set_half_open_max_probes(*saved);
                }
            }
        }

        Err(LlmError::Unknown(
            "Aggressive streaming retry exhausted — all providers failed".to_string(),
        ))
    }

    fn fire_event(
        &self,
        provider_name: &str,
        success: bool,
        latency_ms: f64,
        tokens: u32,
        model: &str,
        phase: AttemptPhase,
    ) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_maintenance_window_detects_empero() {
        assert!(is_maintenance_window(
            "We are switching the free endpoint to new models. retrying in 29m 57s"
        ));
        assert!(!is_maintenance_window("503 Service Unavailable"));
        assert!(!is_maintenance_window("rate limit exceeded"));
    }
}