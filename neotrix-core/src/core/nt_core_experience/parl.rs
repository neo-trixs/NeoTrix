#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::MutationOp;

/// Configuration for the PARL (Parallel Reinforcement Learning) evaluator.
///
/// Controls batch size, per-evaluation timeout, and result history retention.
#[derive(Debug, Clone)]
pub struct ParlConfig {
    /// Number of mutation candidates to evaluate in parallel per batch.
    pub batch_size: usize,
    /// Maximum time (milliseconds) to wait for a single evaluation before timing out.
    pub timeout_ms: u64,
    /// Maximum number of historical results to retain for statistics.
    pub max_history: usize,
}

impl Default for ParlConfig {
    fn default() -> Self {
        Self {
            batch_size: 8,
            timeout_ms: 30_000,
            max_history: 1000,
        }
    }
}

/// Outcome of evaluating a single mutation candidate.
#[derive(Debug, Clone)]
pub struct ParlOutcome {
    /// Quality score in [0, 1].
    pub score: f64,
    /// Whether the mutation compiled successfully.
    pub compiles: bool,
    /// Human-readable description or diagnostic message.
    pub description: String,
    /// Wall-clock time spent evaluating, in milliseconds.
    pub latency_ms: u64,
}

/// A complete evaluation result for a single mutation candidate.
#[derive(Debug, Clone)]
pub struct ParlResult {
    /// Sequential ID for this proposal (index within the batch).
    pub proposal_id: u64,
    /// The mutation operation that was evaluated.
    pub op: MutationOp,
    /// Outcome of the parallel evaluation.
    pub outcome: ParlOutcome,
    /// Timestamp captured when the evaluation task was spawned.
    pub timestamp: Instant,
}

/// Aggregate statistics across all evaluations retained in the history buffer.
#[derive(Debug, Clone, Default)]
pub struct ParlStats {
    /// Total number of evaluation results retained.
    pub total_evaluated: u64,
    /// Average score across all retained results.
    pub avg_score: f64,
    /// Highest score observed across all retained results.
    pub best_score: f64,
    /// Average latency per evaluation, in milliseconds.
    pub avg_latency_ms: f64,
    /// Fraction of evaluations that compiled successfully (0.0 – 1.0).
    pub compile_rate: f64,
}

/// Parallel Reinforcement Learning evaluator for SEAL mutations.
///
/// Instead of evaluating mutation candidates one-at-a-time, `ParlEvaluator`
/// dispatches a batch of candidates as concurrent `tokio::task::spawn_blocking`
/// tasks, collects their outcomes, and returns results ranked by score descending.
///
/// A per-task timeout prevents any single evaluation from hanging indefinitely.
#[derive(Debug)]
pub struct ParlEvaluator {
    pub config: ParlConfig,
    results: VecDeque<ParlResult>,
}

impl ParlEvaluator {
    /// Create a new evaluator with the given configuration.
    ///
    /// The results buffer is pre-allocated to `min(max_history, 1000)` slots.
    pub fn new(config: ParlConfig) -> Self {
        let cap = config.max_history.min(1000);
        Self {
            config,
            results: VecDeque::with_capacity(cap),
        }
    }

