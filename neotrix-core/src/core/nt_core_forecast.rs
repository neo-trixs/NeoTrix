//! # NT-FORECAST — 信息推演 Agent
//!
//! 基于文献 6 原则 + 交易项目范式 + NeoTrix 现有组件构建的通用信息推演引擎：
//!
//! 1. **多情景树**（Schoemaker / ToT / LATS）— 输出 3-5 条演化路线而非单点
//! 2. **概率校准**（Gneiting / Halawi / KalshiBench）— Brier/ECE 监控 + 弃权机制
//! 3. **因果链驱动**（Pearl SCM / CausalStock）— Abduction-Action-Prediction 三步
//! 4. **事件驱动动态更新**（Ding 2015 / Wu 2020）— 结构化事件 + 时效衰减
//! 5. **多尺度共振**（gold_prediction 项目）— 多 horizon 趋势一致性
//! 6. **元认知反思**（Reflexion / ReTreVal）— 置信理由 + 校准反馈闭环
//!
//! 复用 NeoTrix 现有组件：E8AbductionBridge（因果推演）、SimulateEngine（情景模拟）、
//! SelfTest（T1 自测注册）。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::core::nt_core_e8::e8_abduction_bridge::E8AbductionBridge;
use crate::core::nt_core_e8::domain_transition::{CoTLength, E8TaskType};
use crate::core::nt_core_self_test::SelfTest;
use crate::neotrix::l1_body_impl::nt_io_provider::types::LlmRequest;

// ─────────────────────────────────────────────────────────────
// ① 结构化事件（Ding 2015: Actor-Action-Object + 时效衰减）
// ─────────────────────────────────────────────────────────────

/// 结构化事件 — 事件影响力随时间指数衰减，带方向性（利多/利空/中性）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredEvent {
    pub actor: String,
    pub action: String,
    pub object: String,
    /// 事件原始影响力 [0,1]
    pub impact: f64,
    /// 时效衰减率（越大衰减越快）
    pub decay_rate: f64,
    /// 事件发生的时间步
    pub occurred_at: u64,
    /// 方向性：+1 利多 / -1 利空 / 0 中性（决定情景树偏斜方向）
    pub valence: f64,
}

impl StructuredEvent {
    pub fn new(actor: &str, action: &str, object: &str, impact: f64) -> Self {
        Self {
            actor: actor.to_string(),
            action: action.to_string(),
            object: object.to_string(),
            impact: impact.clamp(0.0, 1.0),
            decay_rate: 0.1,
            occurred_at: 0,
            valence: 1.0,
        }
    }

    /// 带方向的构造器 — 情报摄取时显式指定利多/利空/中性。
    pub fn new_signed(
        actor: &str,
        action: &str,
        object: &str,
        impact: f64,
        valence: f64,
    ) -> Self {
        let mut ev = Self::new(actor, action, object, impact);
        ev.valence = valence.clamp(-1.0, 1.0);
        ev
    }

    /// 在给定当前时间步下的有效影响力（指数衰减）。
    pub fn effective_impact(&self, current_time: u64) -> f64 {
        let age = current_time.saturating_sub(self.occurred_at) as f64;
        self.impact * (-self.decay_rate * age).exp()
    }
}

/// 事件流 — 聚合多个事件的衰减后影响力。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStream {
    pub events: Vec<StructuredEvent>,
    pub current_time: u64,
}

