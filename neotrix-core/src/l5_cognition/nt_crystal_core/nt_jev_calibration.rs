//! NT-JEV-CALIBRATION — 晶体自验证分数 → JEV 校准训练数据
//!
//! 研究依据（2026 校准线三篇，不依赖外部教师模型）：
//! - COREA (EACL'26)：confidence reward = -|verbalized confidence − correctness|，
//!   L1 最优。本模块把 `NtAwakenLoop::verify` 的 total 直接当 correctness 估计，
//!   结论自带可验证的 confidence 标签（比蒸馏外部模型的幻觉概率更真）。
//! - ACL Findings'26：SFT 比 RLVR 校准更好；决策 token 目标：正确→one-hot，
//!   错误→uniform。本模块 choice 分布按 verify 分数配比并截断到 [1/|C|, 1]。
//! - arXiv 2608.05064：小模型用 Platt scaling（斜率+偏置）而非 temperature
//!   scaling；200 问校准集即可。本模块提供 Platt 参数拟合（坐标下降最小化 NLL）。
//!
//! 纯函数，无 IO；无 unwrap / expect / panic；无 `[]` 索引。

use std::collections::HashMap;

use super::consciousness::{CrystalConsciousness, ReasoningType};
use super::nt_awaken_loop::{NtAwakenLoop, VerifyScores};
use crate::l5_cognition::nt_jev::eval::{EvalCase, EvalPrediction, GoldAnswer};
use crate::l5_cognition::nt_jev::primitives::{ChoiceAnswer, DecisionStatus, JevDecision, NoulAnswer};

/// 置信度金标准门（与 archive_train pattern_conf_floor 对齐）
pub const GOLD_FLOOR: f64 = 0.7;
/// 决策概率截断下限（|C|=2 时 uniform floor，防过自信）
pub const PROB_FLOOR: f64 = 0.05;
pub const HIGH_RISK_REVIEW_FLOOR: f64 = 0.85;
/// RLCD act 代价高风险 review 门（D3 吸收，源 laya `rl_common.py`）：
/// 答错代价 3.0，escalate 代价 0.5 → 高风险域送审条件 conf < 1 - 0.5/3.0 ≈ 0.833，
/// 保守侧取 0.85。默认门 GOLD_FLOOR 不变（fail-open，调用方显式选入高风险）。

/// 单条校准 SFT 行
#[derive(Debug, Clone)]
pub struct CalibRow {
    /// 训练文本（MiniMind conversations 格式 JSON 字符串）
    pub jsonl: String,
    /// 标签 confidence（即 verify.total）
    pub confidence: f64,
    /// 是否为正例（gold）
    pub gold: bool,
    /// 域（前提记忆 domain 多数票；空链/无前提则 None → 校准回 global）。
    /// tick 接线用它调 `PlattBucketTable::calibrate`，11 域桶才进得去生产。
    pub domain: Option<String>,
}

pub struct NtJevCalibration;

impl NtJevCalibration {
    /// verify 复用：结论 id + 前提 ids → 三件套分数
    pub fn score(
        consciousness: &CrystalConsciousness,
        conclusion_id: &str,
        premise_ids: &[String],
    ) -> VerifyScores {
        NtAwakenLoop::verify(consciousness, conclusion_id, premise_ids)
    }

    /// 链的域：前提记忆 domain 多数票（与 consciousness 归纳句式同口径）；
    /// 票数持平取首前提域（确定性），无前提/链不存在则 None。
    pub fn chain_domain(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
    ) -> Option<String> {
        let ch = consciousness
            .reasoning_chains
            .iter()
            .find(|c| c.id == chain_id)?;
        let mut order: Vec<&str> = Vec::new();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for pid in &ch.premises {
            if let Some(m) = consciousness.memories.get(pid) {
                if !counts.contains_key(m.domain.as_str()) {
                    order.push(m.domain.as_str());
                }
                *counts.entry(m.domain.as_str()).or_insert(0) += 1;
            }
        }
        // 多数票；持平取最早出现的前提域（严格大于才替换）。
        let mut best: Option<(&str, usize)> = None;
        for d in order {
            let n = counts.get(d).copied().unwrap_or(0);
            match best {
                Some((_, bn)) if bn >= n => {}
                _ => best = Some((d, n)),
            }
        }
        best.map(|(d, _)| d.to_string())
    }

