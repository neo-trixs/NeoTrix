//! 晶体记忆图 — 双向邻接索引（推理链「前提选择」的图基础）
//!
//! # 为什么必须是**双向**
//!
//! 2026-09-28 实测（64,674 条记忆的全库统计）推翻了「沿 `connections` 正向
//! 多跳取前提」的直觉：
//!
//! ```text
//! profile (锚点)   出度 0    入度 39     ← facet 全部指向锚点
//! pitfall/drug/…   出度 1
//! 全库 max degree = 2   中位度 1
//! ```
//!
//! 正向走 1 跳到锚点即**死路**（锚点出度 0），第 2 跳得 0 节点。
//! 故前提候选只能取 **in-neighbors**（谁指向我）。
//! 反向结构恰好理想：锚点的入邻域 = 该主题全部 facet，天然跨域。
//!
//! 另注：`connected_ratio`（有连接占比）曾被当作「图已就绪」的证据，那是错的
//! —— 85.1% 连通但 max degree 仅 2，结构上近乎无用。**连通率不等于可用图。**
//!
//! # 确定性是硬要求
//!
//! 本索引的每个输出都排序。理由有实据：`HashMap` 迭代序不确定，而
//! `sync_to_consciousness` 正是按 HashMap 序 `insert(id, mem)` 覆盖，
//! 2026-09-28 那次 id 碰撞事故里「每组哪条记忆存活」就是随机的。
//! 索引若不确定，会把同类随机性引入前提选择。
//!
//! # 与既有验证器的关系（不重复造）
//!
//! 前提「够不够格」不由本索引裁决，交 `NtAwakenLoop`：
//!   - `verify`            四分量链分（novelty/grounding/bridge）
//!   - `verify_robust`     子集稳健（两半前提都须达标，剔「只在一半前提上成立」的捷径）
//!   - `premise_necessity` 留一必要性（delta≈0 = 搭便车前提）
//! 本索引只负责「**给候选**」，裁决留给验证器 —— 职责不重叠。

use std::collections::{HashMap, HashSet};

use super::consciousness::Memory;

/// 双向邻接索引。`out` = 本节点指向谁；`inn` = 谁指向本节点。
///
/// 不落盘：由 `CrystalConsciousness::rebuild_adjacency()` 在 load 后重建
/// （与 `connected_count` 同策略：派生状态不入盘，避免与 memories 不同步）。
#[derive(Debug, Clone, Default)]
pub struct Adjacency {
    out: HashMap<String, Vec<String>>,
    inn: HashMap<String, Vec<String>>,
}

impl Adjacency {
    /// 从记忆集合构建。O(V+E)；悬空目标**保留在 out 里**（便于诊断），
    /// 但不进 `inn` 的任何统计——悬挂边是数据缺陷，不该被当作图结构。
    pub fn from_memories(memories: &HashMap<String, Memory>) -> Self {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        let mut inn: HashMap<String, Vec<String>> = HashMap::new();
        for (id, m) in memories {
            let mut targets: Vec<String> = m
                .connections
                .iter()
                .filter(|t| memories.contains_key(*t))
                .cloned()
                .collect();
            targets.sort();
            targets.dedup();
            if targets.is_empty() {
                continue;
            }
            for t in &targets {
                inn.entry(t.clone()).or_default().push(id.clone());
            }
            out.insert(id.clone(), targets);
        }
        for v in inn.values_mut() {
            v.sort();
            v.dedup();
        }
        Adjacency { out, inn }
    }