impl EventStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, mut ev: StructuredEvent) {
        ev.occurred_at = self.current_time;
        self.events.push(ev);
        if self.events.len() > 256 {
            self.events.remove(0);
        }
    }

    pub fn advance(&mut self, steps: u64) {
        self.current_time += steps;
    }

    /// 时间推进（别名，语义更贴近推演时钟）。
    pub fn tick(&mut self, steps: u64) {
        self.advance(steps);
    }

    /// 聚合所有事件的有效影响力 — 指数软饱和（1-exp(-total)）。
    ///
    /// 区别于线性求和后 clamp：软饱和在多事件叠加时不会立刻打满，
    /// 保留区分度（1 个强事件 ≈ 0.63，5 个同权事件 ≈ 0.99 仍能分辨）。
    pub fn aggregate_impact(&self) -> f64 {
        let total: f64 = self
            .events
            .iter()
            .map(|e| e.effective_impact(self.current_time))
            .sum();
        1.0 - (-total).exp()
    }

    /// 净方向 — 事件按方向（valence × 影响力）加权的净信号，[-1, +1]。
    ///
    /// +1 = 一致利多，-1 = 一致利空，0 = 多空对冲/中性。
    pub fn net_direction(&self) -> f64 {
        let signed: f64 = self
            .events
            .iter()
            .map(|e| e.effective_impact(self.current_time) * e.valence)
            .sum();
        // 双曲正切饱和：小信号线性，大信号收敛到 ±1
        signed.tanh()
    }

    /// 方向一致性 — 事件间方向一致程度 [0,1]。
    ///
    /// = |净加权| / Σ|加权|：单事件或全同向 → 1；多空对冲 → 0。
    /// 度量的是"分歧度"而非信号强度，避免单事件被误判为低共识。
    pub fn consensus(&self) -> f64 {
        let abs_total: f64 = self
            .events
            .iter()
            .map(|e| e.effective_impact(self.current_time).abs())
            .sum();
        if abs_total <= 1e-12 {
            return 0.0;
        }
        let signed: f64 = self
            .events
            .iter()
            .map(|e| e.effective_impact(self.current_time) * e.valence)
            .sum();
        (signed.abs() / abs_total).clamp(0.0, 1.0)
    }

    /// 主导事件（影响力最大者）。
    pub fn dominant_event(&self) -> Option<&StructuredEvent> {
        self.events
            .iter()
            .max_by(|a, b| {
                a.effective_impact(self.current_time)
                    .partial_cmp(&b.effective_impact(self.current_time))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

// ─────────────────────────────────────────────────────────────
// ② 情景树（Schoemaker / ToT）
// ─────────────────────────────────────────────────────────────

/// 情景树节点 — 一个演化状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioNode {
    pub id: u64,
    pub name: String,
    pub state: String,
    /// 条件概率 [0,1]
    pub probability: f64,
    /// 置信度 [0,1]
    pub confidence: f64,
    /// 先行指标（用于实时监控）
    pub leading_indicators: Vec<String>,
    /// 失效条件（触发即推翻该情景）
    pub invalidation: Vec<String>,
    /// LLM 生成的情景叙事（可选 — 无 LLM 池子时为 None，降级确定性描述）
    pub narrative: Option<String>,
}

/// 情景树 — 演化路线图。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioTree {
    pub nodes: Vec<ScenarioNode>,
    pub edges: Vec<(u64, u64)>,
    next_id: u64,
}

impl ScenarioTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(
        &mut self,
        name: &str,
        state: &str,
        probability: f64,
        confidence: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(ScenarioNode {
            id,
            name: name.to_string(),
            state: state.to_string(),
            probability: probability.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            leading_indicators: Vec::new(),
            invalidation: Vec::new(),
            narrative: None,
        });
        id
    }

    pub fn add_edge(&mut self, parent: u64, child: u64) {
        self.edges.push((parent, child));
    }

    pub fn node(&self, id: u64) -> Option<&ScenarioNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: u64) -> Option<&mut ScenarioNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// 叶子节点（无出边）— 即最终演化路线。
    pub fn leaves(&self) -> Vec<&ScenarioNode> {
        let has_child: std::collections::HashSet<u64> =
            self.edges.iter().map(|(p, _)| *p).collect();
        self.nodes
            .iter()
            .filter(|n| !has_child.contains(&n.id))
            .collect()
    }

    /// 叶子概率之和（应≈1，用于校准检查）。
    pub fn leaf_probability_sum(&self) -> f64 {
        self.leaves().iter().map(|n| n.probability).sum()
    }

    /// 归一化叶子概率（确保和为 1）。
    pub fn normalize_leaves(&mut self) {
        let sum = self.leaf_probability_sum();
        if sum > 0.0 {
            let leaf_ids: Vec<u64> = self.leaves().iter().map(|n| n.id).collect();
            for n in self.nodes.iter_mut() {
                if leaf_ids.contains(&n.id) {
                    n.probability /= sum;
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// ③ 概率校准（Gneiting / Halawi）
// ─────────────────────────────────────────────────────────────

/// 校准跟踪器 — Brier score + ECE 监控。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalibrationTracker {
    /// (预测概率, 实际结果 0/1)
    pub records: Vec<(f64, f64)>,
}

impl CalibrationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, prob: f64, outcome: bool) {
        self.records.push((prob.clamp(0.0, 1.0), if outcome { 1.0 } else { 0.0 }));
        if self.records.len() > 4096 {
            self.records.remove(0);
        }
    }

    /// Brier score — 越低越好（0 完美，0.25 随机）。
    pub fn brier_score(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .records
            .iter()
            .map(|(p, y)| (p - y) * (p - y))
            .sum();
        sum / self.records.len() as f64
    }

    /// Expected Calibration Error — 分箱后 |准确率 - 平均置信度| 的加权均值。
    pub fn ece(&self, n_bins: usize) -> f64 {
        if self.records.is_empty() || n_bins == 0 {
            return 0.0;
        }
        let mut bins: Vec<(f64, f64, usize)> = vec![(0.0, 0.0, 0); n_bins];
        for (p, y) in &self.records {
            let bin = ((p * n_bins as f64).floor() as usize).min(n_bins - 1);
            bins[bin].0 += p;
            bins[bin].1 += y;
            bins[bin].2 += 1;
        }
        let total = self.records.len() as f64;
        let mut ece = 0.0;
        for (conf_sum, acc_sum, count) in bins {
            if count > 0 {
                let conf = conf_sum / count as f64;
                let acc = acc_sum / count as f64;
                ece += (count as f64 / total) * (acc - conf).abs();
            }
        }
        ece
    }

    /// 校准置信下限 — 若 ECE 高则下调置信度（防过度自信）。
    pub fn calibrated_confidence(&self, raw: f64) -> f64 {
        let penalty = self.ece(10);
        (raw - penalty).clamp(0.0, 1.0)
    }
}

// ─────────────────────────────────────────────────────────────
// ④ 推演输出契约（三要素：概率 + 置信理由 + 弃权）
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub target: String,
    pub tree: ScenarioTree,
    /// 置信理由（含知识缺口）
    pub confidence_reason: String,
    /// 弃权信号 — 信息不足时置 true
    pub abstain: bool,
    pub generated_at: u64,
}

// ─────────────────────────────────────────────────────────────
// ④.5 LLM 叙事层 — 内部自动调用 NT-IO LLM 池子生成情景叙事
// ─────────────────────────────────────────────────────────────

/// 情景叙事生成器 — 懒加载 NT-IO GatewayV2（LLM 池子），
/// 内部自动路由到可用 provider；无 provider 或调用失败时优雅降级。
pub struct LlmNarrator {
    /// 内部缓存 gateway 实例（避免每次推演重复探测本地端点）
    gateway: std::sync::OnceLock<gateway_handle::GatewayHandle>,
    /// LLM 模型名（None = 让池子自动选择）
    model: Option<String>,
    /// 单次叙事生成的最大 token（控制成本）
    max_tokens: u32,
}

/// Gateway 句柄 — 持有池子实例，`complete` 时自动 block_on。
mod gateway_handle {
    use crate::neotrix::l1_body_impl::nt_io_provider::factory;
    use crate::neotrix::l1_body_impl::nt_io_provider::gateway::GatewayV2;
    use crate::neotrix::l1_body_impl::nt_io_provider::types::{LlmRequest, LlmResponse};
    /// 持有 GatewayV2 并封装同步调用（池子内部自动选择 provider）。
    pub struct GatewayHandle(GatewayV2);

    impl GatewayHandle {
        pub fn new() -> Self {
            Self(factory::create_gateway())
        }

        /// 已注册的 provider 名列表（含 `provider/model` 形式，如 `api-airforce/grok-4.1-mini:free`）。
        pub fn providers(&self) -> Vec<String> {
            self.0.providers()
        }

