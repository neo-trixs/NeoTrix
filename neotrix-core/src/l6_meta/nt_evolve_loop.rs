//! nt_evolve_loop — EVO-07 在线进化闭环（reef Serve/Observe/Grow/Commit 思想）。
//!
//! 收据＋评分＋版本化 artifact＋热切换，全部内存纯逻辑：`ArtifactStore`
//! 最多保留 [`MAX_VERSIONS`] 版，超限丢弃最旧；`decide` 按分做
//! promote/hold/rollback 三决策。同步纯逻辑，无 IO / 全局状态。

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// artifact 保留版数。
pub const MAX_VERSIONS: usize = 8;
/// 晋升线 / 回滚线。
pub const PROMOTE_SCORE: f64 = 0.8;
pub const ROLLBACK_SCORE: f64 = 0.3;

/// 服务收据。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServeReceipt {
    pub id: String,
    pub artifact: String,
    pub version: u64,
}

/// 观测报告（score 恒箝位 [0,1]）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObserveReport {
    pub score: f64,
    pub feedback: String,
}

impl ObserveReport {
    pub fn new(score: f64, feedback: impl Into<String>) -> Self {
        let score = if score.is_nan() {
            0.0
        } else {
            score.clamp(0.0, 1.0)
        };
        Self { score, feedback: feedback.into() }
    }
}

/// 生长决策。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowDecision {
    Promote,
    Hold,
    Rollback,
}

/// 按分决策（纯函数）。
pub fn decide(score: f64) -> GrowDecision {
    if score >= PROMOTE_SCORE {
        GrowDecision::Promote
    } else if score <= ROLLBACK_SCORE {
        GrowDecision::Rollback
    } else {
        GrowDecision::Hold
    }
}

/// 版本化 artifact 库（内存）。
#[derive(Debug, Default, Clone)]
pub struct ArtifactStore {
    versions: VecDeque<(u64, String)>,
    current: u64,
    next: u64,
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self { versions: VecDeque::new(), current: 0, next: 1 }
    }

    /// 发布新版，返回版本号（超限丢弃最旧）。
    pub fn publish(&mut self, content: impl Into<String>) -> u64 {
        let v = self.next;
        self.next += 1;
        self.versions.push_back((v, content.into()));
        while self.versions.len() > MAX_VERSIONS {
            self.versions.pop_front();
        }
        if self.current == 0 {
            self.current = v;
        }
        v
    }

    pub fn get(&self, version: u64) -> Option<&str> {
        self.versions.iter().find(|(v, _)| *v == version).map(|(_, c)| c.as_str())
    }

    pub fn list_versions(&self) -> Vec<u64> {
        self.versions.iter().map(|(v, _)| *v).collect()
    }

    pub fn current(&self) -> u64 {
        self.current
    }

    /// 热切换 current 指针（版本不存在返回 false）。
    pub fn switch(&mut self, version: u64) -> bool {
        if self.versions.iter().any(|(v, _)| *v == version) {
            self.current = version;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_clamped_and_nan_zero() {
        assert_eq!(ObserveReport::new(2.0, "x").score, 1.0);
        assert_eq!(ObserveReport::new(-1.0, "x").score, 0.0);
        assert_eq!(ObserveReport::new(f64::NAN, "x").score, 0.0);
    }

    #[test]
    fn versions_evict_oldest_beyond_cap() {
        let mut s = ArtifactStore::new();
        for i in 0..10 {
            s.publish(format!("c{i}"));
        }
        let vs = s.list_versions();
        assert_eq!(vs.len(), MAX_VERSIONS);
        assert!(!vs.contains(&1));
        assert!(vs.contains(&10));
        assert!(s.get(1).is_none());
    }

    #[test]
    fn switch_moves_current_only_for_known() {
        let mut s = ArtifactStore::new();
        let v1 = s.publish("a");
        let v2 = s.publish("b");
        assert_eq!(s.current(), v1);
        assert!(s.switch(v2));
        assert_eq!(s.current(), v2);
        assert!(!s.switch(999));
        assert_eq!(s.current(), v2);
    }

    #[test]
    fn decide_boundaries() {
        assert_eq!(decide(0.8), GrowDecision::Promote);
        assert_eq!(decide(0.3), GrowDecision::Rollback);
        assert_eq!(decide(0.5), GrowDecision::Hold);
        assert_eq!(decide(1.0), GrowDecision::Promote);
        assert_eq!(decide(0.0), GrowDecision::Rollback);
    }
}
