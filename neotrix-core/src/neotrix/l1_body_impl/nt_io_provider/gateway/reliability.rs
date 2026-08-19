use std::collections::HashMap;

use super::super::generation_classifier::{GenerationRecord, LlmPurpose, TaskType};
use super::*;

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
    pub fn set_generation_classification(&mut self, enabled: bool) {
        self.generation_classification_enabled = enabled;
    }

    pub fn generation_classification_enabled(&self) -> bool {
        self.generation_classification_enabled
    }

    /// 记录一次生成分类到 analytics (供 activity analytics 聚合)。
    /// 在成功响应完成路径调用 — 与 heal_and_cache_response 同位置。
    pub(super) fn tag_generation(
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
    pub fn generation_analytics_snapshot(&self) -> (u64, HashMap<String, u64>, HashMap<String, u64>, HashMap<String, u64>) {
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
    pub(super) fn heal_and_cache_response(&self, request: &LlmRequest, response: LlmResponse) -> LlmResponse {
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
    pub(super) fn prompt_text(&self, request: &LlmRequest) -> String {
        request
            .messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 缓存 key 硬化: 除消息内容外, 纳入会影响响应语义的请求指纹
    /// (max_tokens / thinking_budget / tools / structured_output / prefix 标记)。
    ///
    /// 旧 key 仅拼 messages 内容: 同一提示词在不同 max_tokens 或不同工具集下的
    /// 请求会错误共享缓存 — 可能命中被截断输出 (Length) 或带 tool_calls 的响应,
    /// 属质量损失型 bug。改为指纹后, 缓存命中语义与请求完全一致。
    pub(super) fn prompt_cache_key(&self, request: &LlmRequest) -> String {
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