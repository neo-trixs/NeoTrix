use super::types::{FusionDeliberator, TrtRollout, TrtStrategy};

// ─── TRT: Test-time Recursive Thinking ───

impl FusionDeliberator {
    pub fn trt_think(&mut self, query: &[u8], n_rollouts: usize) -> Vec<TrtRollout> {
        let mut rollouts = Vec::with_capacity(n_rollouts);
        for i in 0..n_rollouts {
            if i > 0 {
                self.adapt_trt_strategy(&rollouts);
            }
            let strategy = self.trt_strategy.clone();
            let thoughts = self.generate_trt_perspectives(query, &strategy);
            let verify_score = if strategy.self_verify {
                self.trt_self_verify(&thoughts)
            } else {
                0.5
            };
            let knowledge = self.accumulate_trt_knowledge(&thoughts, &rollouts);
            let accepted = verify_score >= 0.3;
            rollouts.push(TrtRollout {
                idx: i,
                query: query.to_vec(),
                strategy,
                thoughts,
                self_verify_score: verify_score,
                accumulated_knowledge: knowledge,
                accepted,
            });
        }
        self.trt_history.extend(rollouts.clone());
        if self.trt_history.len() > 50 {
            self.trt_history.drain(0..self.trt_history.len() - 50);
        }
        rollouts
    }

    fn generate_trt_perspectives(&self, query: &[u8], strategy: &TrtStrategy) -> Vec<Vec<u8>> {
        let mut thoughts = Vec::with_capacity(strategy.n_perspectives);
        for p in 0..strategy.n_perspectives {
            let perspective = self.vsa_trt_perspective(query, p, strategy.explore_rate);
            thoughts.push(perspective);
        }
        thoughts
    }

    fn vsa_trt_perspective(&self, query: &[u8], seed: usize, explore_rate: f64) -> Vec<u8> {
        let mut result = query.to_vec();
        for i in 0..result.len() {
            let bias = ((seed.wrapping_mul(31).wrapping_add(i)) % 256) as u8;
            let explore = (seed as f64 * 0.1).fract() < explore_rate;
            if explore {
                result[i] ^= bias;
            }
        }
        result
    }

