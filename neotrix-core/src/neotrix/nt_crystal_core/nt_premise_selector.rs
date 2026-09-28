//! NT-PREMISE-SELECTOR — 推理链前提选择（语义簇 · 确定性 · 反 monoculture）
//!
//! # 它修的缺陷
//!
//! 改造前 `NtAwakenLoop::cycle()` 的前提是**按位置**取的：
//! `ids.get(round % ids.len())`（本域列表第 N 条）与 `ids.get((round+1) % ids.len())`
//! （第 N+1 条）。而那个 `ids` 来自 `HashMap::values()` —— **顺序本身不确定**。
//! 于是每条链的前提既无语义关联（本域内两条任意记忆），又逐次运行都不同。
//!
//! 本模块把前提来源换成 `Adjacency` 的**入邻域**（`in_neighbors`）：锚点
//! （疾病档案）的入邻居就是它的全部 facet（pitfall / drug / summary …），
//! 天然跨域、天然同指 —— 这才是「前提」的定义。详见 `nt_graph_index`
//! 的模块文档（in-star 结构实测）。
//!
//! # 三条硬约束
//!
//! 1. **确定性**：每个输出列表都定序。理由有实据 —— 2026-09-28 的 id 碰撞
//!    事故中「每组哪条记忆存活」就是 `HashMap` 序随机造成的；`select` 的
//!    窗口偏移只由 `rotation` 决定，不看任何迭代序。
//! 2. **语义先于数量**：单轮 d≈3–5 条（`DEFAULT_PER_ROUND_CAP`），靠
//!    `facet_cluster` 的**域多样性**贪心取，而不是把整个入邻域塞进一条链。
//! 3. **反 monoculture**：入度 ≥ `MEGA_HUB_MAX_IN_DEGREE` 的巨型枢纽**不得**
//!    做种子。理由见 `Adjacency::specificity` 文档：入度 935 的判分锚点，
//!    经它取到的 925 个入邻居彼此毫无语义关联。
//!
//! # 种子策略（`resolve_seed`）
//!
//! 三级降级，全部定序：
//! 1. `Requested` —— 请求的 id 本身即合格锚点（入度 ∈ [门, 反枢纽闸]）。
//! 2. `LeafWalk` —— 请求的是叶子（入度 0）：沿**出边走一跳**。语料是
//!    in-star，叶子的出边终点就是它的锚点，故一跳足够；锚点出度 0，
//!    第 2 跳必空，故不走（这正是 `nt_graph_index` 要建双向索引的原因）。
//! 3. `HubFallback` —— 前两条不成立（id 不存在 / 是巨型枢纽 / 出边无合格
//!    终点）：按**特异度**取最优合格锚点。排序**不比较浮点**：
//!    `specificity = 1/ln(1+入度)` 对入度严格单调降，故直接比入度，
//!    精确、无 NaN 风险。
//!
//! # 降级路径（`PremiseSet::degraded`）
//!
//! 图内**完全没有**入边（新建 / 未连边的语料）时，`for_round` 退回
//! 「本域 id 定序后按轮次轮转」，且跨域节奏改用「别域 id 定序轮转」补一条
//! （保留旧 `cross_premise` 的**意图**但去掉它的 HashMap 序随机性）。
//!
//! 保留降级而非直接放弃的理由：此时不存在任何可利用的语义信息，若直接放弃，
//! 觉醒循环会**永久 0 提案**（静默停摆，`proposed == 0` 会被误读成
//! 「没有可推理的东西」）。且降级路径比旧代码**更确定**。
//!
//! 与验证器的分工不重叠：`nt_graph_index` 只给候选，裁决仍在
//! `NtAwakenLoop::verify` / `verify_robust` / `premise_necessity`。

use std::borrow::Cow;
use std::collections::HashMap;

use super::consciousness::{CrystalConsciousness, Memory, ReasoningType};
use super::nt_graph_index::Adjacency;

/// 单轮前提数上界（d≈3–5）。
///
/// 区间来自 MCMH 式多链证据的实践做法：单链给 3–5 条**互不重复**的前提，
/// 证据量靠多轮（`select_chains`）累加，而不是靠单轮堆更多条 —— 后者会把
/// 同一 facet 的近重复塞进同一条链，把 `verify` 的新颖度稀释掉。
/// 单轮前提数上界。
///
/// 2026-09-28 由 5 降为 **3**，依据是活库实测（`nt_verify_sim.py`，200 锚点 ×
/// 5 档 cap 扫描）：留一必要性算出的**搭便车前提**（|delta|<0.01，对结论无贡献）
/// 随 cap 单调上升 —— cap 2/3/4/5/8 → 0% / 34.5% / 42.8% / 46.8% / 51.9%。
/// 即 cap=5 时**近一半前提是凑数的**，它们只稀释信号、不增加信息。
/// 降到 3：搭便车率降 12 个点，而跨域率不变（各档均 99.0%），
/// total 中位数与标准差亦不变（0.995 / ~0.022）→ **无代价**。
pub const DEFAULT_PER_ROUND_CAP: usize = 3;

