#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum FactorExpr {
    Constant(f64),
    Feature(&'static str),
    Add(Box<FactorExpr>, Box<FactorExpr>),
    Mul(Box<FactorExpr>, Box<FactorExpr>),
    Sub(Box<FactorExpr>, Box<FactorExpr>),
    Div {
        num: Box<FactorExpr>,
        den: Box<FactorExpr>,
    },
}

impl FactorExpr {
    pub fn evaluate(&self, features: &HashMap<&str, f64>) -> f64 {
        match self {
            FactorExpr::Constant(v) => *v,
            FactorExpr::Feature(name) => *features.get(name).unwrap_or(&0.0),
            FactorExpr::Add(a, b) => a.evaluate(features) + b.evaluate(features),
            FactorExpr::Mul(a, b) => a.evaluate(features) * b.evaluate(features),
            FactorExpr::Sub(a, b) => a.evaluate(features) - b.evaluate(features),
            FactorExpr::Div { num, den } => {
                let d = den.evaluate(features);
                if d.abs() < 1e-10 {
                    0.0
                } else {
                    num.evaluate(features) / d
                }
            }
        }
    }

    pub fn complexity(&self) -> usize {
        match self {
            FactorExpr::Constant(_) | FactorExpr::Feature(_) => 1,
            FactorExpr::Add(a, b) | FactorExpr::Mul(a, b) | FactorExpr::Sub(a, b) => {
                1 + a.complexity() + b.complexity()
            }
            FactorExpr::Div { num, den } => 1 + num.complexity() + den.complexity(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FactorCandidate {
    pub id: u64,
    pub expression: FactorExpr,
    pub sharpe_ratio: f64,
    pub ic: f64,
}

impl FactorCandidate {
    pub fn new(id: u64, expression: FactorExpr) -> Self {
        Self {
            id,
            expression,
            sharpe_ratio: 0.0,
            ic: 0.0,
        }
    }
}

pub struct FactorMiner {
    next_id: u64,
    features: Vec<&'static str>,
}

impl FactorMiner {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            features: vec!["price", "volume", "ma5", "ma20", "rsi", "volatility"],
        }
    }

    fn random_expr(&self, rng: &mut u64) -> FactorExpr {
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let choice = (*rng % 6) as usize;
        match choice {
            0 => FactorExpr::Constant(((*rng % 100) as f64) / 10.0),
            1 => {
                let idx = (*rng % self.features.len() as u64) as usize;
                FactorExpr::Feature(self.features[idx])
            }
            _ => {
                let a = Box::new(self.random_expr(rng));
                let b = Box::new(self.random_expr(rng));
                match choice {
                    2 => FactorExpr::Add(a, b),
                    3 => FactorExpr::Mul(a, b),
                    4 => FactorExpr::Sub(a, b),
                    _ => FactorExpr::Div { num: a, den: b },
                }
            }
        }
    }

    pub fn generate_candidates(&mut self, population: usize) -> Vec<FactorCandidate> {
        let mut rng: u64 = 42;
        let mut candidates = Vec::with_capacity(population);
        for _ in 0..population {
            let expr = self.random_expr(&mut rng);
            let id = self.next_id;
            self.next_id += 1;
            candidates.push(FactorCandidate::new(id, expr));
        }
        candidates
    }

    pub fn backtest(&self, _candidate: &FactorCandidate, data: &[(f64, f64)]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0);
        }
        let mut returns_sum = 0.0;
        let mut returns_sq_sum = 0.0;
        let mut ic_numerator = 0.0;
        let mut ic_denom_a = 0.0;
        let mut ic_denom_b = 0.0;
        let mut n = 0;

        for &(actual_return, predicted) in data {
            returns_sum += actual_return;
            returns_sq_sum += actual_return * actual_return;
            ic_numerator += (actual_return - 0.0) * (predicted - 0.0);
            ic_denom_a += (actual_return - 0.0) * (actual_return - 0.0);
            ic_denom_b += (predicted - 0.0) * (predicted - 0.0);
            n += 1;
        }

        let nf = n as f64;
        let mean_return = returns_sum / nf;
        let variance = (returns_sq_sum / nf) - (mean_return * mean_return);
        let sharpe = if variance > 0.0 {
            mean_return / variance.sqrt()
        } else {
            0.0
        };

        let denom = (ic_denom_a * ic_denom_b).sqrt();
        let ic = if denom > 0.0 {
            ic_numerator / denom
        } else {
            0.0
        };

        (ic, sharpe)
    }

