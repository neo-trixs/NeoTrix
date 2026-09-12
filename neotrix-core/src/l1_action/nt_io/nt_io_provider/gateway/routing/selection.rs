use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;
use std::sync::Arc;
use std::time::Instant;

use crate::l1_action::nt_io::nt_io_provider/common::factory::{create_provider, ProviderConfig};
use crate::l1_action::nt_io::nt_io_provider/catalog::free_catalog::{FreeModelCatalog, FreeModelEntry};
use crate::l1_action::nt_io::nt_io_provider/health::rate_limiter::RateLimiter;
use crate::l1_action::nt_io::nt_io_provider/health::rate_profiles::get_rate_profile;
use super::*;

impl GatewayV2 {
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

    pub fn register_provider(&self, name: &str, provider: Arc<dyn LlmProvider>, is_free: bool) {
        self.register_provider_with_category(name, provider, is_free, ProviderCategory::Cloud)
    }

    pub fn register_provider_with_category(
        &self,
        name: &str,
        provider: Arc<dyn LlmProvider>,
        is_free: bool,
        category: ProviderCategory,
    ) {
        {
            let mut providers = self.providers.write().unwrap_or_else(|e| {
                log::warn!("[gateway] providers RwLock poisoned: {}", e);
                e.into_inner()
            });
            providers.insert(name.to_string(), provider);
        }
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
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });

        // Tier 1: Available free providers (preferred)
        let free_best = states
            .iter()
            .filter(|(_, s)| s.is_available() && s.is_free)
            .max_by(|(na, a), (nb, b)| {
                // 同分按 total_calls 升序轮询 (CONTEXT.md: 低 total_calls 优先均衡),
                // 名字兜底消除 HashMap 遍历序偶发 (D13 确定性)。
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.total_calls.cmp(&b.total_calls).reverse())
                    .then(na.cmp(nb))
            })
            .map(|(name, _)| name.clone());

        if free_best.is_some() {
            return free_best;
        }

        // Tier 2: Available paid providers if free-first is off OR all free exhausted
        if !self.prefer_free {
            return states
                .iter()
                .filter(|(_, s)| s.is_available())
                .max_by(|(na, a), (nb, b)| {
                    a.composite_score()
                        .partial_cmp(&b.composite_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.total_calls.cmp(&b.total_calls).reverse())
                        .then(na.cmp(nb))
                })
                .map(|(name, _)| name.clone());
        }

        // Tier 3: Any available provider (free-first exhausted all free, allow paid)
        states
            .iter()
            .filter(|(_, s)| s.is_available())
            .max_by(|(na, a), (nb, b)| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.total_calls.cmp(&b.total_calls).reverse())
                    .then(na.cmp(nb))
            })
            .map(|(name, _)| name.clone())
    }

    /// 构建候选链 — 从池子**实际注册名**动态排序, 而非硬编码。
    ///
    /// 规则 (按优先级):
    /// 1. 请求 model 含 `{provider}/` 前缀且该 provider 已注册 → 前缀 provider 第一候选
    ///    (如 `llm7/codestral-latest` → `llm7`; `api-airforce/grok-4.1-mini:free` → 该完整注册名)
    /// 2. 其余按 free 优先 + is_available 优先 + composite_score 降序
    /// 3. 去重; 候选全部来自 self.states 实际注册名, 数量上限 `limit`
    pub fn build_candidate_chain(&self, model: &str, limit: usize) -> Vec<String> {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let mut chain: Vec<String> = Vec::new();

        // 1. 前缀 provider 优先 (完整注册名匹配优先, 再退化到裸 provider 名)
        if let Some(prefix) = model.split('/').next().filter(|p| !p.is_empty()) {
            // 完整注册名: `llm7/codestral-latest` 恰好是 catalog 注册名时直接用
            // (该模型被锁定则跳过, 路由到其它 provider)
            if states.contains_key(model)
                && !chain.contains(&model.to_string())
                && !states.get(model).map_or(false, |s| s.is_model_locked(model))
            {
                chain.push(model.to_string());
            }
            // 裸 provider 名: `llm7` keyless 注册名
            if states.contains_key(prefix)
                && !chain.contains(&prefix.to_string())
                && !states.get(prefix).map_or(false, |s| s.is_model_locked(model))
            {
                chain.push(prefix.to_string());
            }
        }

        // 2. 池子其余注册名按 available + free + 有调用记录 + score 排序
        //    (有实际调用记录的 provider 优先于从未尝试的 — 后者默认 EMA 0.8 会虚高)
        let mut rest: Vec<(&String, f64, bool, bool, u64)> = states
            .iter()
            .map(|(name, s)| {
                (
                    name,
                    s.composite_score(),
                    s.is_free,
                    s.is_available(),
                    s.total_calls,
                )
            })
            .collect();
        rest.sort_by(|a, b| {
            // available 优先
            b.3.cmp(&a.3)
                // free 优先
                .then(b.2.cmp(&a.2))
                // 有调用记录优先 (避免未尝试 provider 默认 EMA 虚高)
                .then((b.4 > 0).cmp(&(a.4 > 0)))
                // score 降序
                .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        for (name, _, _, _, _) in rest {
            if chain.len() >= limit {
                break;
            }
            if chain.contains(name) {
                continue;
            }
            // 请求了具体模型且该 provider 上此模型被锁定 → 跳过, 路由到其它 provider
            if !model.is_empty() && states.get(name).map_or(false, |s| s.is_model_locked(model)) {
                continue;
            }
            chain.push(name.clone());
        }

        chain
    }

    /// quota-aware 回退链 — 吸收 `diegosouzapw/OmniRoute`: 当某 provider 触发配额/限流,
    /// 返回**排除已耗尽 provider** 的候选链, 交由执行器自动降级到下一流 (R-P42 强化现有
    /// `build_candidate_chain` 排序逻辑, 仅叠加 exclude 过滤, 不新建平行路由)。
    ///
    /// 等价于 OmniRoute 的 "quota-aware auto-fallback": 主选耗尽 → 自动跳到次优可用 provider。
    pub fn quota_aware_fallback_chain(
        &self,
        model: &str,
        exhausted: &[String],
        limit: usize,
    ) -> Vec<String> {
        let chain = self.build_candidate_chain(model, limit + exhausted.len());
        chain
            .into_iter()
            .filter(|name| !exhausted.iter().any(|e| e == name))
            .take(limit)
            .collect()
    }

    /// Register providers from FreeModelCatalog discovered entries.
    /// For each entry where the required API key env var is set (or keyless),
    /// create a provider and register it.
    pub fn register_from_catalog(&self, entries: &[FreeModelEntry]) {
        for entry in entries {
            let name = format!("{}/{}", entry.provider, entry.model_id);
            if self.providers.read().unwrap_or_else(|e| e.into_inner()).contains_key(&name) {
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
            let provider = create_provider(ProviderConfig {
                provider_type: entry.provider_type,
                api_key,
                base_url: Some(entry.base_url.clone()),
                model: Some(entry.model_id.clone()),
                timeout_secs: 60,
                proxy: proxy_from_env(),
            });
            self.register_provider_with_category(
                &name,
                provider.into(),
                entry.is_free,
                ProviderCategory::Cloud,
            );
            log::info!(
                "[gateway] Registered from catalog: {} ({})",
                name,
                entry.display_name
            );
        }
    }

    pub fn provider_status(&self) -> Vec<serde_json::Value> {        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        states
            .iter()
            .map(|(name, state)| {
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
            })
            .collect()
    }

    /// 当前可用 (circuit 未开) 的免费 provider 名称列表 — 用于池子充足度自检。
    pub fn available_free_providers(&self) -> Vec<String> {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        states
            .iter()
            .filter(|(_, s)| s.is_free && s.is_available())
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// 池子充足度自检: 可用免费 provider 数 >= `min` 即视为充足。
    /// 目标 "自有 LLM 池子里保持充足的模型" 的量化门槛 — 低于该数应触发 catalog 补充。
    pub fn is_pool_sufficient(&self, min: usize) -> bool {
        self.available_free_providers().len() >= min
    }

    /// 池子充足度报告 (JSON) — 供 telemetry / CLI 观测当前自有池健康度。
    pub fn pool_sufficiency_report(&self, min: usize) -> serde_json::Value {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let total = states.len();
        let free_total = states.values().filter(|s| s.is_free).count();
        let free_available = states
            .values()
            .filter(|s| s.is_free && s.is_available())
            .count();
        let locked_models: usize = states
            .values()
            .map(|s| s.model_locks.iter().filter(|(_, &until)| Instant::now() < until).count())
            .sum();
        serde_json::json!({
            "total_providers": total,
            "free_total": free_total,
            "free_available": free_available,
            "model_locks_active": locked_models,
            "min_required": min,
            "sufficient": free_available >= min,
        })
    }

    /// 从 FreeModelCatalog 重新发现并补充可用 keyless 源 (对齐 OmniRoute Radar 刷新)。
    /// 幂等: 已注册名跳过。`&self` 即可调用 (providers 内部可变性) — 故可由
    /// `complete_with_selection` 在池子偏薄时按需自愈 (T3 生产接线), 或由后台循环周期调用,
    /// 使自有 LLM 池始终维持充足模型, 而非启动一次性注册后静止。
    /// `cooldown_secs` 防止单次请求链内重复全量刷新 (刷新本身有 I/O 成本)。
    pub async fn reconcile_pool_from_catalog(&self, cooldown_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        {
            let mut last = self.last_reconcile_ts.lock().unwrap_or_else(|e| e.into_inner());
            if now.saturating_sub(*last) < cooldown_secs {
                return;
            }
            *last = now;
        }
        // catalog.refresh() 为阻塞网络 I/O, 必须在 spawn_blocking 中执行,
        // 否则在 async 运行时内直接调用会触发 tokio 1.52+ 的 "blocking in async" panic。
        let mut catalog = FreeModelCatalog::new();
        let discovered = tokio::task::spawn_blocking(move || catalog.refresh())
            .await
            .unwrap_or_default();
        let before = self.providers().len();
        self.register_from_catalog(&discovered);
        let after = self.providers().len();
        if after > before {
            log::info!(
                "[gateway] reconcile_pool_from_catalog: +{} new providers ({}→{})",
                after - before,
                before,
                after
            );
        }
    }

    /// 池子偏薄时按需补充 (需求驱动自愈): 仅当可用免费 provider 低于 `min_free` 时触发,
    /// 且受 `reconcile_pool_from_catalog` 内置 cooldown 节流。T3 生产接线点 — 每次请求入口调用。
    ///
    /// 必须经 `self_heal_reconcile` 开关门禁: 默认 (单测) 关, 避免每个请求入口触发阻塞式
    /// catalog 网络刷新 (破坏测试确定性 + 生产误触发); 仅 `create_gateway_async` 置 true。
    pub async fn ensure_pool_sufficient(&self, min_free: usize, cooldown_secs: u64) {
        if !self
            .self_heal_reconcile
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        if !self.is_pool_sufficient(min_free) {
            log::warn!(
                "[gateway] pool insufficient (min_free={}): triggering reconcile_pool_from_catalog",
                min_free
            );
            self.reconcile_pool_from_catalog(cooldown_secs).await;
            log::info!(
                "[gateway] {}",
                super::LlmPoolHealth::summarize(self, min_free)
            );
        }
    }
    /// 当前被 L3 模型级熔断锁定的 (provider, model) 数量 — 池健康度指标。
    pub fn model_locked_count(&self) -> usize {
        self.states
            .read()
            .unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            })
            .values()
            .map(|s| {
                s.model_locks
                    .iter()
                    .filter(|(_, &until)| Instant::now() < until)
                    .count()
            })
            .sum()
    }

    /// 已注册 provider 名称列表
    pub fn providers(&self) -> Vec<String> {
        self.states
            .read()
            .unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            })
            .keys()
            .cloned()
            .collect()
    }

    /// 查询 provider 的安全分类
    pub fn category_of(&self, name: &str) -> Option<ProviderCategory> {
        self.states
            .read()
            .unwrap_or_else(|e| {
                log::warn!("[gateway] states RwLock poisoned: {}", e);
                e.into_inner()
            })
            .get(name)
            .map(|s| s.category)
    }

    /// 默认 provider 名称 (注册的第一个, 无则空串)
    pub fn default_provider_name(&self) -> String {
        self.default_name
            .read()
            .unwrap_or_else(|e| {
                log::warn!("[gateway] default_name RwLock poisoned: {}", e);
                e.into_inner()
            })
            .clone()
    }

    /// 解析默认模型 — 从池子**实际注册名**选最佳可用者, 而非硬编码。
    ///
    /// 当调用方未显式指定模型 (如 `default`) 时, 用候选链第一个可用注册名作为完整
    /// model 名 (含 `{provider}/{model_id}` 或裸 `{provider}` 格式), 保证整体链路
    /// 从池子真实状态出发, 而非写死某个 provider。
    /// 同步版 (async 版见 `resolve_default_model`, 优先 llm7/codestral-latest)。
    pub fn resolve_default_model_sync(&self) -> String {
        let chain = self.build_candidate_chain("", 3);
        chain
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    }
}