use std::collections::HashMap;

use serde::Serialize;

use crate::core::nt_core_llm::{LlmRequest, LlmResponse};
use crate::l1_action::nt_io::nt_io_provider::common::generation_classifier::{GenerationRecord, LlmPurpose, TaskType};
use super::super::GatewayV2;
use super::*;

// ═══════════════════════════════════════════════════════════════════
// Pool Health — LLM 池健康探测器 (NT-REPAIR 自愈节点 / SelfTest T1)
// ═══════════════════════════════════════════════════════════════════

/// LLM 池健康快照 — 由 `LlmPoolHealth::evaluate` 产出, 供日志/断言/自愈决策消费。
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct PoolHealthReport {
    /// 已注册 provider 总数
    pub total: usize,
    /// 免费 (keyless) provider 数
    pub free: usize,
    /// 付费 provider 数
    pub paid: usize,
    /// 本地 (主体) provider 数
    pub local: usize,
    /// 当前被 L3 模型级熔断锁定的数量
    pub model_locked: usize,
    /// 是否满足 `min_free` 充足阈值
    pub sufficient: bool,
    /// 充足阈值 (免费 provider 下限)
    pub min_free: usize,
}

/// LLM 池健康探测器 — T1 存在 (impl) + T3 生产接线 (由 `ensure_pool_sufficient` 调用其 `evaluate`)。
pub struct LlmPoolHealth;

impl LlmPoolHealth {
    /// 对 `gw` 当前池子做健康评估, 返回结构化报告。纯读, 无副作用。
    pub fn evaluate(gw: &GatewayV2, min_free: usize) -> PoolHealthReport {
        let providers = gw.providers();
        let total = providers.len();
        let free = gw.available_free_providers().len();
        let model_locked = gw.model_locked_count();
        let sufficient = gw.is_pool_sufficient(min_free);
        let local = providers
            .iter()
            .filter(|n| n.contains("ollama") || n.contains("vllm") || n.contains("llama"))
            .count();
        PoolHealthReport {
            total,
            free,
            paid: total.saturating_sub(free),
            local,
            model_locked,
            sufficient,
            min_free,
        }
    }