    pub fn rank_and_select(
        &self,
        mut candidates: Vec<FactorCandidate>,
        top_k: usize,
    ) -> Vec<FactorCandidate> {
        candidates.sort_by(|a, b| {
            b.sharpe_ratio
                .partial_cmp(&a.sharpe_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.into_iter().take(top_k).collect()
    }

    pub fn evaluate_with_features(
        &self,
        candidate: &FactorCandidate,
        feature_rows: &[HashMap<&str, f64>],
        returns: &[f64],
    ) -> (f64, f64) {
        let predictions: Vec<f64> = feature_rows
            .iter()
            .map(|row| candidate.expression.evaluate(row))
            .collect();
        let paired: Vec<(f64, f64)> = predictions
            .into_iter()
            .zip(returns.iter().copied())
            .map(|(p, r)| (r, p))
            .collect();
        self.backtest(candidate, &paired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_candidates_creates_n() {
        let mut miner = FactorMiner::new();
        let candidates = miner.generate_candidates(10);
        assert_eq!(candidates.len(), 10);
    }

    #[test]
    fn test_backtest_returns_ic_and_sharpe() {
        let miner = FactorMiner::new();
        let expr = FactorExpr::Constant(1.0);
        let cand = FactorCandidate::new(1, expr);
        let data = vec![(0.01, 0.02), (0.02, 0.03), (-0.01, -0.02)];
        let (ic, sharpe) = miner.backtest(&cand, &data);
        assert!(ic >= -1.0 && ic <= 1.0);
        assert!(sharpe.is_finite());
    }

    #[test]
    fn test_rank_and_select_returns_top_k() {
        let mut miner = FactorMiner::new();
        let mut candidates = miner.generate_candidates(20);
        for (i, c) in candidates.iter_mut().enumerate() {
            c.sharpe_ratio = i as f64;
        }
        let top = miner.rank_and_select(candidates, 5);
        assert_eq!(top.len(), 5);
        assert!(top[0].sharpe_ratio >= top[1].sharpe_ratio);
    }

    #[test]
    fn test_backtest_empty_data() {
        let miner = FactorMiner::new();
        let expr = FactorExpr::Constant(1.0);
        let cand = FactorCandidate::new(1, expr);
        let (ic, sharpe) = miner.backtest(&cand, &[]);
        assert_eq!(ic, 0.0);
        assert_eq!(sharpe, 0.0);
    }

    #[test]
    fn test_expression_evaluate_constant() {
        let expr = FactorExpr::Constant(3.14);
        let features = HashMap::new();
        assert!((expr.evaluate(&features) - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_expression_evaluate_feature() {
        let expr = FactorExpr::Feature("price");
        let mut features = HashMap::new();
        features.insert("price", 100.0);
        assert!((expr.evaluate(&features) - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_expression_complexity() {
        let expr = FactorExpr::Add(
            Box::new(FactorExpr::Constant(1.0)),
            Box::new(FactorExpr::Feature("price")),
        );
        assert_eq!(expr.complexity(), 3);
    }

    #[test]
    fn test_division_by_near_zero() {
        let expr = FactorExpr::Div {
            num: Box::new(FactorExpr::Constant(5.0)),
            den: Box::new(FactorExpr::Constant(0.0)),
        };
        let features = HashMap::new();
        assert_eq!(expr.evaluate(&features), 0.0);
    }

    #[test]
    fn test_evaluate_with_features() {
        let miner = FactorMiner::new();
        let expr = FactorExpr::Feature("price");
        let cand = FactorCandidate::new(1, expr);
        let features = vec![
            HashMap::from([("price", 100.0)]),
            HashMap::from([("price", 102.0)]),
        ];
        let returns = vec![0.01, 0.02];
        let (_ic, sharpe) = miner.evaluate_with_features(&cand, &features, &returns);
        assert!(sharpe.is_finite());
    }

    #[test]
    fn test_candidate_id_increments() {
        let mut miner = FactorMiner::new();
        let c1 = miner.generate_candidates(1);
        let c2 = miner.generate_candidates(1);
        assert_eq!(c2[0].id, c1[0].id + 1);
    }
}
