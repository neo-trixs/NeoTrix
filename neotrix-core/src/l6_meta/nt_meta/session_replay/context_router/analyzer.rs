#![deny(clippy::unwrap_used)]

use super::context::ContextSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analysis result derived from a series of context snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    pub common_patterns: Vec<String>,
    pub task_distribution: HashMap<String, u64>,
    pub avg_turns_per_task: f64,
    pub bottlenecks: Vec<String>,
}

/// Analyzer that extracts patterns and bottlenecks from snapshot history.
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a slice of context snapshots.
    pub fn analyze(&self, snapshots: &[ContextSnapshot]) -> ContextAnalysis {
        if snapshots.is_empty() {
            return ContextAnalysis {
                common_patterns: Vec::new(),
                task_distribution: HashMap::new(),
                avg_turns_per_task: 0.0,
                bottlenecks: Vec::new(),
            };
        }

        let mut task_counts: HashMap<String, u64> = HashMap::new();
        let mut active_agent_counts: HashMap<String, u64> = HashMap::new();
        let mut total_turns: u64 = 0;
        let mut consecutive_same_task = 0u64;
        let mut max_consecutive = 0u64;

        for (i, snap) in snapshots.iter().enumerate() {
            // Track task distribution
            if let Some(task) = &snap.state.current_task {
                *task_counts.entry(task.clone()).or_insert(0) += 1;

                // Track consecutive same task (bottleneck indicator)
                if i > 0 {
                    if let Some(prev_task) = &snapshots[i - 1].state.current_task {
                        if prev_task == task {
                            consecutive_same_task += 1;
                        } else {
                            max_consecutive = max_consecutive.max(consecutive_same_task);
                            consecutive_same_task = 0;
                        }
                    }
                }
            }

            // Track agent usage
            for agent in &snap.state.active_agents {
                *active_agent_counts.entry(agent.clone()).or_insert(0) += 1;
            }

            total_turns += snap.turn;
        }

        max_consecutive = max_consecutive.max(consecutive_same_task);

        // Build common patterns
        let mut common_patterns = Vec::new();

        let dominant_task = task_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(t, c)| (t.clone(), *c));
        if let Some((task, count)) = dominant_task {
            if count > 1 {
                common_patterns.push(format!("dominant task: '{task}' ({count}x)"));
            }
        }

        let dominant_agent = active_agent_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(a, c)| (a.clone(), *c));
        if let Some((agent, count)) = dominant_agent {
            if count > 1 {
                common_patterns.push(format!("dominant agent: '{agent}' ({count}x)"));
            }
        }

        let unique_tasks = task_counts.len();
        if unique_tasks > 1 {
            common_patterns.push(format!("{unique_tasks} distinct tasks observed"));
        }

        // Detect bottlenecks
        let mut bottlenecks = Vec::new();

        if max_consecutive >= 3 {
            bottlenecks.push(format!(
                "repeated task for {max_consecutive} consecutive snapshots"
            ));
        }

        for (task, count) in &task_counts {
            let total: u64 = task_counts.values().sum();
            if total > 0 && *count as f64 / total as f64 > 0.7 {
                bottlenecks.push(format!("task '{task}' dominates ({count}/{total})"));
            }
        }

        let avg_turns = if snapshots.is_empty() {
            0.0
        } else {
            total_turns as f64 / snapshots.len() as f64
        };

        ContextAnalysis {
            common_patterns,
            task_distribution: task_counts,
            avg_turns_per_task: avg_turns,
            bottlenecks,
        }
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::nt_meta::session_replay::context_router::context::ContextState;

    fn make_snapshots(tasks: &[&str]) -> Vec<ContextSnapshot> {
        tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let state = ContextState::new().with_task(*task).with_turn(i as u64);
                ContextSnapshot {
                    state,
                    timestamp: 1000 + i as u128,
                    turn: i as u64,
                }
            })
            .collect()
    }

    #[test]
    fn test_analyze_empty() {
        let analyzer = ContextAnalyzer::new();
        let result = analyzer.analyze(&[]);
        assert!(result.common_patterns.is_empty());
        assert!(result.task_distribution.is_empty());
        assert_eq!(result.avg_turns_per_task, 0.0);
        assert!(result.bottlenecks.is_empty());
    }

    #[test]
    fn test_analyze_single_task_repetition() {
        let analyzer = ContextAnalyzer::new();
        let snaps = make_snapshots(&["summarize", "summarize", "summarize", "summarize"]);
        let result = analyzer.analyze(&snaps);
        assert!(result.bottlenecks.iter().any(|b| b.contains("dominates")));
        assert!(result
            .common_patterns
            .iter()
            .any(|p| p.contains("dominant task")));
    }

    #[test]
    fn test_analyze_diverse_tasks() {
        let analyzer = ContextAnalyzer::new();
        let snaps = make_snapshots(&["search", "code", "debug", "review"]);
        let result = analyzer.analyze(&snaps);
        assert!(result
            .common_patterns
            .iter()
            .any(|p| p.contains("4 distinct")));
    }

    #[test]
    fn test_analyze_consecutive_bottleneck() {
        let analyzer = ContextAnalyzer::new();
        let snaps = make_snapshots(&["debug", "debug", "debug", "debug", "code"]);
        let result = analyzer.analyze(&snaps);
        assert!(result.bottlenecks.iter().any(|b| b.contains("consecutive")));
    }

    #[test]
    fn test_analyze_single_snapshot() {
        let analyzer = ContextAnalyzer::new();
        let snaps = make_snapshots(&["unique_task"]);
        let result = analyzer.analyze(&snaps);
        assert_eq!(result.task_distribution.len(), 1);
        assert_eq!(result.avg_turns_per_task, 0.0);
    }

    #[test]
    fn test_analyze_agent_tracking() {
        let analyzer = ContextAnalyzer::new();
        let snaps: Vec<ContextSnapshot> = (0..3)
            .map(|i| {
                let state = ContextState::new()
                    .with_task("task")
                    .with_turn(i)
                    .with_agent("agent_a")
                    .with_agent("agent_b");
                ContextSnapshot {
                    state,
                    timestamp: 1000 + i as u128,
                    turn: i as u64,
                }
            })
            .collect();
        let result = analyzer.analyze(&snaps);
        // Both agents should appear in patterns as dominant
        assert!(result
            .common_patterns
            .iter()
            .any(|p| p.contains("agent_a")));
    }

    #[test]
    fn test_analyze_two_tasks_no_bottleneck() {
        let analyzer = ContextAnalyzer::new();
        let snaps = make_snapshots(&["task_a", "task_b", "task_a", "task_b"]);
        let result = analyzer.analyze(&snaps);
        assert!(result.bottlenecks.is_empty());
        assert!(result
            .common_patterns
            .iter()
            .any(|p| p.contains("2 distinct")));
    }

    #[test]
    fn test_analyzer_default() {
        let analyzer = ContextAnalyzer::default();
        let result = analyzer.analyze(&[]);
        assert!(result.common_patterns.is_empty());
    }
}
