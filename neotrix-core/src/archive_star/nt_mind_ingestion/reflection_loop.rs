use super::ReflectionRound;

pub struct ReflectionLoop {
    pub max_rounds: usize,
    pub convergence_threshold: f64,
    pub rounds: Vec<ReflectionRound>,
    pub converged: bool,
}

impl ReflectionLoop {
    pub fn new(max_rounds: usize, convergence_threshold: f64) -> Self {
        Self {
            max_rounds,
            convergence_threshold,
            rounds: Vec::with_capacity(max_rounds),
            converged: false,
        }
    }

    pub fn current_round(&self) -> usize {
        self.rounds.len() + 1
    }

    pub fn should_continue(&self) -> bool {
        !self.converged && self.rounds.len() < self.max_rounds
    }

    pub fn record_round(&mut self, insights: Vec<String>, clarity_delta: f64) {
        let round_num = self.rounds.len() + 1;
        let converged = clarity_delta.abs() < self.convergence_threshold
            || round_num >= self.max_rounds;
        self.rounds.push(ReflectionRound {
            round: round_num,
            insights,
            clarity_delta,
            converged,
        });
        if converged {
            self.converged = true;
        }
    }

    pub fn total_rounds(&self) -> usize {
        self.rounds.len()
    }

    pub fn all_insights(&self) -> Vec<String> {
        self.rounds.iter()
            .flat_map(|r| r.insights.clone())
            .collect()
    }

    pub fn latest_clarity(&self) -> f64 {
        self.rounds.last().map(|r| r.clarity_delta).unwrap_or(1.0)
    }

    pub fn best_quality(&self) -> f64 {
        let count = self.rounds.len();
        if count == 0 {
            return 0.0;
        }
        let total: f64 = self.rounds.iter().enumerate().map(|(i, r)| {
            let weight = (i as f64 + 1.0) / count as f64;
            (1.0 - r.clarity_delta.abs()) * weight
        }).sum();
        (total / count as f64).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct QualityMonitor {
    pub threshold: f64,
    pub scores: Vec<f64>,
}

impl QualityMonitor {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            scores: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, clarity: f64, entity_count: usize, relation_count: usize) -> f64 {
        let clarity_score = (1.0 - clarity.abs()).max(0.0).min(1.0);
        let density_score = if entity_count > 0 {
            let ratio = relation_count as f64 / entity_count as f64;
            (ratio / 3.0).min(1.0)
        } else {
            0.0
        };
        let score = 0.6 * clarity_score + 0.4 * density_score;
        self.scores.push(score);
        score
    }

    pub fn is_acceptable(&self) -> bool {
        self.scores.last().copied().unwrap_or(0.0) >= self.threshold
    }

    pub fn average_score(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        self.scores.iter().sum::<f64>() / self.scores.len() as f64
    }
}
