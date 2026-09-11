use crate::agents::SimAgent;

/// A mutation to be tested in the sandbox.
#[derive(Debug, Clone)]
pub struct Mutation {
    pub trait_index: usize,
    pub delta: f32,
    pub description: String,
}

/// Statistics about sandbox operations.
#[derive(Debug, Clone, Default)]
pub struct SandboxStats {
    pub tested: u32,
    pub promoted: u32,
    pub rejected: u32,
    pub avg_fitness_delta: f32,
}

/// Evolution sandbox that tests mutations in isolation before promoting to the main population.
pub struct EvolutionSandbox {
    pub sandbox_agents: Vec<SimAgent>,
    pub sandbox_world: WorldState,
    pub promotion_threshold: f32,
    stats: SandboxStats,
}

/// Simplified world state for sandbox testing.
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub tick: u64,
    pub resource_scarcity: f32,
    pub threat_level: f32,
}

impl EvolutionSandbox {
    pub fn new(promotion_threshold: f32) -> Self {
        Self {
            sandbox_agents: Vec::new(),
            sandbox_world: WorldState::default(),
            promotion_threshold,
            stats: SandboxStats::default(),
        }
    }

    /// Test a mutation on an agent in the sandbox and return the fitness delta.
    pub fn test_mutation(&mut self, agent: &SimAgent, mutation: Mutation) -> f32 {
        let baseline_fitness = agent.fitness() as f32;

        // Create a mutated clone
        let mut mutated = agent.clone();
        match mutation.trait_index {
            0 => mutated.core.health = (mutated.core.health + mutation.delta * 100.0).clamp(0.0, 100.0),
            1 => mutated.core.energy = (mutated.core.energy + mutation.delta * 100.0).clamp(0.0, 100.0),
            2 => mutated.personality.sociability = (mutated.personality.sociability + mutation.delta).clamp(0.0, 1.0),
            3 => mutated.personality.curiosity = (mutated.personality.curiosity + mutation.delta).clamp(0.0, 1.0),
            4 => mutated.personality.aggression = (mutated.personality.aggression + mutation.delta).clamp(0.0, 1.0),
            5 => mutated.personality.cooperativeness = (mutated.personality.cooperativeness + mutation.delta).clamp(0.0, 1.0),
            _ => {}
        }

        let mutated_fitness = mutated.fitness() as f32;

        // Simulate a few ticks in sandbox
        self.sandbox_agents.clear();
        self.sandbox_agents.push(mutated.clone());
        for _ in 0..10 {
            self.sandbox_tick();
        }
        let post_sim_fitness = self.sandbox_agents.first()
            .map(|a| a.fitness() as f32)
            .unwrap_or(mutated_fitness);

        self.sandbox_agents.clear();
        self.stats.tested += 1;

        post_sim_fitness - baseline_fitness
    }

    /// Promote an agent from the sandbox to the main population.
    pub fn promote(&mut self, agent: SimAgent) -> Option<SimAgent> {
        let fitness = agent.fitness() as f32;
        if fitness >= self.promotion_threshold {
            self.stats.promoted += 1;
            Some(agent)
        } else {
            self.reject(agent);
            None
        }
    }

    /// Reject an agent from the sandbox.
    pub fn reject(&mut self, agent: SimAgent) {
        self.stats.rejected += 1;
        // Agent is simply dropped
        drop(agent);
    }

    /// Get sandbox statistics.
    pub fn get_sandbox_stats(&self) -> SandboxStats {
        self.stats.clone()
    }

    fn sandbox_tick(&mut self) {
        self.sandbox_world.tick += 1;
        for agent in &mut self.sandbox_agents {
            if !agent.is_alive() {
                continue;
            }
            // Simple simulation: energy decays, health adjusts
            agent.core.energy = (agent.core.energy - 0.5).max(0.0);
            agent.core.hunger = (agent.core.hunger + 1.0).min(100.0);
            if agent.core.hunger > 80.0 {
                agent.core.health -= 0.5;
            }
            if agent.core.energy <= 0.0 || agent.core.health <= 0.0 {
                agent.core.alive = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn test_mutation_returns_fitness_delta() {
        let mut sandbox = EvolutionSandbox::new(0.5);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.core.energy = 80.0;
        let mutation = Mutation {
            trait_index: 1,
            delta: 0.1,
            description: "increase energy".into(),
        };
        let delta = sandbox.test_mutation(&agent, mutation);
        // Delta can be positive or negative, but the function should return a value
        assert!(delta.abs() < 100.0);
    }

    #[test]
    fn promote_accepts_high_fitness() {
        let mut sandbox = EvolutionSandbox::new(0.3);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.core.energy = 100.0;
        agent.core.health = 100.0;
        let result = sandbox.promote(agent);
        assert!(result.is_some());
        assert_eq!(sandbox.get_sandbox_stats().promoted, 1);
    }

    #[test]
    fn promote_rejects_low_fitness() {
        let mut sandbox = EvolutionSandbox::new(0.9);
        let mut agent = SimAgent::new(1, Vec2::zero());
        agent.core.energy = 10.0;
        agent.core.health = 10.0;
        let result = sandbox.promote(agent);
        assert!(result.is_none());
        assert_eq!(sandbox.get_sandbox_stats().rejected, 1);
    }

    #[test]
    fn sandbox_stats_track_correctly() {
        let mut sandbox = EvolutionSandbox::new(0.5);
        let agent = SimAgent::new(1, Vec2::zero());
        sandbox.test_mutation(&agent, Mutation {
            trait_index: 0,
            delta: 0.1,
            description: "test".into(),
        });
        assert_eq!(sandbox.get_sandbox_stats().tested, 1);
    }
}