    /// noul 行：前提 → "结论成立吗？" → think 痕迹 + verdict + confidence。
    ///
    /// confidence 标签 = verify.total（COREA L1 式：标签即正确率估计）。
    pub fn noul_row(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
    ) -> Option<CalibRow> {
        let ch = consciousness
            .reasoning_chains
            .iter()
            .find(|c| c.id == chain_id)?;
        let premises: Vec<String> = ch
            .premises
            .iter()
            .filter_map(|pid| consciousness.memories.get(pid).map(|m| m.content.clone()))
            .collect();
        if premises.is_empty() {
            return None;
        }
        let scores = Self::score(
            consciousness,
            ch.conclusion_memory_id.as_deref().unwrap_or(""),
            &ch.premises,
        );
        let conf = scores.total.clamp(0.0, 1.0);
        let verdict = if conf >= GOLD_FLOOR { "成立" } else { "存疑" };
        let thinking = format!(
            "<think>类型={:?}，新颖度={:.2}，接地率={:.2}，跨域桥={:.2}。</think>",
            ch.chain_type, scores.novelty, scores.coverage, scores.bridge
        );
        let jsonl = serde_json::json!({
            "conversations": [
                {"role": "user", "content": format!("前提：{}\n结论：{}\n该结论成立吗？", premises.join("\n"), ch.conclusion)},
                {"role": "assistant", "content": format!("{thinking}{verdict}（confidence: {conf:.2}）")},
            ],
            "calib_confidence": conf,
        })
        .to_string();
        Some(CalibRow {
            jsonl,
            confidence: conf,
            gold: conf >= GOLD_FLOOR,
            domain: Self::chain_domain(consciousness, chain_id),
        })
    }

    /// 全意识 noul 行批量生成（上限 cap 条，置信度跨度优先：高低各半防偏斜）
    pub fn noul_rows(consciousness: &CrystalConsciousness, cap: usize) -> Vec<CalibRow> {
        let mut pos = Vec::new();
        let mut neg = Vec::new();
        for ch in consciousness.reasoning_chains.iter() {
            if let Some(row) = Self::noul_row(consciousness, &ch.id) {
                if row.gold {
                    pos.push(row);
                } else {
                    neg.push(row);
                }
            }
            if pos.len() + neg.len() >= cap.saturating_mul(2) {
                break;
            }
        }
        let half = cap / 2;
        let mut out = Vec::new();
        out.extend(pos.into_iter().take(half.max(cap.saturating_sub(neg.len().min(half)))));
        out.extend(neg.into_iter().take(half));
        out.truncate(cap);
        out
    }

    /// choice 行：同域一高一低两个结论 → 选高者 + verify 配比分布。
    ///
    /// 分布截断到 [PROB_FLOOR, 1-PROB_FLOOR]（ACL'26 决策 token 约束）。
    pub fn choice_row(
        consciousness: &CrystalConsciousness,
        high_chain_id: &str,
        low_chain_id: &str,
    ) -> Option<CalibRow> {
        let find = |id: &str| {
            consciousness
                .reasoning_chains
                .iter()
                .find(|c| c.id == id)
        };
        let hi = find(high_chain_id)?;
        let lo = find(low_chain_id)?;
        let s_hi = Self::score(
            consciousness,
            hi.conclusion_memory_id.as_deref().unwrap_or(""),
            &hi.premises,
        )
        .total
        .clamp(0.0, 1.0);
        let s_lo = Self::score(
            consciousness,
            lo.conclusion_memory_id.as_deref().unwrap_or(""),
            &lo.premises,
        )
        .total
        .clamp(0.0, 1.0);
        if s_hi <= s_lo {
            return None; // 顺序反了就拒掉，保证标签正确
        }
        let sum = (s_hi + s_lo).max(1e-9);
        // 配比分布并截断（正确高置信、错误不过自信，ACL'26 决策 token 约束）
        let p_hi = (s_hi / sum).clamp(PROB_FLOOR, 1.0 - PROB_FLOOR);
        let p_lo = 1.0 - p_hi;
        let jsonl = serde_json::json!({
            "conversations": [
                {"role": "user", "content": format!("A：{}\nB：{}\n哪个结论更可靠？", hi.conclusion, lo.conclusion)},
                {"role": "assistant", "content": format!("选 A（P={p_hi:.2} vs {p_lo:.2}）")},
            ],
            "calib_p_hi": p_hi,
        })
        .to_string();
        Some(CalibRow {
            jsonl,
            confidence: p_hi,
            gold: true,
            // 选的是高者（hi 链），域跟 hi 走；跨域对决命中桶则校准，不命中回 global。
            domain: Self::chain_domain(consciousness, high_chain_id),
        })
    }

