use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringParams {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
    pub initial_velocity: f64,
}

impl Default for SpringParams {
    fn default() -> Self {
        SpringParams {
            stiffness: 200.0,
            damping: 20.0,
            mass: 1.0,
            initial_velocity: 0.0,
        }
    }
}

impl SpringParams {
    pub fn new(stiffness: f64, damping: f64) -> Self {
        SpringParams {
            stiffness,
            damping,
            mass: 1.0,
            initial_velocity: 0.0,
        }
    }

    pub fn expressive() -> Self {
        SpringParams { stiffness: 180.0, damping: 15.0, mass: 1.0, initial_velocity: 0.0 }
    }

    pub fn standard() -> Self {
        SpringParams { stiffness: 200.0, damping: 20.0, mass: 1.0, initial_velocity: 0.0 }
    }

    pub fn gentle() -> Self {
        SpringParams { stiffness: 120.0, damping: 25.0, mass: 1.0, initial_velocity: 0.0 }
    }

    pub fn snappy() -> Self {
        SpringParams { stiffness: 400.0, damping: 30.0, mass: 1.0, initial_velocity: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct SpringSimulation {
    params: SpringParams,
    steps: Vec<f64>,
    velocities: Vec<f64>,
}

impl SpringSimulation {
    pub fn new(params: SpringParams) -> Self {
        SpringSimulation {
            params,
            steps: Vec::new(),
            velocities: Vec::new(),
        }
    }

    pub fn simulate(&mut self, duration_secs: f64, dt: f64) -> &[f64] {
        let num_steps = (duration_secs / dt).ceil() as usize;
        self.steps = Vec::with_capacity(num_steps);
        self.velocities = Vec::with_capacity(num_steps);

        let mut position = 1.0;
        let mut velocity = self.params.initial_velocity;

        for _ in 0..num_steps {
            self.steps.push(position);
            self.velocities.push(velocity);

            let spring_force = -self.params.stiffness * position;
            let damping_force = -self.params.damping * velocity;
            let acceleration = (spring_force + damping_force) / self.params.mass;

            velocity += acceleration * dt;
            position += velocity * dt;
        }

        &self.steps
    }

    pub fn position_at(&self, t: f64) -> f64 {
        let idx = t.max(0.0).min(1.0);
        if self.steps.is_empty() {
            return 1.0;
        }
        let i = ((idx * (self.steps.len() - 1) as f64) as usize).min(self.steps.len() - 1);
        self.steps[i]
    }

    pub fn is_at_rest(&self, threshold: f64) -> bool {
        self.steps.last().map_or(true, |&p| p.abs() < threshold)
    }

    pub fn duration_until_rest(&self, threshold: f64, dt: f64) -> f64 {
        let mut position: f64 = 1.0;
        let mut velocity: f64 = self.params.initial_velocity;
        let mut t: f64 = 0.0;
        let max_steps = 10000;
        for _ in 0..max_steps {
            if position.abs() < threshold {
                return t;
            }
            let spring_force = -self.params.stiffness * position;
            let damping_force = -self.params.damping * velocity;
            let acceleration = (spring_force + damping_force) / self.params.mass;
            velocity += acceleration * dt;
            position += velocity * dt;
            t += dt;
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_simulation() {
        let mut sim = SpringSimulation::new(SpringParams::standard());
        let positions = sim.simulate(1.0, 0.016).to_vec();
        assert!(!positions.is_empty());
        assert!((positions[0] - 1.0).abs() < 0.01);
        assert!(positions.last().unwrap().abs() < 0.1);
    }

    #[test]
    fn test_expressive_overshoots() {
        let mut exp = SpringSimulation::new(SpringParams::expressive());
        let pos = exp.simulate(1.0, 0.016).to_vec();
        let has_overshoot = pos.iter().any(|&p| p < 0.0);
        assert!(has_overshoot);
    }

    #[test]
    fn test_duration_until_rest() {
        let params = SpringParams::snappy();
        let sim = SpringSimulation::new(params);
        let dur = sim.duration_until_rest(0.01, 0.016);
        assert!(dur > 0.0 && dur < 2.0);
    }

    #[test]
    fn test_spring_params_defaults() {
        let p = SpringParams::default();
        assert_eq!(p.stiffness, 200.0);
        assert_eq!(p.damping, 20.0);
    }
}
