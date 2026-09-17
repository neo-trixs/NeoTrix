//! S 门控的自编辑闭环 — 灵境引擎 L4「自指闭环」转译。
//!
//! # D≥4 回环证据链
//!
//! ```text
//! 意识循环 (nt_mind_background_loop::handlers_consciousness)
//!   └─ 写入 brain._last_consciousness_quality (Φ/IIT 派生信号, 0.0–1.0)
//!        └─ SEAL 自编辑应用点前 ConstitutionGate 裁决:
//!             · 尚无意识信号 (_consciousness_critique_count == 0) → None → 保守放行并计数
//!             · Some(q) < SELF_EDIT_MIN_CONSCIOUSNESS → 否决, 跳过本次应用 (行为改变)
//!                  └─ 被放行的编辑修改系统自身能力 (ReasoningBrain.apply_micro_edits)
//!                       └─ 下一轮意识质量由被修改后的系统重新产生
//!                            ⇒ 度量信号反过来控制"计算该度量的系统本身"的修改 ⇒ D≥4 自指闭环
//! ```
//!
//! 应用点接线 (R-P79 同 session 生产路径):
//! 1. [`BoundedEditStage`](super::super::neotrix::l8_autonomic_impl) — SEAL 双管线
//!    (seal_pipeline / kernel_iterate_pipeline) 中自编辑队列的唯一处理 stage,
//!    否决即清空待应用队列, 编辑不再流向下游采纳/归档。
//! 2. `SelfIteratingBrain::code_review_iterate` — 唯一直接调用
//!    `ReasoningBrain::apply_micro_edits` 的生产入口, 否决即跳过应用。
//!
//! 计数器暴露 getter (`allowed_count`/`denied_count`/`no_signal_count`),
//! 供 SelfTest 断言门控真实参与行为决策 (T3 行为接地)。
//!
//! # F2 校准数据采集 (W2 配对研究地基)
//!
//! [`ConstitutionGate::judge`] 每次裁决同步 push 一条 [`DecisionRecord`] 进
//! 环形缓冲 (cap [`DECISION_RING_CAP`], append-only, 溢出丢最旧)。缓冲由
//! NT-MIND 后台意识 tick (`handlers_consciousness::handle_awareness`) 定期
//! `drain_decisions()` 排空, 经 KB 场账本落盘:
//!
//! - namespace `gating_decisions`, key `gd_{ts_nanos}`, writer `constitution_gate`;
//!   预期消费者 = 未来校准脚本: 从 kv_store SELECT 该 namespace 即得
//!   {ts_nanos, quality(null=无信号), allowed} 裁决序列, 用于验证 G5 门控阈值
//!   (SELF_EDIT_MIN_CONSCIOUSNESS=0.5) 的真实区分度。
#![forbid(unsafe_code)]

use std::collections::VecDeque;

/// 自编辑应用的最低意识质量阈值。
/// quality < 阈值 → 否决 (严格小于; 等于阈值视为可信, 放行)。
pub const SELF_EDIT_MIN_CONSCIOUSNESS: f64 = 0.5;

/// 裁决环形缓冲容量 (F2): 最近 64 条裁决待排空, 溢出丢最旧。
/// 上限选取: 覆盖两次意识 tick 间的最大裁决量, 同时防无界内存增长。
pub const DECISION_RING_CAP: usize = 64;

/// 单条门控裁决记录 (append-only, 由 [`ConstitutionGate::judge`] 产出)。
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionRecord {
    /// 裁决时刻 (UNIX 纪元纳秒); 兼作 KB 落盘键 `gd_{ts_nanos}` 保证唯一。
    pub ts_nanos: u64,
    /// 裁决时的意识质量信号; `None` 表示尚无信号 (JSON 序列化为 null,
    /// 与数值区分, 校准脚本按 null 过滤无信号窗口)。
    pub quality: Option<f64>,
    /// 本次自编辑应用是否放行。
    pub allowed: bool,
}

impl DecisionRecord {
    /// 单行 JSON 序列化 (KB field_stage value; 校准脚本 serde_json 解析即得配对集)。
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "ts_nanos": self.ts_nanos,
            "quality": self.quality,
            "allowed": self.allowed,
        })
        .to_string()
    }
}

/// 宪法门控: 意识质量信号门控 SEAL 对自身的修改。
///
/// 单实例挂在 `SelfIteratingBrain::_constitution_gate` 上, 跨迭代累计裁决计数;
/// 纯函数判定逻辑见 [`ConstitutionGate::should_allow_self_edit`]。
#[derive(Debug, Clone)]
pub struct ConstitutionGate {
    threshold: f64,
    allowed_count: u64,
    denied_count: u64,
    no_signal_count: u64,
    /// F2 校准: 待排空裁决环形缓冲 (append-only, cap DECISION_RING_CAP)。
    decision_ring: VecDeque<DecisionRecord>,
}

impl Default for ConstitutionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstitutionGate {
    pub fn new() -> Self {
        Self {
            threshold: SELF_EDIT_MIN_CONSCIOUSNESS,
            allowed_count: 0,
            denied_count: 0,
            no_signal_count: 0,
            decision_ring: VecDeque::with_capacity(DECISION_RING_CAP),
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold,
            ..Self::new()
        }
    }

