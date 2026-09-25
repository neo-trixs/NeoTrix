//! 指令队列 — 吸收 `OpenWarcraft3` 的 `g_commands.c` 指令思想.
//!
//! OW3 侧：RTS 帧逻辑由序列化指令驱动（移动/攻击/建造进队列，
//! 按 tick 顺序执行，网络对战靠指令同步而非状态同步）。
//! 本模块是无头数据侧：`Command` 可序列化（`serde`），`CommandQueue`
//! 按 `tick` 排序出队，调用方每固定步 `drain_due(current_tick)` 取出
//! 执行（与 `nt_clock::FixedStepper` 同步使用）。
//!
//! 公理：指令即数据（可经网络/录像回放）；队列按 `(tick, seq)` 稳定排序；
//! 过期/未来 tick 由调用方按固定步消费，本模块不做时间推进。

use serde::{Deserialize, Serialize};

/// 引擎中性指令（RTS 最小集；游戏语义由调用方解释 `kind/payload`）.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    /// 下发时刻的仿真 tick（执行 tick ≥ 此值时可执行）.
    pub tick: u64,
    /// 发起实体（0 表系统指令）.
    pub issuer: u64,
    /// 指令类型（"move"/"attack"/"stop"/…，游戏侧约定）.
    pub kind: String,
    /// 目标点（无目标指令用 `None`）.
    pub target: Option<(f32, f32)>,
    /// 目标实体（集火/跟随用）.
    pub target_entity: Option<u64>,
}

impl Command {
    pub fn new(
        tick: u64,
        issuer: u64,
        kind: &str,
        target: Option<(f32, f32)>,
        target_entity: Option<u64>,
    ) -> Self {
        Self {
            tick,
            kind: kind.to_string(),
            issuer,
            target,
            target_entity,
        }
    }
}

/// 按 tick 出队的指令队列（入队 seq 保序）.
#[derive(Debug, Default)]
pub struct CommandQueue {
    items: Vec<(Command, u64)>,
    seq: u64,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 入队（空 `kind` 拒绝，返回 `Err`）.
    pub fn push(&mut self, cmd: Command) -> Result<(), String> {
        if cmd.kind.is_empty() {
            return Err("指令 kind 为空".to_string());
        }
        let seq = self.seq;
        self.seq += 1;
        self.items.push((cmd, seq));
        Ok(())
    }

    /// 取出全部 `tick <= current_tick` 的指令（按 `(tick, seq)` 排序）.
    pub fn drain_due(&mut self, current_tick: u64) -> Vec<Command> {
        self.items.sort_by(|a, b| {
            a.0.tick.cmp(&b.0.tick).then_with(|| a.1.cmp(&b.1))
        });
        let mut due = Vec::new();
        let mut rest = Vec::new();
        for (cmd, seq) in self.items.drain(..) {
            if cmd.tick <= current_tick {
                due.push((cmd, seq));
            } else {
                rest.push((cmd, seq));
            }
        }
        self.items = rest;
        due.sort_by(|a, b| a.0.tick.cmp(&b.0.tick).then_with(|| a.1.cmp(&b.1)));
        due.into_iter().map(|(c, _)| c).collect()
    }

    /// 序列化快照（录像/网络同步用）.
    pub fn snapshot(&self) -> Vec<Command> {
        self.items.iter().map(|(c, _)| c.clone()).collect()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_ordered_by_tick_then_seq() {
        let mut q = CommandQueue::new();
        q.push(Command::new(3, 1, "move", Some((1.0, 2.0)), None)).unwrap();
        q.push(Command::new(1, 1, "stop", None, None)).unwrap();
        q.push(Command::new(1, 2, "attack", None, Some(9))).unwrap();
        let due = q.drain_due(1);
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].kind, "stop"); // 同 tick 按入队序
        assert_eq!(due[1].kind, "attack");
        assert_eq!(q.len(), 1); // tick=3 的还在
        let rest = q.drain_due(3);
        assert_eq!(rest.len(), 1);
        assert!(q.is_empty());
    }

    #[test]
    fn empty_kind_rejected() {
        let mut q = CommandQueue::new();
        assert!(q.push(Command::new(0, 0, "", None, None)).is_err());
        assert!(q.is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let cmd = Command::new(7, 42, "move", Some((3.0, 4.0)), None);
        let s = serde_json::to_string(&cmd).unwrap();
        let back: Command = serde_json::from_str(&s).unwrap();
        assert_eq!(cmd, back);
    }
}
