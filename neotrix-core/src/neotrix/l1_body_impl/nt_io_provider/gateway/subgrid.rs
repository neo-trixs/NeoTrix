use crate::core::nt_io_cache::text_to_embedding;

use super::super::provider_catalog::lookup_provider;
use super::*;

impl GatewayV2 {
    // ═══════════════════════════════════════════════════════════════════
    // SubGrid Composition (子母阵动态组合)
    // 通过组合已有 provider 节点，构建满足指定通信安全级别的小循环子网格
    // ═══════════════════════════════════════════════════════════════════

    /// 动态组合一个子网格: 从已注册 providers 中选出满足安全画像的子集
    /// `security_profile` 指定目标安全级别; `include_free_only` 限制只组合免费 provider
    pub fn compose_sub_grid(
        &self,
        name: &str,
        security_profile: CommunicationProfile,
        include_free_only: bool,
    ) -> SubGrid {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let mut provider_names: Vec<String> = states
            .iter()
            .filter(|(_, s)| {
                let profile_ok = s
                    .category
                    .default_security_profile()
                    .meets(security_profile);
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
        let best_free = states
            .iter()
            .filter(|(_, s)| {
                s.is_available()
                    && s.is_free
                    && s.category.default_security_profile().meets(required)
            })
            .max_by(|(_, a), (_, b)| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone());
        if best_free.is_some() {
            return best_free;
        }
        // Tier 2: 满足安全级别的任意 provider
        states
            .iter()
            .filter(|(_, s)| {
                s.is_available() && s.category.default_security_profile().meets(required)
            })
            .max_by(|(_, a), (_, b)| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone())
    }

    /// 解析默认模型名（用于交互模式初始化）。
    /// 优先返回真正匿名可用的免费模型 (llm7/codestral-latest — 实测唯一匿名可用流式端点,
    /// 见本文件 2123 行注释; api-airforce 需真 key 401, pollinations 429/402),
    /// 其次走 select_best_for_profile (免费 provider 优先)。
    pub async fn resolve_default_model(&self) -> String {
        let llm7_ok = self
            .states
            .read()
            .map(|s| s.get("llm7").map(|st| st.is_available()).unwrap_or(false))
            .unwrap_or(false);
        if llm7_ok {
            return "llm7/codestral-latest".to_string();
        }
        self.select_best_for_profile(CommunicationProfile::Open)
            .await
            .unwrap_or_else(|| "default".to_string())
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
        self.complete_for_profile_detailed(required, request)
            .await
            .map(|(r, _, _)| r)
    }

    /// 与 complete_for_profile 相同, 但返回 (响应, 实际使用的画像, 实际 provider 名)。
    /// 供 CapabilityCoordinator 报告真实的降级状态 (修复 degraded 恒 false 失真)。
    pub(super) async fn complete_for_profile_detailed(
        &self,
        required: CommunicationProfile,
        request: &LlmRequest,
    ) -> Result<(LlmResponse, CommunicationProfile, String), LlmError> {
        let start = std::time::Instant::now();
        // 缓存接入 (与 complete_with_selection 同口径): 命中直接返回, 免去
        // 子网格选择 + provider 调用整条链路 — 重复的画像内请求 (如能力协调器
        // 对同一结构化抽取的周期调用) 不再重复计费。
        let cache_key = self.prompt_cache_key(request);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_exact(&request.model, &cache_key) {
                if let Ok(response) = serde_json::from_str::<LlmResponse>(&cached) {
                    return Ok((response, required, request.model.clone()));
                }
            }
        }
        // 语义回退 (与 complete_with_selection 同口径): exact miss 后按 embedding 余弦兜底
        {
            let embedding = text_to_embedding(&self.prompt_text(request));
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(cached) = cache.get_semantic(&embedding) {
                    if let Ok(response) = serde_json::from_str::<LlmResponse>(cached) {
                        return Ok((response, required, request.model.clone()));
                    }
                }
            }
        }
        // 前缀锁定: request.model 形如 `{provider}/{model_id}` 时优先该 provider
        // (与 complete_with_selection 前缀路由一致), 避免 llm7/codestral-latest
        // 被 select_best_for_profile 路由到 composite_score 更高的 pollinations (402/404)。
        let prefix_provider: Option<String> = request
            .model
            .split('/')
            .next()
            .filter(|p| !p.is_empty())
            .filter(|p| self.providers.contains_key(*p))
            .map(|p| p.to_string());
        // 前缀 provider 的安全画像满足要求才锁定 (如 Open 画像可用所有 provider)
        let prefix_meets = |p: &str| {
            lookup_provider(p)
                .map(|info| info.security_profile.meets(required))
                .unwrap_or(true)
        };
        let selected = match &prefix_provider {
            Some(p) if prefix_meets(p) => Some(p.clone()),
            _ => self.select_best_for_profile(required).await,
        };
        match selected {
            Some(name) => {
                log::debug!("[gateway] complete_for_profile({:?}) → {}", required, name);
                let result = self.call_provider(&name, request).await;
                let latency = start.elapsed().as_millis() as u64;
                let success = result.is_ok();
                self.record_sub_grid_call(required, success, latency);
                if success {
                    let resp = result;
                    // 回写 exact cache (仅成功路径)
                    if let Ok(response) = &resp {
                        if let Ok(mut cache) = self.cache.lock() {
                            if let Ok(serialized) = serde_json::to_string(response) {
                                cache.set_with_embedding(
                                    &request.model,
                                    &cache_key,
                                    serialized,
                                    text_to_embedding(&self.prompt_text(request)),
                                );
                            }
                        }
                    }
                    resp.map(|r| (r, required, name))
                } else {
                    // Gap 4: 首选 provider 失败 → 降级到更宽松画像重试
                    let err = result
                        .err()
                        .unwrap_or_else(|| LlmError::Unknown("profile call failed".into()));
                    self.degraded_retry(required, request, &err).await
                }
            }
            None => {
                log::warn!(
                    "[gateway] no provider meets {:?} — falling back to default selection",
                    required
                );
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
                value: format!(
                    "{}:{}:{}ms",
                    if success { "ok" } else { "err" },
                    success as u8,
                    latency_ms
                ),
            },
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
            log::warn!(
                "[gateway] profile {:?} degraded → {:?}: {}",
                required,
                target,
                first_err
            );
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
        self.list_sub_grids()
            .into_iter()
            .map(|g| (g.name, g.health))
            .collect()
    }

    /// 检查满足指定画像的子网格中是否有健康可用的 (Gap 4 前置判断)
    pub fn has_healthy_sub_grid(&self, required: CommunicationProfile) -> bool {
        self.sub_grids_meeting(required).iter().any(|grid_name| {
            self.sub_grid_health_report()
                .iter()
                .any(|(name, health)| name == grid_name && health.is_healthy())
        })
    }

    /// 返回当前已组成的子网格中满足给定安全级别的网格名称
    pub fn sub_grids_meeting(&self, required: CommunicationProfile) -> Vec<String> {
        self.list_sub_grids()
            .into_iter()
            .filter(|sg| sg.security_profile.meets(required))
            .map(|sg| sg.name)
            .collect()
    }
}