        /// 单次调用指定 provider — 无重试连打，调用方自行节流（适配 keyless free 限流）。
        pub fn complete_single(&self, provider_name: &str, request: &LlmRequest) -> Result<LlmResponse, String> {
            let rt = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle.block_on(self.0.complete_single(provider_name, request)),
                Err(_) => {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => return Err(format!("runtime: {e}")),
                    };
                    rt.block_on(self.0.complete_single(provider_name, request))
                }
            };
            rt.map_err(|e| e.to_string())
        }
    }
}

impl LlmNarrator {
    pub fn new() -> Self {
        Self {
            gateway: std::sync::OnceLock::new(),
            model: None,
            max_tokens: 512,
        }
    }

    /// 指定模型名（None = 池子自动选择）。
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// 生成情景叙事 — 内部自动调用 LLM 池子。
    ///
    /// 自动从池子构建候选链（llm7 → pollinations → api-airforce），**逐个尝试 +
    /// 每候选自控节流重试**（适配 free 池限流，尊重服务端 retry_after）。
    /// 某个候选非限流失败/空 content 时切下一个候选；全部失败返回 None
    /// （调用方降级到确定性模板）。
    pub fn narrate_scenarios(&self, context: &str) -> Option<String> {
        let handle = self
            .gateway
            .get_or_init(gateway_handle::GatewayHandle::new);

        // ── 构建候选链 (provider注册名, model)：按可用性优先级排序 ──
        let candidates = if let Some(m) = &self.model {
            vec![(self.default_provider(handle.providers()).unwrap_or_default(), m.clone())]
        } else {
            Self::build_candidates(handle.providers())
        };
        if candidates.is_empty() {
            return None; // 池子为空 → 降级
        }

        // Token 优化 (2026-08): ① 输入压缩 — context 按预算截断, 控制每调用成本;
        // ② 调用预算门 — 全候选+全重试总调用 ≤ MAX_NARRATION_CALLS, 防 9 次全量重发。
        let context = Self::truncate_context(context, Self::DEFAULT_CONTEXT_TOKEN_BUDGET);
        let mut budget = CallBudgetImpl::new(Self::MAX_NARRATION_CALLS);

        for (provider, model) in &candidates {
            let mut request = LlmRequest::new(model, &context);
            request.max_tokens = self.max_tokens;
            request.temperature = Some(0.8);

            // 每候选：节流重试（429/并发限制 → 尊重 retry_after；非限流 → 切下一候选）
            let mut attempt = 0;
            let mut last_err: Option<String> = None;
            while attempt < 3 && budget.try_spend() {
                std::thread::sleep(std::time::Duration::from_millis(1200 + attempt * 800));
                match handle.complete_single(provider, &request) {
                    Ok(resp) if !resp.content.trim().is_empty() => {
                        log::info!(
                            "[nt_core_forecast] LLM narrate via {provider}/{model} ok ({} tokens, budget spent {}/{})",
                            resp.usage.total_tokens, budget.spent, budget.max_calls
                        );
                        return Some(resp.content);
                    }
                    Ok(_resp) => {
                        // 空 content（reasoning 模型把内容写进 reasoning 字段）→ 切下一候选
                        log::warn!(
                            "[nt_core_forecast] LLM narrate via {provider}/{model} returned empty content, trying next candidate"
                        );
                        break;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        let is_429 = msg.contains("429")
                            || msg.contains("rate limit")
                            || msg.contains("RateLimit")
                            || msg.contains("Queue full")
                            || msg.contains("concurrent_request_limit")
                            || msg.contains("retry_after");
                        log::warn!(
                            "[nt_core_forecast] LLM narrate attempt {}/3 via {provider}/{model} failed: {e}",
                            attempt + 1
                        );
                        last_err = Some(msg.clone());
                        if budget.exhausted() {
                            log::warn!(
                                "[nt_core_forecast] LLM narrate call budget exhausted ({}/{}) — 停止重试, 降级确定性模板",
                                budget.spent, budget.max_calls
                            );
                            return None;
                        }
                        if !is_429 {
                            break; // 非限流错误 → 切下一候选
                        }
                        // 解析服务端 retry_after（秒），无则指数退避；至少等 3s
                        let retry_after = msg
                            .split("\"retry_after\":")
                            .nth(1)
                            .and_then(|s| s.trim_start().split(|c: char| !c.is_ascii_digit()).next())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(2u64.pow(attempt as u32) * 3);
                        log::info!(
                            "[nt_core_forecast] LLM narrate via {provider}/{model} rate-limited, waiting {retry_after}s before retry"
                        );
                        std::thread::sleep(std::time::Duration::from_secs(retry_after));
                        attempt += 1;
                    }
                }
            }
            if let Some(e) = last_err {
                log::warn!("[nt_core_forecast] LLM narrate candidate {provider}/{model} exhausted: {e}");
            }
        }
        None
    }

    /// 从池子挑一个默认 provider（仅用于显式指定模型时的宿主选择）。
    fn default_provider(&self, names: Vec<String>) -> Option<String> {
        names
            .iter()
            .find(|n| n.starts_with("llm7"))
            .or_else(|| names.iter().find(|n| n.starts_with("pollinations")))
            .or_else(|| names.iter().find(|n| n.starts_with("api-airforce")))
            .or_else(|| names.iter().find(|n| !n.contains("ollama")))
            .cloned()
    }

    /// 构建候选链 (provider注册名, model)：按可用性优先级排序。
    ///
    /// provider 注册名直接用池子里的真实条目（含 model 后缀如 `llm7/codestral-latest`，
    /// 或裸名如 `llm7`），保证 complete_single 能命中。顺序：
    /// 1) llm7 — 首选，匿名 200 可用；先 codestral（非 reasoning，叙事完整），
    ///    再 gpt-oss:20b（reasoning：content 常空但兜底尝试）。
    /// 2) pollinations — 匿名层已关，仅 IP 信誉好时可用。
    /// 3) api-airforce — 有模型条目时用其 model_id。
    fn build_candidates(names: Vec<String>) -> Vec<(String, String)> {
        let mut c = Vec::new();
        // 1) llm7
        for n in &names {
            if n == "llm7" || n.starts_with("llm7/") {
                c.push((n.clone(), "codestral-latest".to_string()));
                c.push((n.clone(), "gpt-oss:20b".to_string()));
                break;
            }
        }
        // 2) pollinations
        if let Some(n) = names.iter().find(|n| n.starts_with("pollinations")) {
            c.push((n.clone(), "openai".to_string()));
        }
        // 3) api-airforce — 有模型条目时用其 model_id
        if let Some(n) = names.iter().find(|n| n.starts_with("api-airforce") && n.contains('/')) {
            if let Some((_, m)) = n.rsplit_once('/') {
                c.push((n.clone(), m.to_string()));
            }
        }
        c
    }