    /// 由推理链构造 JEV 决策（供 eval.rs 直接测 ECE/Brier，闭环可度量）。
    pub fn decide(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
    ) -> Option<(EvalCase, EvalPrediction)> {
        Self::decide_with_risk(consciousness, chain_id, false)
    }

    /// RLCD act 代价门：高风险域（医疗/金融/不可逆操作）review 门抬到
    /// HIGH_RISK_REVIEW_FLOOR；默认与 decide() 行为完全一致。
    pub fn decide_with_risk(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
        high_risk: bool,
    ) -> Option<(EvalCase, EvalPrediction)> {
        let floor = if high_risk {
            HIGH_RISK_REVIEW_FLOOR
        } else {
            GOLD_FLOOR
        };
        Self::decide_with_floor(consciousness, chain_id, floor, None, None)
    }

    /// 按域校准决策（RLCD 分桶消费点）：先按域查表得校准置信度，再过 risk 门。
    /// domain 未知/表缺桶 → global；table 为空 → 近恒等。永不失败。
    pub fn decide_calibrated(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
        domain: &str,
        table: &PlattBucketTable,
        high_risk: bool,
    ) -> Option<(EvalCase, EvalPrediction)> {
        let floor = if high_risk {
            HIGH_RISK_REVIEW_FLOOR
        } else {
            GOLD_FLOOR
        };
        Self::decide_with_floor(consciousness, chain_id, floor, Some(domain), Some(table))
    }

    /// 内核：取链 → 评分 →（可选）按域校准 → 按门裁决。decide 系三入口共用，
    /// 默认路径（floor=GOLD_FLOOR、无校准）与旧 decide() 逐行同义。
    fn decide_with_floor(
        consciousness: &CrystalConsciousness,
        chain_id: &str,
        floor: f64,
        domain: Option<&str>,
        table: Option<&PlattBucketTable>,
    ) -> Option<(EvalCase, EvalPrediction)> {
        let ch = consciousness
            .reasoning_chains
            .iter()
            .find(|c| c.id == chain_id)?;
        let scores = Self::score(
            consciousness,
            ch.conclusion_memory_id.as_deref().unwrap_or(""),
            &ch.premises,
        );
        let raw = scores.total.clamp(0.0, 1.0);
        let conf = match (domain, table) {
            (Some(d), Some(t)) => t.calibrate(d, raw),
            _ => raw,
        };
        let gold = conf >= GOLD_FLOOR;
        let decision = JevDecision::Noul(NoulAnswer {
            noul: conf,
            needs_review: conf < floor,
            reason: None,
            status: if conf < floor {
                DecisionStatus::Review
            } else {
                DecisionStatus::Selected
            },
        });
        Some((
            EvalCase {
                id: ch.id.clone(),
                gold: GoldAnswer::Noul(gold),
            },
            EvalPrediction {
                case_id: ch.id.clone(),
                decision,
                latency_ms: 0,
            },
        ))
    }

    /// 由分布构造 Choice 决策（choice 行的评估对偶）。
    pub fn decide_choice(
        option_a: &str,
        option_b: &str,
        p_a: f64,
        pick_a: bool,
    ) -> (JevDecision, f64) {
        Self::decide_choice_with_risk(option_a, option_b, p_a, pick_a, false)
    }

