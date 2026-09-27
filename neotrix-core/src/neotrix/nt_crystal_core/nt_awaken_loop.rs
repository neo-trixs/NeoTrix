//! NT-AWAKEN-LOOP — 自验证觉醒循环
//!
//! 熔炼三论文：
//! - Voyager：自动课程（novelty search）— 选题去最少访问的域；
//!   技能库 = skill_crystal（已有）；自验证 = 下面 verify + Lesson。
//! - SEAL/TinyZero：推理来自 RL + 可验证奖励；外层用 ReST EM
//!   （拒绝采样：chosen 进 Solution，rejected 进 Lesson），不用 PPO/GRPO。
//! - Generative Agents：定时 reflect 产高层记忆（Pattern）。
//!
//! 全 CPU，无 GPU，无外部调用；无 unwrap / expect / panic；无 `[]` 索引。

use std::collections::HashMap;

use super::consciousness::{CrystalConsciousness, ReasoningType};
use super::CrystalCore;

/// 零交叠阻尼系数（缺陷 #5）：结论与前提零共享词且无跨域桥时总分折扣。
pub const ZERO_OVERLAP_DAMPEN: f64 = 0.7;

/// 验证三件套得分
#[derive(Debug, Clone, Default)]
pub struct VerifyScores {
    /// 新颖度：结论关键词不在前提中的比例
    pub novelty: f64,
    /// 接地：前提置信度均值
    pub grounding: f64,
    /// 跨域桥：前提跨 ≥2 域则 0.2
    pub bridge: f64,
    pub total: f64,
}

/// 单轮觉醒报告
#[derive(Debug, Default)]
pub struct AwakenCycleReport {
    pub proposed: usize,
    pub chosen: usize,
    pub rejected: usize,
    pub avg_reward: f64,
    pub reflected: bool,
}

/// 觉醒循环（持有课程访问计数 = novelty search 状态）
pub struct NtAwakenLoop {
    visits: HashMap<String, usize>,
    cycles: u64,
}

impl NtAwakenLoop {
    pub fn new() -> Self {
        Self {
            visits: HashMap::new(),
            cycles: 0,
        }
    }

