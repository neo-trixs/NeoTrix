//! Result Aggregation
//!
//! Combines outputs from multiple agents using configurable strategies.
//! Supports conflict detection when agents disagree (R-P16 surface disagreement).

use serde::{Deserialize, Serialize};

use super::crew::CrewResult;

/// Strategy for combining multiple agent results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Majority voting — most common output wins
    Voting,
    /// Weighted average — each agent has a trust weight
    WeightedAverage,
    /// First successful result (short-circuit)
    FirstValid,
    /// Merge all successful outputs into a single document
    MergeAll,
}

impl std::fmt::Display for AggregationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregationStrategy::Voting => write!(f, "Voting"),
            AggregationStrategy::WeightedAverage => write!(f, "WeightedAverage"),
            AggregationStrategy::FirstValid => write!(f, "FirstValid"),
            AggregationStrategy::MergeAll => write!(f, "MergeAll"),
        }
    }
}

/// Detected conflict between agent results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub kind: ConflictKind,
    pub agents: Vec<String>,
    pub description: String,
    /// Severity: 0.0 (minor) to 1.0 (critical)
    pub severity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    Contradiction,
    FactualDisagreement,
    ApproachConflict,
    SuccessFailureSplit,
}

impl std::fmt::Display for ConflictKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictKind::Contradiction => write!(f, "Contradiction"),
            ConflictKind::FactualDisagreement => write!(f, "FactualDisagreement"),
            ConflictKind::ApproachConflict => write!(f, "ApproachConflict"),
            ConflictKind::SuccessFailureSplit => write!(f, "SuccessFailureSplit"),
        }
    }
}

/// Aggregated output from multiple agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub output: String,
    pub strategy: AggregationStrategy,
    pub contributors: usize,
    pub conflicts: Vec<Conflict>,
    /// Confidence score: 0.0 (uncertain) to 1.0 (confident)
    pub confidence: f64,
}

impl AggregatedResult {
    fn with_conflicts(mut self, conflicts: Vec<Conflict>) -> Self {
        self.conflicts = conflicts;
        self
    }
}

/// Per-agent weight for weighted aggregation
#[derive(Debug, Clone)]
pub struct AgentWeight {
    pub agent_handle: String,
    pub weight: f64,
}

/// Aggregate results from multiple agents using the given strategy.
pub fn aggregate(
    results: &[CrewResult],
    strategy: AggregationStrategy,
) -> AggregatedResult {
    let successful: Vec<&CrewResult> = results.iter().filter(|r| r.success).collect();

    if successful.is_empty() {
        return AggregatedResult {
            output: String::new(),
            strategy,
            contributors: 0,
            conflicts: vec![Conflict {
                kind: ConflictKind::SuccessFailureSplit,
                agents: results.iter().map(|r| r.agent_handle.clone()).collect(),
                description: "All agents failed".to_string(),
                severity: 1.0,
            }],
            confidence: 0.0,
        };
    }

    let conflicts = detect_conflicts(results);

    let base = match strategy {
        AggregationStrategy::Voting => aggregate_voting(&successful),
        AggregationStrategy::WeightedAverage => {
            let weights: Vec<AgentWeight> = successful
                .iter()
                .map(|r| AgentWeight {
                    agent_handle: r.agent_handle.clone(),
                    weight: 1.0,
                })
                .collect();
            aggregate_weighted(&successful, &weights)
        }
        AggregationStrategy::FirstValid => aggregate_first_valid(&successful),
        AggregationStrategy::MergeAll => aggregate_merge_all(&successful),
    };
    base.with_conflicts(conflicts)
}