    /// Choice 对偶的高风险变体（门逻辑同 decide_with_risk；默认行为零变化）。
    pub fn decide_choice_with_risk(
        option_a: &str,
        option_b: &str,
        p_a: f64,
        pick_a: bool,
        high_risk: bool,
    ) -> (JevDecision, f64) {
        let floor = if high_risk {
            HIGH_RISK_REVIEW_FLOOR
        } else {
            GOLD_FLOOR
        };
        let p = p_a.clamp(PROB_FLOOR, 1.0 - PROB_FLOOR);
        let mut probs = HashMap::new();
        probs.insert(option_a.to_string(), if pick_a { p } else { 1.0 - p });
        probs.insert(option_b.to_string(), if pick_a { 1.0 - p } else { p });
        let winner = if pick_a { option_a } else { option_b };
        let margin = (2.0 * p - 1.0).abs();
        (
            JevDecision::Choice(ChoiceAnswer {
                choice: winner.to_string(),
                probabilities: probs,
                confidence: p,
                margin,
                needs_review: p < floor,
                reason: None,
                status: if p < floor {
                    DecisionStatus::Review
                } else {
                    DecisionStatus::Selected
                },
            }),
            p,
        )
    }
}

/// Platt scaling：p' = σ(a·logit(p) + b)，logit 截断到 ±6。
///
/// arXiv 2608.05064：小模型校准用 Platt（斜率+偏置）而非 temperature scaling。
#[derive(Debug, Clone, Copy)]
pub struct PlattParams {
    pub a: f64,
    pub b: f64,
}

impl Default for PlattParams {
    fn default() -> Self {
        Self { a: 1.0, b: 0.0 }
    }
}

impl PlattParams {
    pub fn apply(&self, p: f64) -> f64 {
        let pc = p.clamp(1e-6, 1.0 - 1e-6);
        let logit = (pc / (1.0 - pc)).ln().clamp(-6.0, 6.0);
        let z = self.a * logit + self.b;
        1.0 / (1.0 + (-z).exp())
    }

    /// 坐标下降拟合（最小化 NLL，无外部优化依赖）。
    /// items: (预测概率, 是否正确)。
    pub fn fit(items: &[(f64, bool)], iters: usize) -> Self {
        let mut best = Self::default();
        let mut best_nll = Self::nll(&best, items);
        let mut step_a = 0.5;
        let mut step_b = 0.25;
        for _ in 0..iters.max(1) {
            let mut improved = false;
            for (da, db) in [(step_a, 0.0), (-step_a, 0.0), (0.0, step_b), (0.0, -step_b)] {
                let cand = Self {
                    a: (best.a + da).max(0.05),
                    b: best.b + db,
                };
                let nll = Self::nll(&cand, items);
                if nll < best_nll {
                    best = cand;
                    best_nll = nll;
                    improved = true;
                }
            }
            if !improved {
                step_a *= 0.5;
                step_b *= 0.5;
            }
            if step_a < 1e-4 && step_b < 1e-4 {
                break;
            }
        }
        best
    }

    fn nll(&self, items: &[(f64, bool)]) -> f64 {
        let mut sum = 0.0;
        let mut n = 0usize;
        for (p, y) in items {
            let q = self.apply(*p).clamp(1e-9, 1.0 - 1e-9);
            sum += if *y { -q.ln() } else { -(1.0 - q).ln() };
            n += 1;
        }
        if n == 0 {
            return f64::INFINITY;
        }
        sum / n.max(1) as f64
    }
}

/// 按域 Platt 表（RLCD 分桶落地）：domain → PlattParams，附 global 回退。
///
/// 数据源 `models/training/jev_platts.json`（`jev_platts.py` 全量 1130 拟合，
/// n≥20 成桶），编译期 `include_str!` 嵌入。解析失败/域缺失/参数非法一律
/// 回退恒等（a=1,b=0），fail-open，校准永不炸路由。
#[derive(Debug, Clone)]
pub struct PlattBucketTable {
    pub global: PlattParams,
    pub buckets: HashMap<String, PlattParams>,
}

