//! Agent Load Balancer
//!
//! Tracks per-agent task load, completed-task count, and average duration.
//! Provides `rebalance()` for distributing tasks across agents proportionally
//! to their available capacity.

use serde::{Deserialize, Serialize};


/// Statistics for a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub agent_id: String,
    pub active_tasks: u32,
    pub completed_tasks: u32,
    pub avg_duration: f64,
}

impl AgentStats {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            active_tasks: 0,
            completed_tasks: 0,
            avg_duration: 0.0,
        }
    }

    /// Total tasks handled (active + completed)
    pub fn total_tasks(&self) -> u32 {
        self.active_tasks + self.completed_tasks
    }
}

/// Manages load tracking and rebalancing across agents
pub struct AgentLoadBalancer {
    stats: Vec<AgentStats>,
}

impl AgentLoadBalancer {
    pub fn new() -> Self {
        Self { stats: Vec::new() }
    }

    /// Register an agent for load tracking
    pub fn register_agent(&mut self, agent_id: &str) {
        if !self.stats.iter().any(|s| s.agent_id == agent_id) {
            self.stats.push(AgentStats::new(agent_id));
        }
    }

    /// Remove an agent from load tracking
    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.stats.retain(|s| s.agent_id != agent_id);
    }

    /// Get the current load for an agent (active_tasks as fraction of max capacity).
    ///
    /// Returns 0.0–1.0 where 1.0 means fully loaded.
    /// Returns 0.0 if agent is not tracked.
    pub fn get_load(&self, agent_id: &str) -> f64 {
        self.stats
            .iter()
            .find(|s| s.agent_id == agent_id)
            .map(|s| {
                // ⚠️ 2026-10-02：原有一行 `let total = s.total_tasks() as f64;`
                // 被算出却**从未参与计算**（分母是硬编码的 10.0）⇒ 死计算。
                // ⛔ 未改用 `total` 作分母：那会**改变 get_load 的语义**，
                //    属于行为变更而非冗余清理。保留硬编码上限，仅删死变量。
                // Normalize against a soft cap of 10 concurrent tasks
                (s.active_tasks as f64 / 10.0).min(1.0)
            })
            .unwrap_or(0.0)
    }

    /// Get load for all agents
    pub fn get_all_loads(&self) -> Vec<(String, f64)> {
        self.stats
            .iter()
            .map(|s| (s.agent_id.clone(), self.get_load(&s.agent_id)))
            .collect()
    }

    /// 该 agent 当前的**活跃任务数**（未归一化的原始计数）。
    ///
    /// 2026-10-02 新增：`get_load` 返回归一化 f64（`active/10`，上限 1.0），
    /// 而 `TaskRouter::route_with_initial_load` 需要**整数计数**做比较。
    /// ⇒ 单独暴露原始计数，避免调用方从归一化值反推（会丢分辨率：
    ///    12 个任务与 10 个任务的归一化值都是 1.0）。
    pub fn active_task_count(&self, agent_id: &str) -> u32 {
        self.stats
            .iter()
            .find(|s| s.agent_id == agent_id)
            .map(|s| s.active_tasks)
            .unwrap_or(0)
    }

    /// Increment active task count for an agent
    pub fn task_started(&mut self, agent_id: &str) {
        if let Some(s) = self.stats.iter_mut().find(|s| s.agent_id == agent_id) {
            s.active_tasks += 1;
        }
    }

    /// Decrement active task count and increment completed count.
    /// Updates rolling average duration.
    pub fn task_completed(&mut self, agent_id: &str, duration: f64) {
        if let Some(s) = self.stats.iter_mut().find(|s| s.agent_id == agent_id) {
            s.active_tasks = s.active_tasks.saturating_sub(1);
            // Exponential moving average for duration
            let n = s.completed_tasks as f64;
            s.avg_duration = if n == 0.0 {
                duration
            } else {
                (s.avg_duration * n + duration) / (n + 1.0)
            };
            s.completed_tasks += 1;
        }
    }

    /// Rebalance: return agent IDs sorted by load (lowest first).
    ///
    /// Agents with lower current load are prioritized for new task assignment.
    pub fn rebalance(&self) -> Vec<String> {
        let mut sorted: Vec<&AgentStats> = self.stats.iter().collect();
        sorted.sort_by(|a, b| {
            let load_a = self.get_load(&a.agent_id);
            let load_b = self.get_load(&b.agent_id);
            load_a
                .partial_cmp(&load_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().map(|s| s.agent_id.clone()).collect()
    }

    /// Get stats for all agents
    pub fn get_agent_stats(&self) -> Vec<AgentStats> {
        self.stats.clone()
    }

    /// Get stats for a specific agent
    pub fn get_stats(&self, agent_id: &str) -> Option<&AgentStats> {
        self.stats.iter().find(|s| s.agent_id == agent_id)
    }

    /// Number of tracked agents
    pub fn len(&self) -> usize {
        self.stats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }
}

impl Default for AgentLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_stats_new() {
        let s = AgentStats::new("a1");
        assert_eq!(s.agent_id, "a1");
        assert_eq!(s.active_tasks, 0);
        assert_eq!(s.completed_tasks, 0);
        assert_eq!(s.avg_duration, 0.0);
        assert_eq!(s.total_tasks(), 0);
    }

    #[test]
    fn load_balancer_register_and_load() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        assert_eq!(lb.get_load("a1"), 0.0);
        assert_eq!(lb.len(), 1);
    }

    #[test]
    fn load_balancer_task_lifecycle() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");

        lb.task_started("a1");
        lb.task_started("a1");
        assert!((lb.get_load("a1") - 0.2).abs() < 0.01); // 2/10

        lb.task_completed("a1", 5.0);
        assert_eq!(lb.get_load("a1"), 0.1); // 1/10
        let stats = lb.get_stats("a1").unwrap();
        assert_eq!(stats.completed_tasks, 1);
        assert!((stats.avg_duration - 5.0).abs() < 0.01);
    }

    #[test]
    fn load_balancer_rolling_average() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");

        lb.task_completed("a1", 2.0);
        lb.task_completed("a1", 8.0);

        let stats = lb.get_stats("a1").unwrap();
        assert!((stats.avg_duration - 5.0).abs() < 0.01);
        assert_eq!(stats.completed_tasks, 2);
    }

    #[test]
    fn load_balancer_rebalance_sorts_by_load() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.register_agent("a2");
        lb.register_agent("a3");

        lb.task_started("a1");
        lb.task_started("a1");
        lb.task_started("a1");
        // a1: 3/10, a2: 0/10, a3: 0/10

        let order = lb.rebalance();
        assert_eq!(order[0], "a2"); // lowest load first
        assert_eq!(order[1], "a3");
        assert_eq!(order[2], "a1"); // highest load last
    }

    #[test]
    fn load_balancer_unregister() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.register_agent("a2");
        lb.unregister_agent("a1");
        assert_eq!(lb.len(), 1);
        assert_eq!(lb.get_load("a1"), 0.0);
        assert!(lb.get_stats("a1").is_none());
    }

    #[test]
    fn load_balancer_unknown_agent() {
        let lb = AgentLoadBalancer::new();
        assert_eq!(lb.get_load("nonexistent"), 0.0);
        assert!(lb.get_stats("nonexistent").is_none());
    }

    #[test]
    fn load_balancer_empty() {
        let lb = AgentLoadBalancer::new();
        assert!(lb.is_empty());
        assert!(lb.rebalance().is_empty());
    }

    #[test]
    fn load_balancer_get_all_loads() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.register_agent("a2");
        lb.task_started("a1");

        let loads = lb.get_all_loads();
        assert_eq!(loads.len(), 2);
    }

    #[test]
    fn load_capped_at_1_0() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        for _ in 0..20 {
            lb.task_started("a1");
        }
        assert_eq!(lb.get_load("a1"), 1.0);
    }

    #[test]
    fn load_balancer_unregister_mid_task() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.task_started("a1");
        lb.unregister_agent("a1");
        assert_eq!(lb.get_load("a1"), 0.0);
        assert!(lb.get_stats("a1").is_none());
    }

    #[test]
    fn agent_stats_total_tasks() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.task_started("a1");
        lb.task_started("a1");
        lb.task_completed("a1", 3.0);
        let stats = lb.get_stats("a1").unwrap();
        assert_eq!(stats.total_tasks(), 2); // 1 active + 1 completed
    }

    #[test]
    fn load_balancer_rolling_average_three_tasks() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.task_completed("a1", 10.0);
        lb.task_completed("a1", 20.0);
        lb.task_completed("a1", 30.0);
        let stats = lb.get_stats("a1").unwrap();
        assert!((stats.avg_duration - 20.0).abs() < 0.01);
        assert_eq!(stats.completed_tasks, 3);
    }

    #[test]
    fn rebalance_preserves_order_with_same_load() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.register_agent("a2");
        lb.register_agent("a3");
        // All have 0 load — order should be stable
        let order = lb.rebalance();
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn task_completed_unknown_agent_no_panic() {
        let mut lb = AgentLoadBalancer::new();
        lb.register_agent("a1");
        lb.task_completed("ghost", 5.0); // should be no-op
        assert_eq!(lb.get_stats("a1").unwrap().completed_tasks, 0);
    }

    #[test]
    fn task_started_unknown_agent_no_panic() {
        let mut lb = AgentLoadBalancer::new();
        lb.task_started("ghost"); // should be no-op
        assert!(lb.is_empty());
    }

    #[test]
    fn get_all_loads_empty() {
        let lb = AgentLoadBalancer::new();
        assert!(lb.get_all_loads().is_empty());
    }

    #[test]
    fn load_balancer_default() {
        let lb = AgentLoadBalancer::default();
        assert!(lb.is_empty());
        assert_eq!(lb.len(), 0);
    }
}
