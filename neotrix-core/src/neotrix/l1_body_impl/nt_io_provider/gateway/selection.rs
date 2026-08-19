use crate::neotrix::l1_body_impl::nt_io_http_factory::proxy_from_env;

use super::super::factory::{create_provider, ProviderConfig};
use super::super::free_catalog::FreeModelEntry;
use super::super::rate_limiter::RateLimiter;
use super::super::rate_profiles::get_rate_profile;
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

    pub fn register_provider(&mut self, name: &str, provider: Box<dyn LlmProvider>, is_free: bool) {
        self.register_provider_with_category(name, provider, is_free, ProviderCategory::Cloud)
    }

    pub fn register_provider_with_category(
        &mut self,
        name: &str,
        provider: Box<dyn LlmProvider>,
        is_free: bool,
        category: ProviderCategory,
    ) {
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
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });

        // Tier 1: Available free providers (preferred)
        let free_best = states
            .iter()
            .filter(|(_, s)| s.is_available() && s.is_free)
            .max_by(|(_, a), (_, b)| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
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
                .max_by(|(_, a), (_, b)| {
                    a.composite_score()
                        .partial_cmp(&b.composite_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(name, _)| name.clone());
        }

        // Tier 3: Any available provider (free-first exhausted all free, allow paid)
        states
            .iter()
            .filter(|(_, s)| s.is_available())
            .max_by(|(_, a), (_, b)| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
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
            if states.contains_key(model) && !chain.contains(&model.to_string()) {
                chain.push(model.to_string());
            }
            // 裸 provider 名: `llm7` keyless 注册名
            if states.contains_key(prefix) && !chain.contains(&prefix.to_string()) {
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
            if !chain.contains(name) {
                chain.push(name.clone());
            }
        }

        chain
    }

    /// Register providers from FreeModelCatalog discovered entries.
    /// For each entry where the required API key env var is set (or keyless),
    /// create a provider and register it.
    pub fn register_from_catalog(&mut self, entries: &[FreeModelEntry]) {
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
            let mut provider = create_provider(ProviderConfig {
                provider_type: entry.provider_type,
                api_key,
                base_url: Some(entry.base_url.clone()),
                model: Some(entry.model_id.clone()),
                timeout_secs: 60,
                proxy: None,
            });
            // 代理注入: 与手工 keyless 注册一致, 本机 fake-ip 分流网络下直连会全部超时
            if let Some(proxy_url) = proxy_from_env() {
                provider.set_proxy(&proxy_url);
            }
            self.register_provider_with_category(
                &name,
                provider,
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

    pub fn provider_status(&self) -> Vec<serde_json::Value> {
        let states = self.states.read().unwrap_or_else(|e| {
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