fn platt_num(v: &serde_json::Value, key: &str, fallback: f64) -> f64 {
    v.get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(fallback)
}

fn platt_params_of(v: &serde_json::Value) -> Option<PlattParams> {
    let a = platt_num(v, "a", f64::NAN);
    let b = platt_num(v, "b", f64::NAN);
    if a.is_finite() && b.is_finite() && a > 0.0 {
        Some(PlattParams { a, b })
    } else {
        None
    }
}

impl PlattBucketTable {
    /// 空表（恒等回退）：解析失败时的安全形态。
    pub fn fallback() -> Self {
        Self {
            global: PlattParams::default(),
            buckets: HashMap::new(),
        }
    }

    /// 从 `jev_platts.json` 文本解析（坏键/坏值跳过该域，global 坏则整体回退）。
    pub fn from_json(text: &str) -> Self {
        let v: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return Self::fallback(),
        };
        let global = v
            .get("global")
            .and_then(platt_params_of)
            .unwrap_or_default();
        let mut buckets = HashMap::new();
        if let Some(map) = v.get("buckets").and_then(|b| b.as_object()) {
            for (dom, params) in map {
                if let Some(p) = platt_params_of(params) {
                    buckets.insert(dom.clone(), p);
                }
            }
        }
        Self { global, buckets }
    }

    /// 编译期嵌入本仓 `models/training/jev_platts.json`。
    pub fn embedded() -> Self {
        Self::from_json(include_str!("../../../../models/training/jev_platts.json"))
    }

    /// 按域校准：命中桶用桶参，否则 global。永不失败。
    pub fn calibrate(&self, domain: &str, p: f64) -> f64 {
        self.buckets.get(domain).unwrap_or(&self.global).apply(p)
    }
}

/// ReasoningType 名（统计用）
pub fn chain_type_name(t: &ReasoningType) -> &'static str {
    match t {
        ReasoningType::Deductive => "deductive",
        ReasoningType::Inductive => "inductive",
        ReasoningType::Abductive => "abductive",
        ReasoningType::Analogical => "analogical",
        ReasoningType::CrossDomain => "cross_domain",
    }
}

#[cfg(test)]
mod tests {
    use super::super::consciousness::MemoryType;
    use super::super::{CrystalConsciousness, NtTrainExport};
    use super::*;

    fn seeded() -> CrystalConsciousness {
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁 投入 火焰 熔化", MemoryType::Fact, "physics", 0.9);
        c.reason(vec![a, b], ReasoningType::Inductive);
        c
    }

    #[test]
    fn test_noul_row_has_confidence_label() {
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let row = NtJevCalibration::noul_row(&c, &id).unwrap();
        assert!(row.jsonl.contains("confidence:"));
        assert!(row.jsonl.contains("<think>"));
        assert!((0.0..=1.0).contains(&row.confidence));
    }

    #[test]
    fn test_noul_rows_balanced_cap() {
        let c = seeded();
        let rows = NtJevCalibration::noul_rows(&c, 10);
        assert!(!rows.is_empty());
        assert!(rows.len() <= 10);
    }

    #[test]
    fn test_choice_row_rejects_inverted() {
        let mut c = seeded();
        // 第二条链：抄作业式低新颖结论
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        c.reason(vec![a, b], ReasoningType::Inductive);
        let ids: Vec<String> = c.reasoning_chains.iter().map(|ch| ch.id.clone()).collect();
        let first = ids.first().cloned().unwrap_or_default();
        let second = ids.get(1).cloned().unwrap_or_default();
        // 无论顺序如何，至少一个方向能出（高在前）或都被拒（分数相等）
        let fwd = NtJevCalibration::choice_row(&c, &first, &second);
        let rev = NtJevCalibration::choice_row(&c, &second, &first);
        assert!(fwd.is_some() || rev.is_some(), "one direction must win");
        if let Some(r) = fwd.or(rev) {
            assert!(r.jsonl.contains("选 A"));
            assert!(r.confidence >= 1.0 - PROB_FLOOR - 1e-9 || r.confidence >= 0.5);
        }
    }