/// 规范锚点形状：入度 ≥ 8。
///
/// 实测（64,674 条记忆）：2,212 个节点入度 ≥ 8，最宽的合法病种枢纽入度 171。
/// 门取 8 而非 2：facet 数不足 8 的簇喂不出一条像样的链。低于此门不入
/// 锚点候选（可用 `with_min_anchor_in_degree` 放宽给小型/早期语料）。
pub const CANONICAL_ANCHOR_IN_DEGREE: usize = 8;

/// 反 monoculture 硬闸：入度 ≥ 此值者**不作为种子**。
///
/// 阈值 400，与 `nt_graph_index::live_graph_index` 的回归闸同阈：远高于任何
/// 语义正确的族/病种枢纽（实测最大 171），又远低于 2026-09-28 事故值 935。
/// 命中即说明有新吸收把整个域连到了单一锚点，此时**任何**经它取到的前提
/// 都不可信 —— 直接弃用，而不是降级使用。
pub const MEGA_HUB_MAX_IN_DEGREE: usize = 400;

/// 域内锚点轮转窗口。
///
/// 锚点按特异度取最优（= 入度最小者，见模块文档）后，若每轮都用同一个，
/// 逐轮前提会高度重复。故在前 `k` 个合格锚点间轮转 `rotation % k`。
pub const DEFAULT_ANCHOR_WINDOW: usize = 4;

/// 种子解析策略（`resolve_seed` 的三级降级）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedStrategy {
    /// 请求的 id 本身即合格锚点。
    Requested,
    /// 叶子（入度 0）沿出边走一跳到锚点（in-star 语料的规范跳数）。
    LeafWalk,
    /// 按特异度定序取的锚点：请求不可用时的兜底（`resolve_seed`），
    /// 或课程域直接选锚点（`seed_for` —— 那里没有具体 id 可请求）。
    HubFallback,
}

/// 一个已解析的种子：id + 怎么找到的 + 度数画像。
#[derive(Debug, Clone, PartialEq)]
pub struct Seed {
    pub id: String,
    pub strategy: SeedStrategy,
    pub in_degree: usize,
    pub out_degree: usize,
    /// `Adjacency::specificity` 的值（IDF 式，单调排序分非概率）。
    pub specificity: f64,
}

/// 一组候选前提 + 其链类型（MCMH 多链证据的一「链」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PremiseChain {
    pub premises: Vec<String>,
    pub chain_type: ReasoningType,
    /// 前提是否跨 ≥2 域（= `chain_type == CrossDomain`，冗余存一份便于
    /// 调用方不 import `ReasoningType` 就能判）。
    pub cross_domain: bool,
}

/// 一轮的前提选择结果。
#[derive(Debug, Clone, PartialEq)]
pub struct PremiseSet {
    /// 使用的种子（降级路径为 `None`）。
    pub seed: Option<Seed>,
    pub premises: Vec<String>,
    pub chain_type: ReasoningType,
    /// 是否走了无图降级路径（图内无任何入边）。
    pub degraded: bool,
}

/// 前提选择器。借用记忆表与邻接，**不复制** 6 万+ 记忆。
///
/// 生命周期设计：选择器只借用 `&HashMap<String, Memory>`，邻接则用 `Cow`
/// ——`with_adjacency` 借用调用方缓存的邻接（推荐），`new` 自建并持有。
/// 于是调用方可在同一轮里先选前提、再可变借用 `CrystalConsciousness`
/// 去 `reason()`（选择器在 `for_round` 返回后即失效，NLL 负责这一点）。
pub struct PremiseSelector<'m, 'a> {
    memories: &'m HashMap<String, Memory>,
    adj: Cow<'a, Adjacency>,
    min_anchor_in_degree: usize,
    anchor_window: usize,
    per_round_cap: usize,
}

impl<'m, 'a> PremiseSelector<'m, 'a> {
    /// 自建邻接（一次性；活库上是 O(V+E) ≈ 6.5 万条边，故调用方宜复用）。
    pub fn new(memories: &'m HashMap<String, Memory>) -> Self {
        Self {
            adj: Cow::Owned(Adjacency::from_memories(memories)),
            memories,
            min_anchor_in_degree: CANONICAL_ANCHOR_IN_DEGREE,
            anchor_window: DEFAULT_ANCHOR_WINDOW,
            per_round_cap: DEFAULT_PER_ROUND_CAP,
        }
    }

    /// 复用已有邻接（推荐：`NtAwakenLoop` 按记忆条数缓存邻接）。
    pub fn with_adjacency(memories: &'m HashMap<String, Memory>, adj: &'a Adjacency) -> Self {
        Self {
            adj: Cow::Borrowed(adj),
            memories,
            min_anchor_in_degree: CANONICAL_ANCHOR_IN_DEGREE,
            anchor_window: DEFAULT_ANCHOR_WINDOW,
            per_round_cap: DEFAULT_PER_ROUND_CAP,
        }
    }

