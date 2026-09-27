//! nt_law_gate — EVO-11 证明门（影子）＋语义网关（Bend LAWS＋TokenHub 思想）。
//!
//! LAWS 规则机验以前先跑影子裁决（`blocks()` 恒 false，只记录触发）；
//! 语义路由按分选路并做 effective-route 归因账本（上限 [`MAX_RECORDS`]）。
//! 全部同步纯逻辑，无 IO / 全局状态。

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// 归因账本上限。
pub const MAX_RECORDS: usize = 100;

/// 法则规则。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawRule {
    pub id: String,
    pub description: String,
    pub severity: u8,
}

/// 影子裁决（记录触发，永不拦截）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowVerdict {
    pub rule_id: String,
    pub triggered: bool,
    pub note: String,
}

impl ShadowVerdict {
    /// 影子模式：永远不拦截。
    pub fn blocks(&self) -> bool {
        false
    }
}

/// 语义路由候选。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticRoute {
    pub route: String,
    pub score: f64,
    pub budget_ms: u64,
}

impl SemanticRoute {
    pub fn scored(route: impl Into<String>, score: f64, budget_ms: u64) -> Self {
        let score = if score.is_nan() { 0.0 } else { score.clamp(0.0, 1.0) };
        Self { route: route.into(), score, budget_ms }
    }
}

/// 按分选路（最高分；空返回 None）。
pub fn pick(routes: &[SemanticRoute]) -> Option<&SemanticRoute> {
    let mut best: Option<&SemanticRoute> = None;
    for r in routes {
        let take = match &best {
            None => true,
            Some(b) => r.score > b.score,
        };
        if take {
            best = Some(r);
        }
    }
    best
}

/// effective-route 归因记录。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveRecord {
    pub route: String,
    pub tokens: u64,
    pub cost_micros: u64,
}

/// 归因账本（内存，超限丢弃最旧）。
#[derive(Debug, Default, Clone)]
pub struct AttributionLedger {
    records: VecDeque<EffectiveRecord>,
}

impl AttributionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, rec: EffectiveRecord) {
        self.records.push_back(rec);
        while self.records.len() > MAX_RECORDS {
            self.records.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 按路由汇总 tokens。
    pub fn tokens_for(&self, route: &str) -> u64 {
        self.records.iter().filter(|r| r.route == route).map(|r| r.tokens).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_never_blocks() {
        let v = ShadowVerdict {
            rule_id: "r".to_owned(),
            triggered: true,
            note: "hit".to_owned(),
        };
        assert!(v.triggered);
        assert!(!v.blocks());
    }

    #[test]
    fn pick_best_and_empty() {
        let routes = vec![
            SemanticRoute::scored("a", 0.4, 100),
            SemanticRoute::scored("b", 0.9, 200),
            SemanticRoute::scored("c", f64::NAN, 50),
        ];
        assert_eq!(pick(&routes).map(|r| r.route.as_str()), Some("b"));
        let empty: Vec<SemanticRoute> = Vec::new();
        assert!(pick(&empty).is_none());
    }

    #[test]
    fn ledger_evicts_and_sums() {
        let mut l = AttributionLedger::new();
        for i in 0..105 {
            l.record(EffectiveRecord {
                route: if i % 2 == 0 { "a".to_owned() } else { "b".to_owned() },
                tokens: 10,
                cost_micros: 1,
            });
        }
        assert_eq!(l.len(), MAX_RECORDS);
        assert_eq!(l.tokens_for("a") + l.tokens_for("b"), (MAX_RECORDS as u64) * 10);
    }

    #[test]
    fn score_clamped() {
        assert_eq!(SemanticRoute::scored("x", 9.0, 1).score, 1.0);
        assert_eq!(SemanticRoute::scored("x", -2.0, 1).score, 0.0);
    }
}
