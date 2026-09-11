use crate::agents::SimAgent;
use crate::foundation::math_bridge::Vec2;

#[derive(Debug, Clone)]
pub struct RVOAgent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub preferred_velocity: Vec2,
    pub max_speed: f32,
    pub neighbor_dist: f32,
    pub time_horizon: f32,
}

impl RVOAgent {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            velocity: Vec2::zero(),
            preferred_velocity: Vec2::zero(),
            max_speed: 2.0,
            neighbor_dist: 15.0,
            time_horizon: 5.0,
        }
    }

    pub fn from_agent(agent: &SimAgent) -> Self {
        Self::new(agent.core.position.x, agent.core.position.y)
    }
}

pub struct RVOSimulator {
    pub agents: Vec<RVOAgent>,
    pub time_step: f32,
    pub default_neighbor_dist: f32,
    pub default_time_horizon: f32,
}

impl RVOSimulator {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            time_step: 0.25,
            default_neighbor_dist: 15.0,
            default_time_horizon: 5.0,
        }
    }

    pub fn add_agent(&mut self, agent: RVOAgent) -> usize {
        let idx = self.agents.len();
        self.agents.push(agent);
        idx
    }

    pub fn remove_agent(&mut self, idx: usize) {
        if idx < self.agents.len() {
            self.agents.remove(idx);
        }
    }

    pub fn set_preferred_velocity(&mut self, idx: usize, vx: f32, vy: f32) {
        if let Some(agent) = self.agents.get_mut(idx) {
            agent.preferred_velocity = Vec2::new(vx, vy);
        }
    }

    pub fn compute_new_velocity(&self, idx: usize) -> Vec2 {
        let agent = &self.agents[idx];
        let neighbors = self.get_neighbors(idx);

        if neighbors.is_empty() {
            return agent.preferred_velocity;
        }

        let mut new_vel = agent.preferred_velocity;

        for &n_idx in &neighbors {
            let neighbor = &self.agents[n_idx];
            let rel_pos = agent.position - neighbor.position;
            let rel_vel = agent.velocity - neighbor.velocity;

            let dist = rel_pos.length();
            if dist < 0.01 {
                continue;
            }

            let combined_radius = 1.0;
            let penetration = combined_radius - dist;

            if penetration > 0.0 {
                let weight = penetration / dist;
                new_vel = new_vel + rel_pos * (weight * 0.5);
            }
        }

        let speed = new_vel.length();
        if speed > agent.max_speed {
            new_vel = new_vel * (agent.max_speed / speed);
        }

        new_vel
    }

    pub fn step(&mut self) {
        let new_velocities: Vec<Vec2> = (0..self.agents.len())
            .map(|i| self.compute_new_velocity(i))
            .collect();

        for (i, vel) in new_velocities.into_iter().enumerate() {
            self.agents[i].velocity = vel;
            self.agents[i].position = self.agents[i].position + vel * self.time_step;
        }
    }

    fn get_neighbors(&self, idx: usize) -> Vec<usize> {
        let agent = &self.agents[idx];
        self.agents
            .iter()
            .enumerate()
            .filter(|(i, other)| {
                if *i == idx {
                    return false;
                }
                agent.position.distance_to(&other.position) <= agent.neighbor_dist
            })
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rvo_agent() {
        let agent = RVOAgent::new(0.0, 0.0);
        assert_eq!(agent.position, Vec2::new(0.0, 0.0));
        assert_eq!(agent.max_speed, 2.0);
    }

    #[test]
    fn test_rvo_simulator() {
        let mut sim = RVOSimulator::new();
        sim.add_agent(RVOAgent::new(0.0, 0.0));
        sim.add_agent(RVOAgent::new(1.0, 0.0));
        assert_eq!(sim.agents.len(), 2);
    }

    #[test]
    fn test_rvo_velocity() {
        let mut sim = RVOSimulator::new();
        let idx = sim.add_agent(RVOAgent::new(0.0, 0.0));
        sim.set_preferred_velocity(idx, 1.0, 0.0);

        let vel = sim.compute_new_velocity(idx);
        assert_eq!(vel, Vec2::new(1.0, 0.0));
    }
}
