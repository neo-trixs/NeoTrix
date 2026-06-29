use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique ID for a sub-consciousness
pub type SubConsciousnessId = u64;

/// The domain a sub-consciousness specializes in
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReasoningDomain {
    KnowledgeQuery,
    Planning,
    SelfInspection,
    Simulation,
    Consensus,
    Custom(String),
}

/// Status of a sub-consciousness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SubConsciousnessStatus {
    Running,
    Completed,
    Failed(String),
    CheckpointPending,
}

/// A spawnable sub-consciousness that runs a focused reasoning pipeline.
/// Each SubConsciousness has its own pipeline, state, and checkpoint cycle.
/// It periodically checkpoints state back to the parent consciousness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubConsciousness {
    pub id: SubConsciousnessId,
    pub domain: ReasoningDomain,
    pub name: String,
    pub created_at: u64,
    pub status: SubConsciousnessStatus,
    pub parent_id: Option<SubConsciousnessId>,

    /// Pipeline nodes specific to this sub-consciousness
    pub pipeline_nodes: Vec<String>,

    /// How often (in cycles) to checkpoint back to parent
    pub checkpoint_interval: u32,

    /// Cycle count since this sub-consciousness started
    pub local_cycle: u64,

    /// Last checkpoint data hash (for integrity)
    pub last_checkpoint_hash: [u8; 32],

    /// Aggregated results to return to parent
    pub output_brief: String,

    /// VSA vector summary of output (for integration)
    pub output_vsa: Option<Vec<u8>>,
}

/// Manages the lifecycle of sub-consciousnesses.
pub struct SubConsciousnessManager {
    /// All sub-consciousnesses, keyed by ID
    pub sub_consciousnesses: HashMap<SubConsciousnessId, SubConsciousness>,
    /// Next ID to assign
    next_id: SubConsciousnessId,
    /// Max active sub-consciousnesses
    max_active: usize,
}

impl SubConsciousnessManager {
    pub fn new(max_active: usize) -> Self {
        Self {
            sub_consciousnesses: HashMap::new(),
            next_id: 1,
            max_active,
        }
    }

    pub fn spawn(
        &mut self,
        domain: ReasoningDomain,
        name: &str,
        checkpoint_interval: u32,
    ) -> Option<SubConsciousnessId> {
        if self.sub_consciousnesses.len() >= self.max_active {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        let sub = SubConsciousness {
            id,
            domain,
            name: name.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            status: SubConsciousnessStatus::Running,
            parent_id: None,
            pipeline_nodes: Vec::new(),
            checkpoint_interval,
            local_cycle: 0,
            last_checkpoint_hash: [0u8; 32],
            output_brief: String::new(),
            output_vsa: None,
        };
        self.sub_consciousnesses.insert(id, sub);
        Some(id)
    }

    pub fn tick(&mut self, id: SubConsciousnessId, _input: &str) -> Result<(), String> {
        let sub = self
            .sub_consciousnesses
            .get_mut(&id)
            .ok_or_else(|| format!("SubConsciousness {} not found", id))?;
        sub.local_cycle += 1;
        if sub.local_cycle % sub.checkpoint_interval as u64 == 0 {
            sub.status = SubConsciousnessStatus::CheckpointPending;
        }
        Ok(())
    }

    pub fn complete(
        &mut self,
        id: SubConsciousnessId,
        brief: &str,
        vsa: Option<Vec<u8>>,
    ) -> Result<(), String> {
        let sub = self
            .sub_consciousnesses
            .get_mut(&id)
            .ok_or_else(|| format!("SubConsciousness {} not found", id))?;
        sub.status = SubConsciousnessStatus::Completed;
        sub.output_brief = brief.to_string();
        sub.output_vsa = vsa;
        Ok(())
    }

    pub fn fail(&mut self, id: SubConsciousnessId, reason: &str) -> Result<(), String> {
        let sub = self
            .sub_consciousnesses
            .get_mut(&id)
            .ok_or_else(|| format!("SubConsciousness {} not found", id))?;
        sub.status = SubConsciousnessStatus::Failed(reason.to_string());
        Ok(())
    }

    pub fn collect_completed(&mut self) -> Vec<(SubConsciousnessId, String, Option<Vec<u8>>)> {
        let mut completed = Vec::new();
        let to_remove: Vec<SubConsciousnessId> = self
            .sub_consciousnesses
            .iter()
            .filter(|(_, s)| s.status == SubConsciousnessStatus::Completed)
            .map(|(id, _)| *id)
            .collect();
        for id in to_remove {
            if let Some(sub) = self.sub_consciousnesses.remove(&id) {
                completed.push((id, sub.output_brief, sub.output_vsa));
            }
        }
        completed
    }

    pub fn active_count(&self) -> usize {
        self.sub_consciousnesses
            .values()
            .filter(|s| s.status == SubConsciousnessStatus::Running)
            .count()
    }

    pub fn get(&self, id: SubConsciousnessId) -> Option<&SubConsciousness> {
        self.sub_consciousnesses.get(&id)
    }

    pub fn get_mut(&mut self, id: SubConsciousnessId) -> Option<&mut SubConsciousness> {
        self.sub_consciousnesses.get_mut(&id)
    }

    pub fn summary(&self) -> Vec<(SubConsciousnessId, String, String)> {
        self.sub_consciousnesses
            .values()
            .map(|s| (s.id, s.name.clone(), format!("{:?}", s.status)))
            .collect()
    }

    pub fn max_active(&self) -> usize {
        self.max_active
    }

    pub fn total_count(&self) -> usize {
        self.sub_consciousnesses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_manager() -> SubConsciousnessManager {
        SubConsciousnessManager::new(5)
    }

    #[test]
    fn test_spawn_sub_consciousness() -> Result<(), String> {
        let mut mgr = fresh_manager();
        let id = mgr.spawn(ReasoningDomain::KnowledgeQuery, "knowledge-search", 10);
        assert!(id.is_some());
        assert_eq!(mgr.active_count(), 1);
        let id = id.ok_or_else(|| "spawn returned None after assert".to_string())?;
        let sub = mgr.get(id).unwrap();
        assert_eq!(sub.name, "knowledge-search");
        assert_eq!(sub.domain, ReasoningDomain::KnowledgeQuery);
        assert_eq!(sub.status, SubConsciousnessStatus::Running);
        assert_eq!(sub.local_cycle, 0);
        Ok(())
    }

    #[test]
    fn test_tick_and_complete() {
        let mut mgr = fresh_manager();
        let id = mgr
            .spawn(ReasoningDomain::Planning, "plan-test", 5)
            .unwrap();

        mgr.tick(id, "input_data").unwrap();
        assert_eq!(mgr.get(id).unwrap().local_cycle, 1);

        mgr.complete(id, "plan result", Some(vec![0u8; 64]))
            .unwrap();
        assert_eq!(
            mgr.get(id).unwrap().status,
            SubConsciousnessStatus::Completed
        );

        let collected = mgr.collect_completed();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].1, "plan result");
        assert_eq!(collected[0].2, Some(vec![0u8; 64]));
    }

