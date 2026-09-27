//! nt_intel_digest — EVO-12 情报 profile 管线＋跨模态 ICL 基准（Horizon＋14011 思想）。
//!
//! rubric 阈值过滤→按 topic 去重（留最高分）→balanced 配额→降序截断
//! （上限 [`MAX_ITEMS`]）；`PairedGap` 度量 clean-vs-deranged 配对收益。
//! 抓取与 enrich 在外层，本文件只做同步纯逻辑。

use serde::{Deserialize, Serialize};

/// 简报上限；单 topic 配额。
pub const MAX_ITEMS: usize = 30;
pub const MAX_PER_TOPIC: usize = 3;

/// 情报条目。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntelItem {
    pub topic: String,
    pub title: String,
    pub score: f64,
    pub source: String,
}

/// 准入 rubric（最低分）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rubric {
    pub min_score: f64,
}

impl Rubric {
    pub fn pass(&self, item: &IntelItem) -> bool {
        !item.score.is_nan() && item.score >= self.min_score
    }
}

/// 配对收益（clean-vs-deranged，14011 Δn 同构）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PairedGap {
    pub clean: f64,
    pub deranged: f64,
}

impl PairedGap {
    pub fn delta(&self) -> f64 {
        self.clean - self.deranged
    }
}

/// 简报构建：过滤→去重（同 topic 留最高）→配额→降序截断。
pub fn build_digest(items: &[IntelItem], rubric: &Rubric) -> Vec<IntelItem> {
    let mut best: Vec<IntelItem> = Vec::new();
    for it in items {
        if !rubric.pass(it) {
            continue;
        }
        if let Some(pos) = best.iter().position(|b| b.topic == it.topic && b.title == it.title) {
            if it.score > best[pos].score {
                best[pos] = it.clone();
            }
            continue;
        }
        best.push(it.clone());
    }
    // 单 topic 配额：同 topic 按分取前 MAX_PER_TOPIC
    let mut topics: Vec<String> = best.iter().map(|b| b.topic.clone()).collect();
    topics.sort();
    topics.dedup();
    let mut out = Vec::new();
    for t in topics {
        let mut group: Vec<IntelItem> =
            best.iter().filter(|b| b.topic == t).cloned().collect();
        group.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        group.truncate(MAX_PER_TOPIC);
        out.extend(group);
    }
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(MAX_ITEMS);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(topic: &str, title: &str, score: f64) -> IntelItem {
        IntelItem {
            topic: topic.to_owned(),
            title: title.to_owned(),
            score,
            source: "s".to_owned(),
        }
    }

    #[test]
    fn threshold_and_dedupe() {
        let r = Rubric { min_score: 0.5 };
        let items = vec![
            item("t", "a", 0.4),
            item("t", "b", 0.6),
            item("t", "b", 0.9),
        ];
        let d = build_digest(&items, &r);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].score, 0.9);
    }

    #[test]
    fn per_topic_quota_and_total_cap() {
        let r = Rubric { min_score: 0.0 };
        let mut items = Vec::new();
        for i in 0..10 {
            items.push(item("t", &format!("x{i}"), 0.9 - (i as f64) * 0.01));
        }
        let d = build_digest(&items, &r);
        assert_eq!(d.len(), MAX_PER_TOPIC);
        // 降序
        assert!(d[0].score >= d[1].score);
    }

    #[test]
    fn nan_rejected() {
        let r = Rubric { min_score: 0.0 };
        let d = build_digest(&[item("t", "a", f64::NAN)], &r);
        assert!(d.is_empty());
    }

    #[test]
    fn paired_gap_delta() {
        let g = PairedGap { clean: 0.75, deranged: 0.5 };
        assert!((g.delta() - 0.25).abs() < 1e-9);
    }
}
