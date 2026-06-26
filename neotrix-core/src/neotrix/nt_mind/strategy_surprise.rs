use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct SuSStats {
    pub step: u64,
    pub strategy_stability: f64,
    pub strategy_surprise: f64,
    pub sus_bonus: f64,
    pub ss_weight: f64,
    pub sus_weight: f64,
    pub strategy_buffer_size: usize,
}

pub struct StrategyAwareSurprise {
    strategy_buffer: VecDeque<Vec<u8>>,
    ss_weight: f64,
    sus_weight: f64,
    window_size: usize,
    _vsa_dim: usize,
    prev_strategy: Option<Vec<u8>>,
    step: u64,
    strategy_stability: f64,
    strategy_surprise: f64,
    last_bonus: f64,
}

impl StrategyAwareSurprise {
    pub fn new(window_size: usize, vsa_dim: usize) -> Self {
        Self {
            strategy_buffer: VecDeque::with_capacity(window_size),
            ss_weight: 0.5,
            sus_weight: 0.5,
            window_size: window_size.max(3),
            _vsa_dim: vsa_dim,
            prev_strategy: None,
            step: 0,
            strategy_stability: 0.0,
            strategy_surprise: 0.0,
            last_bonus: 0.0,
        }
    }

    fn hamming_sim(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let same = a.iter().zip(b.iter()).filter(|(&x, &y)| x == y).count();
        same as f64 / len as f64
    }

    fn bundle(vectors: &[&[u8]]) -> Vec<u8> {
        if vectors.is_empty() {
            return vec![0u8; vectors.first().map(|v| v.len()).unwrap_or(64)];
        }
        let dim = vectors[0].len();
        let mut result = vec![0i32; dim];
        for v in vectors {
            for (i, &b) in v.iter().enumerate() {
                if b != 0 {
                    result[i] += 1;
                } else {
                    result[i] -= 1;
                }
            }
        }
        result
            .into_iter()
            .map(|s| if s >= 0 { 255u8 } else { 0u8 })
            .collect()
    }

    fn rolling_strategy(&self) -> Option<Vec<u8>> {
        if self.strategy_buffer.is_empty() {
            return None;
        }
        let refs: Vec<&[u8]> = self.strategy_buffer.iter().map(|v| v.as_slice()).collect();
        Some(Self::bundle(&refs))
    }

    pub fn update(&mut self, trace: &[u8], outcome: &[u8]) {
        self.step += 1;

        self.strategy_buffer.push_back(trace.to_vec());
        while self.strategy_buffer.len() > self.window_size {
            self.strategy_buffer.pop_front();
        }

        let current_strategy = self.rolling_strategy();
        let before_strategy = self.prev_strategy.clone();

        if let (Some(ref before), Some(ref current)) = (&before_strategy, &current_strategy) {
            self.strategy_stability = Self::hamming_sim(before, current);
        }

        let outcome_sim = Self::hamming_sim(trace, outcome);
        let outcome_decay = 0.3;
        self.strategy_surprise =
            self.strategy_surprise * (1.0 - outcome_decay) + (1.0 - outcome_sim) * outcome_decay;

        self.last_bonus =
            self.ss_weight * self.strategy_stability + self.sus_weight * self.strategy_surprise;

        self.prev_strategy = current_strategy;
    }

    pub fn compute_bonus(&self) -> f64 {
        self.last_bonus.clamp(0.0, 1.0)
    }

    pub fn adapt_weights(&mut self, task_success: f64) {
        let delta = (task_success - 0.5) * 0.1;
        let total_before = self.ss_weight + self.sus_weight;
        if total_before > 0.0 {
            let new_ss = (self.ss_weight + delta).clamp(0.1, 0.9);
            let new_sus = (self.sus_weight - delta).clamp(0.1, 0.9);
            let sum = new_ss + new_sus;
            if sum > 0.0 {
                self.ss_weight = new_ss / sum;
                self.sus_weight = new_sus / sum;
            }
        }
    }

    pub fn stats(&self) -> SuSStats {
        SuSStats {
            step: self.step,
            strategy_stability: self.strategy_stability,
            strategy_surprise: self.strategy_surprise,
            sus_bonus: self.last_bonus,
            ss_weight: self.ss_weight,
            sus_weight: self.sus_weight,
            strategy_buffer_size: self.strategy_buffer.len(),
        }
    }
}

impl Default for StrategyAwareSurprise {
    fn default() -> Self {
        Self::new(10, 64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand_vec(seed: u8, len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| i.wrapping_mul(seed as usize) as u8)
            .collect()
    }

    #[test]
    fn test_new_sus() {
        let sus = StrategyAwareSurprise::new(10, 64);
        assert_eq!(sus.step, 0);
        assert!((sus.compute_bonus() - 0.0).abs() < 0.01);
        assert!(sus.prev_strategy.is_none());
    }

    #[test]
    fn test_update_trace() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        let t = rand_vec(1, 64);
        let o = rand_vec(2, 64);
        sus.update(&t, &o);
        assert_eq!(sus.step, 1);
        assert_eq!(sus.strategy_buffer.len(), 1);
        assert!(sus.prev_strategy.is_some());
    }

    #[test]
    fn test_strategy_stability_similar() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        let t = rand_vec(1, 64);
        for _ in 0..5 {
            sus.update(&t, &t);
        }
        assert!(sus.strategy_stability > 0.9);
    }

    #[test]
    fn test_strategy_surprise_on_shift() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        let t1 = rand_vec(1, 64);
        let t2 = rand_vec(255, 64);
        let o = rand_vec(3, 64);
        for _ in 0..5 {
            sus.update(&t1, &t1);
        }
        let stable_surprise = sus.strategy_surprise;
        sus.update(&t2, &o);
        assert!(sus.strategy_surprise >= stable_surprise);
    }

    #[test]
    fn test_adapt_weights() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        let w_before = sus.ss_weight;
        sus.adapt_weights(0.8);
        assert!((sus.ss_weight - w_before).abs() > 0.0);
    }

    #[test]
    fn test_bonus_bounds() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        for i in 0..10 {
            let t = rand_vec(i, 64);
            let o = rand_vec(i.wrapping_add(1), 64);
            sus.update(&t, &o);
            let bonus = sus.compute_bonus();
            assert!(
                bonus >= 0.0 && bonus <= 1.0,
                "bonus={} out of bounds",
                bonus
            );
        }
    }

    #[test]
    fn test_stats() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        for i in 0..3 {
            sus.update(&rand_vec(i, 64), &rand_vec(i.wrapping_add(1), 64));
        }
        let s = sus.stats();
        assert_eq!(s.step, 3);
        assert!(s.ss_weight > 0.0);
        assert!(s.sus_weight > 0.0);
        assert!(s.strategy_buffer_size > 0);
    }

    #[test]
    fn test_empty_stats() {
        let sus = StrategyAwareSurprise::new(5, 64);
        let s = sus.stats();
        assert_eq!(s.step, 0);
        assert!(s.strategy_buffer_size == 0);
    }

    #[test]
    fn test_consecutive_same_trace() {
        let mut sus = StrategyAwareSurprise::new(5, 64);
        let t = rand_vec(42, 64);
        for _ in 0..10 {
            sus.update(&t, &t);
        }
        let s = sus.stats();
        assert!(s.strategy_stability > 0.95);
        assert!(s.strategy_surprise < 0.2);
    }
}
