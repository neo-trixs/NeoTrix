//! Dispatcher policy — 历史/侵略度启发式 + 路由记忆（纯搬移，行为零变更）。

use super::nt_dispatcher_reduce::SubTaskClass;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 单次 dispatch 记录（SDB v0.6 §评分首填格式对齐；log-only）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchLogRecord {
    /// 任务签名（标题哈希，不存原文，防日志膨胀）。
    pub task_sig: u64,
    pub pred_confidence: f64,
    pub tau: f64,
    pub class: SubTaskClass,
    pub kernel_present: bool,
    pub vp1_pass: bool,
    pub vp2_pass: bool,
    /// VP-3（状态钳位 &0x3f）构造性强制，恒 true，留位供审计。
    pub vp3_pass: bool,
    /// 决策分支：kernel_fast | kernel | cot | reasoning | kernel_direct | direct_llm。
    pub decision: &'static str,
}

/// 路由记忆上限（D-6；满弹旧，防无界增长）。
pub const HISTORY_CAP: usize = 64;
/// 置信阈值下限（V-3：只许调严，调松需 ADR＋安全签字）。
pub const CONFIDENCE_FLOOR: f64 = 0.65;

/// τ 钳位 [0.65, 1.0]；非数回落下限（禁 unwrap/panic 路径）。
pub(crate) fn clamp_confidence(v: f64) -> f64 {
    if !v.is_finite() {
        return CONFIDENCE_FLOOR;
    }
    v.clamp(CONFIDENCE_FLOOR, 1.0)
}

/// 任务签名（确定性哈希）。
pub(crate) fn task_sig(task: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    task.hash(&mut h);
    h.finish()
}

/// 路由记忆追加（有界）。
pub(crate) fn push_history(history: &mut VecDeque<DispatchLogRecord>, rec: DispatchLogRecord) {
    if history.len() >= HISTORY_CAP {
        history.pop_front();
    }
    history.push_back(rec);
}

/// 路由记忆检索（纯函数）：海明邻近（≤8 bit）历史 confidence 均值偏低则建议保守。
/// 返回值仅供日志（log-only）；调制接线 Phase 2b，调用方暂只有日志位（防死代码警告）。
pub(crate) fn suggest_aggression(
    history: &VecDeque<DispatchLogRecord>,
    sig: u64,
    base: f64,
) -> f64 {
    let mut sum = 0.0;
    let mut n = 0u32;
    for r in history
        .iter()
        .filter(|r| (r.task_sig ^ sig).count_ones() <= 8)
    {
        sum += r.pred_confidence;
        n += 1;
    }
    if n == 0 {
        return base;
    }
    let avg = sum / f64::from(n);
    if avg < 0.5 {
        (base - 0.1).max(0.1)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_core_task_dispatcher::nt_dispatcher_reduce::SubTaskClass;

    #[test]
    fn test_confidence_clamp_table() {
        // V-3：只许调严，下限锁死 0.65；非数回落。
        assert_eq!(clamp_confidence(0.0), 0.65);
        assert_eq!(clamp_confidence(0.65), 0.65);
        assert_eq!(clamp_confidence(0.9), 0.9);
        assert_eq!(clamp_confidence(2.0), 1.0);
        assert_eq!(clamp_confidence(f64::NAN), 0.65);
        assert_eq!(clamp_confidence(f64::INFINITY), 0.65);
    }

    fn log_rec(sig: u64, conf: f64) -> DispatchLogRecord {
        DispatchLogRecord {
            task_sig: sig,
            pred_confidence: conf,
            tau: 0.65,
            class: SubTaskClass::Deterministic,
            kernel_present: true,
            vp1_pass: true,
            vp2_pass: true,
            vp3_pass: true,
            decision: "kernel_fast",
        }
    }

    #[test]
    fn test_task_sig_deterministic_and_distinct() {
        assert_eq!(task_sig("abc"), task_sig("abc"));
        assert_ne!(task_sig("abc"), task_sig("abd"));
    }

    #[test]
    fn test_push_history_cap_eviction() {
        let mut h = VecDeque::new();
        for i in 0..(HISTORY_CAP + 5) as u64 {
            push_history(&mut h, log_rec(i, 0.9));
        }
        assert_eq!(h.len(), HISTORY_CAP);
        assert_eq!(h.front().map(|r| r.task_sig), Some(5));
    }

    #[test]
    fn test_suggest_aggression_neighbors() {
        let empty = VecDeque::new();
        assert_eq!(suggest_aggression(&empty, 0, 0.5), 0.5);
        let mut low = VecDeque::new();
        push_history(&mut low, log_rec(0b1010, 0.1));
        assert_eq!(suggest_aggression(&low, 0b1011, 0.5), 0.4);
        let mut high = VecDeque::new();
        push_history(&mut high, log_rec(0b1010, 0.9));
        assert_eq!(suggest_aggression(&high, 0b1011, 0.5), 0.5);
    }
}
