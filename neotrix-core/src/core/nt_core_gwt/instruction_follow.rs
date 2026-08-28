//! InstructionFollowSpecialist — GWT 监控节点，实时追踪「指令发出 → harness 实际跟随」匹配率 (HFR)。
//!
//! 论文发现弱模型 HFR 仅 0.142（Opus 0.757），即 harness 经常不跟随 GWT 下发的指令。
//! 本节点独立于既有 GWT 路由，作为可挂载的监控专型：记录每次 directive 及其是否被 follow，
//! 暴露 `record_directive` / `record_follow` / `hfr()` 接口，供 ConsciousnessTree / SEAL 读取。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单条已下发指令的跟踪记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub id: u64,
    pub content: String,
    pub followed: bool,
}

/// InstructionFollowSpecialist — 实时跟随率监控节点。
#[derive(Debug, Clone, Default)]
pub struct InstructionFollowSpecialist {
    directives: Vec<Directive>,
    by_id: HashMap<u64, usize>,
    next_id: u64,
}

impl InstructionFollowSpecialist {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一条新下发的指令，返回其唯一 id（供后续 `record_follow` 关联）。
    pub fn record_directive(&mut self, content: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.by_id.insert(id, self.directives.len());
        self.directives.push(Directive {
            id,
            content: content.to_string(),
            followed: false,
        });
        id
    }

    /// 记录某条指令被 harness 实际跟随；返回 true 表示该 id 存在并已标记。
    pub fn record_follow(&mut self, directive_id: u64) -> bool {
        if let Some(&idx) = self.by_id.get(&directive_id) {
            self.directives[idx].followed = true;
            true
        } else {
            false
        }
    }

    /// Harness 跟随率 HFR = 已跟随数 / 已下发数；无指令时返回 0.0。
    pub fn hfr(&self) -> f64 {
        let issued = self.directives.len() as f64;
        if issued == 0.0 {
            0.0
        } else {
            let followed = self.directives.iter().filter(|d| d.followed).count() as f64;
            followed / issued
        }
    }

    /// 仍未被跟随的待处理指令数。
    pub fn pending(&self) -> usize {
        self.directives.iter().filter(|d| !d.followed).count()
    }

    /// 已下发指令总数。
    pub fn count(&self) -> usize {
        self.directives.len()
    }

    /// 导出当前所有指令（快照，供审计 / 上报）。
    pub fn snapshot(&self) -> Vec<Directive> {
        self.directives.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_directive_returns_incremental_id() {
        let mut s = InstructionFollowSpecialist::new();
        let a = s.record_directive("activate harness X");
        let b = s.record_directive("run distil");
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(s.count(), 2);
    }

    #[test]
    fn test_hfr_basic_142_like_paper() {
        // 论文弱模型 HFR = 0.142（1000 条中 142 条被跟随）。
        let mut s = InstructionFollowSpecialist::new();
        let mut ids = Vec::new();
        for i in 0..1000 {
            ids.push(s.record_directive(&format!("d{i}")));
        }
        for id in ids.iter().take(142) {
            assert!(s.record_follow(*id));
        }
        assert!((s.hfr() - 0.142).abs() < 1e-9);
        assert_eq!(s.pending(), 858);
    }

    #[test]
    fn test_record_follow_unknown_id_returns_false() {
        let mut s = InstructionFollowSpecialist::new();
        s.record_directive("x");
        assert!(!s.record_follow(999));
    }

    #[test]
    fn test_hfr_full_follow() {
        let mut s = InstructionFollowSpecialist::new();
        let a = s.record_directive("a");
        let b = s.record_directive("b");
        assert!(s.record_follow(a));
        assert!(s.record_follow(b));
        assert!((s.hfr() - 1.0).abs() < 1e-9);
        assert_eq!(s.pending(), 0);
    }

    #[test]
    fn test_hfr_empty_is_zero() {
        let s = InstructionFollowSpecialist::new();
        assert_eq!(s.hfr(), 0.0);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_record_follow_idempotent() {
        let mut s = InstructionFollowSpecialist::new();
        let a = s.record_directive("a");
        assert!(s.record_follow(a));
        assert!(s.record_follow(a));
        assert_eq!(s.snapshot()[0].followed, true);
        assert_eq!(s.hfr(), 1.0);
    }
}
