use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub parameters: HashMap<String, String>,
}

pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    fn choose_action(
        &self,
        state: &HashMap<String, String>,
        legal_actions: &[Action],
    ) -> Option<Action>;
    fn update(
        &mut self,
        state: &HashMap<String, String>,
        action: &Action,
        reward: f64,
        next_state: &HashMap<String, String>,
    );
}

pub struct GreedyPolicy;

impl GreedyPolicy {
    pub fn new() -> Self {
        Self
    }
}

impl Policy for GreedyPolicy {
    fn name(&self) -> &str {
        "greedy"
    }
    fn choose_action(
        &self,
        _state: &HashMap<String, String>,
        legal_actions: &[Action],
    ) -> Option<Action> {
        legal_actions.first().cloned()
    }
    fn update(
        &mut self,
        _: &HashMap<String, String>,
        _: &Action,
        _: f64,
        _: &HashMap<String, String>,
    ) {
    }
}

impl Default for GreedyPolicy {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RandomPolicy {
    seed: u64,
}

impl RandomPolicy {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl Policy for RandomPolicy {
    fn name(&self) -> &str {
        "random"
    }
    fn choose_action(
        &self,
        _state: &HashMap<String, String>,
        legal_actions: &[Action],
    ) -> Option<Action> {
        if legal_actions.is_empty() {
            None
        } else {
            Some(legal_actions[(self.seed as usize) % legal_actions.len()].clone())
        }
    }
    fn update(
        &mut self,
        _: &HashMap<String, String>,
        _: &Action,
        _: f64,
        _: &HashMap<String, String>,
    ) {
        self.seed = self.seed.wrapping_add(1);
    }
}

pub struct EpsilonGreedyPolicy {
    inner: Box<dyn Policy>,
    epsilon: f64,
    rng_state: AtomicU64,
}

impl EpsilonGreedyPolicy {
    pub fn new(inner: Box<dyn Policy>, epsilon: f64) -> Self {
        Self {
            inner,
            epsilon,
            rng_state: AtomicU64::new(0),
        }
    }
}

impl Policy for EpsilonGreedyPolicy {
    fn name(&self) -> &str {
        "epsilon_greedy"
    }
    fn choose_action(
        &self,
        state: &HashMap<String, String>,
        legal_actions: &[Action],
    ) -> Option<Action> {
        let old = self.rng_state.load(Ordering::Relaxed);
        let new = old
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_state.store(new, Ordering::Relaxed);
        let r = (new >> 33) as f64 / (1u64 << 31) as f64;
        if r < self.epsilon {
            legal_actions.first().cloned()
        } else {
            self.inner.choose_action(state, legal_actions)
        }
    }
    fn update(
        &mut self,
        state: &HashMap<String, String>,
        action: &Action,
        reward: f64,
        next_state: &HashMap<String, String>,
    ) {
        self.inner.update(state, action, reward, next_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy() {
        let p = GreedyPolicy::new();
        let mut s = HashMap::new();
        let a = vec![Action {
            name: "attack".into(),
            parameters: HashMap::new(),
        }];
        assert!(p.choose_action(&s, &a).is_some());
    }

    #[test]
    fn test_random() {
        let p = RandomPolicy::new(42);
        let s = HashMap::new();
        let a = vec![
            Action {
                name: "a".into(),
                parameters: HashMap::new(),
            },
            Action {
                name: "b".into(),
                parameters: HashMap::new(),
            },
        ];
        assert!(p.choose_action(&s, &a).is_some());
    }

    #[test]
    fn test_epsilon_greedy() {
        let p = EpsilonGreedyPolicy::new(Box::new(GreedyPolicy::new()), 0.0);
        let s = HashMap::new();
        let a = vec![Action {
            name: "x".into(),
            parameters: HashMap::new(),
        }];
        assert!(p.choose_action(&s, &a).is_some());
    }
}
