//! # 能力网融合算法 (Capability Network Fusion)
//!
//! 目标: 把"两套并行能力网"统一为单一事实源 — 能力树 crate 注册表
//! (`.neotrix/capability_registry.json`) 吸收 `CapabilityNode` trait 实现节点
//! (l8/l9/l10 中 9 个实现, node_id 全部不在注册表中)。
//!
//! 操作:
//! 1. `fuse_trait_nodes`: trait 节点吸收进注册表 (node_id 归一化 + 挂载)。
//! 2. `dedup_edges`: 去除重复依赖边。
//! 3. `fix_domain_mismatch`: 修正域错配 (以 trait 自身 node_id 域为准)。
//! 4. `centrality`: 融合图上计算度中心性 → 关键枢纽节点 (喂给意识核心 soil)。
//!
//! 设计蓝本: `docs/1-DESIGN/2026-08-14-quantum-state-link-capability-network-fusion.md` §2.3

use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer, RuneSocket, EvolutionOp, EvolutionLogEntry};
use crate::registry::CapabilityRegistry;

/// 外部 trait 能力节点描述 — 由 `CapabilityNode` trait 实现的 node_id/provides 提取。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitNodeDescriptor {
    /// 原始 node_id (trait 实现者提供)
    pub raw_id: String,
    /// 提供的能力标签
    pub provides: Vec<String>,
    /// 星座等级 (0-6)
    pub constellation: u8,
    /// 域 (从 raw_id 或显式给出)
    pub domain: Option<Domain>,
}

/// 融合报告。
#[derive(Debug, Clone, Default)]
pub struct FusionReport {
    /// 吸收的 trait 节点数
    pub absorbed: usize,
    /// 因冲突跳过 (注册表已存在)
    pub skipped_conflict: usize,
    /// 去除的重复边数
    pub deduped_edges: usize,
    /// 修正的域错配数
    pub fixed_domain_mismatch: usize,
    /// 融合后节点总数
    pub total_nodes_after: usize,
    /// 度中心性 top-5 节点 (id, degree)
    pub top_centrality: Vec<(String, usize)>,
    /// 检测到的孤立 trait 节点 (未挂载消费者)
    pub orphan_trait_nodes: Vec<String>,
    /// 统计信息
    pub stats: String,
}