    /// 是否已初始化 gateway（幂等，供诊断用）。
    pub fn is_initialized(&self) -> bool {
        self.gateway.get().is_some()
    }

    // ── Token 消耗优化 (2026-08) ──────────────────────────────────────────
    // 依据外部文献 (SitePoint 2026 / Redis 2026 / tokenoptimize 2026) + 本地盘点:
    // ① 输入压缩: 上下文全量传入无截断 → 每调用全额重付
    // ② 调用预算: 3候选×3重试=9次全量调用 → 超预算即停 (对齐 aggressive-retry 放大成本)
    // ③ CJK 感知估算: 中文按字 1 token, 英文按 4 字符 1 token (对齐 nt_memory_skill_cost.rs:107)

    /// 默认输入上下文 token 预算（对齐 max_tokens=512 的输出预算, 输入控制在输出 2 倍内）。
    pub const DEFAULT_CONTEXT_TOKEN_BUDGET: usize = 800;

    /// CJK 感知 token 估算: 中文(含全角标点)按字符计 1 token, 其余按 4 字符计 1 token。
    pub fn estimate_tokens(text: &str) -> usize {
        let mut cjk = 0usize;
        let mut rest = 0usize;
        for ch in text.chars() {
            if is_cjk(ch) {
                cjk += 1;
            } else {
                rest += 1;
            }
        }
        cjk + rest.div_ceil(4)
    }

    /// 按预算截断上下文: 保留开头, 超预算部分截断并附标记。
    /// 保留语义前置信息 (背景在前, 预测对象在前), 丢弃尾部冗长。
    pub fn truncate_context(context: &str, budget: usize) -> String {
        if context.is_empty() || Self::estimate_tokens(context) <= budget {
            return context.to_string();
        }
        // 二分找到预算内的截断点 (按 char 边界, 不劈开中文字符)
        let marker = "\n…(截断)";
        let marker_cost = Self::estimate_tokens(marker);
        let mut lo = 0usize;
        let mut hi = context.chars().count();
        let chars: Vec<char> = context.chars().collect();
        // 找最大 char 数使 estimate_tokens(chars[..n]) <= budget - marker_cost
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let part: String = chars[..mid].iter().collect();
            if Self::estimate_tokens(&part) <= budget.saturating_sub(marker_cost) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let part: String = chars[..lo].iter().collect();
        let mut out = part;
        out.push_str(marker);
        out
    }

    /// 调用预算门: 限制单次叙事总 LLM 调用次数 (默认 5, 对应 1候选×3重试 的合理上限,
    /// 防止 3候选×3重试=9 次全量调用放大成本, 对齐本地盘点#5 aggressive-retry)。
    ///
    /// 注: 结构定义在 impl 外 (Rust 不允许 impl 内定义 struct), 见 `CallBudgetImpl`。
    pub const MAX_NARRATION_CALLS: u32 = 5;
}

/// 调用预算门别名 — LlmNarrator::CallBudget 的模块级真身 (见 impl 内注释)。
pub type CallBudget = CallBudgetImpl;

impl Default for LlmNarrator {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────
// ⑤ 核心引擎 — Abduction-Action-Prediction 三步
// ─────────────────────────────────────────────────────────────

pub struct ForecastEngine {
    pub abductive: E8AbductionBridge,
    pub events: EventStream,
    pub calibration: CalibrationTracker,
    pub abstain_threshold: f64,
    pub forecast_count: u64,
    /// 可选 LLM 叙事层 — 设置后推演内部自动调用 LLM 池子生成情景叙事
    pub narrator: Option<LlmNarrator>,
}

impl ForecastEngine {
    pub fn new() -> Self {
        Self {
            abductive: E8AbductionBridge::new(0.5),
            events: EventStream::new(),
            calibration: CalibrationTracker::new(),
            abstain_threshold: 0.35,
            forecast_count: 0,
            narrator: None,
        }
    }

    /// 启用 LLM 叙事层 — 推演时内部自动调用 NT-IO LLM 池子。
    /// `model` 传 None 让池子自动选择最佳 provider。
    pub fn with_llm_narrator(mut self, model: Option<&str>) -> Self {
        let mut narrator = LlmNarrator::new();
        if let Some(m) = model {
            narrator = narrator.with_model(m);
        }
        self.narrator = Some(narrator);
        self
    }

    /// 情报摄取 — 注入结构化事件（默认利多方向）。
    pub fn ingest_event(&mut self, actor: &str, action: &str, object: &str, impact: f64) {
        self.events.push(StructuredEvent::new(actor, action, object, impact));
    }

    /// 情报摄取（带方向）— +1 利多 / -1 利空 / 0 中性。
    pub fn ingest_signed_event(
        &mut self,
        actor: &str,
        action: &str,
        object: &str,
        impact: f64,
        valence: f64,
    ) {
        self.events
            .push(StructuredEvent::new_signed(actor, action, object, impact, valence));
    }

    /// 时间推进。
    pub fn tick(&mut self, steps: u64) {
        self.events.tick(steps);
    }

