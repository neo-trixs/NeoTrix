use super::chunk::{sample_categorical, Chunk, ScoreWeights};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionMode {
    Softmax,
    Nash,
}

#[derive(Debug, Clone)]
pub struct NashConfig {
    pub max_iterations: usize,
    pub tolerance: f64,
    pub temperature: f64,
}

impl Default for NashConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            tolerance: 1e-4,
            temperature: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NashStats {
    pub iterations_used: usize,
    pub total_competitions: u64,
    pub nash_equilibrium_found: bool,
}

impl Default for NashStats {
    fn default() -> Self {
        Self {
            iterations_used: 0,
            total_competitions: 0,
            nash_equilibrium_found: false,
        }
    }
}

pub struct NashCompetitionEngine {
    pub config: NashConfig,
    pub stats: NashStats,
    pub total_nash_competitions: u64,
}

impl NashCompetitionEngine {
    pub fn new() -> Self {
        Self {
            config: NashConfig::default(),
            stats: NashStats::default(),
            total_nash_competitions: 0,
        }
    }

    pub fn nash_competition<'a>(
        &mut self,
        chunks: &'a [Chunk],
        weights: &ScoreWeights,
    ) -> Option<&'a Chunk> {
        self.total_nash_competitions += 1;

        if chunks.is_empty() {
            return None;
        }
        if chunks.len() == 1 {
            self.stats = NashStats {
                iterations_used: 0,
                total_competitions: self.total_nash_competitions,
                nash_equilibrium_found: true,
            };
            return Some(&chunks[0]);
        }

        let n = chunks.len();
        let raw_scores: Vec<f64> = chunks.iter().map(|c| c.weight(weights)).collect();
        let _max_raw = raw_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let mut pi = vec![1.0 / n as f64; n];

        for t in 0..self.config.max_iterations {
            let u_i: Vec<f64> = (0..n)
                .map(|i| {
                    let cs = logit_contest(i, &raw_scores, &pi, self.config.temperature);
                    chunks[i].weight(weights) * cs
                })
                .collect();

            let _v_i: Vec<f64> = (0..n)
                .map(|i| {
                    let cs = logit_contest(i, &raw_scores, &pi, self.config.temperature);
                    chunks[i].weight(weights) * cs
                })
                .collect();

            let max_u = u_i.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let shifted_u: Vec<f64> = u_i
                .iter()
                .map(|u| (u - max_u) / self.config.temperature.max(1e-8))
                .collect();
            let exp_u: Vec<f64> = shifted_u.iter().map(|u| u.exp()).collect();
            let sum_exp_u: f64 = exp_u.iter().sum();

            let pi_new: Vec<f64> = if sum_exp_u > 0.0 && sum_exp_u.is_finite() {
                exp_u.iter().map(|e| e / sum_exp_u).collect()
            } else {
                pi.clone()
            };

            let diff: f64 = pi
                .iter()
                .zip(pi_new.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            pi = pi_new;

            if diff < self.config.tolerance {
                self.stats = NashStats {
                    iterations_used: t + 1,
                    total_competitions: self.total_nash_competitions,
                    nash_equilibrium_found: true,
                };
                return Some(sample_categorical(chunks, &pi));
            }
        }

        self.stats = NashStats {
            iterations_used: self.config.max_iterations,
            total_competitions: self.total_nash_competitions,
            nash_equilibrium_found: false,
        };
        Some(sample_categorical(chunks, &pi))
    }
}

fn logit_contest(agent: usize, scores: &[f64], pi: &[f64], temperature: f64) -> f64 {
    let numerator = (scores[agent] / temperature.max(1e-8)).exp() * pi[agent];
    let denominator: f64 = scores
        .iter()
        .zip(pi.iter())
        .map(|(s, p)| (s / temperature.max(1e-8)).exp() * p)
        .sum();
    if denominator > 0.0 && denominator.is_finite() {
        numerator / denominator
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nash_empty() {
        let mut eng = NashCompetitionEngine::new();
        let winner = eng.nash_competition(&[] as &[Chunk], &ScoreWeights::default());
        assert!(winner.is_none());
    }

    fn make_c(name: &str, rel: f64, conf: f64) -> Chunk {
        let mut c = Chunk::new(name, 0, vec![0u8; 16]);
        c.relevance = rel;
        c.confidence = conf;
        c
    }

    #[test]
    fn test_nash_single() {
        let mut eng = NashCompetitionEngine::new();
        let chunks = vec![make_c("only", 0.0, 0.0)];
        let winner = eng.nash_competition(&chunks, &ScoreWeights::default());
        assert_eq!(winner.unwrap().processor_name, "only");
    }

    #[test]
    fn test_nash_selects_higher_weight() {
        let chunks = vec![make_c("low", 0.1, 0.1), make_c("high", 0.9, 0.9)];
        let count_high = (0..30)
            .filter(|_| {
                let mut eng = NashCompetitionEngine::new();
                eng.nash_competition(&chunks, &ScoreWeights::default())
                    .unwrap()
                    .processor_name
                    == "high"
            })
            .count();
        assert!(
            count_high >= 18,
            "high should win ~70%+: got {}/30",
            count_high
        );
    }

    #[test]
    fn test_nash_convergence() {
        let mut eng = NashCompetitionEngine::new();
        let mut c1 = Chunk::new("a", 0, vec![0u8; 16]);
        c1.relevance = 0.5;
        c1.confidence = 0.5;
        let mut c2 = Chunk::new("b", 1, vec![1u8; 16]);
        c2.relevance = 0.5;
        c2.confidence = 0.5;
        let chunks = vec![c1, c2];
        eng.nash_competition(&chunks, &ScoreWeights::default());
        assert_eq!(eng.stats.nash_equilibrium_found, true);
    }

    #[test]
    fn test_nash_stats_tracked() {
        let mut eng = NashCompetitionEngine::new();
        let c = Chunk::new("x", 0, vec![0u8; 16]);
        eng.nash_competition(&[c], &ScoreWeights::default());
        assert_eq!(eng.total_nash_competitions, 1);
    }

    #[test]
    fn test_logit_contest_symmetric() {
        let scores = vec![1.0, 1.0];
        let pi = vec![0.5, 0.5];
        let cs0 = logit_contest(0, &scores, &pi, 1.0);
        let cs1 = logit_contest(1, &scores, &pi, 1.0);
        assert!((cs0 - cs1).abs() < 1e-6);
    }

    #[test]
    fn test_logit_contest_asymmetric() {
        let scores = vec![2.0, 1.0];
        let pi = vec![0.5, 0.5];
        let cs0 = logit_contest(0, &scores, &pi, 1.0);
        let cs1 = logit_contest(1, &scores, &pi, 1.0);
        assert!(cs0 > cs1, "higher score should have higher contest share");
    }
}
