//! NT-HUANGJI-ATLAS — 皇極經世宇宙论种子（标签数据层，逻辑见 nt_cosmo_frames）。
//!
//! 本文件只含名（元會運世/乾坤/取象……）：一切递归/倍增/相序/接地走 frames 构造器。
//! 来源（公有领域）：维基文库四库本目录 / wikipedia 皇極經世 / 刘钢先天易图考 / 沈大成疏。
//! 45 节点 / 44 边（计数见 test_atlas_counts）。无 unwrap / expect / panic。

use super::knowledge_graph::{EdgeType, KnowledgeGraphManager, NodeId};
use super::nt_cosmo_frames::{
    build_doubling, build_phases, build_scale, link_grounding, DoublingSpec, PhaseSpec, ScaleSpec,
};
use std::collections::HashMap;

/// 确定性 embedding [层阶/序号归一/基数对数/0]。
fn emb(layer: f64, idx: f64, count: f64) -> Vec<f64> {
    let norm = if count > 1.0 {
        (count.ln() / 64.0_f64.ln()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    vec![layer, idx.clamp(0.0, 1.0), norm, 0.0]
}

fn meta(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn add(
    kg: &mut KnowledgeGraphManager,
    label: &str,
    node_type: &str,
    embedding: Vec<f64>,
    metadata: HashMap<String, String>,
) -> NodeId {
    let id = kg.add_node(label, embedding, node_type);
    if let Some(n) = kg.nodes.get_mut(&id) {
        n.metadata = metadata;
    }
    id
}

// ── 标签数据表（逻辑无标签，名只在此） ──
const HUI: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];
const DOUBLING_NAMES: [&str; 5] = ["太極", "兩儀", "四象", "八卦", "六十四卦"];
const DOUBLING_PICKS: [u32; 5] = [0, 1, 2, 3, 6];
const GUA: [(&str, &str, &str); 8] = [
    ("乾", "1", "日"),
    ("兌", "2", "月"),
    ("離", "3", "星"),
    ("震", "4", "辰"),
    ("巽", "5", "石"),
    ("坎", "6", "土"),
    ("艮", "7", "火"),
    ("坤", "8", "水"),
];
const PHASES: [(&str, &str); 4] = [
    ("皇", "以道化民"),
    ("帝", "以德教民"),
    ("王", "以功劝民"),
    ("伯", "以力率民"),
];

/// 皇極經世宇宙论全种子。
pub fn seed_huangji_atlas(kg: &mut KnowledgeGraphManager) {
    // ── 源头：人 + 书 + 方法论 ──
    let shaoyong = add(
        kg,
        "邵雍",
        "huangji_founder",
        emb(0.0, 0.0, 1.0),
        meta(&[("朝代", "北宋"), ("生卒", "1011-1077"), ("师承", "李之才")]),
    );
    let book = add(
        kg,
        "皇極經世書",
        "huangji_work",
        emb(0.0, 0.1, 64.0),
        meta(&[(
            "卷篇",
            "十二卷六十四篇/元會運世34/声音律呂16/观物内12/观物外2",
        )]),
    );
    kg.add_edge(&shaoyong, &book, EdgeType::Causal, 1.0);
    let principle = add(
        kg,
        "以物觀物",
        "huangji_principle",
        emb(5.0, 0.0, 1.0),
        meta(&[("出處", "觀物內篇"), ("對立", "不以我觀物")]),
    );
    kg.add_edge(&book, &principle, EdgeType::Semantic, 1.0);

    // ── 加一倍法（frames 倍增链） ──
    let doubling = DoublingSpec {
        depth: 6,
        picks: &DOUBLING_PICKS,
        node_type: "huangji_doubling",
        label: &|p| {
            DOUBLING_PICKS
                .iter()
                .position(|q| *q == p)
                .map(|i| DOUBLING_NAMES[i].to_string())
                .unwrap_or(format!("2^{p}"))
        },
        meta: &|p, count| {
            if count >= 64 {
                meta(&[("2的幂", &p.to_string()), ("注", "二进制字典序")])
            } else {
                meta(&[("2的幂", &p.to_string())])
            }
        },
        embedding: &|p| emb(1.0, f64::from(p) / 6.0, (1u64 << p.min(30)) as f64),
    };
    build_doubling(kg, &doubling);

    // ── 先天八卦 + 取象接地 ──
    let mut pairs = Vec::with_capacity(GUA.len());
    for (i, (name, num, xiang)) in GUA.iter().enumerate() {
        let g = add(
            kg,
            name,
            "huangji_gua",
            emb(2.0, (i + 1) as f64 / 8.0, 8.0),
            meta(&[("先天數", num), ("經世取象", xiang)]),
        );
        let x = add(
            kg,
            xiang,
            "huangji_xiang",
            emb(2.0, (i + 1) as f64 / 8.0, 1.0),
            meta(&[("所屬卦", name)]),
        );
        pairs.push((g, x));
    }
    link_grounding(kg, &pairs, 1.0);

    // ── 元會運世（frames 层级：扇出 [12,30,12]，展开元會两层） ──
    let scale = ScaleSpec {
        fanouts: &[12, 30, 12],
        expand_levels: 2,
        root_label: "元",
        node_type: "huangji_yuanhui",
        label: &|level, idx| match level {
            1 => HUI.get(idx).copied().unwrap_or("會").to_string(),
            2 => "運".to_string(),
            _ => "世".to_string(),
        },
        meta: &|level, idx, _count| match level {
            0 => meta(&[("年數", "129600"), ("下轄會", "12")]),
            1 => meta(&[("年數", "10800"), ("序", &(idx + 1).to_string())]),
            2 => meta(&[("年數", "360"), ("下轄世", "12")]),
            _ => meta(&[("年數", "30")]),
        },
        embedding: &|_level, idx| emb(3.0, idx as f64 / 12.0, 129600.0),
    };
    let levels = build_scale(kg, &scale);

    // ── 皇-帝-王-伯（frames 相序梯）+ 开物/闭物 ──
    let phases = PhaseSpec {
        count: PHASES.len(),
        node_type: "huangji_phase",
        label: &|i| {
            PHASES
                .get(i)
                .map(|p| p.0.to_string())
                .unwrap_or(format!("P{i}"))
        },
        meta: &|i| {
            PHASES
                .get(i)
                .map(|p| meta(&[("治道", p.1)]))
                .unwrap_or_default()
        },
        embedding: &|i| emb(4.0, (i + 1) as f64 / 4.0, 4.0),
    };
    build_phases(kg, &phases);
    let kaiwu = add(
        kg,
        "開物",
        "huangji_phase",
        emb(4.0, 0.1, 1.0),
        meta(&[("於會", "寅")]),
    );
    let biwu = add(
        kg,
        "閉物",
        "huangji_phase",
        emb(4.0, 0.9, 1.0),
        meta(&[("於會", "戌")]),
    );
    // 寅为第 3 会（下标 2），戌为第 11 会（下标 10）。
    if let Some(hui) = levels.get(1) {
        if let (Some(yin), Some(xu)) = (hui.get(2), hui.get(10)) {
            kg.add_edge(&kaiwu, yin, EdgeType::Temporal, 1.0);
            kg.add_edge(&biwu, xu, EdgeType::Temporal, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> KnowledgeGraphManager {
        let mut kg = KnowledgeGraphManager::new();
        seed_huangji_atlas(&mut kg);
        kg
    }

    #[test]
    fn test_atlas_counts() {
        let kg = seeded();
        let (nodes, edges, _) = kg.stats();
        assert_eq!(nodes, 45, "节点数漂移");
        assert_eq!(edges, 44, "边数漂移");
    }

    #[test]
    fn test_xiantian_numbers() {
        let kg = seeded();
        for (name, num) in [
            ("乾", "1"),
            ("兌", "2"),
            ("離", "3"),
            ("震", "4"),
            ("巽", "5"),
            ("坎", "6"),
            ("艮", "7"),
            ("坤", "8"),
        ] {
            match kg.find_by_label(name) {
                Some(n) => assert_eq!(n.metadata.get("先天數").map(String::as_str), Some(num)),
                None => assert!(false, "缺卦 {name}"),
            }
        }
    }

    #[test]
    fn test_quxiang_mapping() {
        let kg = seeded();
        for (name, xiang) in [
            ("乾", "日"),
            ("兌", "月"),
            ("離", "星"),
            ("震", "辰"),
            ("巽", "石"),
            ("坎", "土"),
            ("艮", "火"),
            ("坤", "水"),
        ] {
            match kg.find_by_label(name) {
                Some(n) => {
                    assert_eq!(n.metadata.get("經世取象").map(String::as_str), Some(xiang));
                    assert!(kg.find_by_label(xiang).is_some(), "缺象节点");
                }
                None => assert!(false, "缺卦 {name}"),
            }
        }
    }

    #[test]
    fn test_yuanhui_numbers() {
        let kg = seeded();
        match kg.find_by_label("元") {
            Some(yuan) => {
                assert_eq!(
                    yuan.metadata.get("年數").map(String::as_str),
                    Some("129600")
                )
            }
            None => assert!(false, "缺元"),
        }
        // 十二会：年數 10800 者（元 129600 / 運 360 / 世 30 排除在外）。
        let hui_n = kg
            .nodes
            .values()
            .filter(|n| {
                n.node_type == "huangji_yuanhui"
                    && n.metadata.get("年數").map(String::as_str) == Some("10800")
            })
            .count();
        assert_eq!(hui_n, 12, "十二会");
        for h in ["寅", "戌"] {
            match kg.find_by_label(h) {
                Some(n) => {
                    assert_eq!(n.metadata.get("年數").map(String::as_str), Some("10800"))
                }
                None => assert!(false, "缺會 {h}"),
            }
        }
    }

    #[test]
    fn test_temporal_covers_all_edges() {
        let kg = seeded();
        let now = kg.edges.iter().map(|e| e.valid_from).max().unwrap_or(0);
        assert_eq!(kg.temporal_query(now).len(), kg.edges.len());
    }

    #[test]
    fn test_doubling_chain_causal() {
        let kg = seeded();
        let causal = kg.query_by_type(&EdgeType::Causal);
        // 加一倍法 4 + 邵雍→书 1 = 5 条因果边
        assert_eq!(causal.len(), 5);
    }

    #[test]
    fn test_neighbors_walk() {
        // 融入门：推理可从元出发沿时序边走到十二会。
        let kg = seeded();
        let yuan = kg.find_by_label("元");
        assert!(yuan.is_some(), "缺元");
        let kids = kg.neighbors(
            &yuan.map(|n| n.id.clone()).unwrap_or(NodeId("".into())),
            Some(&EdgeType::Temporal),
        );
        assert_eq!(kids.len(), 12);
    }
}