    /// 生成推演 — Abduction-Action-Prediction 三步。
    ///
    /// - **Abduction**: 用 E8 溯因桥解释当前状态，预测下一状态 + 置信度
    /// - **Action**: 事件流影响力作为干预信号
    /// - **Prediction**: 生成情景树（基准/乐观/悲观），绑定先行指标与失效条件
    pub fn generate_forecast(&mut self, target: &str, base_state: u8) -> Forecast {
        self.forecast_count += 1;

        // Abduction: E8 预测下一状态
        let (predicted, base_conf) = self.abductive.predict_with_abduction(
            base_state,
            E8TaskType::Reasoning,
            CoTLength::Medium,
        );

        // Action: 事件强度 + 净方向（缺陷修复：方向驱动偏斜，非等权 0.8/0.2）
        let strength = self.events.aggregate_impact();
        let direction = self.events.net_direction();
        let consensus = self.events.consensus();

        // 信息量调制置信度：证据越强置信越高（但保留 ECE 惩罚下限）
        let info_conf = (base_conf + (0.25 * strength).min(0.25)).min(1.0);
        let calibrated = self.calibration.calibrated_confidence(info_conf);

        // 弃权判定：事件影响力过低 且 校准置信度低于阈值
        let abstain = strength < self.abstain_threshold && calibrated < self.abstain_threshold;

        // Prediction: 构建情景树（牛/熊/震荡三分支）
        //
        // 概率分配原则（方向一致性驱动）：
        //   - directional = strength × consensus：方向性证据质量（0=分歧, 1=一致）
        //   - 一致利多 (direction→+1) → 牛分支占满 directional
        //   - 一致利空 (direction→-1) → 熊分支占满 directional
        //   - 分歧 (consensus→0) → directional→0，概率让给震荡分支
        //   - 事件越强 (strength↑) → 方向性越极端，震荡越少
        let mut tree = ScenarioTree::new();
        let root = tree.add_node("root", &format!("state_{}", base_state), 1.0, calibrated);

        let directional = strength * consensus;
        let sideways_p = (1.0 - directional).clamp(0.0, 1.0);
        // 方向性质量按 direction 在牛/熊间分配（各保底 0.05 防零概率）
        let bull_p = (directional * (0.5 + 0.5 * direction)).clamp(0.05, 0.95);
        let bear_p = (directional * (0.5 - 0.5 * direction)).clamp(0.05, 0.95);

        let bull = tree.add_node("bull", &format!("state {} (bullish)", predicted), bull_p, calibrated);
        let bear = tree.add_node(
            "bear",
            &format!("state {} (bearish)", predicted.saturating_sub(1)),
            bear_p,
            calibrated,
        );
        let sideways = tree.add_node(
            "sideways",
            &format!("state {} (sideways)", base_state),
            sideways_p.max(0.0),
            calibrated,
        );
        tree.add_edge(root, bull);
        tree.add_edge(root, bear);
        tree.add_edge(root, sideways);
        // 保底 clamp 可能使三分支之和略偏离 1 → 归一化叶子保证合法概率分布
        tree.normalize_leaves();

        // 绑定先行指标与失效条件
        if let Some(b) = tree.node_mut(bull) {
            b.leading_indicators = vec!["net_direction_positive".into(), "strength_rising".into()];
            b.invalidation = vec!["net_direction_flips_negative".into()];
        }
        if let Some(b) = tree.node_mut(bear) {
            b.leading_indicators = vec!["net_direction_negative".into()];
            b.invalidation = vec!["net_direction_breaks_positive".into()];
        }
        if let Some(s) = tree.node_mut(sideways) {
            s.leading_indicators = vec!["consensus_near_zero".into()];
            s.invalidation = vec!["consensus_breaks_0.4".into()];
        }

        // LLM 叙事层 — 内部自动调用 NT-IO LLM 池子生成每个情景的演化叙事。
        // 无 narrator / 调用失败 / 返回空 → 降级到确定性描述（不阻塞推演）。
        if let Some(narrator) = &self.narrator {
            let context = self.build_narrative_context(target, base_state, strength, direction, consensus);
            if let Some(narrative) = narrator.narrate_scenarios(&context) {
                let lines: Vec<&str> = narrative.lines().collect();
                let mut section = 0usize;
                // 期望 LLM 按顺序输出 "bull/bear/sideways: ..." 三段
                for node_id in [bull, bear, sideways] {
                    if section >= lines.len() {
                        break;
                    }
                    // 跳过空行
                    while section < lines.len() && lines[section].trim().is_empty() {
                        section += 1;
                    }
                    if section < lines.len() {
                        let text = lines[section].trim().trim_start_matches(|c| matches!(c, '1'..='9' | '.' | '-' | ' ' | ':' | '•'));
                        if !text.is_empty() && !text.contains("scenario") && !text.contains("Scenario") {
                            if let Some(n) = tree.node_mut(node_id) {
                                n.narrative = Some(text.to_string());
                            }
                        }
                        section += 1;
                    }
                }
            }
        }
        // 兜底：无 LLM 叙事的分支用确定性描述（概率取归一化后节点实际值）
        let deterministic = |name: &str, prob: f64| {
            format!(
                "{name} scenario with estimated probability {:.1}% based on event direction and strength",
                prob * 100.0
            )
        };
        if let Some(b) = tree.node_mut(bull) {
            if b.narrative.is_none() {
                let p = b.probability;
                b.narrative = Some(deterministic("Bullish", p));
            }
        }
        if let Some(b) = tree.node_mut(bear) {
            if b.narrative.is_none() {
                let p = b.probability;
                b.narrative = Some(deterministic("Bearish", p));
            }
        }
        if let Some(s) = tree.node_mut(sideways) {
            if s.narrative.is_none() {
                let p = s.probability;
                s.narrative = Some(deterministic("Sideways", p));
            }
        }

        let reason = format!(
            "E8 predicted state {} (base {:.2}); strength {:.2}; direction {:.2}; consensus {:.2}; calibrated {:.2}",
            predicted, base_conf, strength, direction, consensus, calibrated
        );

        Forecast {
            target: target.to_string(),
            tree,
            confidence_reason: reason,
            abstain,
            generated_at: self.events.current_time,
        }
    }

    /// 构建 LLM 叙事上下文 — 汇总目标、事件流与推演信号。
    pub fn build_narrative_context(
        &self,
        target: &str,
        base_state: u8,
        strength: f64,
        direction: f64,
        consensus: f64,
    ) -> String {
        let mut ctx = String::new();
        ctx.push_str("You are a scenario forecaster. Given the target and the structured intelligence events, ");
        ctx.push_str("write exactly three short scenario narratives, one per line, in this order:\n");
        ctx.push_str("1. Bullish scenario (positive driver)\n2. Bearish scenario (negative driver)\n3. Sideways scenario (consensus/confusion)\n");
        ctx.push_str("Each line: max 60 words, concrete and specific to the events. No labels, no bullets, no numbering.\n\n");
        ctx.push_str(&format!("TARGET: {target}\nBASE_STATE: {base_state}\n"));
        ctx.push_str(&format!(
            "SIGNAL: strength={strength:.2} direction={direction:.2} consensus={consensus:.2}\n\n"
        ));
        ctx.push_str("EVENTS:\n");
        for ev in &self.events.events {
            let valence = if ev.valence > 0.0 { "bullish" } else if ev.valence < 0.0 { "bearish" } else { "neutral" };
            ctx.push_str(&format!(
                "- [{}] {} {} {} (impact {:.2})\n",
                valence, ev.actor, ev.action, ev.object, ev.impact
            ));
        }
        ctx
    }

