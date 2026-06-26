#[derive(Debug, Clone)]
pub struct ModernHopfieldNetwork {
    pub memory_patterns: Vec<Vec<f64>>,
    pub dims: usize,
    pub beta: f64,
}

impl ModernHopfieldNetwork {
    pub fn new(dims: usize, beta: f64) -> Self {
        Self {
            memory_patterns: vec![],
            dims,
            beta,
        }
    }
    pub fn store(&mut self, pattern: Vec<f64>) {
        self.memory_patterns.push(pattern);
    }
    pub fn retrieve(&self, query: &[f64]) -> Vec<f64> {
        if self.memory_patterns.is_empty() {
            return query.to_vec();
        }
        let mut best_score = -f64::INFINITY;
        let mut best = query.to_vec();
        for p in &self.memory_patterns {
            let dot: f64 = p.iter().zip(query).map(|(a, b)| a * b).sum();
            let score = (self.beta * dot).exp();
            if score > best_score {
                best_score = score;
                best = p.clone();
            }
        }
        best
    }
}
