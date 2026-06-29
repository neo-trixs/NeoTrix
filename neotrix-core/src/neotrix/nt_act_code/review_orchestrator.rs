use std::time::Instant;

use super::compatibility_agent::CompatibilityAgent;
use super::review_aggregator::{AgentReviewResult, AggregatedReviewReport, ReviewAggregator};
use super::test_agent::TestEdgeCaseAgent;
use super::ultra_review::{default_review_dimensions, ReviewDimension, UltraReviewEngine};

pub struct ReviewOrchestrator {
    max_concurrent: usize,
    dimensions: Vec<ReviewDimension>,
}

impl Default for ReviewOrchestrator {
    fn default() -> Self {
        Self::new(8)
    }
}

impl ReviewOrchestrator {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            dimensions: default_review_dimensions(),
        }
    }

    pub fn with_dimensions(mut self, dims: Vec<ReviewDimension>) -> Self {
        self.dimensions = dims;
        self
    }

    pub fn run_parallel_review(&self, code: &str, file: &str) -> AggregatedReviewReport {
        let start = Instant::now();

        let n_dim = self.dimensions.len();
        let extra_agents = 2;
        let total = n_dim + extra_agents;

        let chunk_size = (total + self.max_concurrent - 1) / self.max_concurrent;
        let n_batches = (total + chunk_size - 1) / chunk_size;

        let mut all_results: Vec<AgentReviewResult> = Vec::new();

        std::thread::scope(|s| {
            for batch in 0..n_batches {
                let batch_start = batch * chunk_size;

                let mut handles = Vec::new();

                for offset in 0..chunk_size {
                    let idx = batch_start + offset;
                    if idx >= total {
                        break;
                    }

                    if idx < n_dim {
                        let dim = self.dimensions[idx];
                        handles.push(s.spawn(move || run_dimension_agent(code, file, dim)));
                    } else if idx == n_dim {
                        handles.push(s.spawn(move || TestEdgeCaseAgent::review(code, file)));
                    } else if idx == n_dim + 1 {
                        handles.push(s.spawn(move || CompatibilityAgent::review(code, file)));
                    }
                }

                for h in handles {
                    if let Ok(result) = h.join() {
                        all_results.push(result);
                    }
                }
            }
        });

        let mut report = ReviewAggregator::aggregate(file, all_results);
        report.duration_ms = start.elapsed().as_millis() as u64;
        report
    }
}

fn run_dimension_agent(code: &str, _file: &str, dim: ReviewDimension) -> AgentReviewResult {
    let agent_start = Instant::now();
    let (issues, score) = UltraReviewEngine::run_dimension_review(code, dim);
    let elapsed = agent_start.elapsed().as_millis() as u64;

    let label = dim.label().to_string();
    let agent_name = format!("agent-{}", label.to_lowercase().replace(' ', "-"));

    AgentReviewResult {
        agent_name,
        dimension: label,
        issues,
        score,
        duration_ms: elapsed,
    }
}

pub fn quick_review_agents() -> Vec<&'static str> {
    vec!["security", "correctness", "test-edge", "compatibility"]
}
