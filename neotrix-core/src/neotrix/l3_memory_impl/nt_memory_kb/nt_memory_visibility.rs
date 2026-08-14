//! 三值可见性过滤 — 吸收自 xai-org/x-algorithm (visibility-filtering:
//! ALLOW / INTERSTITIAL / DROP 而非二元 allow/deny)。
//!
//! 作为检索/推荐管线的**末端决策层**: MMR 重排之后、返回之前, 按候选的
//! 硬信号 (安全/去重/质量) 与软信号 (相关度/新颖度) 做三值裁定 —
//! 比二元过滤多一个 INTERSTITIAL (插入候选) 档位, 让低相关但高新颖的内容
//! 有"占位展示"机会, 而非被直接丢弃。

use super::nt_memory_types::SearchResult;

/// 三值可见性裁定 — 语义同 x-algorithm visibility-filtering:
/// - `Drop`:   不可展示 (硬阻断: 违规/低质/重复)。
/// - `Interstitial`: 插入候选 — 可展示但降权 (新颖但弱相关)。
/// - `Allow`:  正常展示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    Drop,
    Interstitial,
    Allow,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Drop => "DROP",
            Self::Interstitial => "INTERSTITIAL",
            Self::Allow => "ALLOW",
        }
    }
}

/// 可见性过滤配置。
#[derive(Debug, Clone)]
pub struct VisibilityConfig {
    /// 硬性最低相关度 — 低于此直接 Drop (硬阻断)。
    pub min_relevance: f64,
    /// Interstitial 档的相关度下界 — 低于此 Drop, 高于此且低于 Allow 档则插入候选。
    pub interstitial_lower_bound: f64,
    /// 是否启用硬阻断 (危险内容信号 → Drop)。
    pub hard_block: bool,
    /// 每批最多保留的 Interstitial 数 (避免大量弱相关内容占据版面)。
    pub max_interstitials: usize,
}

impl Default for VisibilityConfig {
    fn default() -> Self {
        Self {
            min_relevance: 0.05,
            interstitial_lower_bound: 0.15,
            hard_block: true,
            max_interstitials: 2,
        }
    }
}

/// 单条候选的可见性裁定结果。
#[derive(Debug, Clone)]
pub struct VisibilityVerdict {
    pub node_id: String,
    pub visibility: Visibility,
    pub reason: String,
}

/// 对一批候选做三值可见性裁定。
///
/// 判定顺序 (硬信号优先):
/// 1. 危险内容信号 (signals[0] 为风险分) 且 hard_block → Drop。
/// 2. 相关度 < min_relevance → Drop。
/// 3. 相关度 ∈ [interstitial_lower_bound, min_relevance 之上) → 视新颖信号
///    (signals[1]) 给 Interstitial (新颖度不足则 Drop)。
/// 4. 其余 → Allow。
///
/// 返回与输入**同序**的全量裁定 (含 Drop), 由下游决定消费视图 —
/// 过滤层只做决策标记, 不重排 (语义同 x-algorithm visibility-filtering)。
pub fn filter_visibility(
    results: Vec<SearchResult>,
    config: &VisibilityConfig,
) -> Vec<VisibilityVerdict> {
    let mut out = Vec::with_capacity(results.len());
    let mut interstitial_used = 0usize;

    for r in results {
        let risk = r.signals.as_ref().map(|s| s[0]).unwrap_or(0.0);
        let novelty = r.signals.as_ref().map(|s| s[1]).unwrap_or(0.0);

        // 硬信号 1: 危险内容阻断。
        if config.hard_block && risk > 0.8 {
            out.push(VisibilityVerdict {
                node_id: r.node.id,
                visibility: Visibility::Drop,
                reason: format!("risk signal {:.2} > 0.8 (hard block)", risk),
            });
            continue;
        }
        // 硬信号 2: 相关度过低。
        if r.score < config.min_relevance {
            out.push(VisibilityVerdict {
                node_id: r.node.id,
                visibility: Visibility::Drop,
                reason: format!("relevance {:.3} < {:.3}", r.score, config.min_relevance),
            });
            continue;
        }
        // 软信号: 中等相关 → Interstitial 候选 (需要新颖度支撑, 且限量)。
        if r.score < config.interstitial_lower_bound {
            if novelty > 0.5 && interstitial_used < config.max_interstitials {
                interstitial_used += 1;
                out.push(VisibilityVerdict {
                    node_id: r.node.id,
                    visibility: Visibility::Interstitial,
                    reason: format!("novel but weak (relevance {:.3})", r.score),
                });
            } else {
                out.push(VisibilityVerdict {
                    node_id: r.node.id,
                    visibility: Visibility::Drop,
                    reason: if interstitial_used >= config.max_interstitials {
                        "interstitial quota exhausted".to_string()
                    } else {
                        format!("weak relevance {:.3} + low novelty {:.2}", r.score, novelty)
                    },
                });
            }
            continue;
        }
        out.push(VisibilityVerdict {
            node_id: r.node.id,
            visibility: Visibility::Allow,
            reason: "relevance above threshold".to_string(),
        });
    }
    out
}

