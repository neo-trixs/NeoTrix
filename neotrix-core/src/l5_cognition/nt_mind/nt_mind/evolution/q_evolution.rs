#[derive(Debug, Clone)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub q_value: f64,
    pub visits: u32,
}

pub struct QEvolution {
    strategies: Vec<Strategy>,
    learning_rate: f64,
    epsilon: f64,
}

impl QEvolution {
    pub fn new(learning_rate: f64, epsilon: f64) -> Self {
        Self {
            strategies: Vec::new(),
            learning_rate,
            epsilon,
        }
    }
    pub fn add_strategy(&mut self, id: &str, name: &str) {
        self.strategies.push(Strategy {
            id: id.to_string(),
            name: name.to_string(),
            q_value: 0.0,
            visits: 0,
        });
    }
    pub fn select(&self) -> Option<&Strategy> {
        if self.strategies.is_empty() {
            return None;
        }
        if self.strategies.len() == 1 {
            return self.strategies.first();
        }
        let use_random = (rand::random::<f64>()) < self.epsilon;
        if use_random {
            let idx = (rand::random::<f64>() * self.strategies.len() as f64) as usize;
            self.strategies.get(idx.min(self.strategies.len() - 1))
        } else {
            self.strategies.iter().max_by(|a, b| {
                a.q_value
                    .partial_cmp(&b.q_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        }
    }
    pub fn update(&mut self, id: &str, reward: f64) {
        if let Some(s) = self.strategies.iter_mut().find(|s| s.id == id) {
            s.visits += 1;
            s.q_value += self.learning_rate * (reward - s.q_value);
        }
    }
    pub fn best(&self) -> Option<&Strategy> {
        self.strategies.iter().max_by(|a, b| {
            a.q_value
                .partial_cmp(&b.q_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
    pub fn count(&self) -> usize {
        self.strategies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_and_select() {
        let mut q = QEvolution::new(0.1, 0.1);
        q.add_strategy("s1", "greedy");
        q.add_strategy("s2", "explore");
        assert!(q.select().is_some());
    }
    #[test]
    fn test_update() {
        let mut q = QEvolution::new(0.1, 0.0);
        q.add_strategy("s1", "test");
        q.update("s1", 1.0);
        assert!(q.best().unwrap().q_value > 0.0);
    }
}