    /// 校准反馈 — 用主导叶子概率记录（非叶子概率和：恒 1 无信息量）。
    pub fn resolve_outcome(&mut self, forecast: &Forecast, occurred: bool) {
        let leaf_prob = forecast
            .tree
            .leaves()
            .iter()
            .map(|n| n.probability)
            .fold(0.0, f64::max);
        self.calibration.record(leaf_prob, occurred);
    }
}

impl Default for ForecastEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for ForecastEngine {
    fn name(&self) -> &'static str {
        "ForecastEngine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.forecast_count == 0 {
            failures.push("no forecast generated".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// ⑥ KB 持久化层 — 推演记录落盘到 knowledge.db kv_store
//    namespace: 'forecast'（与 experience 同库不同命名空间）
// ─────────────────────────────────────────────────────────────

/// 推演落盘记录 — 序列化后的完整推演（情景树 + 概率 + 置信理由）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRecord {
    pub target: String,
    pub forecast: Forecast,
    pub calibration: Vec<(f64, f64)>,
    pub event_stream: Vec<StructuredEvent>,
    pub created_at: u64,
}

impl ForecastEngine {
    /// KB 落盘 — 将推演记录写入 `~/.neotrix/knowledge.db` 的 kv_store，
    /// namespace='forecast'，key=`forecast_{target}_{ts}`。
    pub fn persist_to_kb(&self, forecast: &Forecast) -> Result<(), String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = format!("{}/.neotrix/knowledge.db", home);
        let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(30)).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=30000; PRAGMA synchronous=NORMAL;")
            .map_err(|e| e.to_string())?;

        let record = ForecastRecord {
            target: forecast.target.clone(),
            forecast: forecast.clone(),
            calibration: self.calibration.records.clone(),
            event_stream: self.events.events.clone(),
            created_at: self.events.current_time,
        };
        let value = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        // key 含 forecast_count 保证同一时间步多次推演不互相覆盖
        let key = format!(
            "forecast_{}_{}_{}",
            forecast.target, self.events.current_time, self.forecast_count
        );
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        conn.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["forecast", key, value, ts as i64],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 检索最近 N 条推演记录（按 target 过滤可选）。
    pub fn recent_forecasts(target: Option<&str>, limit: usize) -> Vec<ForecastRecord> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = format!("{}/.neotrix/knowledge.db", home);
        let Ok(conn) = rusqlite::Connection::open(&db_path) else {
            return Vec::new();
        };
        let pattern = match target {
            Some(t) => format!("forecast_{}%", t),
            None => "forecast_%".to_string(),
        };
        let Ok(mut stmt) = conn.prepare(
            "SELECT value FROM kv_store WHERE namespace='forecast' AND key LIKE ?1 ORDER BY updated_at DESC LIMIT ?2",
        ) else {
            return Vec::new();
        };
        let Ok(rows) = stmt.query_map(rusqlite::params![pattern, limit as i64], |r| r.get::<_, String>(0)) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for row in rows.flatten() {
            if let Ok(rec) = serde_json::from_str::<ForecastRecord>(&row) {
                out.push(rec);
            }
        }
        out
    }
}

/// CJK 字符判断（中文/日文/韩文 + 全角标点）— 用于 CJK 感知 token 估算。
fn is_cjk(ch: char) -> bool {
    let c = ch as u32;
    (0x4E00..=0x9FFF).contains(&c)   // CJK 统一表意文字
        || (0x3400..=0x4DBF).contains(&c) // CJK 扩展 A
        || (0x3000..=0x303F).contains(&c) // CJK 符号/标点
        || (0xFF00..=0xFFEF).contains(&c) // 全角形式
}

/// 调用预算门 — 限制单次叙事总 LLM 调用次数。
#[derive(Debug, Clone)]
pub struct CallBudgetImpl {
    /// 已花费调用数。
    pub spent: u32,
    /// 总预算上限。
    pub max_calls: u32,
}

impl CallBudgetImpl {
    /// 构造预算门。
    pub fn new(max_calls: u32) -> Self {
        Self { spent: 0, max_calls }
    }

    /// 尝试花费一次调用; 超预算返回 false。
    pub fn try_spend(&mut self) -> bool {
        if self.spent >= self.max_calls {
            return false;
        }
        self.spent += 1;
        true
    }