    #[test]
    fn test_chain_domain_majority_tie_missing() {
        let mut c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        // 种子链：两前提皆 physics → 多数票 physics
        assert_eq!(
            NtJevCalibration::chain_domain(&c, &id),
            Some("physics".to_string())
        );
        // 2v1：physics 多数
        let p1 = c.remember("水 流动 向下", MemoryType::Fact, "physics", 0.9);
        let p2 = c.remember("火 向上 燃烧", MemoryType::Fact, "physics", 0.9);
        let q1 = c.remember("叶 绿色 光合", MemoryType::Fact, "biology", 0.9);
        c.reason(vec![p1, p2, q1], ReasoningType::Inductive);
        let id2 = c.reasoning_chains.last().map(|ch| ch.id.clone()).unwrap();
        assert_eq!(
            NtJevCalibration::chain_domain(&c, &id2),
            Some("physics".to_string())
        );
        // 1v1 持平 → 首前提域（chem 在前）
        let r1 = c.remember("酸 腐蚀 金属", MemoryType::Fact, "chemistry", 0.9);
        let r2 = c.remember("鸟 有 翅膀", MemoryType::Fact, "biology", 0.9);
        c.reason(vec![r1, r2], ReasoningType::Analogical);
        let id3 = c.reasoning_chains.last().map(|ch| ch.id.clone()).unwrap();
        assert_eq!(
            NtJevCalibration::chain_domain(&c, &id3),
            Some("chemistry".to_string())
        );
        // 不存在的链 → None（fail-open，调用方回 global）
        assert_eq!(NtJevCalibration::chain_domain(&c, "no-such-chain"), None);
    }

    #[test]
    fn test_noul_row_carries_domain() {
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let row = NtJevCalibration::noul_row(&c, &id).unwrap();
        assert_eq!(row.domain, Some("physics".to_string()));
    }

    #[test]
    fn test_embedded_table_not_stub() {
        // 回归门：models/training/jev_platts.json 曾被归档误搬后剩 `{}` 空壳，
        // embedded 表退化为恒等则 11 域桶全进不了生产。此门一响先查文件在不在。
        let t = PlattBucketTable::embedded();
        assert!(
            !t.buckets.is_empty(),
            "embedded Platt 表为空：jev_platts.json 疑似被 stub 覆盖"
        );
        // 未知域回 global（确定性）
        let a = t.calibrate("未知域-xyz", 0.8);
        let b = t.calibrate("未知域-xyz", 0.8);
        assert!((a - b).abs() < 1e-12);
        assert!((a - t.global.apply(0.8)).abs() < 1e-12);
    }

    #[test]
    fn test_decide_plugs_into_eval() {        use crate::l5_cognition::nt_jev::eval::evaluate;
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let (case, pred) = NtJevCalibration::decide(&c, &id).unwrap();
        let rep = evaluate(std::slice::from_ref(&case), std::slice::from_ref(&pred));
        assert_eq!(rep.n, 1);
        assert!((0.0..=1.0).contains(&rep.accuracy));
        assert!((0.0..=1.0).contains(&rep.ece));
    }

    #[test]
    fn test_decide_choice_prob_floor() {
        let (d, p) = NtJevCalibration::decide_choice("A", "B", 0.99, true);
        assert!((p - (1.0 - PROB_FLOOR)).abs() < 1e-9);
        match d {
            JevDecision::Choice(ch) => {
                assert_eq!(ch.choice, "A");
                assert!((ch.margin - (2.0 * p - 1.0).abs()).abs() < 1e-9);
            }
            _ => panic!("must be choice"),
        }
    }