    /// 健康度文字摘要 (供日志)。
    pub fn summarize(gw: &GatewayV2, min_free: usize) -> String {
        let r = Self::evaluate(gw, min_free);
        format!(
            "LLM pool health: total={} free={} paid={} local={} model_locked={} sufficient={} (min_free={})",
            r.total, r.free, r.paid, r.local, r.model_locked, r.sufficient, r.min_free
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// Provider Reliability Suite — G: Response Caching / Healing / MarketRouter
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
    // ── G: Provider Reliability Suite 接线 ──────────────────────────

    /// 开关 LRU 响应缓存 (G: Response Caching)
    pub fn enable_response_cache(&mut self, enabled: bool) {
        self.response_cache_enabled = enabled;
    }

    pub fn response_cache_enabled(&self) -> bool {
        self.response_cache_enabled
    }

    /// LRU 响应缓存命中计数 (遥测可见)
    pub fn response_cache_hits(&self) -> u64 {
        self.response_cache.lock().map(|c| c.hit_count()).unwrap_or(0)
    }

    pub fn response_cache_len(&self) -> usize {
        self.response_cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    /// P0-7 lookahead 预取命中计数 (OasisKV, 遥测可见)。
    pub fn response_cache_prefetches(&self) -> u64 {
        self.response_cache_prefetches
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 开关畸形 JSON 修复 (G: Response Healing)
    pub fn set_response_healer(&mut self, enabled: bool) {
        self.response_healer_enabled = enabled;
    }

    pub fn response_healer_enabled(&self) -> bool {
        self.response_healer_enabled
    }

    /// 修复器计数 (heal, unrepairable)
    pub fn response_healer_counters(&self) -> (u64, u64) {
        match self.response_healer.lock() {
            Ok(h) => (h.heal_count(), h.unrepairable_count()),
            Err(_) => (0, 0),
        }
    }

    // ── F6: Generation Classification ─────────────────────────────

    /// 开关生成分类打标 (F6: Generation Classifier)
    pub(crate) fn _set_generation_classification(&mut self, enabled: bool) {
        self.generation_classification_enabled = enabled;
    }

    pub fn generation_classification_enabled(&self) -> bool {
        self.generation_classification_enabled
    }

    /// 记录一次生成分类到 analytics (供 activity analytics 聚合)。
    /// 在成功响应完成路径调用 — 与 heal_and_cache_response 同位置。
    pub(crate) fn tag_generation(
        &self,
        request: &LlmRequest,
        response: &LlmResponse,
        provider_name: &str,
        latency_ms: f64,
        tokens: u32,
        success: bool,
    ) {
        if !self.generation_classification_enabled {
            return;
        }
        let prompt = request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let classification = match self.generation_classifier.lock() {
            Ok(c) => c.classify(&prompt, &response.content),
            Err(e) => {
                log::warn!("[gateway] generation_classifier poisoned: {}", e);
                return;
            }
        };
        // llm_calls ledger 归因: 从分类任务类型推断业务用途 (AgentTurn 兜底)。
        let purpose = match classification.task_type {
            TaskType::ToolUse => LlmPurpose::ToolUse,
            TaskType::Summarization => LlmPurpose::Summarization,
            TaskType::Extraction => LlmPurpose::ToolUse,
            _ => LlmPurpose::AgentTurn,
        };
        let record = GenerationRecord {
            model: format!("{}/{}", provider_name, request.model),
            classification,
            prompt_len: prompt.len(),
            response_len: response.content.len(),
            latency_ms: latency_ms as u64,
            tokens,
            success,
            purpose,
        };
        if let Ok(mut analytics) = self.generation_analytics.lock() {
            analytics.record(&record);
        }
    }

    /// F6: analytics 快照 — (total, by_task_type, by_complexity, by_domain)
    pub(crate) fn _generation_analytics_snapshot(&self) -> (u64, HashMap<String, u64>, HashMap<String, u64>, HashMap<String, u64>) {
        match self.generation_analytics.lock() {
            Ok(a) => (
                a.total,
                a.distribution("task_type"),
                a.distribution("complexity"),
                a.distribution("domain"),
            ),
            Err(_) => (0, HashMap::new(), HashMap::new(), HashMap::new()),
        }
    }

    /// G: MarketRouter 周期重估 tick hook — 从当前 states 重算市场权重。
    /// 由外部循环 (Auto Exacto) 周期性调用; 内部受 5min 间隔约束。
    /// `&self` (纯内部锁) — 使 Arc<GatewayV2> 可直接被后台循环 tick。
    pub fn maybe_re_evaluate(&self) -> bool {
        let states = self.states.read().unwrap_or_else(|e| {
            log::warn!("[gateway] states RwLock poisoned: {}", e);
            e.into_inner()
        });
        let refs: Vec<&ProviderState> = states.values().collect();
        match self.market_router.lock() {
            Ok(mut router) => router.re_evaluate(&refs),
            Err(e) => {
                log::warn!("[gateway] market_router Mutex poisoned: {}", e);
                false
            }
        }
    }

    /// G: Response Healing + Caching — 成功响应后处理 (修复畸形 JSON, 回写 LRU 缓存)
    pub(crate) fn heal_and_cache_response(&self, request: &LlmRequest, response: LlmResponse) -> LlmResponse {
        let mut response = response;
        if self.response_healer_enabled {
            if let Ok(mut healer) = self.response_healer.lock() {
                if response.content.contains('{') || response.content.contains('[') {
                    response.content = healer.heal(&response.content);
                }
            }
        }
        if self.response_cache_enabled {
            let rc_key = ResponseCache::key_for_request(&request.model, &self.prompt_cache_key(request));
            if let Ok(mut rc) = self.response_cache.lock() {
                match serde_json::to_string(&response) {
                    Ok(serialized) => rc.insert(&rc_key, serialized),
                    Err(_) => rc.insert(&rc_key, response.content.clone()),
                }
                let hints = rc.lookahead_hints(&rc_key);
                if !hints.is_empty() {
                    let (_, _) = rc.prefetch_lookahead(&hints);
                    self.response_cache_prefetches.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        response
    }

    /// 请求的纯文本提示 (消息内容拼接) — 用于 embedding 与 token 估算。
    pub(crate) fn prompt_text(&self, request: &LlmRequest) -> String {
        request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 缓存 key 硬化: 除消息内容外, 纳入会影响响应语义的请求指纹
    /// (max_tokens / thinking_budget / tools / structured_output / prefix 标记)。
    pub(crate) fn prompt_cache_key(&self, request: &LlmRequest) -> String {
        let content = self.prompt_text(request);
        let mut tools: Vec<&str> = request
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>();
        tools.sort_unstable();
        let tools_fp = tools.join(",");
        let structured_fp = match &request.structured_output {
            Some(s) => serde_json::to_string(s).unwrap_or_default(),
            None => String::new(),
        };
        format!(
            "{}|max={}|think={:?}|tools=[{}]|struct={}|prefix={:?}",
            content,
            request.max_tokens,
            request.thinking_budget,
            tools_fp,
            structured_fp,
            request.cacheable_prefix_tokens
        )
    }
}

#[cfg(test)]
mod tests {
use super::super::*;

    #[test]
    fn test_pool_health_evaluate_empty_pool() {
        let gw = GatewayV2::new();
        let r = LlmPoolHealth::evaluate(&gw, 3);
        assert_eq!(r.total, 0);
        assert_eq!(r.free, 0);
        assert!(!r.sufficient, "空池不应判定为充足");
        assert_eq!(r.min_free, 3);
    }

    #[test]
    fn test_pool_health_summary_format() {
        let gw = GatewayV2::new();
        let s = LlmPoolHealth::summarize(&gw, 3);
        assert!(s.contains("LLM pool health"), "摘要应含前缀");
        assert!(s.contains("total=0"), "空池 total=0");
    }
}