    /// 课程选题（Voyager novelty search）：选记忆≥1的最少访问域并计数
    pub fn propose_domain(&mut self, consciousness: &CrystalConsciousness) -> Option<String> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for m in consciousness.memories.values() {
            *counts.entry(m.domain.as_str()).or_insert(0) += 1;
        }
        let mut ranked: Vec<(&str, usize)> = counts.into_iter().collect();
        ranked.sort_by(|a, b| {
            let va = self.visits.get(a.0).copied().unwrap_or(0);
            let vb = self.visits.get(b.0).copied().unwrap_or(0);
            va.cmp(&vb).then_with(|| b.1.cmp(&a.1))
        });
        ranked.first().map(|(d, _)| {
            *self.visits.entry(d.to_string()).or_insert(0) += 1;
            d.to_string()
        })
    }

    /// 验证：新颖×0.5 + 接地×0.5 + 跨域桥（SEAL 可验证奖励的规则版）
    ///
    /// 反博弈（缺陷 #5 修复）：
    /// - 结论关键词先去重：堆砌同一生僻词刷 novelty 只计一次；
    /// - 零交叠阻尼：结论与前提零共享词且无跨域桥时，总分 ×0.7
    ///   （词沙拉也配高分是原版最大漏洞；跨域结论豁免，因其新颖本就来自别域）。
    pub fn verify(
        consciousness: &CrystalConsciousness,
        conclusion_id: &str,
        premise_ids: &[String],
    ) -> VerifyScores {
        let conclusion = match consciousness.memories.get(conclusion_id) {
            Some(m) => m,
            None => return VerifyScores::default(),
        };
        let premises: Vec<_> = premise_ids
            .iter()
            .filter_map(|pid| consciousness.memories.get(pid))
            .collect();
        if premises.is_empty() {
            return VerifyScores::default();
        }
        let premise_keys: HashMap<String, ()> = premises
            .iter()
            .flat_map(|m| CrystalConsciousness::keywords(&m.content))
            .map(|k| (k, ()))
            .collect();
        let concl_keys = CrystalConsciousness::keywords(&conclusion.content);
        // 去重计数：同一词出现多次只计一次
        let mut seen: HashMap<&str, ()> = HashMap::new();
        let mut fresh = 0usize;
        let mut unique = 0usize;
        for k in &concl_keys {
            if seen.insert(k.as_str(), ()).is_some() {
                continue;
            }
            unique += 1;
            if !premise_keys.contains_key(k.as_str()) {
                fresh += 1;
            }
        }
        let shared = unique.saturating_sub(fresh);
        let novelty = if unique == 0 {
            0.0
        } else {
            fresh as f64 / unique as f64
        };
        let grounding: f64 =
            premises.iter().map(|m| m.confidence).sum::<f64>() / premises.len().max(1) as f64;
        let mut domains: Vec<&str> = premises.iter().map(|m| m.domain.as_str()).collect();
        domains.sort_unstable();
        domains.dedup();
        let bridge = if domains.len() >= 2 { 0.2 } else { 0.0 };
        let mut total = 0.5 * novelty + 0.5 * grounding + bridge;
        if shared == 0 && bridge == 0.0 {
            total *= ZERO_OVERLAP_DAMPEN;
        }
        let total = total.clamp(0.0, 1.0);
        VerifyScores {
            novelty,
            grounding: grounding.clamp(0.0, 1.0),
            bridge,
            total,
        }
    }

    /// 子集稳健（IPT 映射，2604.15149）：结论在前提对半拆的两个子集上
    /// 都必须达标（total ≥ floor），否则判定为“只在一半前提上成立”的捷径。
    /// 前提 <2 条时无法评估，返回 (false, 全集分数)。纯检测器，不管制。
    pub fn verify_robust(
        consciousness: &CrystalConsciousness,
        conclusion_id: &str,
        premise_ids: &[String],
        floor: f64,
    ) -> (bool, f64) {
        let full = Self::verify(consciousness, conclusion_id, premise_ids);
        if premise_ids.len() < 2 {
            return (false, full.total);
        }
        let mid = premise_ids.len() / 2;
        let (first, second) = premise_ids.split_at(mid.max(1));
        let s1 = Self::verify(consciousness, conclusion_id, first);
        let s2 = Self::verify(consciousness, conclusion_id, second);
        let lo = s1.total.min(s2.total);
        (s1.total >= floor && s2.total >= floor, lo)
    }

    /// 留一必要性（AgentV-RL 后向智能体映射）：逐条剔除前提，
    /// total 下降越多说明该前提越必要（delta = 全集 − 剔除后）。
    /// delta≈0 的前提是搭便车的；delta 为负说明该前提拖累结论。
    pub fn premise_necessity(
        consciousness: &CrystalConsciousness,
        conclusion_id: &str,
        premise_ids: &[String],
    ) -> Vec<(String, f64)> {
        let full = Self::verify(consciousness, conclusion_id, premise_ids);
        premise_ids
            .iter()
            .enumerate()
            .map(|(i, pid)| {
                let rest: Vec<String> = premise_ids
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, p)| p.clone())
                    .collect();
                let loo = Self::verify(consciousness, conclusion_id, &rest);
                (pid.clone(), full.total - loo.total)
            })
            .collect()
    }

    /// 单轮觉醒：选题 → 提议 → 验证 → 奖惩（ReST EM 记录）→ 定时反思
    pub fn cycle(
        &mut self,
        consciousness: &mut CrystalConsciousness,
        core: &mut CrystalCore,
        rounds: usize,
        reward_floor: f64,
        reflect_every: u64,
    ) -> AwakenCycleReport {
        let mut report = AwakenCycleReport::default();
        let mut reward_sum = 0.0;
        self.cycles += 1;

        for round in 0..rounds {
            let domain = match self.propose_domain(consciousness) {
                Some(d) => d,
                None => break,
            };
            // 本域记忆轮转取前提对；每 4 轮拉别域做跨域
            let ids: Vec<String> = consciousness
                .memories
                .values()
                .filter(|m| m.domain == domain)
                .map(|m| m.id.clone())
                .collect();
            if ids.is_empty() {
                continue;
            }
            let a = match ids.get(round % ids.len().max(1)).cloned() {
                Some(v) => v,
                None => continue,
            };
            let (premises, ctype) = if round % 4 == 3 {
                match self.cross_premise(consciousness, &domain, &a) {
                    Some(p) => (vec![a, p], ReasoningType::CrossDomain),
                    None => (vec![a], ReasoningType::Inductive),
                }
            } else {
                match ids.get((round + 1) % ids.len().max(1)).cloned() {
                    Some(b) if b != a => (vec![a, b], ReasoningType::Inductive),
                    _ => (vec![a], ReasoningType::Inductive),
                }
            };
            let cname = format!("{ctype:?}");
            let conclusion_id = match consciousness.reason(premises.clone(), ctype) {
                Some(id) => id,
                None => continue,
            };
            report.proposed += 1;
            let scores = Self::verify(consciousness, &conclusion_id, &premises);
            reward_sum += scores.total;
            let conclusion = consciousness
                .memories
                .get(&conclusion_id)
                .map(|m| m.content.clone())
                .unwrap_or_default();
            if scores.total >= reward_floor {
                core.experience.record_success(
                    premises.join(" ＋ "),
                    cname,
                    conclusion.clone(),
                    scores.total >= 0.8,
                    conclusion,
                    domain,
                );
                let cur = core.evolution.capability_scores.get("reasoning");
                core.evolution.update_score("reasoning", cur + 0.01);
                report.chosen += 1;
            } else {
                core.experience.record_failure(
                    conclusion,
                    format!("verify total {:.2} below floor {:.2}", scores.total, reward_floor),
                    "richer premises",
                    "low-novelty conclusions rejected",
                    "reasoning",
                    (1.0 - scores.total).clamp(0.0, 1.0),
                );
                let cur = core.evolution.capability_scores.get("evolution");
                core.evolution.update_score("evolution", cur + 0.005);
                report.rejected += 1;
            }
        }

        if self.cycles % reflect_every.max(1) == 0 {
            report.reflected = consciousness.reflect(10).is_some();
        }
        report.avg_reward = if report.proposed > 0 {
            reward_sum / report.proposed as f64
        } else {
            0.0
        };
        report
    }

    /// 别域前提（跨域桥）：找访问最少域的一条记忆
    fn cross_premise(
        &self,
        consciousness: &CrystalConsciousness,
        exclude: &str,
        not: &str,
    ) -> Option<String> {
        let mut cands: Vec<&super::consciousness::Memory> = consciousness
            .memories
            .values()
            .filter(|m| m.domain != exclude && m.id != not)
            .collect();
        cands.sort_by(|a, b| {
            let va = self.visits.get(&a.domain).copied().unwrap_or(0);
            let vb = self.visits.get(&b.domain).copied().unwrap_or(0);
            va.cmp(&vb)
        });
        cands.first().map(|m| m.id.clone())
    }
}