    /// 本节点指向的目标（已排序）。无则空切片。
    pub fn out_neighbors(&self, id: &str) -> &[String] {
        self.out.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 谁指向本节点（已排序）—— 前提候选的**唯一正确来源**。
    pub fn in_neighbors(&self, id: &str) -> &[String] {
        self.inn.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// (出度, 入度)
    pub fn degree(&self, id: &str) -> (usize, usize) {
        (self.out_neighbors(id).len(), self.in_neighbors(id).len())
    }

    /// 边总数（去重后、仅计双方都在库内的）。
    pub fn edge_count(&self) -> usize {
        self.out.values().map(|v| v.len()).sum()
    }

    /// 跨域边数 —— `update_phase` 的 `cross_domain >= 5` 门的可测代理量。
    ///
    /// 口径：一条边若两端 `Memory.domain` 不同则计 1。与相位判定同源
    /// （相位按 `chain_type == CrossDomain` 数**链**，此处数**边**；
    /// 边是链的必要素材，边为 0 则链必为 0，故这是可先行的下界指标）。
    pub fn cross_domain_edges(&self, memories: &HashMap<String, Memory>) -> usize {
        let mut n = 0usize;
        for (id, targets) in &self.out {
            let Some(src) = memories.get(id) else {
                continue;
            };
            for t in targets {
                if let Some(dst) = memories.get(t) {
                    if src.domain != dst.domain {
                        n += 1;
                    }
                }
            }
        }
        n
    }

    /// 入邻域按**域多样性**贪心采样（MCMH 多链证据 d≈3–5 的依据：
    /// 多样性优于数量——多条同域近重复前提提供的信息远少于多条跨域前提）。
    ///
    /// 每域先取 1 条（按 id 序，确定性），再按剩余域轮转补足到 `cap`。
    /// 返回 (选中的 id, 是否跨 ≥2 域)。`cap == 0` 视作不限。
    pub fn facet_cluster(
        &self,
        anchor: &str,
        memories: &HashMap<String, Memory>,
        cap: usize,
    ) -> (Vec<String>, bool) {
        let cands = self.in_neighbors(anchor);
        // 按域分桶，桶内按 id 序（in_neighbors 已排序，故保持即可）
        let mut buckets: Vec<(String, Vec<String>)> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        for id in cands {
            let Some(m) = memories.get(id) else { continue };
            let d = m.domain.clone();
            match index.get(&d).copied() {
                Some(i) => {
                    if let Some(b) = buckets.get_mut(i) {
                        b.1.push(id.clone());
                    }
                }
                None => {
                    index.insert(d.clone(), buckets.len());
                    buckets.push((d, vec![id.clone()]));
                }
            }
        }
        // 域内按「谁指向我」的入邻域大小升序（入邻域小 = 更专属 = 信息量高），
        // 同值再按 id 序 —— 保证确定性。
        for (_d, ids) in buckets.iter_mut() {
            ids.sort_by(|x, y| {
                self.in_neighbors(x)
                    .len()
                    .cmp(&self.in_neighbors(y).len())
                    .then_with(|| x.cmp(y))
            });
        }
        buckets.sort_by(|a, b| a.0.cmp(&b.0)); // 域序确定 → 输出确定
        let mut out: Vec<String> = Vec::new();
        let mut round = 0usize;
        loop {
            let before = out.len();
            for (_d, ids) in &buckets {
                if let Some(id) = ids.get(round) {
                    out.push(id.clone());
                }
            }
            if out.len() == before {
                break;
            }
            round += 1;
            if cap > 0 && out.len() >= cap {
                break;
            }
        }
        if cap > 0 && out.len() > cap {
            out.truncate(cap);
        }
        let cross = buckets.len() >= 2;
        (out, cross)
    }

    /// 诊断快照：总度数（出度+入度）分布 + 枢纽。
    ///
    /// **必须以 `memories` 的键为准迭代，不能只遍历 `out`**：锚点出度为 0
    /// 根本不会成为 `out` 的键，只遍历 `out` 会**静默漏掉全部锚点**
    /// （实测语料 2194 个锚点，正是最该被度分布统计到的那批）。
    /// 本函数只负责含 out 边的节点，入边边数用 `edge_count` / `cross_domain_edges`。
    pub fn degree_histogram(&self, memories: &HashMap<String, Memory>) -> HashMap<usize, usize> {
        let mut h: HashMap<usize, usize> = HashMap::new();
        for id in memories.keys() {
            let (o, i) = self.degree(id);
            *h.entry(o + i).or_insert(0) += 1;
        }
        h
    }

    /// 边信息量（IDF 式特异度）：`1 / ln(1 + 入度(目标))`。
    ///
    /// 值域 `(0, +∞)`（入度 0 → 1.0；入度 1 → 1.44；入度 935 → 0.13）。
    /// 它是**单调排序分**不是概率，只用于「同一域内优先挑更专属的候选」这一相对次序。
    ///
    /// 为何需要：实测活库有**入度 935 的巨型枢纽**（`M-062005` = 判分器锚点，
    /// 925 份 rubric 全指向它）。这类节点是**通用连接件**——经它取到的 925 个
    /// 入邻居彼此**毫无语义关联**（925 个不同疾病族的环境）。若按入邻居数直接
    /// 采样，会大量产出「前提同指率极低」的垃圾链。
    /// 度数即泛化度：入度越大越泛化 → 特异度越低 → 采样时排后。
    /// 这是 IR 里 idf 的同构用法（高频词信息量低）。
    pub fn specificity(&self, target: &str) -> f64 {
        let n = self.in_neighbors(target).len() as f64;
        if n <= 0.0 {
            return 1.0;
        }
        1.0 / (1.0 + n).ln()
    }

    /// 巨型枢纽诊断：入度 ≥ `k` 的节点（按入度降序，入度同则按 id 序）。
    /// 供 ops 工具/评审发现「一个锚点吞掉整个域」这类 monoculture。
    pub fn mega_hubs(&self, k: usize) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .inn
            .iter()
            .filter(|(_, t)| t.len() >= k)
            .map(|(id, t)| (id.clone(), t.len()))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// 孤立节点数（出度 0 且入度 0）。
    pub fn isolated_count(&self, memories: &HashMap<String, Memory>) -> usize {
        memories
            .keys()
            .filter(|id| self.degree(id).0 == 0 && self.degree(id).1 == 0)
            .count()
    }

    /// 入度 ≥ `k` 的节点（= 枢纽）。当前语料实测为 0（max in-degree 即锚点）。
    pub fn hubs(&self, k: usize) -> Vec<String> {
        let mut v: Vec<String> = self
            .inn
            .iter()
            .filter(|(_, t)| t.len() >= k)
            .map(|(id, _)| id.clone())
            .collect();
        v.sort();
        v
    }

    /// 去重后的域集合（确定性序）。
    pub fn domains(&self, memories: &HashMap<String, Memory>) -> Vec<String> {
        let mut s: HashSet<String> = HashSet::new();
        for m in memories.values() {
            s.insert(m.domain.clone());
        }
        let mut v: Vec<String> = s.into_iter().collect();
        v.sort();
        v
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

    /// 真实语料形状：锚点出度 0、入度 N；facet 出度 1 指向锚点。
    ///
    /// 夹具边表（in-star 核对用，勿凭印象改断言）：
    ///   P1→A  P2→A  D1→A  S1→A（同域）  R1→A  R1→B
    /// 故 A 出度 0 / 入度 5（P1,P2,D1,S1,R1）；B 出度 0 / 入度 1（R1）。
    fn in_star() -> HashMap<String, Memory> {
        let mut m = HashMap::new();
        m.insert(
            "A".into(),
            mem("A", "medical-disease", "profile asthma", &[]),
        );
        m.insert(
            "P1".into(),
            mem("P1", "medical-clinical", "pitfall 1", &["A"]),
        );
        m.insert(
            "P2".into(),
            mem("P2", "medical-clinical", "pitfall 2", &["A"]),
        );
        m.insert("D1".into(), mem("D1", "medical-pharma", "drug 1", &["A"]));
        m.insert(
            "S1".into(),
            mem("S1", "medical-disease", "summary 1", &["A"]),
        );
        // 跨病边：B 是第二个锚点，R1 同时指向 A 和 B
        m.insert("B".into(), mem("B", "medical-disease", "profile copd", &[]));
        m.insert(
            "R1".into(),
            mem("R1", "medical-clinical", "related", &["A", "B"]),
        );
        m
    }

    #[test]
    fn in_star_shape_is_detected() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        // 锚点：出度 0、入度 5 —— 正向走会死路，故必须走反向
        assert_eq!(a.degree("A"), (0, 5), "anchor must be in-star");
        assert_eq!(a.out_neighbors("A").len(), 0);
        assert_eq!(a.in_neighbors("A"), &["D1", "P1", "P2", "R1", "S1"]);
        // facet 出度 1；B 出度 0 入度 1（被 R1 指向）
        assert_eq!(a.degree("P1"), (1, 0));
        assert_eq!(a.degree("B"), (0, 1));
    }

    #[test]
    fn forward_walk_dead_ends_but_in_walk_reaches_cluster() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        // 正向：从 P1 走 1 跳到 A，A 无出边 → 2 跳必空（这正是要建双向的理由）
        let fwd1 = a.out_neighbors("P1");
        assert_eq!(fwd1, &["A"]);
        assert!(
            a.out_neighbors(fwd1[0].as_str()).is_empty(),
            "正向第 2 跳必须为空"
        );
        // 反向：从 A 走入邻域拿到整簇 5 条
        assert_eq!(a.in_neighbors("A").len(), 5);
    }

    #[test]
    fn facet_cluster_is_domain_diverse_and_deterministic() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        let (sel, cross) = a.facet_cluster("A", &ms, 0);
        assert!(cross, "3 个域 → 跨域");
        // 确定性：连跑两次完全一致（HashMap 序不确定 → 必须靠排序兜住）
        let (sel2, _) = a.facet_cluster("A", &ms, 0);
        assert_eq!(sel, sel2);
        // 域序 medical-clinical < medical-disease < medical-pharma，
        // round0 每域取 1 → 前 3 条必为 [P1(clinical), S1(disease), D1(pharma)]
        assert_eq!(&sel[..3], &["P1", "S1", "D1"], "域多样性贪心应每域先取 1");
        let d0: HashSet<String> = sel
            .iter()
            .take(3)
            .filter_map(|i| ms.get(i).map(|m| m.domain.clone()))
            .collect();
        assert_eq!(d0.len(), 3, "前 3 条应覆盖 3 个域，实得 {:?}", sel);
        assert_eq!(sel.len(), 5, "cap=0 → 全取 5 条");
        // cap 生效
        let (capped, _) = a.facet_cluster("A", &ms, 2);
        assert_eq!(capped.len(), 2);
    }