    /// 从晶体意识取选择器（只需其 `memories`，自建邻接）。
    pub fn from_consciousness(consciousness: &'m CrystalConsciousness) -> Self {
        Self::new(&consciousness.memories)
    }

    /// 放宽/收紧锚点入度门（默认 `CANONICAL_ANCHOR_IN_DEGREE` = 8）。
    /// 小型或尚未连边的语料可下调；下调会拉低簇的质量，仅供早期语料。
    pub fn with_min_anchor_in_degree(mut self, k: usize) -> Self {
        self.min_anchor_in_degree = k;
        self
    }

    /// 域内锚点轮转窗口（默认 `DEFAULT_ANCHOR_WINDOW` = 4）。
    pub fn with_anchor_window(mut self, k: usize) -> Self {
        self.anchor_window = k;
        self
    }

    /// 单轮前提数上界（默认 `DEFAULT_PER_ROUND_CAP` = 3）。传 0 → 选不出前提。
    pub fn with_per_round_cap(mut self, k: usize) -> Self {
        self.per_round_cap = k;
        self
    }

    /// 该节点是否合格锚点：入度落在 [门, 反枢纽闸] 内。
    ///
    /// **不要求出度为 0**：语料里出度 0 是「典型」锚点形状（档案类），
    /// 但跨域桥记忆自己也可以有出边；把出度 0 当硬条件会误杀合法锚点。
    pub fn is_anchor(&self, id: &str) -> bool {
        let in_deg = self.adj.in_neighbors(id).len();
        in_deg >= self.min_anchor_in_degree && in_deg <= MEGA_HUB_MAX_IN_DEGREE
    }

    /// 该节点度数画像（邻接透传，便于调用方免于自建 `Adjacency`）。
    pub fn degree(&self, id: &str) -> (usize, usize) {
        self.adj.degree(id)
    }

    /// 合格锚点候选，已按**特异度降序**（= 入度升序，入度同则 id 升）定序。
    ///
    /// `domain = Some(d)` 限域（`NtAwakenLoop` 的课程选题是「域」粒度的）。
    /// 取入度升序而非降序是刻意的：入度越大越泛化，其入邻域越可能是
    /// 「一堆互不相干的同域/异域记忆」——见模块文档的反 monoculture 约束。
    pub fn anchors(&self, domain: Option<&str>) -> Vec<Seed> {
        let mut v: Vec<Seed> = Vec::new();
        for id in self.adj.hubs(self.min_anchor_in_degree) {
            if !self.is_anchor(&id) {
                continue;
            }
            if let Some(d) = domain {
                match self.domain_of(&id) {
                    Some(md) if md == d => {}
                    _ => continue,
                }
            }
            v.push(self.describe(&id, SeedStrategy::HubFallback));
        }
        v.sort_by(|a, b| a.in_degree.cmp(&b.in_degree).then_with(|| a.id.cmp(&b.id)));
        v
    }

    /// 解析种子：请求 → 叶子走一跳 → 定序兜底。
    ///
    /// 返回 `None` 只在一种情形：**全库没有任何合格锚点**（图内无入边，
    /// 或全库都是被反枢纽闸挡掉的巨型枢纽）。此时调用方应走降级路径，
    /// 不要退而拿巨型枢纽当种子 —— 那正是 2026-09-28 monoculture 事故的形状。
    pub fn resolve_seed(&self, requested: Option<&str>) -> Option<Seed> {
        if let Some(id) = requested {
            if self.is_anchor(id) {
                return Some(self.describe(id, SeedStrategy::Requested));
            }
            if self.adj.in_neighbors(id).is_empty() {
                // 叶子：出边终点就是它的锚点（in-star）。out_neighbors 已定序，
                // 故取第一个合格终点即为定序结果，无需再排。
                for t in self.adj.out_neighbors(id) {
                    if self.is_anchor(t) {
                        return Some(self.describe(t, SeedStrategy::LeafWalk));
                    }
                }
            }
        }
        self.anchors(None).into_iter().next()
    }

    /// 域内种子（课程域 → 锚点），在特异度最高的 `anchor_window` 个之间轮转
    /// `rotation`，以免逐轮取同一个锚点。域内无合格锚点时定序回全库兜底。
    pub fn seed_for(&self, domain: &str, rotation: usize) -> Option<Seed> {
        let ranked = self.anchors(Some(domain));
        if !ranked.is_empty() {
            let window = self.anchor_window.max(1);
            let idx = rotation % ranked.len().min(window);
            if let Some(s) = ranked.get(idx) {
                return Some(s.clone());
            }
        }
        self.anchors(None).into_iter().next()
    }

