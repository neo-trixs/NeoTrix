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

use std::collections::{HashMap, HashSet};

// ReasoningType 不再在此具名：链类型改由 PremiseSelector 判定后带回
// （跨 ≥2 域 → CrossDomain），故此处只留 CrystalConsciousness。
use super::consciousness::CrystalConsciousness;
use super::nt_graph_index::Adjacency;
use super::nt_premise_selector::PremiseSelector;
use super::{CrystalCore, Memory};

/// 零交叠阻尼系数（缺陷 #5）：结论与前提零共享词且无跨域桥时总分折扣。
pub const ZERO_OVERLAP_DAMPEN: f64 = 0.7;

/// 跨域桥满分（D9：旧实现把 0.2 当常数，实为「域数≥2 即满分」）。
pub const BRIDGE_MAX: f64 = 0.2;
/// `0.5*novelty + 0.5*coverage` 的**理论峰值**。
/// f(t) = 0.5·4t(1−t) + 0.5·(1−t) = 0.5 + 1.5t − 2t²，峰值在 t=0.375，值 0.78125。
/// 归一用它，使 base 的量程恰为 [0, BASE_WEIGHT]。
pub const NOVELTY_GROUND_PEAK: f64 = 0.78125;
/// base 份额（其余留给 bridge），两者相加恰为 1.0。
pub const BASE_WEIGHT: f64 = 0.8;
/// 「前提是否图连通」判定时，每条前提最多扫描多少条出边。
/// 枢纽入度可达 171（曾达 935），不设上限会退化成 O(枢纽度×前提数)。
/// 扫描前先排序，故截断是**确定性**的。
pub const BRIDGE_SCAN_CAP: usize = 32;

/// 前提两两之间是否图连通（直接相连，或共享至少一个邻居）。
/// 共享邻居的典型形态：同一锚点的 facet（都指向同一个 profile）。
fn premises_pairwise_linked(premises: &[&Memory]) -> bool {
    if premises.len() < 2 {
        return false;
    }
    // 每条前提的邻居集合（排序 + 截断，确定性）
    let nbrs: Vec<HashSet<&str>> = premises
        .iter()
        .map(|m| {
            let mut v: Vec<&str> = m.connections.iter().map(|s| s.as_str()).collect();
            v.sort_unstable();
            v.dedup();
            v.truncate(BRIDGE_SCAN_CAP);
            v.into_iter().collect()
        })
        .collect();
    let ids: Vec<&str> = premises.iter().map(|m| m.id.as_str()).collect();
    for i in 0..premises.len() {
        for j in (i + 1)..premises.len() {
            // 直接相连
            if nbrs[i].contains(ids[j]) || nbrs[j].contains(ids[i]) {
                return true;
            }
            // 共享邻居
            if nbrs[i].iter().any(|a| nbrs[j].contains(*a)) {
                return true;
            }
        }
    }
    false
}

/// 验证三件套得分
#[derive(Debug, Clone, Default)]
pub struct VerifyScores {
    /// 新颖度：结论关键词不在前提中的比例
    pub novelty: f64,
    /// 接地率：结论关键词**被前提覆盖**的比例（= 1 − fresh/unique）。
    /// 2026-09-28 由 `grounding` 改名并重定义：旧值是「前提 confidence 均值」，
    /// 实测标准差 0.012、近乎常量，零判别力，且名不副实。
    pub coverage: f64,
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
    /// 全集分达标但**子集稳健**不达标者（捷径链）—— 反刷分的核心观测量。
    /// 若它占比高，说明选择器给的冗余前提太多（多链证据 d 取值需调）。
    pub robust_rejected: usize,
    /// 留一 delta≈0 的「搭便车」前提总数（对结论无实际贡献）。
    pub free_rider_premises: usize,
    /// `total` 饱和在 1.0 的链数 —— **判别力告警指标**。
    ///
    /// 2026-09-28 实测（`nt_verify_sim.py`，活库 300 锚点）：`reason()` 产出的是
    /// 「固定中文框架 + 前提片段拷贝」，使词汇新鲜占比 t 结构性地钉在 0.4 附近
    /// （接近 total 曲线峰值），于是**任何词法分量的方差上限约 0.02**
    /// （total 标准差 0.0196、留一 delta 标准差 0.0184，两者同量级）。
    /// 结论：**在没有 judge 模型的条件下，词法判分器无法区分链的优劣**，
    /// 这是 `reason()` 模板化的结构性后果，不是公式 bug。
    /// 故本计数器让"门无判别力"成为**每 tick 可见的事实**，而不是等下一个人
    /// 再花一天重新推导。饱和率若长期 >10%，说明判分器已失效，应上模型判分
    /// （GraphRAG 式 rubric / AnyBURL 式规则），而非继续调系数。
    pub score_saturated: usize,
}