    #[test]
    fn cross_domain_edges_counted() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        // 跨域边：P1→A(clinical→disease), P2→A, D1→A(pharma→disease), S1→A(同域不计), R1→A, R1→B
        assert_eq!(a.cross_domain_edges(&ms), 5, "S1→A 同域应被排除");
    }

    #[test]
    fn dangling_targets_excluded_from_structure() {
        let mut ms = in_star();
        ms.insert(
            "X".into(),
            mem("X", "medical-clinical", "dangling", &["NOPE"]),
        );
        let a = Adjacency::from_memories(&ms);
        // 悬空目标保留在 out（可诊断），但不进 inn
        assert_eq!(a.out_neighbors("X"), &[] as &[String]);
        assert!(a.in_neighbors("NOPE").is_empty());
        assert_eq!(a.degree("X"), (0, 0));
        // 仅 X 孤立：B 虽出度 0，但有入边 R1→B
        assert_eq!(a.isolated_count(&ms), 1);
    }

    #[test]
    fn hubs_and_histogram() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        assert_eq!(a.hubs(3), vec!["A".to_string()], "A 入度 5");
        assert!(a.hubs(6).is_empty(), "无入度 ≥6 者");
        let h = a.degree_histogram(&ms);
        // 8 节点：A: 0+5=5；R1: 2+0=2；P1,P2,D1,S1,B: 1
        assert_eq!(h.get(&5).copied(), Some(1), "A 总度 5");
        assert_eq!(h.get(&2).copied(), Some(1), "R1 总度 2");
        assert_eq!(h.get(&1).copied(), Some(5), "P1,P2,D1,S1,B 各 1");
        assert_eq!(h.values().sum::<usize>(), ms.len(), "直方图须覆盖全部节点");
        assert_eq!(a.edge_count(), 6); // P1,P2,D1,S1→A(4) + R1→A,B(2)
        assert_eq!(a.domains(&ms).len(), 3);
    }

    #[test]
    fn specificity_demotes_generic_hubs() {
        let ms = in_star();
        let a = Adjacency::from_memories(&ms);
        // A 入度 5（较泛化）；B 入度 1（专属）
        let sa = a.specificity("A");
        let sb = a.specificity("B");
        assert!(sa < sb, "入度大的枢纽特异度必须更低: A={} B={}", sa, sb);
        // 值域 (0,+∞)：入度 0→1.0，入度 1→1/ln2≈1.44，入度 5→1/ln6≈0.56
        assert!(sa > 0.0 && sb > 0.0, "特异度为正");
        assert!(
            a.specificity("B") > 1.0,
            "入度 1 时 1/ln2≈1.44 > 1（排序分非概率）"
        );
        assert_eq!(a.specificity("nonexistent"), 1.0, "无入边者按最专属处理");
        // mega_hubs 按入度降序
        assert_eq!(a.mega_hubs(3), vec![("A".to_string(), 5)]);
        assert!(a.mega_hubs(99).is_empty());
    }

    #[test]
    fn empty_input_is_safe() {
        let a = Adjacency::from_memories(&HashMap::new());
        assert_eq!(a.edge_count(), 0);
        assert!(a.out_neighbors("nope").is_empty());
        assert!(a.in_neighbors("nope").is_empty());
        assert_eq!(a.degree("nope"), (0, 0));
        assert_eq!(a.cross_domain_edges(&HashMap::new()), 0);
        assert!(a.hubs(1).is_empty());
        assert!(a.domains(&HashMap::new()).is_empty());
    }
}