    /// 语义簇采样：锚点入邻域 → 域多样性贪心（`facet_cluster`）→ 跳前
    /// `rotation * cap` 条再取 `cap` 条。
    ///
    /// 跳前缀是关键：`facet_cluster` 返回的是**完整轮转序的前缀**，
    /// 只放大 `cap` 再截断会得到与第 0 轮**完全相同**的集合（每轮都在取
    /// 头几条）。显式 skip 才让第 k 轮取到簇的第 k 个窗口。
    pub fn select(&self, seed: &str, rotation: usize) -> Vec<String> {
        let cap = self.per_round_cap;
        if cap == 0 {
            return Vec::new();
        }
        let window = cap.saturating_mul(rotation.saturating_add(1));
        let (all, _cross) = self.adj.facet_cluster(seed, self.memories, window);
        all.into_iter()
            .skip(rotation.saturating_mul(cap))
            // 自环防御：`connect(x, x)` 会让锚点成为自己的入邻居，
            // 那种 id 绝不能当自己的前提。
            .filter(|id| id.as_str() != seed)
            .take(cap)
            .collect()
    }

    /// MCMH 式多链证据：第 k 组 = 簇采样的第 k 个窗口，故各组**天然不重叠**
    /// （直到簇被取尽）。`chains` 过大时后几组会重复/变空，故只返回非空组。
    pub fn select_chains(&self, seed: &str, chains: usize) -> Vec<PremiseChain> {
        let mut out: Vec<PremiseChain> = Vec::new();
        for k in 0..chains {
            let premises = self.select(seed, k);
            if premises.is_empty() {
                continue;
            }
            let cross = self.domains_of(&premises).len() >= 2;
            out.push(PremiseChain {
                chain_type: if cross {
                    ReasoningType::CrossDomain
                } else {
                    ReasoningType::Inductive
                },
                cross_domain: cross,
                premises,
            });
        }
        out
    }

    /// 一轮前提选择（`NtAwakenLoop::cycle` 的入口）。
    ///
    /// `prefer_cross` 为真时（循环的 `round % 4 == 3` 节奏）**尝试**保证跨域：
    /// - 图路径：簇内若已跨域则不动；否则补一条**与该簇真相关**的跨域桥
    ///   （`cross_bridge`）。补不上就诚实留 `Inductive` —— 宁可少一条跨域链，
    ///   也不把无关记忆硬塞进一条语义连贯的簇。
    /// - 降级路径：按 id 定序取一条别域记忆（保留旧 `cross_premise` 的意图，
    ///   去掉它的 `HashMap` 序随机性），保证无图语料也能产跨域链
    ///   （`update_phase` 的 `cross_domain >= 5` 门依赖它）。
    ///
    /// 返回 `None` 只表示「本轮一条前提都取不到」（空域 / 簇为空 / cap=0），
    /// 调用方应跳过本轮 —— 与改造前 `ids.is_empty() => continue` 同义。
    pub fn for_round(&self, domain: &str, round: usize, prefer_cross: bool) -> Option<PremiseSet> {
        if let Some(seed) = self.seed_for(domain, round) {
            let mut premises = self.select(&seed.id, round);
            if premises.is_empty() {
                return None;
            }
            if prefer_cross && self.domains_of(&premises).len() < 2 {
                if let Some(x) = self.cross_bridge(&seed.id, domain) {
                    // 桥可能与簇内某条同 id（锚点来自别域时的边缘情形）
                    if !premises.contains(&x) {
                        premises.push(x);
                    }
                }
            }
            let cross = self.domains_of(&premises).len() >= 2;
            return Some(PremiseSet {
                chain_type: if cross {
                    ReasoningType::CrossDomain
                } else {
                    ReasoningType::Inductive
                },
                seed: Some(seed),
                premises,
                degraded: false,
            });
        }
        // 降级：图内无入边（本域 id 定序轮转）
        let mut premises = self.domain_fallback(domain, round);
        if premises.is_empty() {
            return None;
        }
        if prefer_cross && self.domains_of(&premises).len() < 2 {
            if let Some(x) = self.foreign_domain_id(domain, round) {
                if !premises.contains(&x) {
                    premises.push(x);
                }
            }
        }
        let cross = self.domains_of(&premises).len() >= 2;
        Some(PremiseSet {
            chain_type: if cross {
                ReasoningType::CrossDomain
            } else {
                ReasoningType::Inductive
            },
            seed: None,
            premises,
            degraded: true,
        })
    }

    /// 跨域桥：取一条**与该簇真相关**的别域素材，优先簇内成员自身
    /// （它本来就属别域），退而取「簇内成员的出邻域终点」（跨域边）。
    /// 两者皆无 → `None`。
    pub fn cross_bridge(&self, seed: &str, home_domain: &str) -> Option<String> {
        for id in self.adj.in_neighbors(seed) {
            match self.domain_of(id) {
                Some(d) if d != home_domain => return Some(id.clone()),
                _ => continue,
            }
        }
        for id in self.adj.in_neighbors(seed) {
            for t in self.adj.out_neighbors(id) {
                if t == seed {
                    continue;
                }
                match self.domain_of(t) {
                    Some(d) if d != home_domain => return Some(t.clone()),
                    _ => continue,
                }
            }
        }
        None
    }

