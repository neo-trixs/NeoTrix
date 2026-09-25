//! NT-COSMO-FRAMES — 宇宙论结构框架（皇極架构逻辑的标签剥离版）。
//!
//! 只留骨，不留皮：递归层级 / 二进制倍增 / 相序衰减 / 符号接地四帧，
//! 所有名（元會運世/乾坤/邵雍……）皆由调用方经闭包注入，本模块无一硬编码标签。
//! 纯构造 + KnowledgeGraphManager 薄调用；无 unwrap / expect / panic。

use super::knowledge_graph::{EdgeType, KnowledgeGraphManager, NodeId};
use std::collections::HashMap;

/// 递归层级帧：一串扇出数（如皇極 [12, 30, 12] = 元→會→運→世）。
pub struct ScaleSpec<'a> {
    /// 每层扇出；层数 = fanouts.len() + 1（含根层 0）。
    pub fanouts: &'a [u64],
    /// 展开层数（含根层）；之后每层只建 1 个模式节点（防 12×30×12 实例爆炸）。
    pub expand_levels: usize,
    pub root_label: &'a str,
    pub node_type: &'a str,
    /// (层, 序号) → 标签；层 0 序号 0 为根（忽略 root_label 亦可，此处直接用 root）。
    pub label: &'a dyn Fn(usize, usize) -> String,
    /// (层, 序号, 本层基数) → 元数据（如年數由调用方算扇出积）。
    pub meta: &'a dyn Fn(usize, usize, u64) -> HashMap<String, String>,
    pub embedding: &'a dyn Fn(usize, usize) -> Vec<f64>,
}

/// 建层级，返回 levels[层][序号]。边：父→子 Temporal；展开叶→模式权重 0.5，其余 1.0。
pub fn build_scale(kg: &mut KnowledgeGraphManager, spec: &ScaleSpec<'_>) -> Vec<Vec<NodeId>> {
    let mut levels: Vec<Vec<NodeId>> = Vec::new();
    // 根层。
    let root = kg.add_node(spec.root_label, (spec.embedding)(0, 0), spec.node_type);
    if let Some(n) = kg.nodes.get_mut(&root) {
        n.metadata = (spec.meta)(0, 0, 1);
    }
    levels.push(vec![root]);
    let mut level_count: u64 = 1;
    for (li, fanout) in spec.fanouts.iter().enumerate() {
        let level = li + 1;
        level_count = level_count.saturating_mul(*fanout.max(&1));
        // 超出展开层则只建 1 个模式节点；展开层宽 = 扇出乘积（真扇出）。
        let width = if level < spec.expand_levels {
            spec.fanouts[..level]
                .iter()
                .fold(1usize, |a, f| a.saturating_mul(*f as usize))
        } else {
            1usize
        };
        let mut ids = Vec::with_capacity(width);
        for idx in 0..width {
            let id = kg.add_node(
                &(spec.label)(level, idx),
                (spec.embedding)(level, idx),
                spec.node_type,
            );
            if let Some(n) = kg.nodes.get_mut(&id) {
                n.metadata = (spec.meta)(level, idx, level_count);
            }
            ids.push(id);
        }
        // 父层连本层：宽 1（模式节点）则父全连；否则子 c 归父 c*P/W（整除即真扇出）。
        // 单父 1→N 为真扇出（如元→十二會 12 边）。
        let parents = levels.last().cloned().unwrap_or_default();
        if width == 1 {
            let w = if parents.len() > 1 && level >= spec.expand_levels {
                0.5
            } else {
                1.0
            };
            for p in &parents {
                kg.add_edge(p, &ids[0], EdgeType::Temporal, w);
            }
        } else {
            for (ci, c) in ids.iter().enumerate() {
                let pi = ci * parents.len() / width;
                if let Some(p) = parents.get(pi) {
                    kg.add_edge(p, c, EdgeType::Temporal, 1.0);
                }
            }
        }
        levels.push(ids);
    }
    levels
}

/// 倍增链帧：2^n 递归细分（如皇極 depth=6，取幂 [0,1,2,3,6]）。
pub struct DoublingSpec<'a> {
    pub depth: u32,
    /// 取哪些幂建节点（必须升序；终点一般含 depth）。
    pub picks: &'a [u32],
    pub node_type: &'a str,
    pub label: &'a dyn Fn(u32) -> String,
    pub meta: &'a dyn Fn(u32, u64) -> HashMap<String, String>,
    pub embedding: &'a dyn Fn(u32) -> Vec<f64>,
}

/// 建倍增链（Causal 边串起所选幂），返回所选节点。
pub fn build_doubling(kg: &mut KnowledgeGraphManager, spec: &DoublingSpec<'_>) -> Vec<NodeId> {
    let mut ids = Vec::with_capacity(spec.picks.len());
    let mut prev: Option<NodeId> = None;
    for p in spec.picks {
        let count = 1u64 << p.min(&spec.depth).min(&30);
        let id = kg.add_node(&(spec.label)(*p), (spec.embedding)(*p), spec.node_type);
        if let Some(n) = kg.nodes.get_mut(&id) {
            n.metadata = (spec.meta)(*p, count);
        }
        if let Some(q) = prev {
            kg.add_edge(&q, &id, EdgeType::Causal, 1.0);
        }
        prev = Some(id.clone());
        ids.push(id);
    }
    ids
}