    /// 纯函数判定: 意识质量是否允许本轮自编辑应用。
    /// - `None` (尚无意识信号) → 保守放行 (不因信号缺失阻塞演化);
    /// - `Some(q)`, q < threshold → 否决;
    /// - `Some(q)`, q >= threshold → 放行。
    pub fn should_allow_self_edit(quality: Option<f64>, threshold: f64) -> bool {
        match quality {
            None => true,
            Some(q) => q >= threshold,
        }
    }

    /// 带计数的裁决入口 (应用点调用)。返回是否放行。
    /// 每次裁决同步 push 一条 [`DecisionRecord`] 进环形缓冲 (F2 校准采集)。
    pub fn judge(&mut self, quality: Option<f64>) -> bool {
        if quality.is_none() {
            self.no_signal_count += 1;
        }
        let allowed = Self::should_allow_self_edit(quality, self.threshold);
        if allowed {
            self.allowed_count += 1;
        } else {
            self.denied_count += 1;
        }
        let ts_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        if self.decision_ring.len() >= DECISION_RING_CAP {
            self.decision_ring.pop_front();
        }
        self.decision_ring.push_back(DecisionRecord {
            ts_nanos,
            quality,
            allowed,
        });
        allowed
    }

    /// 排空并返回全部待持久化裁决 (F2): 清空语义 — 调用后缓冲为空,
    /// 由意识 tick 逐条 field_stage 落盘 KB `gating_decisions` namespace。
    pub fn drain_decisions(&mut self) -> Vec<DecisionRecord> {
        std::mem::take(&mut self.decision_ring).into()
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    pub fn allowed_count(&self) -> u64 {
        self.allowed_count
    }

    pub fn denied_count(&self) -> u64 {
        self.denied_count
    }

    /// 无信号保守放行的次数 (与 allowed_count 重叠计入)。
    pub fn no_signal_count(&self) -> u64 {
        self.no_signal_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_quality_allows_conservatively() {
        // 尚无意识信号 → 保守放行, 不阻塞演化
        assert!(ConstitutionGate::should_allow_self_edit(None, SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_low_quality_denied() {
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.2), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.49), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.0), SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_threshold_boundary_allows() {
        // 否决条件是严格小于: 等于阈值放行
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.5), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.51), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(ConstitutionGate::should_allow_self_edit(Some(1.0), SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_custom_threshold_respected() {
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.7), 0.8).eq(&false));
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.9), 0.8));
    }

    #[test]
    fn test_judge_counts_all_three_outcomes() {
        let mut gate = ConstitutionGate::new();
        assert!(gate.judge(None), "无信号放行");
        assert!(gate.judge(Some(0.9)), "高质量放行");
        assert!(!gate.judge(Some(0.1)), "低质量否决");
        assert_eq!(gate.allowed_count(), 2);
        assert_eq!(gate.denied_count(), 1);
        assert_eq!(gate.no_signal_count(), 1);
    }

    #[test]
    fn test_decision_ring_cap_64_drops_oldest() {
        // F2: 环形缓冲 cap=64, 溢出丢最旧 — 70 次裁决只留最近 64 条
        let mut gate = ConstitutionGate::with_threshold(0.5);
        for i in 0..70 {
            gate.judge(Some(if i % 2 == 0 { 0.9 } else { 0.1 }));
        }
        let drained = gate.drain_decisions();
        assert_eq!(drained.len(), DECISION_RING_CAP);
        // 最旧的 6 条 (i=0..6) 已被挤出; 幸存首条对应 i=6 (偶数 → 放行)
        assert!(drained[0].allowed, "首条应为 i=6 的放行裁决");
        // 末条对应 i=69 (奇数 → 否决)
        assert!(!drained[63].allowed, "末条应为 i=69 的否决裁决");
    }

    #[test]
    fn test_drain_clears_buffer() {
        // F2: drain 清空语义 — 排空后再取为空, 计数不受影响
        let mut gate = ConstitutionGate::new();
        gate.judge(None);
        gate.judge(Some(0.8));
        let first = gate.drain_decisions();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].quality, None);
        assert_eq!(first[1].quality, Some(0.8));
        assert!(first[0].allowed && first[1].allowed);
        assert!(gate.drain_decisions().is_empty(), "二次排空应为空");
        // 计数器独立于缓冲, 不因排空重置
        assert_eq!(gate.allowed_count(), 2);
        assert_eq!(gate.no_signal_count(), 1);
    }

    #[test]
    fn test_decision_record_json_parseable() {
        // F2: KB 落盘 value 契约 — 单行 JSON, 校准脚本 serde_json 可解析;
        // None quality 序列化为 null (与数值区分)
        let with_signal = DecisionRecord {
            ts_nanos: 1_700_000_000_000_000_000,
            quality: Some(0.42),
            allowed: false,
        };
        let v: serde_json::Value = serde_json::from_str(&with_signal.to_json()).unwrap();
        assert_eq!(v["ts_nanos"], serde_json::json!(1_700_000_000_000_000_000u64));
        assert_eq!(v["quality"], serde_json::json!(0.42));
        assert_eq!(v["allowed"], serde_json::json!(false));

        let no_signal = DecisionRecord {
            ts_nanos: 1,
            quality: None,
            allowed: true,
        };
        let v: serde_json::Value = serde_json::from_str(&no_signal.to_json()).unwrap();
        assert!(v["quality"].is_null(), "无信号应序列化为 null");
        assert_eq!(v["allowed"], serde_json::json!(true));
    }
}