    /// 前提覆盖的域（去重 + 定序）。
    pub fn domains_of(&self, premise_ids: &[String]) -> Vec<String> {
        let mut v: Vec<String> = premise_ids
            .iter()
            .filter_map(|id| self.domain_of(id))
            .collect();
        v.sort();
        v.dedup();
        v
    }

    /// 降级取前提：本域记忆**按 id 定序**后按 `rotation` 轮转，取 `cap` 条。
    ///
    /// 为什么需要它：全无连接的语料里不存在任何可利用的语义信息，此时若
    /// 直接放弃，觉醒循环会永久 0 提案（静默停摆）。且此路径比改造前**更
    /// 确定** —— 旧代码按 `HashMap` 序取位置，逐次运行取到的两条都不同。
    pub fn domain_fallback(&self, domain: &str, rotation: usize) -> Vec<String> {
        let cap = self.per_round_cap;
        if cap == 0 {
            return Vec::new();
        }
        let mut ids: Vec<String> = self
            .memories
            .values()
            .filter(|m| m.domain == domain)
            .map(|m| m.id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        if ids.is_empty() {
            return Vec::new();
        }
        // rotate_left 会可变借用；`% ids.len()` 是不可变借用 → E0502。先取长度。
        let n_ids = ids.len();
        ids.rotate_left(rotation % n_ids);
        ids.truncate(cap);
        ids
    }

    /// 降级路径的别域素材：非本域记忆按 id 定序，按 `rotation` 轮转。
    pub fn foreign_domain_id(&self, home_domain: &str, rotation: usize) -> Option<String> {
        let mut ids: Vec<String> = self
            .memories
            .values()
            .filter(|m| m.domain != home_domain)
            .map(|m| m.id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        let idx = rotation % ids.len().max(1);
        ids.get(idx).cloned()
    }

    fn describe(&self, id: &str, strategy: SeedStrategy) -> Seed {
        let (out_degree, in_degree) = self.adj.degree(id);
        Seed {
            id: id.to_string(),
            strategy,
            in_degree,
            out_degree,
            specificity: self.adj.specificity(id),
        }
    }

    fn domain_of(&self, id: &str) -> Option<String> {
        self.memories.get(id).map(|m| m.domain.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::consciousness::MemoryType;

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

    /// in-star 夹具（边表勿凭印象改断言）：
    ///
    /// ```text
    /// 锚点 A（medical-disease，出度 0，入度 9）
    ///   clinical × 3  P1 P2 P3 → A     （P1 同时是叶子：入度 0、出度 1）
    ///   pharma    × 3  D1 D2 D3 → A
    ///   disease   × 3  S1 S2 S3 → A
    /// 锚点 S（medical-disease，出度 0，入度 9）：S4..S9 全为 disease → 单域簇
    /// 巨型枢纽 H（rubric，出度 0，入度 420）→ 越过反枢纽闸，永不做种子
    /// 孤立岛 I1 I2 I3（physics，互不连接）→ 任何锚点的簇里都不该出现
    /// ```
    ///
    /// H 用 420 > `MEGA_HUB_MAX_IN_DEGREE`(400) 构造，模拟 2026-09-28 事故
    /// 形状（rubric 全连单一判分锚点，925 份入邻域彼此无关）。
    fn fixture() -> HashMap<String, Memory> {
        let mut ms: HashMap<String, Memory> = HashMap::new();
        for (id, domain, content) in [
            ("P1", "medical-clinical", "pitfall one"),
            ("P2", "medical-clinical", "pitfall two"),
            ("P3", "medical-clinical", "pitfall three"),
            ("D1", "medical-pharma", "drug one"),
            ("D2", "medical-pharma", "drug two"),
            ("D3", "medical-pharma", "drug three"),
            ("S1", "medical-disease", "summary one"),
            ("S2", "medical-disease", "summary two"),
            ("S3", "medical-disease", "summary three"),
        ] {
            ms.insert(id.to_string(), mem(id, domain, content, &["A"]));
        }
        ms.insert(
            "A".to_string(),
            mem("A", "medical-disease", "profile asthma", &[]),
        );
        // S 必须与 A **同入度**（各 9）：`anchors_prefer_specific_over_generic`
        // 的判据是「入度同 → id 序」，且 `premises_stay_inside_the_anchor_cluster`
        // 依赖 S 能被 `for_round(.., 1, ..)` 轮转到。
        // 原为 `4..=9`（6 条）⇒ S 入度 6 < `CANONICAL_ANCHOR_IN_DEGREE`(8)
        // ⇒ S 被 `is_anchor` 直接滤掉，只剩 A —— 夹具与它自己注释里
        // 「A/S 入度 9」矛盾。生产门 8 是 64,674 条实测定的，不动。
        for i in 4..=12 {
            let id = format!("S{i}");
            ms.insert(
                id.clone(),
                mem(&id, "medical-disease", "summary same cluster", &["S"]),
            );
        }
        ms.insert(
            "S".to_string(),
            mem("S", "medical-disease", "profile copd", &[]),
        );
        for i in 0..420 {
            let id = format!("R{i:04}");
            ms.insert(id.clone(), mem(&id, "rubric", "rubric row", &["H"]));
        }
        ms.insert("H".to_string(), mem("H", "rubric", "grader anchor", &[]));
        for i in 1..=3 {
            let id = format!("I{i}");
            ms.insert(id.clone(), mem(&id, "physics", &format!("island {i}"), &[]));
        }
        ms
    }

    fn sel<'m, 'a>(ms: &'m HashMap<String, Memory>, adj: &'a Adjacency) -> PremiseSelector<'m, 'a> {
        PremiseSelector::with_adjacency(ms, adj)
    }

    #[test]
    fn seed_walks_leaf_to_its_anchor() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        // P1 是叶子（入度 0，出度 1 指向 A）
        assert_eq!(adj.degree("P1"), (1, 0), "P1 必须是叶子");
        let seed = s.resolve_seed(Some("P1")).expect("leaf must resolve");
        assert_eq!(seed.id, "A", "叶子的出边终点就是它的锚点");
        assert_eq!(seed.strategy, SeedStrategy::LeafWalk);
        // A 本身合格 → 原样返回
        let direct = s.resolve_seed(Some("A")).expect("A is an anchor");
        assert_eq!(direct.id, "A");
        assert_eq!(direct.strategy, SeedStrategy::Requested);
        assert!(direct.in_degree >= CANONICAL_ANCHOR_IN_DEGREE);
    }

    #[test]
    fn mega_hub_is_never_a_seed() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        assert!(
            adj.degree("H").1 > MEGA_HUB_MAX_IN_DEGREE,
            "夹具前提：H 必须越过反枢纽闸"
        );
        // 候选锚点里不得有 H
        let all = s.anchors(None);
        assert!(!all.is_empty(), "夹具必须有合格锚点");
        assert!(
            all.iter().all(|a| a.id != "H"),
            "巨型枢纽不得进锚点候选: {:?}",
            all.iter().map(|a| a.id.clone()).collect::<Vec<_>>()
        );
        // 显式点名 H 也必须被换掉（而不是盲用）
        let seed = s.resolve_seed(Some("H")).expect("必须有兜底锚点");
        assert_ne!(seed.id, "H");
        assert_eq!(seed.strategy, SeedStrategy::HubFallback);
        // 兜底取的是特异度最高（入度最小）者
        assert_eq!(seed.in_degree, 9, "A/S 入度 9 < 任何其他合格锚点");
        assert!(seed.specificity > 0.0);
    }

    #[test]
    fn anchors_prefer_specific_over_generic() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        let ranked: Vec<String> = s.anchors(None).into_iter().map(|a| a.id).collect();
        assert_eq!(
            ranked,
            vec!["A".to_string(), "S".to_string()],
            "入度同 → id 序"
        );
        // 域内筛选
        let dom: Vec<String> = s
            .anchors(Some("rubric"))
            .into_iter()
            .map(|a| a.id)
            .collect();
        assert!(dom.is_empty(), "rubric 域只剩巨型枢纽 → 无合格锚点");
    }

    #[test]
    fn premises_stay_inside_the_anchor_cluster() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        let set = s
            .for_round("medical-disease", 0, false)
            .expect("锚点 A 存在且簇非空");
        assert!(!set.degraded);
        let seed = set.seed.as_ref().expect("图路径必有种子");
        assert_eq!(seed.id, "A");
        assert!(!set.premises.is_empty());
        assert!(set.premises.len() <= DEFAULT_PER_ROUND_CAP);
        for p in &set.premises {
            assert!(
                adj.in_neighbors("A").contains(p),
                "前提 {p} 必须属于锚点 A 的入邻域（语义同指）"
            );
            assert!(!p.starts_with('I'), "孤立岛成员 {p} 不得成为前提");
        }
        // 轮转窗口内换一个锚点 → 前提随之换成 S 的簇
        let next = s.for_round("medical-disease", 1, false).expect("S 也有簇");
        let nseed = next.seed.as_ref().expect("图路径必有种子");
        assert_eq!(nseed.id, "S", "窗口内轮转到次优锚点");
        for p in &next.premises {
            assert!(
                adj.in_neighbors("S").contains(p),
                "前提 {p} 必须属于 S 的簇"
            );
        }
    }