/// 相序梯帧：N 个有序相位，边权重线性衰减（治道/能量衰减抽象）。
pub struct PhaseSpec<'a> {
    pub count: usize,
    pub node_type: &'a str,
    pub label: &'a dyn Fn(usize) -> String,
    pub meta: &'a dyn Fn(usize) -> HashMap<String, String>,
    pub embedding: &'a dyn Fn(usize) -> Vec<f64>,
}

/// 建相序链（Temporal 边，权重 1.0→0.8 线性衰减），返回相位节点。
pub fn build_phases(kg: &mut KnowledgeGraphManager, spec: &PhaseSpec<'_>) -> Vec<NodeId> {
    let mut ids = Vec::with_capacity(spec.count);
    let mut prev: Option<NodeId> = None;
    for i in 0..spec.count {
        let id = kg.add_node(&(spec.label)(i), (spec.embedding)(i), spec.node_type);
        if let Some(n) = kg.nodes.get_mut(&id) {
            n.metadata = (spec.meta)(i);
        }
        if let Some(q) = prev {
            let w = 1.0 - 0.2 * (i as f64) / (spec.count.max(1) as f64);
            kg.add_edge(&q, &id, EdgeType::Temporal, w);
        }
        prev = Some(id.clone());
        ids.push(id);
    }
    ids
}

/// 符号接地：已建节点的 (符号 → 现象) 语义边批量落子（如卦→取象）。
pub fn link_grounding(kg: &mut KnowledgeGraphManager, pairs: &[(NodeId, NodeId)], weight: f64) {
    for (sym, phen) in pairs {
        kg.add_edge(sym, phen, EdgeType::Semantic, weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_map() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn test_scale_bounds_total() {
        // 扇出 [2, 3] 全展开 = 1+2+6；只展开 2 层 = 1+2+1（模式）。
        let mut kg = KnowledgeGraphManager::new();
        let full = ScaleSpec {
            fanouts: &[2, 3],
            expand_levels: 99,
            root_label: "R",
            node_type: "t",
            label: &|l, i| format!("L{l}-{i}"),
            meta: &|_, _, _| empty_map(),
            embedding: &|_, _| vec![0.0],
        };
        let lv = build_scale(&mut kg, &full);
        assert_eq!(lv.iter().map(Vec::len).sum::<usize>(), 9);
        // 单父扇出：根→2 子（2 边）+ 2 父各→3 子（6 边）= 8。
        assert_eq!(kg.edges.len(), 8);
        let mut kg2 = KnowledgeGraphManager::new();
        let capped = ScaleSpec {
            fanouts: &[2, 3],
            expand_levels: 2,
            root_label: "R",
            node_type: "t",
            label: &|l, i| format!("L{l}-{i}"),
            meta: &|_, _, _| empty_map(),
            embedding: &|_, _| vec![0.0],
        };
        let lv2 = build_scale(&mut kg2, &capped);
        assert_eq!(lv2.iter().map(Vec::len).sum::<usize>(), 4);
        // 模式边权重 0.5（叶→模式），其余 1.0。
        let schema_edges = kg2
            .edges
            .iter()
            .filter(|e| (e.weight - 0.5).abs() < 1e-9)
            .count();
        assert_eq!(schema_edges, 2);
    }

    #[test]
    fn test_doubling_powers_of_two() {
        let mut kg = KnowledgeGraphManager::new();
        let spec = DoublingSpec {
            depth: 4,
            picks: &[0, 2, 4],
            node_type: "t",
            label: &|p| format!("2^{p}"),
            meta: &|_, _| empty_map(),
            embedding: &|_| vec![0.0],
        };
        let ids = build_doubling(&mut kg, &spec);
        assert_eq!(ids.len(), 3);
        assert_eq!(kg.query_by_type(&EdgeType::Causal).len(), 2);
    }

    #[test]
    fn test_phase_decay_weights() {
        let mut kg = KnowledgeGraphManager::new();
        let spec = PhaseSpec {
            count: 4,
            node_type: "t",
            label: &|i| format!("P{i}"),
            meta: &|_| empty_map(),
            embedding: &|_| vec![0.0],
        };
        let ids = build_phases(&mut kg, &spec);
        assert_eq!(ids.len(), 4);
        let ws: Vec<f64> = kg.edges.iter().map(|e| e.weight).collect();
        assert_eq!(ws.len(), 3);
        assert!(ws[0] > ws[1] && ws[1] > ws[2], "权重应衰减");
    }

    #[test]
    fn test_grounding_links() {
        let mut kg = KnowledgeGraphManager::new();
        let a = kg.add_node("s", vec![0.0], "t");
        let b = kg.add_node("p", vec![0.0], "t");
        link_grounding(&mut kg, &[(a, b)], 0.9);
        assert_eq!(kg.query_by_type(&EdgeType::Semantic).len(), 1);
    }
}