    #[test]
    fn test_capacity_limit_enforcement() {
        let mut mgr = SubConsciousnessManager::new(2);
        assert!(mgr.spawn(ReasoningDomain::KnowledgeQuery, "a", 1).is_some());
        assert!(mgr.spawn(ReasoningDomain::Planning, "b", 1).is_some());
        assert!(mgr.spawn(ReasoningDomain::Simulation, "c", 1).is_none());
        assert_eq!(mgr.active_count(), 2);
    }

    #[test]
    fn test_tick_triggers_checkpoint_pending() {
        let mut mgr = fresh_manager();
        let id = mgr
            .spawn(ReasoningDomain::SelfInspection, "self-check", 3)
            .unwrap();
        mgr.tick(id, "").unwrap();
        mgr.tick(id, "").unwrap();
        mgr.tick(id, "").unwrap();
        assert_eq!(
            mgr.get(id).unwrap().status,
            SubConsciousnessStatus::CheckpointPending
        );
        assert_eq!(mgr.get(id).unwrap().local_cycle, 3);
    }

    #[test]
    fn test_fail_and_active_count() {
        let mut mgr = fresh_manager();
        let id = mgr
            .spawn(ReasoningDomain::Custom("test".into()), "t", 1)
            .unwrap();
        mgr.fail(id, "timeout").unwrap();
        assert_eq!(mgr.active_count(), 0);
        assert_eq!(mgr.total_count(), 1);
    }

    #[test]
    fn test_collect_completed_only_removes_completed() {
        let mut mgr = fresh_manager();
        let id_a = mgr.spawn(ReasoningDomain::KnowledgeQuery, "a", 1).unwrap();
        let id_b = mgr.spawn(ReasoningDomain::Planning, "b", 1).unwrap();
        mgr.complete(id_a, "done", None).unwrap();
        let collected = mgr.collect_completed();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, id_a);
        assert!(mgr.get(id_b).is_some());
        assert!(mgr.get(id_a).is_none());
    }

    #[test]
    fn test_summary() {
        let mut mgr = fresh_manager();
        mgr.spawn(ReasoningDomain::KnowledgeQuery, "kq", 1).unwrap();
        mgr.spawn(ReasoningDomain::Planning, "pl", 1).unwrap();
        let summary = mgr.summary();
        assert_eq!(summary.len(), 2);
        for (_, name, status) in &summary {
            assert_eq!(status, "Running");
            assert!(["kq", "pl"].contains(&name.as_str()));
        }
    }

    #[test]
    fn test_tick_nonexistent_returns_error() {
        let mut mgr = fresh_manager();
        let result = mgr.tick(999, "data");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("999"));
    }
}