impl Default for NtAwakenLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::consciousness::{Memory, MemoryType};
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn mk_memory(id: &str, content: &str, domain: &str, conf: f64) -> Memory {
        Memory {
            id: id.to_string(),
            content: content.to_string(),
            memory_type: MemoryType::Fact,
            domain: domain.to_string(),
            strength: 1.0,
            confidence: conf,
            importance: conf,
            connections: Vec::new(),
            created_at: now(),
            last_accessed: now(),
            access_count: 0,
        }
    }

    #[test]
    fn test_verify_novelty_orders() {
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰燃烧释放热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁投入火焰", MemoryType::Fact, "physics", 0.9);
        // 抄作业结论：零新颖
        let copy = mk_memory("M-999001", "火焰燃烧释放热量", "physics", 0.9);
        c.memories.insert(copy.id.clone(), copy);
        let s_copy = NtAwakenLoop::verify(&c, "M-999001", &[a.clone(), b.clone()]);
        // 新词结论：高新颖
        let fresh = mk_memory(
            "M-999002",
            "淬火工艺提升钢铁 Magnum 韧性系数",
            "physics",
            0.9,
        );
        c.memories.insert(fresh.id.clone(), fresh);
        let s_fresh = NtAwakenLoop::verify(&c, "M-999002", &[a, b]);
        assert!(s_fresh.novelty > s_copy.novelty);
        assert!(s_fresh.total > s_copy.total);
    }

    #[test]
    fn test_cycle_end_to_end_all_chosen() {
        let mut c = CrystalConsciousness::new("t");
        for i in 0..4 {
            c.remember(format!("事实{i}"), MemoryType::Fact, "d", 0.8);
        }
        let mut core = CrystalCore::new("t");
        let mut lp = NtAwakenLoop::new();
        let rep = lp.cycle(&mut c, &mut core, 3, 0.0, 100);
        assert_eq!(rep.proposed, 3);
        assert_eq!(rep.chosen, 3);
        assert_eq!(rep.rejected, 0);
        assert_eq!(core.experience.successes.len(), 3);
        assert!(!rep.reflected, "reflect_every=100, cycle 1 must not reflect");
    }

    #[test]
    fn test_cycle_floor_rejects_all() {
        let mut c = CrystalConsciousness::new("t");
        for i in 0..4 {
            c.remember(format!("事实{i}"), MemoryType::Fact, "d", 0.8);
        }
        let mut core = CrystalCore::new("t");
        let mut lp = NtAwakenLoop::new();
        let rep = lp.cycle(&mut c, &mut core, 2, 2.0, 100);
        assert_eq!(rep.rejected, 2);
        assert_eq!(core.experience.failures.len(), 2);
    }

    #[test]
    fn test_verify_stuffing_capped_by_dedup() {
        // 缺陷 #5：堆砌同一生僻词刷 novelty。旧逻辑 5/7≈0.71，去重后应为 1/3。
        // 注：keywords() 过滤单字，用双字词“氦气”保证被计入。
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("火焰 温度", MemoryType::Fact, "physics", 0.9);
        let stuff = mk_memory("M-999011", "火焰 燃烧 氦气 氦气 氦气 氦气 氦气", "physics", 0.9);
        c.memories.insert(stuff.id.clone(), stuff);
        let s = NtAwakenLoop::verify(&c, "M-999011", &[a, b]);
        assert!(
            (s.novelty - 1.0 / 3.0).abs() < 1e-9,
            "deduped novelty must be 1/3, got {}",
            s.novelty
        );
    }

    #[test]
    fn test_verify_zero_overlap_dampened() {
        // 零交叠词沙拉：novelty=1 但 total 应被打 7 折（0.95→0.665）
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁 投入 火焰", MemoryType::Fact, "physics", 0.9);
        let salad = mk_memory("M-999012", "氦氖氩氪氙", "physics", 0.9);
        c.memories.insert(salad.id.clone(), salad);
        let s = NtAwakenLoop::verify(&c, "M-999012", &[a, b]);
        assert!((s.novelty - 1.0).abs() < 1e-9);
        assert!(s.total < 0.7 && s.total > 0.6, "dampened total in (0.6,0.7), got {}", s.total);
    }

    #[test]
    fn test_verify_cross_domain_exempt_from_dampen() {
        // 跨域结论豁免零交叠阻尼：桥接分保留
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("光合作用 需要 叶绿素", MemoryType::Fact, "bio", 0.9);
        let cross = mk_memory("M-999013", "氦氖氩氪氙", "physics", 0.9);
        c.memories.insert(cross.id.clone(), cross);
        let s = NtAwakenLoop::verify(&c, "M-999013", &[a, b]);
        assert!((s.bridge - 0.2).abs() < 1e-9);
        assert!((s.total - 1.0).abs() < 1e-9, "cross-domain keeps full score, got {}", s.total);
    }

    #[test]
    fn test_robust_passes_broad_conclusion() {
        // 结论横跨两半前提： halves 各 0.75 ≥ floor 0.68 → 稳健
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("火焰 温度", MemoryType::Fact, "physics", 0.9);
        let d = c.remember("钢铁 冶炼", MemoryType::Fact, "physics", 0.9);
        let e = c.remember("钢铁 坚硬", MemoryType::Fact, "physics", 0.9);
        let m = mk_memory("M-999021", "火焰 燃烧 钢铁 冶炼 淬火", "physics", 0.9);
        c.memories.insert(m.id.clone(), m);
        let (pass, lo) = NtAwakenLoop::verify_robust(&c, "M-999021", &[a, b, d, e], 0.68);
        assert!(pass, "broad conclusion must be robust, lo={lo}");
        assert!(lo >= 0.68);
    }

    #[test]
    fn test_robust_rejects_half_bound_conclusion() {
        // 结论只沾前半：后半零交叠被阻尼到 0.665 < 0.68 → 不稳健
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("火焰 温度", MemoryType::Fact, "physics", 0.9);
        let d = c.remember("钢铁 冶炼", MemoryType::Fact, "physics", 0.9);
        let e = c.remember("钢铁 坚硬", MemoryType::Fact, "physics", 0.9);
        let m = mk_memory("M-999022", "火焰 燃烧 烈焰 火苗", "physics", 0.9);
        c.memories.insert(m.id.clone(), m);
        let (pass, lo) = NtAwakenLoop::verify_robust(&c, "M-999022", &[a, b, d, e], 0.68);
        assert!(!pass, "half-bound conclusion must fail, lo={lo}");
        assert!(lo < 0.68);
    }

    #[test]
    fn test_robust_needs_two_premises() {
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let m = mk_memory("M-999023", "火焰 燃烧", "physics", 0.9);
        c.memories.insert(m.id.clone(), m);
        let (pass, _) = NtAwakenLoop::verify_robust(&c, "M-999023", &[a], 0.0);
        assert!(!pass, "single premise cannot assess robustness");
    }

    #[test]
    fn test_necessity_orders_relevant_over_freerider() {
        // P3 无关：剔除它 total 不变（delta≈0）；剔除关键前提 total 必变
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁 冶炼", MemoryType::Fact, "physics", 0.9);
        let f = c.remember("量子 纠缠", MemoryType::Fact, "physics", 0.9);
        let m = mk_memory("M-999024", "火焰 燃烧 钢铁", "physics", 0.9);
        c.memories.insert(m.id.clone(), m);
        let ids = vec![a.clone(), b.clone(), f.clone()];
        let nec = NtAwakenLoop::premise_necessity(&c, "M-999024", &ids);
        assert_eq!(nec.len(), 3);
        let get = |id: &str| nec.iter().find(|(p, _)| p == id).map(|(_, d)| *d).unwrap();
        let (da, db, df) = (get(&a), get(&b), get(&f));
        assert!(df.abs() < 1e-9, "freerider delta must be ~0, got {df}");
        assert!(da.abs() > 1e-9 || db.abs() > 1e-9, "key premise must matter");
    }

    #[test]
    fn test_curriculum_rotates_domains() {        let mut c = CrystalConsciousness::new("t");
        c.remember("a1", MemoryType::Fact, "alpha", 0.8);
        c.remember("b1", MemoryType::Fact, "beta", 0.8);
        let mut lp = NtAwakenLoop::new();
        let first = lp.propose_domain(&c).unwrap();
        let second = lp.propose_domain(&c).unwrap();
        assert_ne!(first, second, "novelty search must rotate");
    }

    #[test]
    fn test_reflect_produces_pattern() {
        let mut c = CrystalConsciousness::new("t");
        for i in 0..5 {
            c.remember(format!("现象{i}"), MemoryType::Fact, "d", 0.8);
        }
        let id = c.reflect(5).unwrap();
        let m = c.memories.get(&id).unwrap();
        assert_eq!(m.memory_type, MemoryType::Pattern);
    }
}