fn aggregate_voting(results: &[&CrewResult]) -> AggregatedResult {
    if results.is_empty() {
        return AggregatedResult {
            output: String::new(),
            strategy: AggregationStrategy::Voting,
            contributors: 0,
            conflicts: Vec::new(),
            confidence: 0.0,
        };
    }
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for r in results {
        if let Some(entry) = counts.iter_mut().find(|(o, _)| *o == r.output.as_str()) {
            entry.1 += 1;
        } else {
            counts.push((&r.output, 1));
        }
    }
    let winner = counts.iter().max_by_key(|(_, c)| *c).unwrap();
    let total = results.len();
    AggregatedResult {
        output: winner.0.to_string(),
        strategy: AggregationStrategy::Voting,
        contributors: total,
        conflicts: Vec::new(),
        confidence: winner.1 as f64 / total as f64,
    }
}

fn aggregate_weighted(
    results: &[&CrewResult],
    weights: &[AgentWeight],
) -> AggregatedResult {
    if results.is_empty() {
        return AggregatedResult {
            output: String::new(),
            strategy: AggregationStrategy::WeightedAverage,
            contributors: 0,
            conflicts: Vec::new(),
            confidence: 0.0,
        };
    }
    let mut best: Option<(&CrewResult, f64)> = None;
    for r in results {
        let w = weights
            .iter()
            .find(|w| w.agent_handle == r.agent_handle)
            .map(|w| w.weight)
            .unwrap_or(1.0);
        if best.is_none_or(|(_, bw)| w > bw) {
            best = Some((r, w));
        }
    }
    let (winner, _) = best.unwrap();
    let total_weight: f64 = weights.iter().map(|w| w.weight).sum();
    AggregatedResult {
        output: winner.output.clone(),
        strategy: AggregationStrategy::WeightedAverage,
        contributors: results.len(),
        conflicts: Vec::new(),
        confidence: if total_weight > 0.0 { total_weight.min(1.0) } else { 0.0 },
    }
}

fn aggregate_first_valid(results: &[&CrewResult]) -> AggregatedResult {
    let first = results.first().unwrap();
    AggregatedResult {
        output: first.output.clone(),
        strategy: AggregationStrategy::FirstValid,
        contributors: 1,
        conflicts: Vec::new(),
        confidence: 1.0,
    }
}

fn aggregate_merge_all(results: &[&CrewResult]) -> AggregatedResult {
    let outputs: Vec<&str> = results.iter().map(|r| r.output.as_str()).collect();
    AggregatedResult {
        output: outputs.join("\n---\n"),
        strategy: AggregationStrategy::MergeAll,
        contributors: results.len(),
        conflicts: Vec::new(),
        confidence: 1.0,
    }
}