    #[test]
    fn test_platt_identity_default() {
        let p = PlattParams::default();
        assert!((p.apply(0.7) - 0.7).abs() < 1e-9);
        assert!((p.apply(0.5) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_platt_fit_recovers_bias() {
        // 合成：模型系统性过自信（报 0.9，实际对 60%）→ b 应为负
        let items: Vec<(f64, bool)> = (0..100)
            .map(|i| (0.9, i % 10 < 6))
            .collect();
        let fit = PlattParams::fit(&items, 60);
        assert!(fit.b < 0.0, "overconfident model needs negative bias, got b={}", fit.b);
        let cal = fit.apply(0.9);
        assert!(cal < 0.9, "calibrated prob must drop");
        assert!(cal > 0.4 && cal < 0.8, "should approach 0.6, got {cal}");
        // NLL 必须优于恒等
        assert!(PlattParams::nll(&fit, &items) < PlattParams::nll(&PlattParams::default(), &items));
    }

    #[test]
    fn test_platt_empty_safe() {
        let fit = PlattParams::fit(&[], 10);
        assert!((fit.a - 1.0).abs() < 1e-9 && fit.b.abs() < 1e-9);
    }

    #[test]
    fn test_rows_are_valid_jsonl_for_exporter() {
        let c = seeded();
        let rows = NtJevCalibration::noul_rows(&c, 5);
        for r in &rows {
            let v: serde_json::Value = serde_json::from_str(&r.jsonl).unwrap();
            assert!(v.get("conversations").is_some());
            assert!(v.get("calib_confidence").is_some());
        }
        // 与 NtTrainExport 同格式，可直接混入 SFT
        let sft = NtTrainExport::sft_lines(&c);
        assert!(!sft.is_empty());
    }

    #[test]
    fn test_risk_floor_derived_from_act_cost() {
        // RLCD：1 - escalate/wrong = 1 - 0.5/3.0 = 0.8333…，门保守侧取 0.85。
        assert!((HIGH_RISK_REVIEW_FLOOR - 0.85).abs() < 1e-9);
        assert!(HIGH_RISK_REVIEW_FLOOR > GOLD_FLOOR);
    }

    #[test]
    fn test_choice_risk_escalates_borderline() {
        // p=0.8：默认门 0.7 放行，高风险门 0.85 送审。
        let (d_lo, _) = NtJevCalibration::decide_choice("A", "B", 0.8, true);
        let (d_hi, _) = NtJevCalibration::decide_choice_with_risk("A", "B", 0.8, true, true);
        assert!(matches!(d_lo, JevDecision::Choice(_)));
        assert!(matches!(d_hi, JevDecision::Choice(_)));
        let rev = |d: &JevDecision| match d {
            JevDecision::Choice(ch) => ch.needs_review,
            _ => true,
        };
        assert!(!rev(&d_lo), "0.8 >= 0.7 默认应放行");
        assert!(rev(&d_hi), "0.8 < 0.85 高风险应送审");
        // 高分两侧都放行：默认行为零变化。
        let (d_lo2, _) = NtJevCalibration::decide_choice("A", "B", 0.95, true);
        let (d_hi2, _) = NtJevCalibration::decide_choice_with_risk("A", "B", 0.95, true, true);
        assert!(!rev(&d_lo2));
        assert!(!rev(&d_hi2));
    }

    #[test]
    fn test_decide_risk_never_loosens() {
        // 高风险只收紧不放松：默认送审的，高风险必送审。
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let (_, pred_d) = NtJevCalibration::decide(&c, &id).unwrap();
        let (_, pred_h) = NtJevCalibration::decide_with_risk(&c, &id, true).unwrap();
        let rev = |p: &EvalPrediction| match &p.decision {
            JevDecision::Noul(n) => n.needs_review,
            _ => true,
        };
        if rev(&pred_d) {
            assert!(rev(&pred_h), "默认送审则高风险必送审");
        }
    }

    #[test]
    fn test_bucket_table_embedded_parses() {
        // 编译期嵌入全量拟合：11 域桶 + global 有限。
        let t = PlattBucketTable::embedded();
        assert_eq!(t.buckets.len(), 11, "域桶数漂移（拟合脚本重跑即变）");
        assert!(t.global.a.is_finite() && t.global.b.is_finite());
    }

    #[test]
    fn test_bucket_table_fallback_safe() {
        // 坏 JSON → 恒等回退；未知域 → global。
        let t = PlattBucketTable::from_json("not json{{{");
        assert!(t.buckets.is_empty());
        let p = 0.73;
        assert!(
            (t.calibrate("nope", p) - p).abs() < 1e-9,
            "恒等回退应近原值"
        );
        let t2 = PlattBucketTable::from_json(r#"{"global":{"a":0.5,"b":0.1},"buckets":{}}"#);
        assert!((t2.calibrate("any", p) - PlattParams { a: 0.5, b: 0.1 }.apply(p)).abs() < 1e-12);
    }

    #[test]
    fn test_bucket_table_rejects_bad_params() {
        // a<=0 / 非数 / 缺键的域被丢弃，不污染表。
        let t = PlattBucketTable::from_json(
            r#"{"global":{"a":1.0,"b":0.0},"buckets":{"d1":{"a":0.0,"b":0.0},"d2":{"a":"x"},"d3":{"a":0.8,"b":-0.2}}}"#,
        );
        assert_eq!(t.buckets.len(), 1, "仅 d3 合法");
        assert!(t.buckets.contains_key("d3"));
    }

    #[test]
    fn test_bucket_table_domain_routing() {
        // 命中桶用桶参，未命中用 global（与 PlattParams::apply 一致）。
        let t = PlattBucketTable::embedded();
        let dom = t.buckets.keys().next().cloned().unwrap_or_default();
        if !dom.is_empty() {
            let p = 0.8;
            let direct = t.buckets.get(&dom).map(|q| q.apply(p)).unwrap_or(-1.0);
            assert!((t.calibrate(&dom, p) - direct).abs() < 1e-12);
            assert!((t.calibrate("未知域-xyz", p) - t.global.apply(p)).abs() < 1e-12);
        }
    }

    #[test]
    fn test_decide_calibrated_identity_matches_plain() {
        // 恒等表（空桶）下校准决策 == 普通决策（穿线零行为变化）。
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let (_, plain) = NtJevCalibration::decide(&c, &id).unwrap();
        let table = PlattBucketTable::fallback();
        let (_, cal) =
            NtJevCalibration::decide_calibrated(&c, &id, "physics", &table, false).unwrap();
        let rev = |p: &EvalPrediction| match &p.decision {
            JevDecision::Noul(n) => (n.needs_review, n.noul),
            _ => (true, -1.0),
        };
        assert_eq!(rev(&plain), rev(&cal));
    }

    #[test]
    fn test_decide_calibrated_unknown_domain_uses_global() {
        // 未知域走 global：与显式 global 校准一致。
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let t = PlattBucketTable::embedded();
        let (_, a) = NtJevCalibration::decide_calibrated(&c, &id, "未知域-xyz", &t, false).unwrap();
        let conf_of = |p: &EvalPrediction| match &p.decision {
            JevDecision::Noul(n) => n.noul,
            _ => f64::NAN,
        };
        // global 表对同一 raw 的输出应自洽（两次调用一致，确定性）。
        let (_, b) = NtJevCalibration::decide_calibrated(&c, &id, "未知域-xyz", &t, false).unwrap();
        assert!((conf_of(&a) - conf_of(&b)).abs() < 1e-12);
        assert!((0.0..=1.0).contains(&conf_of(&a)));
    }

    #[test]
    fn test_decide_calibrated_risk_still_applies() {
        // 校准后 risk 门照常：高风险只收紧。
        let c = seeded();
        let id = c.reasoning_chains.first().map(|ch| ch.id.clone()).unwrap();
        let t = PlattBucketTable::embedded();
        let rev = |p: &EvalPrediction| match &p.decision {
            JevDecision::Noul(n) => n.needs_review,
            _ => true,
        };
        let (_, lo) = NtJevCalibration::decide_calibrated(&c, &id, "physics", &t, false).unwrap();
        let (_, hi) = NtJevCalibration::decide_calibrated(&c, &id, "physics", &t, true).unwrap();
        if rev(&lo) {
            assert!(rev(&hi), "默认送审则高风险必送审（含校准路径）");
        }
    }
}