    /// 是否已耗尽。
    pub fn exhausted(&self) -> bool {
        self.spent >= self.max_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_decay() {
        let mut stream = EventStream::new();
        stream.push(StructuredEvent::new("fed", "cut", "rates", 0.8));
        let initial = stream.aggregate_impact();
        stream.tick(10);
        let decayed = stream.aggregate_impact();
        assert!(decayed < initial, "event should decay over time");
        assert!(decayed >= 0.0);
    }

    #[test]
    fn test_event_aggregate_bounded() {
        let mut stream = EventStream::new();
        for _ in 0..5 {
            stream.push(StructuredEvent::new("a", "b", "c", 1.0));
        }
        let impact = stream.aggregate_impact();
        assert!(impact <= 1.0, "aggregate impact must be clamped to [0,1]");
    }

    #[test]
    fn test_scenario_tree_leaves() {
        let mut tree = ScenarioTree::new();
        let root = tree.add_node("root", "s0", 1.0, 0.8);
        let a = tree.add_node("a", "s1", 0.6, 0.8);
        let b = tree.add_node("b", "s2", 0.4, 0.8);
        tree.add_edge(root, a);
        tree.add_edge(root, b);
        assert_eq!(tree.leaves().len(), 2);
        assert!((tree.leaf_probability_sum() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_leaves() {
        let mut tree = ScenarioTree::new();
        let root = tree.add_node("root", "s0", 1.0, 0.8);
        let a = tree.add_node("a", "s1", 0.7, 0.8);
        let b = tree.add_node("b", "s2", 0.7, 0.8);
        tree.add_edge(root, a);
        tree.add_edge(root, b);
        tree.normalize_leaves();
        assert!((tree.leaf_probability_sum() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_brier_score_perfect() {
        let mut cal = CalibrationTracker::new();
        cal.record(1.0, true);
        cal.record(0.0, false);
        assert!(cal.brier_score() < 1e-9);
    }

    #[test]
    fn test_brier_score_random() {
        let mut cal = CalibrationTracker::new();
        cal.record(0.5, true);
        cal.record(0.5, false);
        assert!((cal.brier_score() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_ece_overconfident() {
        let mut cal = CalibrationTracker::new();
        // 高置信但结果相反 → 过度自信 → ECE 高
        cal.record(0.9, false);
        cal.record(0.9, false);
        let ece = cal.ece(10);
        assert!(ece > 0.5);
    }

    #[test]
    fn test_calibrated_confidence_penalty() {
        let mut cal = CalibrationTracker::new();
        cal.record(0.9, false);
        cal.record(0.9, false);
        let raw = 0.9;
        let calibrated = cal.calibrated_confidence(raw);
        assert!(calibrated < raw, "overconfidence should be penalized");
    }

    #[test]
    fn test_forecast_generates_tree() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.8);
        let forecast = engine.generate_forecast("gold", 1);
        assert_eq!(forecast.tree.leaves().len(), 3, "三情景：牛/熊/震荡");
        assert!(!forecast.confidence_reason.is_empty());
        assert!(!forecast.abstain);
    }

    /// 缺陷回归 1：多事件不应线性饱和打满 — 区分度保留。
    #[test]
    fn test_aggregate_soft_saturation_distinguishes() {
        let mut one = EventStream::new();
        one.push(StructuredEvent::new("a", "b", "c", 0.9));
        let mut five = EventStream::new();
        for _ in 0..5 {
            five.push(StructuredEvent::new("a", "b", "c", 0.9));
        }
        let i1 = one.aggregate_impact();
        let i5 = five.aggregate_impact();
        assert!(i5 > i1, "更多事件应更高");
        assert!(i1 < 0.95, "单事件不应直接打满");
        assert!(i1 > 0.3, "强事件应贡献显著");
        assert!(i5 <= 1.0);
    }

    /// 缺陷回归 2：净方向驱动偏斜 — 利多事件牛分支概率更高。
    #[test]
    fn test_net_direction_bullish_bias() {
        let mut bull = ForecastEngine::new();
        bull.ingest_event("fed", "cut", "rates", 0.9); // 利多
        let f_bull = bull.generate_forecast("gold", 1);
        let bull_prob = f_bull.tree.node(1).map(|n| n.probability).unwrap_or(0.0);

        let mut bear = ForecastEngine::new();
        bear.ingest_signed_event("fed", "hike", "rates", 0.9, -1.0); // 利空
        let f_bear = bear.generate_forecast("gold", 1);
        let bear_bear_prob = f_bear.tree.node(2).map(|n| n.probability).unwrap_or(0.0);

        assert!(bull_prob > 0.5, "利多事件牛概率应 >0.5, got {bull_prob}");
        assert!(bear_bear_prob > 0.5, "利空事件熊概率应 >0.5, got {bear_bear_prob}");
    }

    /// 缺陷回归 3：多空对冲 → 震荡分支概率高。
    #[test]
    fn test_conflicting_events_sideways_dominates() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.9); // 利多
        engine.ingest_signed_event("war", "escalate", "oil", 0.9, -1.0); // 利空
        let f = engine.generate_forecast("gold", 1);
        // 三叶节点：bull=1, bear=2, sideways=3
        let sideways = f.tree.node(3).map(|n| n.probability).unwrap_or(0.0);
        let bull = f.tree.node(1).map(|n| n.probability).unwrap_or(0.0);
        assert!(sideways > bull, "对冲时震荡概率应高于单一牛分支: s={sideways:.2} b={bull:.2}");
    }

    /// 缺陷回归 4：resolve_outcome 记录主导叶子概率（信息量），而非恒 1 的和。
    #[test]
    fn test_resolve_outcome_records_dominant_leaf() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.9);
        let forecast = engine.generate_forecast("gold", 1);
        engine.resolve_outcome(&forecast, true);
        let recorded = engine.calibration.records[0].0;
        assert!(recorded < 1.0, "主导叶子概率应 <1, got {recorded}");
        assert!(recorded > 0.0);
    }

    /// 缺陷回归 5：三情景叶子概率归一化为合法分布（和=1）。
    #[test]
    fn test_three_branch_probabilities_normalize() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.9);
        let f = engine.generate_forecast("gold", 1);
        let sum = f.tree.leaf_probability_sum();
        assert!((sum - 1.0).abs() < 1e-6, "叶子概率和应=1, got {sum}");
        assert_eq!(f.tree.leaves().len(), 3);
    }

    /// 缺陷回归 6：无 LLM 池子时叙事降级为确定性描述，不阻塞推演。
    #[test]
    fn test_narrative_fallback_without_llm() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.9);
        let f = engine.generate_forecast("gold", 1);
        // 未启用 narrator → 所有叶子有确定性叙事
        for leaf in f.tree.leaves() {
            let n = leaf.narrative.as_deref().unwrap_or("");
            assert!(!n.is_empty(), "leaf {} 应有兜底叙事", leaf.name);
            assert!(n.contains("scenario"), "应为确定性描述, got: {n}");
        }
    }

    /// 叙事上下文构建 — 事件方向信息正确编码。
    #[test]
    fn test_narrative_context_includes_events() {
        let mut engine = ForecastEngine::new();
        engine.ingest_signed_event("fed", "cut", "rates", 0.9, 1.0);
        engine.ingest_signed_event("war", "escalate", "oil", 0.8, -1.0);
        let ctx = engine.build_narrative_context("gold", 1, 0.9, 0.2, 0.3);
        assert!(ctx.contains("[bullish] fed cut rates"), "利多事件应编码, got: {ctx}");
        assert!(ctx.contains("[bearish] war escalate oil"), "利空事件应编码");
        assert!(ctx.contains("TARGET: gold"));
    }