/// Detect conflicts between agent results (R-P16 surface disagreement)
pub fn detect_conflicts(results: &[CrewResult]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    let successful: Vec<&CrewResult> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<&CrewResult> = results.iter().filter(|r| !r.success).collect();

    if !successful.is_empty() && !failed.is_empty() {
        conflicts.push(Conflict {
            kind: ConflictKind::SuccessFailureSplit,
            agents: failed.iter().map(|r| r.agent_handle.clone()).collect(),
            description: format!(
                "{} agents succeeded, {} failed",
                successful.len(),
                failed.len()
            ),
            severity: 0.7,
        });
    }

    let mut task_groups: std::collections::HashMap<String, Vec<&CrewResult>> =
        std::collections::HashMap::new();
    for r in &successful {
        task_groups
            .entry(r.task_id.clone())
            .or_default()
            .push(r);
    }

    for (task_id, group) in &task_groups {
        if group.len() < 2 {
            continue;
        }
        let mut outputs: Vec<&str> = group.iter().map(|r| r.output.as_str()).collect();
        outputs.dedup();
        if outputs.len() > 1 {
            let mut agents: Vec<String> = group.iter().map(|r| r.agent_handle.clone()).collect();
            agents.sort();
            agents.dedup();
            conflicts.push(Conflict {
                kind: ConflictKind::Contradiction,
                agents,
                description: format!("Task {} has {} different outputs", task_id, outputs.len()),
                severity: 0.9,
            });
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::crew::CrewResult;

    fn make_results() -> Vec<CrewResult> {
        vec![
            CrewResult {
                agent_handle: "a".into(),
                task_id: "t1".into(),
                output: "answer1".into(),
                success: true,
                iterations_used: 1,
            },
            CrewResult {
                agent_handle: "b".into(),
                task_id: "t1".into(),
                output: "answer1".into(),
                success: true,
                iterations_used: 1,
            },
            CrewResult {
                agent_handle: "c".into(),
                task_id: "t1".into(),
                output: "answer2".into(),
                success: true,
                iterations_used: 1,
            },
        ]
    }

    #[test]
    fn voting_picks_majority() {
        let results = make_results();
        let agg = aggregate(&results, AggregationStrategy::Voting);
        assert_eq!(agg.output, "answer1");
        assert!((agg.confidence - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn first_valid_returns_first() {
        let results = make_results();
        let agg = aggregate(&results, AggregationStrategy::FirstValid);
        assert_eq!(agg.output, "answer1");
        assert_eq!(agg.contributors, 1);
    }

    #[test]
    fn merge_all_concatenates() {
        let results = make_results();
        let agg = aggregate(&results, AggregationStrategy::MergeAll);
        assert!(agg.output.contains("answer1"));
        assert!(agg.output.contains("answer2"));
        assert_eq!(agg.contributors, 3);
    }

    #[test]
    fn all_failed_returns_empty() {
        let results = vec![CrewResult {
            agent_handle: "a".into(),
            task_id: "t1".into(),
            output: "err".into(),
            success: false,
            iterations_used: 1,
        }];
        let agg = aggregate(&results, AggregationStrategy::Voting);
        assert!(agg.output.is_empty());
        assert_eq!(agg.confidence, 0.0);
        assert_eq!(agg.conflicts.len(), 1);
    }

    #[test]
    fn conflict_detection_contradiction() {
        let results = make_results();
        let conflicts = detect_conflicts(&results);
        assert!(conflicts.iter().any(|c| c.kind == ConflictKind::Contradiction));
    }

    #[test]
    fn conflict_detection_success_failure_split() {
        let results = vec![
            CrewResult {
                agent_handle: "a".into(),
                task_id: "t1".into(),
                output: "ok".into(),
                success: true,
                iterations_used: 1,
            },
            CrewResult {
                agent_handle: "b".into(),
                task_id: "t1".into(),
                output: "err".into(),
                success: false,
                iterations_used: 1,
            },
        ];
        let conflicts = detect_conflicts(&results);
        assert!(conflicts.iter().any(|c| c.kind == ConflictKind::SuccessFailureSplit));
    }

    #[test]
    fn no_conflict_when_consensus() {
        let results = vec![
            CrewResult {
                agent_handle: "a".into(),
                task_id: "t1".into(),
                output: "same".into(),
                success: true,
                iterations_used: 1,
            },
            CrewResult {
                agent_handle: "b".into(),
                task_id: "t1".into(),
                output: "same".into(),
                success: true,
                iterations_used: 1,
            },
        ];
        let conflicts = detect_conflicts(&results);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn strategy_display() {
        assert_eq!(AggregationStrategy::Voting.to_string(), "Voting");
        assert_eq!(AggregationStrategy::WeightedAverage.to_string(), "WeightedAverage");
        assert_eq!(AggregationStrategy::FirstValid.to_string(), "FirstValid");
        assert_eq!(AggregationStrategy::MergeAll.to_string(), "MergeAll");
    }

    #[test]
    fn weighted_picks_highest_weight() {
        let results = make_results();
        let weights = vec![
            AgentWeight { agent_handle: "a".into(), weight: 0.5 },
            AgentWeight { agent_handle: "b".into(), weight: 1.0 },
            AgentWeight { agent_handle: "c".into(), weight: 0.3 },
        ];
        let agg = aggregate_weighted(
            &results.iter().collect::<Vec<_>>(),
            &weights,
        );
        assert_eq!(agg.output, "answer1");
    }
}