/// 便捷: 返回可见 (非 Drop) 的 ID 列表 (下游主消费视图)。
pub fn visible_ids(verdicts: &[VisibilityVerdict]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|v| v.visibility != Visibility::Drop)
        .map(|v| v.node_id.clone())
        .collect()
}

/// 便捷: 返回只含 Allow 的 ID 列表。
pub fn allowed_ids(verdicts: &[VisibilityVerdict]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|v| v.visibility == Visibility::Allow)
        .map(|v| v.node_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::{KnowledgeNode, NodeType};

    fn res(id: &str, score: f64, signals: Option<[f64; 4]>) -> SearchResult {
        SearchResult {
            node: KnowledgeNode {
                id: id.into(),
                node_type: NodeType::Concept,
                title: id.into(),
                summary: None,
                content: None,
                url: None,
                domain: None,
                language: "en".into(),
                confidence: 1.0,
                importance: 0.5,
                created_at: 0,
                updated_at: 0,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
            },
            score,
            matched_on: vec![],
            signals,
        }
    }

    #[test]
    fn high_relevance_is_allowed() {
        let cfg = VisibilityConfig::default();
        let verdicts = filter_visibility(vec![res("a", 0.9, Some([0.1, 0.2, 0.0, 0.0]))], &cfg);
        assert_eq!(verdicts[0].visibility, Visibility::Allow);
    }

    #[test]
    fn low_relevance_is_dropped() {
        let cfg = VisibilityConfig::default();
        let verdicts = filter_visibility(vec![res("a", 0.01, Some([0.1, 0.0, 0.0, 0.0]))], &cfg);
        assert_eq!(verdicts[0].visibility, Visibility::Drop);
        assert!(visible_ids(&verdicts).is_empty());
    }

    #[test]
    fn high_risk_hard_blocked_even_if_relevant() {
        let cfg = VisibilityConfig::default();
        let verdicts = filter_visibility(vec![res("a", 0.9, Some([0.95, 0.0, 0.0, 0.0]))], &cfg);
        assert_eq!(verdicts[0].visibility, Visibility::Drop);
        assert!(verdicts[0].reason.contains("hard block"));
    }

    #[test]
    fn novelty_promotes_to_interstitial() {
        let cfg = VisibilityConfig::default();
        // 弱相关 + 高新颖 → Interstitial
        let verdicts = filter_visibility(vec![res("a", 0.08, Some([0.1, 0.9, 0.0, 0.0]))], &cfg);
        assert_eq!(verdicts[0].visibility, Visibility::Interstitial);
        // 弱相关 + 低新颖 → Drop
        let verdicts = filter_visibility(vec![res("b", 0.08, Some([0.1, 0.1, 0.0, 0.0]))], &cfg);
        assert_eq!(verdicts[0].visibility, Visibility::Drop);
    }

    #[test]
    fn interstitials_capped() {
        let cfg = VisibilityConfig {
            max_interstitials: 2,
            ..Default::default()
        };
        let results = (0..5)
            .map(|i| res(&format!("n{}", i), 0.08, Some([0.1, 0.9, 0.0, 0.0])))
            .collect();
        let verdicts = filter_visibility(results, &cfg);
        let interstitial_count = verdicts.iter().filter(|v| v.visibility == Visibility::Interstitial).count();
        assert_eq!(interstitial_count, 2, "interstitials capped at max_interstitials");
        // 超额的弱相关内容被 Drop (quota exhausted)
        assert!(verdicts.iter().any(|v| v.visibility == Visibility::Drop));
    }

    #[test]
    fn visible_and_allowed_ids() {
        let cfg = VisibilityConfig::default();
        let verdicts = filter_visibility(
            vec![
                res("ok", 0.9, Some([0.0, 0.0, 0.0, 0.0])),
                res("weak", 0.08, Some([0.0, 0.9, 0.0, 0.0])),
                res("bad", 0.01, Some([0.0, 0.0, 0.0, 0.0])),
            ],
            &cfg,
        );
        assert_eq!(allowed_ids(&verdicts), vec!["ok".to_string()]);
        assert_eq!(visible_ids(&verdicts), vec!["ok".to_string(), "weak".to_string()]);
    }
}
