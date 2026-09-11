use crate::agents::SimAgent;

/// Budget status after checking an agent's resource usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetStatus {
    WithinBudget,
    ApproachingLimit,
    Exceeded,
}

/// Resource usage metrics for an agent.
#[derive(Debug, Clone)]
pub struct BudgetUsage {
    pub actions_this_tick: u32,
    pub memory_bytes: usize,
    pub cpu_time_ms: f32,
}

/// Enforces per-agent resource budgets to prevent runaway agents.
pub struct BudgetEnforcer {
    pub max_actions_per_tick: u32,
    pub max_memory_bytes: usize,
    pub max_cpu_time_ms: f32,
}

impl BudgetEnforcer {
    pub fn new(max_actions: u32, max_memory: usize, max_cpu_ms: f32) -> Self {
        Self {
            max_actions_per_tick: max_actions,
            max_memory_bytes: max_memory,
            max_cpu_time_ms: max_cpu_ms,
        }
    }

    /// Check an agent's current budget status.
    pub fn check(&self, agent: &SimAgent) -> BudgetStatus {
        let usage = self.get_usage(agent);
        if usage.actions_this_tick > self.max_actions_per_tick
            || usage.memory_bytes > self.max_memory_bytes
            || usage.cpu_time_ms > self.max_cpu_time_ms
        {
            BudgetStatus::Exceeded
        } else if usage.actions_this_tick as f32 > self.max_actions_per_tick as f32 * 0.8
            || usage.memory_bytes as f64 > self.max_memory_bytes as f64 * 0.8
            || usage.cpu_time_ms > self.max_cpu_time_ms * 0.8
        {
            BudgetStatus::ApproachingLimit
        } else {
            BudgetStatus::WithinBudget
        }
    }

    /// Enforce budget limits on an agent by clamping excessive resource usage.
    pub fn enforce(&self, agent: &mut SimAgent) {
        let action_count = agent.recent_actions.len() as u32;
        if action_count > self.max_actions_per_tick {
            let excess = (action_count - self.max_actions_per_tick) as usize;
            agent.recent_actions.drain(..excess);
        }
        // Energy cost scaling: agents over budget lose extra energy
        if self.check(agent) == BudgetStatus::Exceeded {
            agent.core.energy = (agent.core.energy - 2.0).max(0.0);
        }
    }

    /// Get the current resource usage of an agent.
    pub fn get_usage(&self, agent: &SimAgent) -> BudgetUsage {
        BudgetUsage {
            actions_this_tick: agent.recent_actions.len() as u32,
            memory_bytes: Self::estimate_memory(agent),
            cpu_time_ms: Self::estimate_cpu(agent),
        }
    }

    fn estimate_memory(agent: &SimAgent) -> usize {
        let base = std::mem::size_of::<SimAgent>();
        let actions = agent.recent_actions.iter().map(|a| a.len()).sum::<usize>();
        let skills = agent.skills.iter().map(|s| s.len()).sum::<usize>();
        base + actions + skills
    }

    fn estimate_cpu(agent: &SimAgent) -> f32 {
        // Heuristic: more recent actions = more CPU used
        agent.recent_actions.len() as f32 * 0.5
    }
}

impl Default for BudgetEnforcer {
    fn default() -> Self {
        Self::new(50, 1024 * 1024, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn within_budget_for_idle_agent() {
        let enforcer = BudgetEnforcer::default();
        let agent = SimAgent::new(1, Vec2::zero());
        assert_eq!(enforcer.check(&agent), BudgetStatus::WithinBudget);
    }

    #[test]
    fn detects_approaching_limit() {
        let enforcer = BudgetEnforcer::new(5, 1024 * 1024, 100.0);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.recent_actions = vec!["a".into(); 4]; // 4/5 = 80%
        assert_eq!(enforcer.check(&agent), BudgetStatus::ApproachingLimit);
    }

    #[test]
    fn detects_exceeded() {
        let enforcer = BudgetEnforcer::new(3, 1024 * 1024, 100.0);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.recent_actions = vec!["a".into(); 5]; // 5 > 3
        assert_eq!(enforcer.check(&agent), BudgetStatus::Exceeded);
    }

    #[test]
    fn enforce_trims_actions() {
        let enforcer = BudgetEnforcer::new(3, 1024 * 1024, 100.0);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.recent_actions = vec!["a".into(); 5];
        enforcer.enforce(&mut agent);
        assert!(agent.recent_actions.len() <= 3);
    }

    #[test]
    fn get_usage_returns_metrics() {
        let enforcer = BudgetEnforcer::default();
        let agent = SimAgent::new(1, Vec2::zero());
        let usage = enforcer.get_usage(&agent);
        assert_eq!(usage.actions_this_tick, 0);
    }
}