impl CapabilityRegistry {
    /// 域 → 短名 (归一化前缀用): Core→"core", Mind→"mind", ...
    fn domain_slug(domain: Domain) -> &'static str {
        match domain {
            Domain::Core => "core",
            Domain::Mind => "mind",
            Domain::Memory => "memory",
            Domain::World => "world",
            Domain::Act => "act",
            Domain::Shield => "shield",
            Domain::Io => "io",
            Domain::Meta => "meta",
            Domain::Nexus => "nexus",
            Domain::Governance => "governance",
            Domain::Repair => "repair",
        }
    }

    /// 归一化 trait node_id 为注册表命名: `nt_<domain>::<module>::<cap>`。
    /// 规则: 保留原始 id 的"模块段 + 能力段", 域前缀统一为 NT 域小写。
    pub fn normalize_trait_node_id(raw: &str, domain: Domain) -> String {
        let slug = Self::domain_slug(domain);
        let cleaned = raw.replace("::", "_");
        if cleaned.contains(&format!("nt_{}", slug)) {
            cleaned
        } else {
            format!("nt_{}::{}", slug, cleaned)
        }
    }

    /// 融合 trait 能力节点进注册表。
    ///
    /// - 已存在 (id 冲突) → 跳过并计数 (保留注册表现状优先)。
    /// - 不存在 → 注册为 Composite (L1), constellation 取真实值,
    ///   evolution_log 记录吸收来源。
    /// - 吸收后自动执行去重边 + 域错配修正。
    pub fn fuse_trait_nodes(&mut self, descriptors: &[TraitNodeDescriptor], cycle: &str) -> FusionReport {
        let mut report = FusionReport::default();
        for d in descriptors {
            let domain = d.domain.unwrap_or(Domain::Core);
            let normalized = Self::normalize_trait_node_id(&d.raw_id, domain);
            if self.nodes.contains_key(&normalized) {
                report.skipped_conflict += 1;
                continue;
            }
            let mut node = CapabilityNode::new_composite(
                normalized.clone(),
                domain,
                NodeLayer::L1Composite,
                d.provides.clone(),
                vec![],
            );
            node.constellation = Self::constellation_from_u8(d.constellation);
            // rune 槽: 给每个吸收节点分配 Indigo (变换) + Alabaster (监控)
            node.rune_sockets = vec![RuneSocket::Indigo, RuneSocket::Alabaster];
            node.record_evolution(EvolutionLogEntry {
                cycle: cycle.to_string(),
                op: EvolutionOp::Grafting,
                from_nodes: vec![d.raw_id.clone()],
                to_node: Some(normalized.clone()),
                note: "trait 能力节点融合吸收".into(),
                timestamp: chrono::Utc::now(),
            });
            if let Err(e) = self.register(node) {
                eprintln!("[fusion] register skipped {}: {}", normalized, e);
                report.skipped_conflict += 1;
                continue;
            }
            report.absorbed += 1;
        }

        // 去重边 + 域错配修正
        let dedup = self.dedup_duplicate_edges();
        report.deduped_edges = dedup;
        let fixed = self.fix_domain_mismatches();
        report.fixed_domain_mismatch = fixed;

        report.total_nodes_after = self.nodes.len();
        report.top_centrality = self.degree_centrality_top(5);
        report.orphan_trait_nodes = self.orphan_trait_nodes();
        report.stats = format!(
            "nodes={} l0={} l1+={}",
            self.nodes.len(),
            self.nodes.values().filter(|n| n.layer == NodeLayer::L0Primitive).count(),
            self.nodes.values().filter(|n| n.layer != NodeLayer::L0Primitive).count(),
        );
        report
    }

    /// 将 u8 星座值转为 ConstellationLevel (clamp 0-6)。
    pub fn constellation_from_u8(v: u8) -> ConstellationLevel {
        match v.min(6) {
            0 => ConstellationLevel::C0Compile,
            1 => ConstellationLevel::C1UnitTest,
            2 => ConstellationLevel::C2IntegrationTest,
            3 => ConstellationLevel::C3Benchmark,
            4 => ConstellationLevel::C4MainPipeline,
            5 => ConstellationLevel::C5SelfHealing,
            _ => ConstellationLevel::C6EvolutionLoop,
        }
    }

    /// 去重重复依赖边 — 统计并移除重复的 (from, to) 对。
    /// 通过 requires 字段去重 (DAG 边由 requires 派生, 见 add_dependency 同步逻辑)。
    pub fn dedup_duplicate_edges(&mut self) -> usize {
        let mut removed = 0usize;
        let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in &ids {
            let node = self.nodes.get_mut(id).expect("id exists");
            let mut unique = Vec::new();
            for req in node.requires.clone() {
                let pair = (id.clone(), req.clone());
                if !seen.insert(pair) {
                    removed += 1;
                    continue; // 重复 → 丢弃
                }
                unique.push(req);
            }
            node.requires = unique;
        }
        removed
    }

    /// 修正域错配 — 以 node_id 内的域前缀为准校正 node.domain。
    /// 例如 `nt_act::a` 若 domain 误为其他域, 按 id 前缀修正。
    pub fn fix_domain_mismatches(&mut self) -> usize {
        let mut fixed = 0usize;
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in &ids {
            let node = self.nodes.get_mut(id).expect("id exists");
            // 从 id 提取域前缀
            if let Some(prefix) = id.split("::").next() {
                let candidate = if prefix.starts_with("nt_") {
                    let slug = &prefix["nt_".len()..];
                    format!("NT-{}", slug.to_uppercase())
                } else if prefix.starts_with("exp::") || prefix.contains("nt-") {
                    continue; // exp:: 节点保留原域
                } else {
                    continue;
                };
                if let Some(domain) = Domain::from_str(&candidate) {
                    if node.domain != domain {
                        node.domain = domain;
                        fixed += 1;
                    }
                }
            }
        }
        fixed
    }

    /// 度中心性 top-k — 依赖图中出入度之和最大的节点 = 能力网枢纽。
    pub fn degree_centrality_top(&self, k: usize) -> Vec<(String, usize)> {
        let mut degrees: Vec<(String, usize)> = self
            .nodes
            .iter()
            .map(|(id, n)| {
                let deg = n.requires.len() + n.dependents.len();
                (id.clone(), deg)
            })
            .collect();
        degrees.sort_by(|a, b| b.1.cmp(&a.1));
        degrees.truncate(k);
        degrees
    }

    /// 孤立 trait 节点 — 无消费者 (dependents 空) 且提供能力标签的节点。
    pub fn orphan_trait_nodes(&self) -> Vec<String> {
        self.nodes
            .values()
            .filter(|n| n.dependents.is_empty() && !n.provides.is_empty() && n.requires.is_empty())
            .map(|n| n.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_trait_node_id() {
        assert_eq!(
            CapabilityRegistry::normalize_trait_node_id("nt_mind::safety::safety_core", Domain::Mind),
            "nt_mind_safety_safety_core"
        );
        assert_eq!(
            CapabilityRegistry::normalize_trait_node_id("nt_core_retrieval", Domain::Core),
            "nt_core_retrieval"
        );
    }

    #[test]
    fn test_dedup_duplicate_edges() {
        let mut reg = CapabilityRegistry::new();
        reg.register(CapabilityNode::new_primitive("nt_act::a".into(), Domain::Act, vec!["read".into()])).unwrap();
        reg.register(CapabilityNode::new_primitive("nt_act::b".into(), Domain::Act, vec!["write".into()])).unwrap();
        // 构造重复依赖: requires 中同一 target 出现两次
        let c_node = reg.get_mut("nt_act::a").unwrap();
        c_node.requires = vec!["nt_act::b".into(), "nt_act::b".into()];
        let removed = reg.dedup_duplicate_edges();
        assert_eq!(removed, 1, "应移除 1 条重复边");
        let node = reg.get("nt_act::a").unwrap();
        assert_eq!(node.requires.len(), 1);
    }

    #[test]
    fn test_fuse_trait_nodes_absorbs() {
        let mut reg = CapabilityRegistry::new();
        reg.register(CapabilityNode::new_primitive("nt_act::base".into(), Domain::Act, vec!["read".into()])).unwrap();
        let descriptors = vec![
            TraitNodeDescriptor {
                raw_id: "nt_mind::safety::safety_core".into(),
                provides: vec!["safety_guard".into()],
                constellation: 3,
                domain: Some(Domain::Mind),
            },
            TraitNodeDescriptor {
                raw_id: "nt_core_retrieval".into(),
                provides: vec!["hybrid_retrieval".into()],
                constellation: 2,
                domain: Some(Domain::Core),
            },
        ];
        let report = reg.fuse_trait_nodes(&descriptors, "test-cycle");
        assert_eq!(report.absorbed, 2);
        assert_eq!(report.skipped_conflict, 0);
        assert_eq!(report.total_nodes_after, 3);
        // 归一化 id 已入注册表
        assert!(reg.get("nt_mind_safety_safety_core").is_some());
        assert!(reg.get("nt_core_retrieval").is_some());
    }

    #[test]
    fn test_fuse_skips_conflict() {
        let mut reg = CapabilityRegistry::new();
        reg.register(CapabilityNode::new_primitive("nt_core_retrieval".into(), Domain::Core, vec!["hybrid_retrieval".into()])).unwrap();
        let descriptors = vec![TraitNodeDescriptor {
            raw_id: "nt_core_retrieval".into(),
            provides: vec!["hybrid_retrieval".into()],
            constellation: 2,
            domain: Some(Domain::Core),
        }];
        let report = reg.fuse_trait_nodes(&descriptors, "t");
        assert_eq!(report.absorbed, 0);
        assert_eq!(report.skipped_conflict, 1);
    }

    #[test]
    fn test_degree_centrality_top() {
        let mut reg = CapabilityRegistry::new();
        reg.register(CapabilityNode::new_primitive("p1".into(), Domain::Core, vec!["x".into()])).unwrap();
        reg.register(CapabilityNode::new_primitive("p2".into(), Domain::Core, vec!["y".into()])).unwrap();
        reg.register(CapabilityNode::new_primitive("p3".into(), Domain::Core, vec!["z".into()])).unwrap();
        // c1 依赖 p1/p2/p3 → 中心性最高
        reg.register(CapabilityNode::new_composite(
            "c1".into(), Domain::Core, NodeLayer::L1Composite,
            vec!["c".into()], vec!["p1".into(), "p2".into(), "p3".into()],
        )).unwrap();
        for p in ["p1", "p2", "p3"] {
            let _ = reg.add_dependency("c1", p);
        }
        let top = reg.degree_centrality_top(1);
        assert_eq!(top[0].0, "c1");
        assert!(top[0].1 >= 3);
    }

    #[test]
    fn test_orphan_trait_nodes_detection() {
        let mut reg = CapabilityRegistry::new();
        // 无消费者且无依赖 → 孤儿
        reg.register(CapabilityNode::new_primitive("nt_act::solo".into(), Domain::Act, vec!["read".into()])).unwrap();
        // 有消费者的节点 → 非孤儿
        reg.register(CapabilityNode::new_primitive("nt_act::hub".into(), Domain::Act, vec!["write".into()])).unwrap();
        reg.register(CapabilityNode::new_composite(
            "nt_act::consumer".into(), Domain::Act, NodeLayer::L1Composite,
            vec!["merge".into()], vec!["write".into()],
        )).unwrap();
        reg.add_dependency("nt_act::consumer", "nt_act::hub").unwrap();
        let orphans = reg.orphan_trait_nodes();
        assert!(orphans.contains(&"nt_act::solo".to_string()));
        assert!(!orphans.contains(&"nt_act::hub".to_string()));
    }
}