/// 活库验收（默认忽略）：`cargo test -p neotrix --lib live_graph_index -- --ignored --nocapture`
///
/// 一举两得：
///   1. **证明 `CocoonStore::load()` 真能吃下活库**（跨会话未竟的 Rust 侧实证）；
///   2. 在**真实 6 万+ 记忆**上验证本索引的设计前提（in-star / 跨域边 / 域多样性）。
///
/// 只读，不落盘。
#[cfg(test)]
mod live {
    use super::*;
    use crate::neotrix::nt_crystal_core::cocoons::CocoonStore;

    #[test]
    #[ignore = "reads real 52MB cocoons.json, seconds"]
    fn live_graph_index() {
        let store = CocoonStore::load();
        let stats = store.stats();
        let mut all: HashMap<String, Memory> = HashMap::new();
        for c in store.cocoons.values() {
            for m in &c.memories {
                all.insert(m.id.clone(), m.clone());
            }
        }
        eprintln!(
            "[live] cocoons={} memories={} unique={} total_recalled={}",
            stats.cocoon_count,
            all.len(),
            stats.total_memories,
            all.len()
        );
        assert!(!all.is_empty(), "CocoonStore::load() 返回空 —— 库未被解析");
        assert_eq!(
            all.len(),
            stats.total_memories,
            "落盘条数与 stats 必须一致（否则有重号被 HashMap 覆盖）"
        );

        let a = Adjacency::from_memories(&all);
        let hist = a.degree_histogram(&all);
        let mut hv: Vec<(usize, usize)> = hist.into_iter().collect();
        hv.sort_unstable();
        eprintln!("[live] degree_hist(total)={:?}", hv);
        eprintln!(
            "[live] edges={} isolated={} hubs(in>=8)={} cross_domain_edges={} domains={}",
            a.edge_count(),
            a.isolated_count(&all),
            a.hubs(8).len(),
            a.cross_domain_edges(&all),
            a.domains(&all).len()
        );

        // 设计前提 1：连通率高但枢纽稀缺（连通率 ≠ 可用图）
        let connected = all.len() - a.isolated_count(&all);
        eprintln!(
            "[live] connected_ratio={:.3}",
            connected as f64 / all.len() as f64
        );

        // 设计前提 2：存在「出度 0、入度 ≫1」的锚点，且其入邻域跨域
        let hub_ids = a.hubs(8);
        let anchors: Vec<String> = hub_ids
            .iter()
            .filter(|id| a.out_neighbors(id).is_empty())
            .cloned()
            .collect();
        assert!(!anchors.is_empty(), "未找到出度 0 的锚点 —— 结构与实测不符");
        let anchor: &str = anchors.first().map(|s| s.as_str()).unwrap_or("");
        let (sel, cross) = a.facet_cluster(anchor, &all, 0);
        let doms: std::collections::BTreeSet<String> = sel
            .iter()
            .filter_map(|i| all.get(i).map(|m| m.domain.clone()))
            .collect();
        eprintln!(
            "[live] anchor={} in_deg={} cluster={} domains={:?} cross={}",
            anchor,
            a.in_neighbors(anchor).len(),
            sel.len(),
            doms,
            cross
        );
        assert!(cross, "锚点入邻域应跨 ≥2 域");
        assert!(sel.len() >= 2, "簇过小，无法构成前提集");

        // 回归闸：禁止 monoculture 复活。2026-09-28 曾因 RL 蒸馏把 925 份
        // rubric 全连到同一判分锚点，致该锚点入度 935（通用连接件 → 经它取到的
        // 入邻居彼此无语义关联 → 前提同指率崩）。修复后 max in-degree = 171。
        // 阈值取 400：远高于任何「语义正确的族/病种枢纽」（实测最大 171），
        // 又远低于曾经的 935，故既能捕获同类回归又不会误报。
        let mega = a.mega_hubs(400);
        eprintln!("[live] mega_hubs(>=400)={:?}", mega);
        assert!(
            mega.is_empty(),
            "检出 monoculture 枢纽 {:?} —— 检查新吸收是否又把整个域连到单一锚点",
            mega
        );
        let top = a.mega_hubs(1);
        let (widest, wn) = top.first().cloned().unwrap_or((String::new(), 0));
        eprintln!("[live] widest_hub={} in_degree={}", widest, wn);
    }
}