/// 觉醒循环（持有课程访问计数 = novelty search 状态）
pub struct NtAwakenLoop {
    visits: HashMap<String, usize>,
    cycles: u64,
    /// 前提选择用的邻接缓存 `Some((记忆条数, 邻接))`。
    /// 派生状态不落盘（同 `connected_count` 策略），按记忆条数判失效：
    /// `reason()` 每轮都铸新记忆，故轮次不是可靠判据。
    /// 命中则复用 —— 活库 6 万+ 记忆，每轮重建是 O(V+E)。
    adj_cache: Option<(usize, Adjacency)>,
}

impl NtAwakenLoop {
    pub fn new() -> Self {
        Self {
            visits: HashMap::new(),
            cycles: 0,
            adj_cache: None,
        }
    }

    /// 取邻接（必要时重建并缓存）。
    ///
    /// 陈旧的邻接会让前提选择看见已经不存在的边；`from_memories` 只把
    /// 双方都在库内的边计入，故**记忆被删**后必须重建才准。
    fn cached_adjacency<'a>(
        &'a mut self,
        consciousness: &CrystalConsciousness,
    ) -> Option<&'a Adjacency> {
        let n = consciousness.memories.len();
        let stale = match self.adj_cache.as_ref() {
            Some((cached, _)) => *cached != n,
            None => true,
        };
        if stale {
            self.adj_cache = Some((n, Adjacency::from_memories(&consciousness.memories)));
        }
        self.adj_cache.as_ref().map(|(_, adj)| adj)
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
        // ── D8 修复：novelty 的反向激励 ────────────────────────────────
        // 旧式 `novelty = fresh / unique`：结论关键词越是**不出现在前提里**，
        // 分越高 —— 这是**奖励编造**（发虚词即得分），与「新颖 = 有据的新增」
        // 本意相反。`ZERO_OVERLAP_DAMPEN` 只挡「零交叠」这一种极端，
        // 「90% 都是新词」的胡编仍能拿高分。
        //
        // 现式：`novelty = 4·t·(1−t)`，t = fresh/unique
        //   t=0   （纯复述）   → 0    无新增，不算新颖
        //   t=0.5 （一半有据一半新增）→ 1.0  峰值，恰是「有据的新发现」
        //   t=1   （全是新词） → 0    纯编造，不给分
        // 取 4 作系数使峰值仍为 1.0 → `total` 的量纲与旧版一致，
        // 不必连带重标定 GOLD_FLOOR（但分数分布会变，见 handoff）。
        let t = if unique == 0 {
            0.0
        } else {
            fresh as f64 / unique as f64
        };
        let novelty = 4.0 * t * (1.0 - t);
        // ── A 修复：grounding 曾是**常量**，零判别力 ──────────────────
        // 旧式 grounding = 前提 confidence 的均值。实测活库 6.4 万条记忆的
        // confidence 中位 0.75、标准差 0.012、**只有 2 个取值** → 该分量在
        // 全语料近乎常数，total 的方差几乎全被它压平。
        // 它测的也不是"接地"，而是"前提的平均自评置信度"——名不副实。
        //
        // 现式 coverage = 结论关键词**被前提覆盖**的比例 = 1 − t。
        // 这才是"接地"的定义：结论有多少内容能在前提里找到出处。
        // 副作用：coverage 与 novelty 由构造互补（和为 1），二者峰值不重合，
        // 判分器首次具备真实区分度（见 nt_verify_sim.py 实测）。
        let coverage = 1.0 - t;
        let mut domains: Vec<&str> = premises.iter().map(|m| m.domain.as_str()).collect();
        domains.sort_unstable();
        domains.dedup();
        // ── D9 修复：bridge 是常数不是质量 ──────────────────────────────
        // 旧式 `0.2 if domains>=2`：域数只是**必要条件**。两条毫不相干的记忆
        // 跨了域就照拿满分（实测曾出现 935 入度通用枢纽下的整域 rubric 互不相干）。
        // 现按实据打分（0 / 0.5 / 1.0 档），命中两项才给满：
        //   ① 前提之间**图连通**（直接相连，或共享一个邻居，如同属一个锚点）
        //   ② 结论与**每条**前提都有词面交集（否则是并置，不是「桥」）
        let bridge = if domains.len() >= 2 {
            let linked = premises_pairwise_linked(&premises);
            let mut per_premise_shared = 0usize;
            for pm in &premises {
                // 必须先把 keywords() 的返回值绑到具名变量：它返回**拥有的**
                // Vec<String>，直接链式 `.iter().map(as_str).collect()` 得到的是
                // 借用该临时量的 HashSet，语句结束即析构（E0716）。
                let pm_kw = CrystalConsciousness::keywords(&pm.content);
                let pk: HashSet<&str> = pm_kw.iter().map(|s| s.as_str()).collect();
                if concl_keys.iter().any(|k| pk.contains(k.as_str())) {
                    per_premise_shared += 1;
                }
            }
            let bridges_all = per_premise_shared == premises.len();
            let score = match (linked, bridges_all) {
                (true, true) => 1.0,
                (true, false) | (false, true) => 0.5,
                (false, false) => 0.0,
            };
            score * BRIDGE_MAX
        } else {
            0.0
        };
        // ── 饱和修复：让 total 的**理论最大值恰为 1.0** ──────────────────
        // 旧式 `0.5*novelty + 0.5*grounding + bridge` 上界是 0.5+0.5+0.2 = **1.2**，
        // 于是高分链一律被 `clamp(1.0)` 截成**同一个值**，排序信息全部丢失
        // （实测 69% 的链 total 饱和在 1.0，标准差仅 0.018 → 门形同虚设）。
        // 现按 NOVELTY_GROUND_PEAK 归一：base ≤ BASE_WEIGHT，base+bridge ≤ 1.0，
        // clamp 退化为**永不触发的兜底**，高分段排序得以保留。
        let base = (0.5 * novelty + 0.5 * coverage) / NOVELTY_GROUND_PEAK * BASE_WEIGHT;
        let mut total = base + bridge;
        if shared == 0 && bridge == 0.0 {
            total *= ZERO_OVERLAP_DAMPEN;
        }
        let total = total.clamp(0.0, 1.0);
        VerifyScores {
            novelty,
            coverage,
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
            // 前提选择：图语义簇（锚点 in-neighbors，域多样性贪心）取代旧的
            // 「按位置取相邻两条」—— 后者与语义无关，只是列表下标邻居，
            // 且 ids 来自 HashMap 序，逐次运行取到的两条都不同。
            // 跨域节奏（round % 4 == 3）与计数器语义保持不变；
            // 链类型改由选择器按前提实际覆盖的域判定（跨 ≥2 域 → CrossDomain）。
            let prefer_cross = round % 4 == 3;
            // 选择器在本块内借用邻接与记忆表；块结束（返回自有 PremiseSet）后
            // 借用即释放，`consciousness` 随后才能可变借用去 reason()。
            let picked = {
                let adj = match self.cached_adjacency(consciousness) {
                    Some(a) => a,
                    None => continue,
                };
                PremiseSelector::with_adjacency(&consciousness.memories, adj).for_round(
                    &domain,
                    round,
                    prefer_cross,
                )
            };
            let set = match picked {
                Some(s) => s,
                None => continue,
            };
            let premises = set.premises;
            let ctype = set.chain_type;
            let cname = format!("{ctype:?}");
            let conclusion_id = match consciousness.reason(premises.clone(), ctype) {
                Some(id) => id,
                None => continue,
            };
            report.proposed += 1;
            let scores = Self::verify(consciousness, &conclusion_id, &premises);
            // ── 步 5：JEV 裁决闭环 ──────────────────────────────────────
            // 旧式只看 `scores.total >= reward_floor`。但 total 是**全集**分数，
            // 一条「只靠其中一半前提就成立」的链同样能过 —— 这正是
            // `verify_robust`（IPT 2604.15149 子集稳健）要拦的捷径。
            // 现在两门都要过：全集分 + 前提对半拆各自达标。
            let (robust_ok, robust_lo) =
                Self::verify_robust(consciousness, &conclusion_id, &premises, reward_floor);
            let accept = scores.total >= reward_floor && robust_ok;
            if !accept && scores.total >= reward_floor && !robust_ok {
                // 全集够分但子集不稳 = 捷径链，单独计数以便观测「选择性」
                report.robust_rejected += 1;
            }
            // 留一必要性：delta≈0 的前提是搭便车（对结论无贡献）。
            // 计入诊断，不自动剔除（剔除会让 recall 更难解释，且需重跑 reason）。
            let necessity = Self::premise_necessity(consciousness, &conclusion_id, &premises);
            report.free_rider_premises +=
                necessity.iter().filter(|(_pid, d)| d.abs() < 0.01).count();
            if scores.total >= 0.999 {
                report.score_saturated += 1;
            }
            reward_sum += scores.total;
            let conclusion = consciousness
                .memories
                .get(&conclusion_id)
                .map(|m| m.content.clone())
                .unwrap_or_default();
            if accept {
                let _ = robust_lo;
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
                    format!(
                        "verify total {:.2} below floor {:.2} (robust_ok={}, subset_lo={:.2})",
                        scores.total, reward_floor, robust_ok, robust_lo
                    ),
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
    ///
    /// 已被 `PremiseSelector::cross_bridge` / `foreign_domain_id` 取代
    /// （2026-09-28 前提选择改造）：本函数从**全库**任意位置抓一条别域记忆，
    /// 与链内前提无语义关联，且 `cands` 的输入是 `HashMap::values()` —
    /// `sort_by` 稳定，同访问数时保留 HashMap 序 → **逐次运行结果不同**。
    /// 保留于此仅作对照，勿再调用。
    #[allow(dead_code)]
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
        // 前提按空格切词（keywords() 只在空白/标点处切）
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁 投入 火焰", MemoryType::Fact, "physics", 0.9);
        // 抄作业结论：与前提逐词相同 ⇒ t=0 ⇒ D8 novelty=0
        let copy = mk_memory("M-999001", "火焰 燃烧 释放 热量", "physics", 0.9);
        c.memories.insert(copy.id.clone(), copy);
        let s_copy = NtAwakenLoop::verify(&c, "M-999001", &[a.clone(), b.clone()]);
        // **有据的新发现**：部分词接地、部分词新增 ⇒ t 落在 0.5 峰值附近。
        // D8 契约下 novelty=4t(1-t) 只在 t≈0.5 最高，t=1（纯新词）判 0 ——
        // 原夹具的「淬火工艺提升钢铁 Magnum 韧性系数」与前提零交集 ⇒ t=1
        // ⇒ novelty 与 copy 同为 0，断言 `>` 永远不成立。峰值在**半接地**处。
        let fresh = mk_memory(
            "M-999002",
            "火焰 燃烧 淬火 工艺 韧性",
            "physics",
            0.9,
        );
        c.memories.insert(fresh.id.clone(), fresh);
        let s_fresh = NtAwakenLoop::verify(&c, "M-999002", &[a, b]);
        assert!(
            (s_copy.novelty - 0.0).abs() < 1e-9,
            "纯复述 t=0 ⇒ novelty 必须为 0，实得 {}",
            s_copy.novelty
        );
        assert!(
            s_fresh.novelty > s_copy.novelty,
            "半接地结论(t≈0.6)应高于纯复述(t=0)：{} vs {}",
            s_fresh.novelty,
            s_copy.novelty
        );
        assert!(s_fresh.total > s_copy.total);
    }

    #[test]
    fn test_cycle_end_to_end_all_chosen() {
        let mut c = CrystalConsciousness::new("t");
        // 夹具前提：每轮都必须落在**有 ≥3 条成员**的域上。
        // `propose_domain` 按 (访问次数少者, 成员多者) 轮转，而每轮 `reason()`
        // 会**新造一条属于新域的结论**；原夹具 4 条全在 "d"，于是第 2 轮就轮转到
        // 那个只有 1 条成员的结论域 ⇒ `domain_fallback` 只回 1 条前提 ⇒
        // `verify_robust` 因 `len() < 2` 硬拒（单前提「链」不算链）⇒ chosen 2≠3。
        // 铺 3 个域 × 4 条，轮转落点恒有 ≥3 条成员。
        for d in 0..3 {
            for i in 0..4 {
                c.remember(
                    format!("域{d}事实{i}"),
                    MemoryType::Fact,
                    format!("d{d}"),
                    0.8,
                );
            }
        }
        let mut core = CrystalCore::new("t");
        let mut lp = NtAwakenLoop::new();
        let rep = lp.cycle(&mut c, &mut core, 3, 0.0, 100);
        assert_eq!(rep.proposed, 3);
        assert_eq!(rep.chosen, 3);
        assert_eq!(rep.rejected, 0);
        assert_eq!(core.experience.successes.len(), 3);
        assert!(
            !rep.reflected,
            "reflect_every=100, cycle 1 must not reflect"
        );
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
        // 缺陷 #5：堆砌同一生僻词刷 novelty。
        // **不变量是「堆砌无效」，不是 novelty 的具体数值**：把「氦气」写 5 遍
        // 必须与只写 1 遍**完全等价**。旧断言写成 `novelty == 1/3`（线性
        // `fresh/unique` 的值），D8 改成抛物线后去重反而把 t 推向峰值
        // （t=1/3 ⇒ 8/9），「封顶」在 novelty 数值上已无法表达。
        // 改成断言等价性：与公式无关，且直接锁死去重逻辑。
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("火焰 温度", MemoryType::Fact, "physics", 0.9);
        // 注：keywords() 过滤单字，用双字词「氦气」保证被计入。
        let stuff = mk_memory(
            "M-999011",
            "火焰 燃烧 氦气 氦气 氦气 氦气 氦气",
            "physics",
            0.9,
        );
        c.memories.insert(stuff.id.clone(), stuff);
        let once = mk_memory("M-999012b", "火焰 燃烧 氦气", "physics", 0.9);
        c.memories.insert(once.id.clone(), once);
        let s_stuff = NtAwakenLoop::verify(&c, "M-999011", &[a.clone(), b.clone()]);
        let s_once = NtAwakenLoop::verify(&c, "M-999012b", &[a, b]);
        assert!(
            (s_stuff.novelty - s_once.novelty).abs() < 1e-12,
            "堆砌 5 遍「氦气」必须与只写 1 遍等价：{} vs {}",
            s_stuff.novelty,
            s_once.novelty
        );
        assert!(
            (s_stuff.total - s_once.total).abs() < 1e-12,
            "total 同样不应被堆砌影响：{} vs {}",
            s_stuff.total,
            s_once.total
        );
    }

    #[test]
    fn test_verify_zero_overlap_dampened() {
        // 零交叠词沙拉。D8 下 novelty=4t(1-t) 在 t=1（纯新词）判 **0** ——
        // 比旧契约（novelty=1 再靠 total 打折）更直接地兑现本测试的本意。
        // 旧断言 `novelty == 1.0` 写的是线性契约，已过期。
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("火焰 燃烧 释放 热量", MemoryType::Fact, "physics", 0.9);
        let b = c.remember("钢铁 投入 火焰", MemoryType::Fact, "physics", 0.9);
        let salad = mk_memory("M-999012", "氦氖氩氪氙", "physics", 0.9);
        c.memories.insert(salad.id.clone(), salad);
        // 对照：结构相同但**沾一个前提词**（氦氖氩氪氙 整段仍是新词）
        // ⇒ t=0.5 峰值 且 shared>0 ⇒ 不触发 ZERO_OVERLAP_DAMPEN
        let cmp = mk_memory("M-999012b", "氦氖氩氪氙 火焰", "physics", 0.9);
        c.memories.insert(cmp.id.clone(), cmp);
        let s = NtAwakenLoop::verify(&c, "M-999012", &[a.clone(), b.clone()]);
        let s_cmp = NtAwakenLoop::verify(&c, "M-999012b", &[a, b]);
        assert!(
            (s.novelty - 0.0).abs() < 1e-9,
            "纯新词词沙拉 t=1 ⇒ D8 novelty 必须为 0，实得 {}",
            s.novelty
        );
        // 阻尼是**相对**效应：同样零交叠，加一个前提词即不被阻尼
        assert!(
            s.total < s_cmp.total,
            "零交叠应被 ZERO_OVERLAP_DAMPEN 压低：{} vs 沾词对照 {}",
            s.total,
            s_cmp.total
        );
    }

    #[test]
    fn test_verify_cross_domain_exempt_from_dampen() {
        // 跨域结论豁免零交叠阻尼：桥接分保留。
        // 夹具必须真的挣到 BRIDGE_MAX：(linked, bridges_all) == (true, true)
        //   - linked  取决于**图边**（connections），不是关键词共现 ⇒ 前提互连
        //   - bridges_all 取决于结论是否与**每条**前提都有共享词
        // 原夹具两条前提无 connections 且结论与前提零共享 ⇒ (false,false)
        // ⇒ bridge=0，断言 0.2 永远不成立。
        let mut c = CrystalConsciousness::new("t");
        let mut pa = mk_memory("PA", "火焰 燃烧", "physics", 0.9);
        let mut pb = mk_memory("PB", "光合作用 叶绿素", "bio", 0.9);
        pa.connections = vec!["PB".to_string()];
        pb.connections = vec!["PA".to_string()];
        c.memories.insert(pa.id.clone(), pa);
        c.memories.insert(pb.id.clone(), pb);
        // 结论同时搭上两条前提（燃烧 / 叶绿素），另有 1 个新词
        let cross = mk_memory("M-999013", "燃烧 叶绿素 耐热合金", "physics", 0.9);
        c.memories.insert(cross.id.clone(), cross);
        let s = NtAwakenLoop::verify(
            &c,
            "M-999013",
            &["PA".to_string(), "PB".to_string()],
        );
        assert!(
            (s.bridge - BRIDGE_MAX).abs() < 1e-9,
            "跨域且搭满两条前提 ⇒ 桥接分应为满额 {}，实得 {}",
            BRIDGE_MAX,
            s.bridge
        );
        // 豁免阻尼：有据的跨域结论不该被压到低位
        assert!(
            s.total > 0.6,
            "跨域有据结论不应被零交叠阻尼压低，实得 total={}（novelty={}）",
            s.total,
            s.novelty
        );
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
        assert!(
            da.abs() > 1e-9 || db.abs() > 1e-9,
            "key premise must matter"
        );
    }

    #[test]
    fn test_curriculum_rotates_domains() {
        let mut c = CrystalConsciousness::new("t");
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

/// D8 / D9 回归测试（2026-09-28）。
///
/// 背景：这两处是**静默的激励错位**——不崩、不报错，只是让评分器奖励错误行为：
///   D8 `novelty = fresh/unique` → 结论越是发虚词分越高（奖励编造）
///   D9 `bridge = 0.2 if domains>=2` → 两条毫不相干的记忆跨域即满分（常数非质量）
#[cfg(test)]
mod verify_incentive_tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::consciousness::{Memory, MemoryType};

    fn mem(id: &str, domain: &str, content: &str, conns: &[&str]) -> Memory {
        Memory {
            id: id.to_string(),
            content: content.to_string(),
            memory_type: MemoryType::Fact,
            domain: domain.to_string(),
            strength: 1.0,
            confidence: 0.8,
            importance: 0.5,
            connections: conns.iter().map(|c| c.to_string()).collect(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
        }
    }

    fn cs(ms: Vec<Memory>) -> CrystalConsciousness {
        let mut c = CrystalConsciousness::new("verify-test");
        for m in ms {
            c.memories.insert(m.id.clone(), m);
        }
        c
    }

    /// D8 核心性质：**全新鲜词（纯编造）novelty 必须为 0**，而旧式会给满分。
    #[test]
    fn d8_pure_fabrication_scores_zero_novelty() {
        // 前提只有 alpha/beta；结论全是没见过的词 → t=1 → novelty=0
        let c = cs(vec![
            mem("P", "d1", "alpha beta", &[]),
            mem("C", "d1", "zulu yonder vex", &[]),
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["P".to_string()]);
        assert!(
            s.novelty < 1e-9,
            "纯编造（全部关键词不在前提）novelty 应为 0，实得 {} —— D8 未修",
            s.novelty
        );
    }

    /// D8 峰值性质：半有据半新增才是「有据的新颖」，novelty 达峰。
    #[test]
    fn d8_grounded_novelty_peaks_at_half() {
        let c = cs(vec![
            mem("P", "d1", "alpha beta gamma", &[]),
            // 3 个词：alpha 有据，kappa/lambda 新增 → t=2/3
            mem("C", "d1", "alpha kappa lambda", &[]),
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["P".to_string()]);
        let t = 2.0 / 3.0;
        let expect = 4.0 * t * (1.0 - t);
        assert!(
            (s.novelty - expect).abs() < 1e-9,
            "{} vs {}",
            s.novelty,
            expect
        );
        // t=2/3 属「偏编造」侧，novelty 应明显低于峰值 1.0
        assert!(s.novelty < 1.0, "t=2/3 不该拿满分");
    }

    /// D8 回归护栏：纯复述（t=0）仍为 0，且量纲未变（峰值仍是 1.0）。
    #[test]
    fn d8_restatement_zero_and_range_preserved() {
        let c = cs(vec![
            mem("P", "d1", "alpha beta", &[]),
            mem("C1", "d1", "alpha beta", &[]), // t=0
        ]);
        let s = NtAwakenLoop::verify(&c, "C1", &["P".to_string()]);
        assert!(s.novelty < 1e-9, "纯复述不该算新颖");
        // 峰值可达 1.0（旧量纲），故 GOLD_FLOOR 无需重标定
        let c2 = cs(vec![
            mem("P", "d1", "alpha beta", &[]),
            mem("C2", "d1", "alpha kappa", &[]), // t=1/2 → 峰值
        ]);
        let s2 = NtAwakenLoop::verify(&c2, "C2", &["P".to_string()]);
        assert!(
            (s2.novelty - 1.0).abs() < 1e-9,
            "峰值应仍为 1.0，实得 {}",
            s2.novelty
        );
    }

    /// D9：跨域但**图不相连**且结论与前提无交集 → bridge 必须远低于满分。
    #[test]
    fn d9_unrelated_cross_domain_gets_no_bridge_credit() {
        let c = cs(vec![
            mem("A", "d1", "alpha", &[]),
            mem("B", "d2", "beta", &[]), // 跨域但零连接、零共享邻居
            mem("C", "d1", "zulu yonder vex wan", &[]), // 结论与两条都无交集
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["A".to_string(), "B".to_string()]);
        assert!(
            s.bridge < 1e-9,
            "不相连且无交集的跨域对不得拿桥分，实得 {} —— D9 未修",
            s.bridge
        );
    }

    /// D9 正向：图连通（共享邻居 = 同属一锚点）+ 结论与每条前提都有交集 → 满分。
    #[test]
    fn d9_linked_cross_domain_earns_full_bridge() {
        let c = cs(vec![
            mem("ANCHOR", "d0", "anchor", &[]),
            mem("A", "d1", "alpha", &["ANCHOR"]), // 都指向同一锚点 → 共享邻居
            mem("B", "d2", "beta", &["ANCHOR"]),  // 跨域
            // 结论与 A、B 都有词面交集
            mem("C", "d0", "alpha beta gamma", &[]),
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["A".to_string(), "B".to_string()]);
        assert!(
            (s.bridge - BRIDGE_MAX).abs() < 1e-9,
            "连通且逐条有交集应给满 {}，实得 {}",
            BRIDGE_MAX,
            s.bridge
        );
    }

    /// D9 中间档：连通但结论只与其中一条有交集 → 半分。
    #[test]
    fn d9_half_credit_when_only_linked() {
        let c = cs(vec![
            mem("ANCHOR", "d0", "anchor", &[]),
            mem("A", "d1", "alpha", &["ANCHOR"]),
            mem("B", "d2", "beta", &["ANCHOR"]),
            mem("C", "d0", "alpha zulu yonder vex", &[]), // 只与 A 有交集
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["A".to_string(), "B".to_string()]);
        assert!(
            (s.bridge - BRIDGE_MAX * 0.5).abs() < 1e-9,
            "半分档，期望 {}，实得 {}",
            BRIDGE_MAX * 0.5,
            s.bridge
        );
    }

    /// 单域前提不得有桥分（域数是必要条件）。
    #[test]
    fn d9_single_domain_has_zero_bridge() {
        let c = cs(vec![
            mem("A", "d1", "alpha", &[]),
            mem("B", "d1", "beta", &[]),
            mem("C", "d1", "alpha beta gamma", &[]),
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["A".to_string(), "B".to_string()]);
        assert!(s.bridge < 1e-9, "单域不得有桥分");
    }

    /// 空前提 / 缺结论不得 panic，返回 default。
    #[test]
    fn empty_and_missing_are_safe() {
        let c = cs(vec![mem("A", "d1", "alpha", &[])]);
        let s1 = NtAwakenLoop::verify(&c, "A", &[]);
        assert_eq!(s1.total, 0.0);
        let s2 = NtAwakenLoop::verify(&c, "NOPE", &["A".to_string()]);
        assert_eq!(s2.total, 0.0);
        let s3 = NtAwakenLoop::verify(&c, "A", &["NOPE".to_string()]);
        assert_eq!(s3.total, 0.0);
        let c_empty = CrystalConsciousness::new("e");
        let s4 = NtAwakenLoop::verify(&c_empty, "x", &["y".to_string()]);
        assert_eq!(s4.total, 0.0);
    }

    /// A 修复：coverage 必须是**词汇接地率**，不是前提 confidence 均值。
    /// 构造两条 confidence 相同但词面重叠度不同的链 → coverage 必须不同。
    #[test]
    fn a_coverage_measures_vocab_overlap_not_confidence() {
        // 前提置信度**完全相同**（0.8），只有词面重叠度不同
        let c = cs(vec![
            mem("PA", "d1", "alpha beta gamma delta", &[]),
            mem("PB", "d1", "alpha beta gamma delta", &[]),
            // 结论 4 个词全在前提里 → t=0 → coverage=1
            mem("C1", "d1", "alpha beta gamma delta", &[]),
            // 结论一半是新词 → t=0.5 → coverage=0.5
            mem("C2", "d1", "alpha beta zulu yonder", &[]),
        ]);
        let s1 = NtAwakenLoop::verify(&c, "C1", &["PA".to_string(), "PB".to_string()]);
        let s2 = NtAwakenLoop::verify(&c, "C2", &["PA".to_string(), "PB".to_string()]);
        assert!(
            (s1.coverage - 1.0).abs() < 1e-9,
            "全重叠 → coverage=1，实得 {}",
            s1.coverage
        );
        assert!(
            (s2.coverage - 0.5).abs() < 1e-9,
            "半重叠 → coverage=0.5，实得 {}",
            s2.coverage
        );
        // 前提 confidence 相同，coverage 却不同 → 证明不再由 confidence 决定
        assert!(s1.coverage > s2.coverage);
    }

    /// 饱和修复：total 的**理论最大值恰为 1.0**（clamp 永不触发）。
    /// 这是 69% 饱和的根因修复 —— 旧式上界 1.2 会被 clamp 截平、排序信息尽失。
    #[test]
    fn saturation_fix_total_never_exceeds_one_by_construction() {
        // 构造让 novelty 与 coverage 同时接近峰值的极难样本：
        // t=0.375 → novelty=1.0, coverage=0.625 → base=0.8；bridge 给满 0.2 → 1.0
        // 结论 8 词，其中 3 新 5 旧 → t=3/8=0.375
        let old8 = "w1 w2 w3 w4 w5";
        let new3 = "n1 n2 n3";
        let c = cs(vec![
            mem("PA", "d1", old8, &[]),
            mem("PB", "d2", old8, &["PA"]), // 跨域 + 直接相连 → bridge 满分
            mem("C", "d1", &format!("{} {}", old8, new3), &[]),
        ]);
        let s = NtAwakenLoop::verify(&c, "C", &["PA".to_string(), "PB".to_string()]);
        assert!(
            s.total <= 1.0 + 1e-9,
            "total 不得超 1.0（clamp 必须永不生效），实得 {}",
            s.total
        );
        // base + bridge 的构造上界
        let base_max = NOVELTY_GROUND_PEAK / NOVELTY_GROUND_PEAK * BASE_WEIGHT;
        assert!(
            base_max + BRIDGE_MAX <= 1.0 + 1e-9,
            "构造上界须 ≤ 1.0：{}+{}",
            base_max,
            BRIDGE_MAX
        );
        // 该样本 novelty 峰值可达 1.0
        assert!(
            s.novelty > 0.9,
            "t=0.375 附近 novelty 应接近峰值，实得 {}",
            s.novelty
        );
    }

    /// 判别力回归：t 从 0 扫到尾，total 必须**非单调平坦**
    /// （旧实现在 t>0.85 一律 clamp 成 1.0，尾部完全无梯度）。
    #[test]
    fn discrimination_total_varies_across_t_sweep() {
        // 扫描范围说明：`old` 恒为 10 个词，故 t = n_new/(10+n_new)，
        // **t=1 在有旧词时不可达**。原扫描取 `0..=10`，其末端 n_new=10
        // 对应 t=0.5 —— 恰是抛物线峰值，却被断言「必须显著低于中段」
        // 并在注释里称作 "t=1" ⇒ 断言方向与实际曲线相反，永远失败。
        // 现扫到 n_new=30（t=0.75，越过峰值进入下降段），并把尾部与
        // **算出的峰值下标**比，而不是硬编码 5。
        const OLD_N: usize = 10;
        let mut totals = Vec::new();
        let mut ts = Vec::new();
        for n_new in 0..=30usize {
            let old: Vec<String> = (0..OLD_N).map(|i| format!("w{}", i)).collect();
            let new: Vec<String> = (0..n_new).map(|i| format!("n{}", i)).collect();
            let mut terms = old.clone();
            terms.extend(new);
            let c = cs(vec![
                mem("PA", "d1", &old.join(" "), &[]),
                mem("C", "d1", &terms.join(" "), &[]),
            ]);
            let s = NtAwakenLoop::verify(&c, "C", &["PA".to_string()]);
            totals.push(s.total);
            ts.push(n_new as f64 / (OLD_N + n_new) as f64);
        }
        let max = totals.iter().cloned().fold(f64::MIN, f64::max);
        let min = totals.iter().cloned().fold(f64::MAX, f64::min);
        assert!(
            max - min > 0.2,
            "t 扫描下 total 应有真实跨度（判别力），实得 max={:.3} min={:.3} 跨度={:.3}",
            max,
            min,
            max - min
        );
        // 峰值下标由 t 算出（t=0.5 处），不硬编码
        let peak = ts
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1 - 0.5)
                    .abs()
                    .partial_cmp(&(b.1 - 0.5).abs())
                    .unwrap()
            })
            .map(|(i, _)| i)
            .expect("扫描非空");
        let tail = totals.len() - 1;
        assert!(
            tail > peak,
            "扫描必须越过峰值才有下降段：peak={} tail={}",
            peak,
            tail
        );
        // 尾部不得饱和：越过峰值后必须回落到峰值以下
        assert!(
            totals[tail] < totals[peak] - 0.05,
            "纯编造尾部(t={:.2})必须低于峰值(t={:.2})，实得 尾部:{:.3} vs 峰值:{:.3}",
            ts[tail],
            ts[peak],
            totals[tail],
            totals[peak]
        );
    }

    /// `premises_pairwise_linked` 的三种形态 + 截断确定性。
    #[test]
    fn pairwise_linked_shapes() {
        let a = mem("A", "d1", "a", &["X"]);
        let b = mem("B", "d2", "b", &["X"]); // 共享邻居 X
        assert!(premises_pairwise_linked(&[&a, &b]));
        let c = mem("C", "d2", "c", &["Y"]);
        let d = mem("D", "d1", "d", &["Y"]);
        assert!(premises_pairwise_linked(&[&c, &d]));
        let e = mem("E", "d1", "e", &["Z"]);
        let f = mem("F", "d2", "f", &["W"]); // 互不相干
        assert!(!premises_pairwise_linked(&[&e, &f]));
        // 直接相连
        let g = mem("G", "d1", "g", &["H"]);
        let h = mem("H", "d2", "h", &[]);
        assert!(premises_pairwise_linked(&[&g, &h]));
        // 单条 / 空
        assert!(!premises_pairwise_linked(&[&a]));
        let empty: Vec<&Memory> = Vec::new();
        assert!(!premises_pairwise_linked(&empty));
    }

    /// 截断必须确定性：构造超多出边，重复调用结果一致。
    #[test]
    fn bridge_scan_truncation_is_deterministic() {
        let mut many: Vec<String> = (0..(BRIDGE_SCAN_CAP * 3))
            .map(|i| format!("M-{:06}", i))
            .collect();
        many.push("TARGET".to_string()); // 排在截断窗口之外
        many.sort();
        let a = mem(
            "A",
            "d1",
            "a",
            &many.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        );
        let b = mem("B", "d2", "b", &[]);
        let r1 = premises_pairwise_linked(&[&a, &b]);
        let r2 = premises_pairwise_linked(&[&a, &b]);
        assert_eq!(r1, r2, "截断后仍须确定性");
    }
}