    /// 启用 LLM 叙事层后，生成推演不因缺 provider 而 panic（优雅降级）。
    ///
    /// llm7 匿名可用时走真实 LLM 叙事；不可用时降级确定性叙事。两种都合法，
    /// 关键是：不 panic、每个叶子都有非空叙事。
    #[test]
    fn test_llm_narrator_enabled_falls_back_gracefully() {
        let mut engine = ForecastEngine::new().with_llm_narrator(None);
        engine.ingest_event("fed", "cut", "rates", 0.9);
        let f = engine.generate_forecast("gold", 1);
        for leaf in f.tree.leaves() {
            let narr = leaf.narrative.as_deref().unwrap_or("");
            assert!(!narr.trim().is_empty(), "每个叶子都应有叙事（LLM 或确定性）");
        }
    }

    #[test]
    fn test_forecast_abstain_on_low_signal() {
        let mut engine = ForecastEngine::new();
        // 无事件 + 低置信 → 弃权
        let forecast = engine.generate_forecast("gold", 1);
        assert!(forecast.abstain);
    }

    #[test]
    fn test_resolve_outcome_updates_calibration() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.8);
        let forecast = engine.generate_forecast("gold", 1);
        engine.resolve_outcome(&forecast, true);
        assert_eq!(engine.calibration.records.len(), 1);
    }

    #[test]
    fn test_selftest_after_forecast() {
        let mut engine = ForecastEngine::new();
        engine.ingest_event("fed", "cut", "rates", 0.8);
        engine.generate_forecast("gold", 1);
        assert!(engine.self_test().is_ok());
    }

    // ── 候选链构建（多 provider 故障转移）──

    #[test]
    fn test_build_candidates_llm7_first_with_model_fallback() {
        let names = vec![
            "api-airforce/grok-4.1-mini:free".to_string(),
            "llm7/codestral-latest".to_string(),
            "pollinations".to_string(),
        ];
        let c = LlmNarrator::build_candidates(names);
        // llm7 优先，且 codestral 在 gpt-oss 前（非 reasoning 优先）
        assert_eq!(c[0], ("llm7/codestral-latest".to_string(), "codestral-latest".to_string()));
        assert_eq!(c[1], ("llm7/codestral-latest".to_string(), "gpt-oss:20b".to_string()));
        // pollinations 次选
        assert_eq!(c[2], ("pollinations".to_string(), "openai".to_string()));
        // api-airforce 有模型条目时用其 model_id
        assert_eq!(c[3], ("api-airforce/grok-4.1-mini:free".to_string(), "grok-4.1-mini:free".to_string()));
    }

    #[test]
    fn test_build_candidates_bare_llm7_name() {
        // 池子只有裸名 llm7（factory 注册，无 catalog 条目）
        let names = vec!["llm7".to_string(), "ollama".to_string()];
        let c = LlmNarrator::build_candidates(names);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], ("llm7".to_string(), "codestral-latest".to_string()));
        assert_eq!(c[1], ("llm7".to_string(), "gpt-oss:20b".to_string()));
    }

    #[test]
    fn test_build_candidates_empty_pool() {
        let c = LlmNarrator::build_candidates(vec![]);
        assert!(c.is_empty());
    }

    #[test]
    fn test_build_candidates_skips_ollama_only() {
        // 只有 ollama（本地）→ 无候选（不把本地模型当 free 池）
        let c = LlmNarrator::build_candidates(vec!["ollama".to_string()]);
        assert!(c.is_empty());
    }

    // ── Token 消耗优化 (2026-08): CJK 感知估算 + context 截断 + 调用预算门 ──

    #[test]
    fn test_estimate_tokens_cjk_aware() {
        // 中文按字计 1 token, 英文按 4 字符计 1 token (对齐 nt_memory_skill_cost 启发式)
        let cn = LlmNarrator::estimate_tokens("这是中文测试句子");
        assert_eq!(cn, 8); // 8 个中文字符
        let en = LlmNarrator::estimate_tokens("hello world");
        assert_eq!(en, 3); // 11 字符 / 4 = 2.75 → 3
        let mixed = LlmNarrator::estimate_tokens("中文 mixed 测试");
        assert!(mixed > 0);
        let empty = LlmNarrator::estimate_tokens("");
        assert_eq!(empty, 0);
    }

    #[test]
    fn test_truncate_context_respects_budget() {
        // 长 context 应被截断到预算内 (保留开头, 附截断标记)
        let long = "这是很长的中文内容".repeat(200); // 2000+ 字
        let truncated = LlmNarrator::truncate_context(&long, 100);
        assert!(LlmNarrator::estimate_tokens(&truncated) <= 110, "应在预算内: {} tokens", LlmNarrator::estimate_tokens(&truncated));
        assert!(truncated.contains("…(截断)"), "应带截断标记");
        assert!(truncated.starts_with("这是"), "保留开头");

        // 短 context 不应被截断
        let short = "简短内容";
        assert_eq!(LlmNarrator::truncate_context(short, 1000), short);
    }

    #[test]
    fn test_budget_gate_blocks_excessive_calls() {
        // 总预算门: 超过 max_calls 时停止重试 (防止 3候选×3重试=9次全量调用)
        let mut gate = super::CallBudgetImpl::new(5);
        assert!(gate.try_spend());
        for _ in 0..4 {
            assert!(gate.try_spend());
        }
        assert!(!gate.try_spend(), "第 6 次调用应被预算门拦截");
        assert_eq!(gate.spent, 5);
        assert!(gate.exhausted());
    }

    #[test]
    fn test_truncate_context_cjk_budget_aligns_max_tokens() {
        // 对齐外部文献 (SitePoint 2026): 输出受限 → 输入也应受限, 控制每调用成本
        let context = "预测背景: 黄金市场当前处于多头动能, 美联储 9 月降息预期升温, 地缘风险溢价支撑避险需求。".repeat(50);
        let budget = LlmNarrator::DEFAULT_CONTEXT_TOKEN_BUDGET;
        let truncated = LlmNarrator::truncate_context(&context, budget);
        let est = LlmNarrator::estimate_tokens(&truncated);
        assert!(est <= budget + 10, "截断后 {} tokens 应 ≤ 预算 {}+10", est, budget);
    }
}