    #[test]
    fn cross_domain_classification_follows_premise_domains() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        // A 的簇跨 3 域 → CrossDomain
        let a = s.for_round("medical-disease", 0, false).expect("A");
        assert_eq!(a.chain_type, ReasoningType::CrossDomain);
        let doms = s.domains_of(&a.premises);
        assert_eq!(
            doms,
            vec![
                "medical-clinical".to_string(),
                "medical-disease".to_string(),
                "medical-pharma".to_string()
            ]
        );
        // S 的簇单域 → Inductive
        let sp = s.for_round("medical-disease", 1, false).expect("S");
        assert_eq!(sp.chain_type, ReasoningType::Inductive);
        assert_eq!(
            s.domains_of(&sp.premises),
            vec!["medical-disease".to_string()]
        );
        // 显式跨域开关也不能把无关记忆塞进单域簇：补不上桥就诚实留 Inductive
        let spx = s.for_round("medical-disease", 1, true).expect("S");
        assert_eq!(
            spx.chain_type,
            ReasoningType::Inductive,
            "无真相关跨域素材 → 不硬塞"
        );
        for p in &spx.premises {
            assert!(
                adj.in_neighbors("S").contains(p),
                "前提 {p} 仍须属于 S 的簇"
            );
        }
    }

    #[test]
    fn chains_are_disjoint_and_cover_the_cluster() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        let chains = s.select_chains("A", 3);
        assert!(!chains.is_empty());
        // 第 k 组 = 簇的第 k 个窗口 → 组间不重叠
        let mut union: Vec<String> = Vec::new();
        for c in &chains {
            for p in &c.premises {
                assert!(
                    !union.contains(p),
                    "前提 {p} 在多链里重复 —— 证据量没有累加"
                );
                union.push(p.clone());
            }
        }
        // 3 组 × cap 5 ≥ 簇深 9 → 覆盖整个入邻域
        let mut want = adj.in_neighbors("A").to_vec();
        want.sort();
        union.sort();
        assert_eq!(union, want, "多链应恰好覆盖簇内全部 facet");
        // 单组不超过 cap
        for c in &chains {
            assert!(c.premises.len() <= DEFAULT_PER_ROUND_CAP);
        }
    }

    #[test]
    fn rotation_actually_moves_the_window() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        let r0 = s.select("A", 0);
        let r1 = s.select("A", 1);
        assert!(!r0.is_empty() && !r1.is_empty());
        assert_ne!(r0, r1, "第 k 轮必须取簇的不同窗口");
        assert!(r1.iter().all(|p| !r0.contains(p)), "窗口不重叠");
        // 深旋转不 panic，只是变短
        let deep = s.select("A", 99);
        assert!(deep.is_empty());
    }

    #[test]
    fn self_loop_never_becomes_its_own_premise() {
        let mut ms = fixture();
        // connect(x,x) 会让锚点成为自己的入邻居
        if let Some(a) = ms.get_mut("A") {
            a.connections.push("A".to_string());
        }
        let adj = Adjacency::from_memories(&ms);
        // 夹具前提：自环确实造出了「A 是自己的入邻居」，否则下面的断言是空的
        assert!(
            adj.in_neighbors("A").contains(&"A".to_string()),
            "自环未生效，测试将失去意义"
        );
        let s = sel(&ms, &adj);
        // 第 1 轮的窗口才覆盖到自环位置，必须逐个排掉
        assert!(adj.facet_cluster("A", &ms, 0).0.contains(&"A".to_string()));
        for r in 0..2 {
            for p in s.select("A", r) {
                assert_ne!(p, "A", "锚点不能当自己的前提（轮次 {r}）");
            }
        }
    }

    #[test]
    fn selection_is_deterministic_across_runs() {
        let ms = fixture();
        // 连跑两次：邻接重建 + 选择全流程，输出必须逐字相同。
        // 只取 2 个轮次：锚点窗口宽 2、簇深 9，第 2 轮起窗口会被取空
        // （`for_round` 返回 None），确定性验证不需要那些轮次。
        let run = || -> Vec<PremiseSet> {
            let adj = Adjacency::from_memories(&ms);
            let s = sel(&ms, &adj);
            (0..2)
                .map(|r| {
                    s.for_round("medical-disease", r, r % 2 == 1)
                        .expect("夹具必可选出")
                })
                .collect()
        };
        assert_eq!(run(), run(), "前提选择不得逐次运行漂移");
        // 显式多链同样确定
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        assert_eq!(s.select_chains("A", 4), s.select_chains("A", 4));
    }

    #[test]
    fn degrades_when_graph_has_no_in_edges() {
        // 全孤立语料（新建/未连边）→ 降级路径，仍须选出确定的前提
        let mut ms: HashMap<String, Memory> = HashMap::new();
        for (id, dom) in [("Z1", "d"), ("Z2", "d"), ("Z3", "d"), ("Y1", "other")] {
            ms.insert(id.to_string(), mem(id, dom, "content", &[]));
        }
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj);
        assert!(s.anchors(None).is_empty(), "无入边 → 无锚点");
        assert!(s.resolve_seed(Some("Z1")).is_none());
        let set = s.for_round("d", 0, false).expect("降级仍须给出前提");
        assert!(set.degraded);
        assert!(set.seed.is_none());
        assert_eq!(set.premises, vec!["Z1", "Z2", "Z3"], "域内 id 定序");
        assert_eq!(set.chain_type, ReasoningType::Inductive);
        // 跨域节奏：降级路径补一条别域记忆（id 定序）
        let cross = s.for_round("d", 1, true).expect("降级 + 跨域");
        assert_eq!(cross.chain_type, ReasoningType::CrossDomain);
        assert!(cross.premises.contains(&"Y1".to_string()));
        // 轮转确定性
        assert_eq!(
            s.domain_fallback("d", 1),
            vec!["Z2".to_string(), "Z3".to_string(), "Z1".to_string()]
        );
    }

    #[test]
    fn empty_and_missing_input_is_safe() {
        let empty: HashMap<String, Memory> = HashMap::new();
        let adj = Adjacency::from_memories(&empty);
        let s = sel(&empty, &adj);
        assert!(s.anchors(None).is_empty());
        assert!(s.anchors(Some("d")).is_empty());
        assert!(s.resolve_seed(None).is_none());
        assert!(s.resolve_seed(Some("nope")).is_none());
        assert!(s.select("nope", 0).is_empty());
        assert!(s.select_chains("nope", 3).is_empty());
        assert!(s.select_chains("nope", 0).is_empty());
        assert!(s.domain_fallback("d", 0).is_empty());
        assert!(s.foreign_domain_id("d", 0).is_none());
        assert!(s.cross_bridge("nope", "d").is_none());
        assert!(s.domains_of(&["nope".to_string()]).is_empty());
        assert_eq!(s.degree("nope"), (0, 0));
        assert!(!s.is_anchor("nope"));
        assert!(s.for_round("d", 0, false).is_none());
        assert!(s.for_round("d", 0, true).is_none());
    }

    #[test]
    fn cap_zero_yields_nothing() {
        let ms = fixture();
        let adj = Adjacency::from_memories(&ms);
        let s = sel(&ms, &adj).with_per_round_cap(0);
        assert!(s.select("A", 0).is_empty());
        assert!(s.select_chains("A", 2).is_empty());
        assert!(s.for_round("medical-disease", 0, false).is_none());
        // 降级路径同样为空
        let e: HashMap<String, Memory> = HashMap::new();
        let ea = Adjacency::from_memories(&e);
        assert!(sel(&e, &ea)
            .with_per_round_cap(0)
            .domain_fallback("d", 0)
            .is_empty());
    }

    #[test]
    fn lowered_anchor_gate_widens_candidates() {
        let mut ms: HashMap<String, Memory> = HashMap::new();
        // 小簇锚点：入度 3 < 规范门 8
        for (id, dom) in [("c1", "x"), ("c2", "y"), ("c3", "y")] {
            ms.insert(id.to_string(), mem(id, dom, "facet", &["h"]));
        }
        ms.insert("h".to_string(), mem("h", "x", "small anchor", &[]));
        let adj = Adjacency::from_memories(&ms);
        let strict = sel(&ms, &adj);
        assert!(
            strict.anchors(None).is_empty(),
            "入度 3 < 规范门 8 → 不是锚点"
        );
        let loose = sel(&ms, &adj).with_min_anchor_in_degree(3);
        let ranked: Vec<String> = loose.anchors(None).into_iter().map(|a| a.id).collect();
        assert_eq!(ranked, vec!["h".to_string()]);
        let set = loose.for_round("x", 0, false).expect("放宽门后可选取");
        assert_eq!(set.seed.map(|s| s.id), Some("h".to_string()));
        assert!(set.premises.len() >= 2, "应取到整个小簇");
    }

    #[test]
    fn builds_from_consciousness() {
        let mut c = CrystalConsciousness::new("t");
        let a = c.remember("profile asthma", MemoryType::Fact, "medical-disease", 0.9);
        let mut ids: Vec<String> = Vec::new();
        for i in 0..8 {
            let domain = if i % 2 == 0 {
                "medical-clinical"
            } else {
                "medical-pharma"
            };
            let id = c.remember(&format!("facet {i}"), MemoryType::Fact, domain, 0.8);
            c.connect(&id, &a);
            ids.push(id);
        }
        let s = PremiseSelector::from_consciousness(&c);
        let seed = s.seed_for("medical-disease", 0).expect("a 应为锚点");
        assert_eq!(seed.id, a);
        assert_eq!(seed.in_degree, 8);
        let set = s
            .for_round("medical-disease", 0, false)
            .expect("应选出前提");
        assert_eq!(set.chain_type, ReasoningType::CrossDomain, "facet 跨两域");
        for p in &set.premises {
            assert!(ids.contains(p), "前提 {p} 必须来自 a 的入邻域");
        }
    }
}