    /// Evaluate a batch of mutation candidates in parallel.
    ///
    /// Each candidate is evaluated by calling `eval_fn` inside a
    /// `tokio::task::spawn_blocking` task (the scoring function is assumed to
    /// be CPU-bound).  A `tokio::time::timeout` guard prevents any single
    /// evaluation from running longer than `config.timeout_ms`.
    ///
    /// Results are returned sorted by `score` descending.  They are also
    /// appended to the internal history buffer (bounded by `max_history`).
    pub async fn evaluate_batch(
        &mut self,
        proposals: Vec<MutationOp>,
        eval_fn: impl Fn(MutationOp) -> ParlOutcome + Send + Sync + 'static,
    ) -> Vec<ParlResult> {
        if proposals.is_empty() {
            return vec![];
        }

        let batch_size = proposals.len();
        let timeout_ms = self.config.timeout_ms;
        let eval_fn = std::sync::Arc::new(eval_fn);
        let mut handles = Vec::with_capacity(batch_size);

        for (i, op) in proposals.into_iter().enumerate() {
            let eval_fn = std::sync::Arc::clone(&eval_fn);

            let idx = i;
            handles.push(tokio::spawn(async move {
                let proposal_id = idx as u64;
                let start = Instant::now();
                let timeout = Duration::from_millis(timeout_ms);

                // Clone op so the original is available for the error path.
                let op_for_eval = op.clone();

                let result = tokio::time::timeout(
                    timeout,
                    tokio::task::spawn_blocking(move || {
                        let outcome = eval_fn(op_for_eval);
                        outcome
                    }),
                )
                .await;

                match result {
                    Ok(Ok(mut outcome)) => {
                        outcome.latency_ms = 1.max(start.elapsed().as_millis() as u64);
                        ParlResult {
                            proposal_id,
                            op,
                            outcome,
                            timestamp: start,
                        }
                    }
                    _ => ParlResult {
                        proposal_id,
                        op,
                        outcome: ParlOutcome {
                            score: 0.0,
                            compiles: false,
                            description: "evaluation timeout or task error".into(),
                            latency_ms: timeout_ms,
                        },
                        timestamp: start,
                    },
                }
            }));
        }

        let mut results: Vec<ParlResult> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        // Sort by score descending.
        results.sort_by(|a, b| {
            b.outcome
                .score
                .partial_cmp(&a.outcome.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Append to bounded history.
        for r in &results {
            self.results.push_back(r.clone());
        }
        while self.results.len() > self.config.max_history {
            self.results.pop_front();
        }

        results
    }

    /// Return the single highest-scoring result from the history buffer, if any.
    pub fn best_result(&self) -> Option<&ParlResult> {
        self.results.iter().max_by(|a, b| {
            a.outcome
                .score
                .partial_cmp(&b.outcome.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Aggregate statistics across all results currently in the history buffer.
    pub fn stats(&self) -> ParlStats {
        let total = self.results.len() as u64;
        if total == 0 {
            return ParlStats::default();
        }

        let mut sum_score = 0.0f64;
        let mut sum_latency = 0u64;
        let mut best_score = 0.0f64;
        let mut compile_count = 0u64;

        for r in &self.results {
            let s = r.outcome.score;
            sum_score += s;
            sum_latency += r.outcome.latency_ms;
            if s > best_score {
                best_score = s;
            }
            if r.outcome.compiles {
                compile_count += 1;
            }
        }

        ParlStats {
            total_evaluated: total,
            avg_score: sum_score / total as f64,
            best_score,
            avg_latency_ms: sum_latency as f64 / total as f64,
            compile_rate: compile_count as f64 / total as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_eval(op: MutationOp) -> ParlOutcome {
        let score = match &op {
            MutationOp::TuneParam { delta, .. } => delta.abs().clamp(0.0, 1.0),
            MutationOp::RewriteHandler { .. } => 0.6,
            MutationOp::AddHandler { .. } => 0.4,
            MutationOp::SwapPolicy { .. } => 0.3,
            MutationOp::RewritePrimitive { .. } => 0.7,
            MutationOp::RewriteMeta { .. } => 0.5,
            MutationOp::SelfModifyProposal { .. } => 0.5,
        };
        ParlOutcome {
            score,
            compiles: score > 0.2,
            description: format!("score={:.4}", score),
            latency_ms: 1,
        }
    }

    #[tokio::test]
    async fn test_evaluate_batch_returns_results() {
        let proposals = vec![
            MutationOp::TuneParam {
                target: "a".into(),
                delta: 0.3,
            },
            MutationOp::TuneParam {
                target: "b".into(),
                delta: 0.7,
            },
            MutationOp::RewriteHandler {
                name: "c".into(),
                code: "ok".into(),
            },
            MutationOp::SwapPolicy {
                gates: vec!["x".into(), "y".into()],
            },
        ];

        let mut evaluator = ParlEvaluator::new(ParlConfig::default());
        let results = evaluator.evaluate_batch(proposals, dummy_eval).await;

        assert_eq!(results.len(), 4);
        assert!(results[0].outcome.score >= results[1].outcome.score);
        assert!(results[1].outcome.score >= results[2].outcome.score);
        assert!(results[2].outcome.score >= results[3].outcome.score);
    }

    #[tokio::test]
    async fn test_best_result_is_highest_score() {
        let proposals = vec![
            MutationOp::TuneParam {
                target: "x".into(),
                delta: 0.1,
            },
            MutationOp::TuneParam {
                target: "y".into(),
                delta: 0.9,
            },
            MutationOp::TuneParam {
                target: "z".into(),
                delta: 0.5,
            },
        ];

        let mut evaluator = ParlEvaluator::new(ParlConfig::default());
        let results = evaluator.evaluate_batch(proposals, dummy_eval).await;

        assert_eq!(results.len(), 3);
        let best = evaluator.best_result();
        assert!(best.is_some());
        assert!(best.unwrap().outcome.score >= 0.8);
        assert!(best.unwrap().outcome.compiles);
    }

    #[tokio::test]
    async fn test_compile_rate_tracking() {
        let proposals = vec![
            MutationOp::TuneParam {
                target: "a".into(),
                delta: 0.1,
            },
            MutationOp::TuneParam {
                target: "b".into(),
                delta: 0.0,
            },
            MutationOp::RewriteHandler {
                name: "c".into(),
                code: "ok".into(),
            },
        ];

        let mut evaluator = ParlEvaluator::new(ParlConfig::default());
        let _ = evaluator.evaluate_batch(proposals, dummy_eval).await;
        let stats = evaluator.stats();

        assert_eq!(stats.total_evaluated, 3);
        assert!(stats.compile_rate > 0.0);
        assert!(stats.avg_score > 0.0);
        assert!(stats.best_score > 0.0);
    }

    #[tokio::test]
    async fn test_empty_batch_returns_empty() {
        let mut evaluator = ParlEvaluator::new(ParlConfig::default());
        let results: Vec<ParlResult> = evaluator.evaluate_batch(vec![], dummy_eval).await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_config_defaults() {
        let config = ParlConfig::default();
        assert_eq!(config.batch_size, 8);
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.max_history, 1000);
    }
}
