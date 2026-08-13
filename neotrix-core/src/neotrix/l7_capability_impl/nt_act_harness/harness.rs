//! nt_act::harness::harness — 单 agent turn 执行 + 输出捕获
//!
//! 节点: nt_act::harness::harness (L0)
//! Provides: agent_harness, agent_turn_execution, output_capture

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HarnessRole {
    Lead,
    Worker,
    Observer,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HarnessTurn {
    pub id: u64,
    pub role: HarnessRole,
    pub instruction: String,
    pub output: Option<String>,
    pub status: TurnStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TurnStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent 执行 Harness — 启动 turn、捕获输出、报告状态
#[derive(Debug, Clone, Default)]
pub struct AgentHarness {
    turns: Vec<HarnessTurn>,
    next_id: u64,
}

impl AgentHarness {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn_turn(
        &mut self,
        role: HarnessRole,
        instruction: &str,
    ) -> Result<u64, NeoTrixError> {
        if instruction.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput("turn 指令不能为空".into()));
        }
        let id = self.next_id;
        self.next_id += 1;
        self.turns.push(HarnessTurn {
            id,
            role,
            instruction: instruction.into(),
            output: None,
            status: TurnStatus::Pending,
        });
        Ok(id)
    }

    pub fn run_turn(
        &mut self,
        id: u64,
        execute: impl FnOnce(&str) -> Result<String, NeoTrixError>,
    ) -> Result<String, NeoTrixError> {
        let turn = self
            .turns
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| NeoTrixError::NotFound(format!("turn {} 未找到", id)))?;
        if turn.status == TurnStatus::Completed {
            return Err(NeoTrixError::InvalidState(format!("turn {} 已执行过", id)));
        }
        turn.status = TurnStatus::Running;
        match execute(&turn.instruction) {
            Ok(out) => {
                turn.output = Some(out.clone());
                turn.status = TurnStatus::Completed;
                Ok(out)
            }
            Err(e) => {
                turn.status = TurnStatus::Failed;
                Err(e)
            }
        }
    }

    pub fn capture_output(&self, id: u64) -> Option<&str> {
        self.turns
            .iter()
            .find(|t| t.id == id)
            .and_then(|t| t.output.as_deref())
    }

    pub fn status(&self, id: u64) -> Option<TurnStatus> {
        self.turns.iter().find(|t| t.id == id).map(|t| t.status)
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    pub fn completed_count(&self) -> usize {
        self.turns
            .iter()
            .filter(|t| t.status == TurnStatus::Completed)
            .count()
    }
}

impl CapabilityNode for AgentHarness {
    fn node_id(&self) -> &str {
        "nt_act::harness::harness"
    }
    fn provides(&self) -> Vec<String> {
        vec![
            "agent_harness".into(),
            "agent_turn_execution".into(),
            "output_capture".into(),
        ]
    }
    fn requires(&self) -> Vec<String> {
        vec![]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Golden]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for AgentHarness {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut h = AgentHarness::new();
        let id = h
            .spawn_turn(HarnessRole::Worker, "编译并报告错误数")
            .map_err(|e| vec![e.to_string()])?;
        let out = h
            .run_turn(id, |_| Ok("0 errors".into()))
            .map_err(|e| vec![e.to_string()])?;
        assert_eq!(out, "0 errors");
        assert_eq!(h.capture_output(id), Some("0 errors"));
        assert_eq!(h.completed_count(), 1);
        // 重复执行应拒绝
        assert!(h.run_turn(id, |_| Ok("again".into())).is_err());
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_act_harness_harness"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_run_capture() {
        let mut h = AgentHarness::new();
        let id = h.spawn_turn(HarnessRole::Lead, "research").unwrap();
        assert_eq!(h.status(id), Some(TurnStatus::Pending));
        let out = h.run_turn(id, |i| Ok(format!("done: {i}"))).unwrap();
        assert_eq!(out, "done: research");
        assert_eq!(h.status(id), Some(TurnStatus::Completed));
        assert_eq!(h.capture_output(id), Some("done: research"));
    }

    #[test]
    fn test_empty_instruction_rejected() {
        let mut h = AgentHarness::new();
        assert!(h.spawn_turn(HarnessRole::Lead, "   ").is_err());
    }

    #[test]
    fn test_failed_turn_status() {
        let mut h = AgentHarness::new();
        let id = h.spawn_turn(HarnessRole::Worker, "boom").unwrap();
        let err = h.run_turn(id, |_| Err(NeoTrixError::InvalidState("crash".into())));
        assert!(err.is_err());
        assert_eq!(h.status(id), Some(TurnStatus::Failed));
        assert_eq!(h.completed_count(), 0);
    }

    #[test]
    fn test_unknown_turn() {
        let mut h = AgentHarness::new();
        assert!(h.run_turn(99, |_| Ok("x".into())).is_err());
        assert!(h.capture_output(99).is_none());
    }

    #[test]
    fn test_replay_rejected() {
        let mut h = AgentHarness::new();
        let id = h.spawn_turn(HarnessRole::Observer, "once").unwrap();
        h.run_turn(id, |_| Ok("out".into())).unwrap();
        assert!(h.run_turn(id, |_| Ok("out2".into())).is_err());
    }
}
