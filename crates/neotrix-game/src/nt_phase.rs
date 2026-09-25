//! 回合相位队列 — 吸收 `pokerogue` 的 `phase-manager` 状态机思想.
//!
//! Pokerogue 侧：战斗是相位流水线（`EncounterPhase → CommandPhase →
//! DamageAnimPhase → FaintPhase → …`），`PhaseManager` 按队列顺序
//! 逐个执行，相位可向前插入（打断/反制）也可条件跳过。
//! 本模块是无头数据侧：`PhaseQueue` 存命名相位队列，调用方每步
//! `pop_next()` 取出执行（执行语义游戏侧定），支持 `push_front`
//! 打断插队与 `skip_if_empty` 式条件；队列可序列化做战斗录像。
//!
//! 公理：相位即数据（`Phase { name, turn }`），管理器不解释语义；
//! 空名相位拒绝入队；执行顺序 FIFO（`push_back`），打断用 `push_front`.

use serde::{Deserialize, Serialize};

/// 单个相位（名 + 所属回合；`data` 为游戏侧载荷，如伤害值/目标）.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub turn: u32,
    pub data: Vec<String>,
}

impl Phase {
    pub fn new(name: &str, turn: u32) -> Self {
        Self { name: name.to_string(), turn, data: Vec::new() }
    }

    pub fn with_data(name: &str, turn: u32, data: Vec<String>) -> Self {
        Self { name: name.to_string(), turn, data }
    }
}

/// 相位队列（Pokerogue `PhaseManager` 的数据侧）.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PhaseQueue {
    queue: Vec<Phase>,
    /// 已执行相位数（录像/统计用）.
    pub completed: u64,
}

impl PhaseQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// 队尾入队（空名 → `Err`）.
    pub fn push_back(&mut self, phase: Phase) -> Result<(), String> {
        if phase.name.is_empty() {
            return Err("相位名为空".to_string());
        }
        self.queue.push(phase);
        Ok(())
    }

    /// 队首插队（打断/反制语义；空名 → `Err`）.
    pub fn push_front(&mut self, phase: Phase) -> Result<(), String> {
        if phase.name.is_empty() {
            return Err("相位名为空".to_string());
        }
        self.queue.insert(0, phase);
        Ok(())
    }

    /// 批量编排一回合（按给定顺序入队）.
    pub fn plan_turn(&mut self, turn: u32, names: &[&str]) -> Result<(), String> {
        for n in names {
            self.push_back(Phase::new(n, turn))?;
        }
        Ok(())
    }

    /// 取出下一个相位（空 → `None`；`completed` 计数+1）.
    pub fn pop_next(&mut self) -> Option<Phase> {
        if self.queue.is_empty() {
            return None;
        }
        let p = self.queue.remove(0);
        self.completed += 1;
        Some(p)
    }

    pub fn peek(&self) -> Option<&Phase> {
        self.queue.first()
    }

    /// 清空未执行相位（战斗提前结束用；已完成计数保留）.
    pub fn clear_pending(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_and_interrupt() {
        let mut q = PhaseQueue::new();
        q.plan_turn(1, &["encounter", "command", "damage"]).unwrap();
        assert_eq!(q.len(), 3);
        // 反制插队到最前
        q.push_front(Phase::new("protect-check", 1)).unwrap();
        assert_eq!(q.pop_next().unwrap().name, "protect-check");
        assert_eq!(q.pop_next().unwrap().name, "encounter");
        assert_eq!(q.completed, 2);
        assert_eq!(q.peek().unwrap().name, "command");
    }

    #[test]
    fn empty_name_rejected() {
        let mut q = PhaseQueue::new();
        assert!(q.push_back(Phase::new("", 1)).is_err());
        assert!(q.push_front(Phase::new("", 1)).is_err());
        assert!(q.plan_turn(1, &["ok", ""]).is_err());
        // "ok" 已在报错前入队：先出 "ok"，之后为空
        assert_eq!(q.pop_next().unwrap().name, "ok");
        assert!(q.pop_next().is_none());
        let mut empty = PhaseQueue::new();
        assert!(empty.pop_next().is_none()); // 空队列 None
    }

    #[test]
    fn clear_keeps_completed() {
        let mut q = PhaseQueue::new();
        q.plan_turn(1, &["a", "b"]).unwrap();
        q.pop_next();
        q.clear_pending();
        assert!(q.is_empty());
        assert_eq!(q.completed, 1);
    }

    #[test]
    fn serde_roundtrip() {
        let mut q = PhaseQueue::new();
        q.plan_turn(2, &["command", "damage"]).unwrap();
        let s = serde_json::to_string(&q).unwrap();
        let back: PhaseQueue = serde_json::from_str(&s).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back.peek().unwrap().name, "command");
    }
}