    fn trt_self_verify(&self, thoughts: &[Vec<u8>]) -> f64 {
        if thoughts.is_empty() {
            return 0.0;
        }
        let n = thoughts.len();
        if n < 2 {
            return 0.5;
        }
        let mut total_sim = 0.0;
        let mut pairs = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = self.vsa_similarity(&thoughts[i], &thoughts[j]);
                total_sim += sim;
                pairs += 1;
            }
        }
        if pairs == 0 {
            return 0.5;
        }
        let avg_sim = total_sim / pairs as f64;
        if avg_sim < 0.3 {
            0.3
        } else if avg_sim > 0.7 {
            0.4
        } else {
            avg_sim + 0.3
        }
    }

    fn accumulate_trt_knowledge(&self, thoughts: &[Vec<u8>], previous: &[TrtRollout]) -> String {
        let mut insights = Vec::new();
        if self.vsa_average(thoughts).is_some() {
            insights.push(format!("perspectives: {}", thoughts.len()));
            if let Some(last) = previous.last() {
                insights.push(format!("prev_strategy: {}", last.accumulated_knowledge));
            }
        }
        insights.join(" | ")
    }

    fn adapt_trt_strategy(&mut self, rollouts: &[TrtRollout]) {
        if rollouts.is_empty() {
            return;
        }
        let recent: Vec<&TrtRollout> = rollouts.iter().rev().take(3).collect();
        let avg_score: f64 =
            recent.iter().map(|r| r.self_verify_score).sum::<f64>() / recent.len() as f64;
        if avg_score < 0.4 {
            self.trt_strategy.explore_rate = (self.trt_strategy.explore_rate + 0.1).min(1.0);
            self.trt_strategy.temperature = (self.trt_strategy.temperature + 0.2).min(2.0);
            self.trt_strategy.label = "exploring".into();
        } else if avg_score > 0.7 {
            self.trt_strategy.explore_rate = (self.trt_strategy.explore_rate - 0.05).max(0.0);
            self.trt_strategy.temperature = (self.trt_strategy.temperature - 0.1).max(0.5);
            self.trt_strategy.label = "exploiting".into();
        } else {
            self.trt_strategy.label = "balanced".into();
        }
    }

    fn vsa_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let same = a[..min_len]
            .iter()
            .zip(b[..min_len].iter())
            .filter(|(x, y)| x == y)
            .count();
        same as f64 / min_len as f64
    }

    fn vsa_average(&self, vectors: &[Vec<u8>]) -> Option<Vec<u8>> {
        if vectors.is_empty() {
            return None;
        }
        let len = vectors[0].len();
        let mut avg = vec![0u8; len];
        for v in vectors {
            for (i, b) in v.iter().enumerate() {
                if i < len {
                    avg[i] = avg[i].wrapping_add(*b);
                }
            }
        }
        let n = vectors.len() as u8;
        for b in &mut avg {
            *b /= n;
        }
        Some(avg)
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::create_test_deliberator;

    #[test]
    fn test_trt_strategy_default() {
        let s = TrtStrategy::default();
        assert_eq!(s.n_perspectives, 4);
        assert!(s.explore_rate > 0.0);
        assert!(s.self_verify);
    }

    #[test]
    fn test_vsa_similarity_identical() {
        let delib = create_test_deliberator();
        let a = vec![1u8, 2, 3, 4];
        let sim = delib.vsa_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vsa_similarity_different() {
        let delib = create_test_deliberator();
        let a = vec![1u8, 2, 3, 4];
        let b = vec![5u8, 6, 7, 8];
        let sim = delib.vsa_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vsa_average() {
        let delib = create_test_deliberator();
        let vectors = vec![vec![10u8, 20, 30], vec![20u8, 30, 40]];
        let avg = delib.vsa_average(&vectors).unwrap();
        assert_eq!(avg, vec![15u8, 25, 35]);
    }

    #[test]
    fn test_trt_self_verify_empty() {
        let delib = create_test_deliberator();
        let score = delib.trt_self_verify(&[]);
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_trt_self_verify_single() {
        let delib = create_test_deliberator();
        let score = delib.trt_self_verify(&[vec![1, 2, 3]]);
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_trt_think_runs() {
        let mut delib = create_test_deliberator();
        let query = vec![0u8; 64];
        let rollouts = delib.trt_think(&query, 3);
        assert_eq!(rollouts.len(), 3);
        for r in &rollouts {
            assert_eq!(r.thoughts.len(), delib.trt_strategy.n_perspectives);
        }
    }

    #[test]
    fn test_adapt_strategy() {
        let mut delib = create_test_deliberator();
        let rollouts = vec![
            TrtRollout {
                idx: 0,
                query: vec![],
                strategy: TrtStrategy::default(),
                thoughts: vec![],
                self_verify_score: 0.2,
                accumulated_knowledge: "".into(),
                accepted: true,
            },
            TrtRollout {
                idx: 1,
                query: vec![],
                strategy: TrtStrategy::default(),
                thoughts: vec![],
                self_verify_score: 0.3,
                accumulated_knowledge: "".into(),
                accepted: true,
            },
        ];
        let before = delib.trt_strategy.explore_rate;
        delib.adapt_trt_strategy(&rollouts);
        assert!(delib.trt_strategy.explore_rate >= before);
    }

    #[test]
    fn test_generate_trt_perspectives() {
        let delib = create_test_deliberator();
        let query = vec![0u8; 64];
        let strategy = TrtStrategy {
            n_perspectives: 5,
            ..Default::default()
        };
        let thoughts = delib.generate_trt_perspectives(&query, &strategy);
        assert_eq!(thoughts.len(), 5);
        for i in 1..thoughts.len() {
            assert!(thoughts[i] != thoughts[0]);
        }
    }
}

#[cfg(test)]
pub(crate) fn create_test_deliberator() -> FusionDeliberator {
    let mut d = FusionDeliberator::default();
    d.trt_strategy = TrtStrategy::default();
    d.trt_history = Vec::new();
    d._trt_max_iters = 10;
    d
}
