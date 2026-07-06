use std::collections::HashMap;

use super::planner::{Plan, PlanningStrategy, Step};
use super::executor::ExecutionResult;

#[derive(Debug, Clone)]
pub struct ReflectionEntry {
    pub step_id: usize,
    pub expected: String,
    pub actual: String,
    pub match_score: f64,
    pub insight: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Reflection {
    pub plan_id: String,
    pub adjusted_plan: Option<Plan>,
    pub insights: Vec<String>,
    pub lessons: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Reflector {
    history: Vec<ReflectionEntry>,
    cycle_count: usize,
    pattern_counts: HashMap<String, usize>,
}

impl Default for Reflector {
    fn default() -> Self {
        Self::new()
    }
}

impl Reflector {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            cycle_count: 0,
            pattern_counts: HashMap::new(),
        }
    }

    pub fn reflect_step(&mut self, step: &Step, result: &ExecutionResult, expected: &str) {
        let actual = if result.success {
            result.output.clone()
        } else {
            result.error.clone().unwrap_or_else(|| "unknown error".to_string())
        };

        let score = if result.success {
            if actual.contains(expected) || expected.contains(&actual) {
                0.95
            } else {
                0.70
            }
        } else {
            0.15
        };

        let insight = if !result.success {
            Some(format!(
                "Step {} failed (score {:.2}): {}",
                step.id, score, actual
            ))
        } else if score < 0.5 {
            Some(format!("Step {} partial match: expected '{}', got '{}'", step.id, expected, actual))
        } else {
            None
        };

        if let Some(ref ins) = insight {
            let key = if ins.contains("failed") { "failure" } else { "partial" };
            *self.pattern_counts.entry(key.to_string()).or_insert(0) += 1;
        }

        self.history.push(ReflectionEntry {
            step_id: step.id,
            expected: expected.to_string(),
            actual,
            match_score: score,
            insight,
        });
    }

    pub fn reflect(&mut self, plan: &Plan, results: &[ExecutionResult]) -> Reflection {
        self.cycle_count += 1;

        let mut insights = Vec::new();
        let mut lessons = Vec::new();
        let mut all_scores = Vec::new();

        for entry in &self.history {
            all_scores.push(entry.match_score);
            if let Some(ref insight) = entry.insight {
                if !insights.contains(insight) {
                    insights.push(insight.clone());
                }
            }
        }

        let avg_score = if all_scores.is_empty() {
            0.0
        } else {
            all_scores.iter().sum::<f64>() / all_scores.len() as f64
        };

        let failure_count = results.iter().filter(|r| !r.success).count();
        let success_rate = if results.is_empty() {
            1.0
        } else {
            (results.len() - failure_count) as f64 / results.len() as f64
        };

        if success_rate < 0.5 {
            lessons.push(format!(
                "High failure rate ({:.0}%): consider simpler decomposition",
                success_rate * 100.0
            ));
        }

        if self.history.len() > 3 {
            let recent: Vec<f64> = self.history.iter().rev().take(3).map(|e| e.match_score).collect();
            let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
            if recent_avg < 0.4 {
                lessons.push("Recent steps show degradation — need strategy shift".to_string());
            }
        }

        let failure_pattern = self.pattern_counts.get("failure").copied().unwrap_or(0);
        if failure_pattern > 2 {
            lessons.push(format!(
                "Recurring failure pattern detected ({} occurrences)",
                failure_pattern
            ));
        }

        let adjusted_plan = if avg_score < 0.4 {
            let new_strategy = match plan.strategy {
                PlanningStrategy::TopDown => PlanningStrategy::BottomUp,
                PlanningStrategy::BottomUp => PlanningStrategy::TopDown,
                PlanningStrategy::Adaptive => PlanningStrategy::TopDown,
            };
            let mut planner = super::planner::Planner::with_strategy(new_strategy);
            let tools: Vec<String> = results.iter().filter_map(|r| {
                if r.success { Some("inferred".to_string()) } else { None }
            }).collect();
            Some(planner.plan(&plan.task, &tools))
        } else {
            None
        };

        let confidence = (avg_score * 0.6 + success_rate * 0.4);

        Reflection {
            plan_id: format!("plan-{}-cycle-{}", plan.task.len(), self.cycle_count),
            adjusted_plan,
            insights,
            lessons,
            confidence: (confidence * 100.0).round() / 100.0,
        }
    }

    pub fn history(&self) -> &[ReflectionEntry] {
        &self.history
    }

    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.pattern_counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(id: usize, desc: &str) -> Step {
        Step { id, description: desc.to_string(), required_tool: None, depends_on: vec![] }
    }

    fn make_result(step_id: usize, success: bool, output: &str) -> ExecutionResult {
        ExecutionResult {
            step_id,
            success,
            output: output.to_string(),
            error: if success { None } else { Some(output.to_string()) },
            duration_ms: 10,
        }
    }

    #[test]
    fn test_reflector_creation() {
        let reflector = Reflector::new();
        assert_eq!(reflector.cycle_count(), 0);
    }

    #[test]
    fn test_reflect_step_success() {
        let mut reflector = Reflector::new();
        let step = make_step(0, "do something");
        let result = make_result(0, true, "doing something");
        reflector.reflect_step(&step, &result, "do something");
        assert_eq!(reflector.history().len(), 1);
        assert!(reflector.history()[0].match_score > 0.5);
    }

    #[test]
    fn test_reflect_step_failure() {
        let mut reflector = Reflector::new();
        let step = make_step(0, "do something");
        let result = make_result(0, false, "error occurred");
        reflector.reflect_step(&step, &result, "do something");
        assert!(reflector.history()[0].match_score < 0.5);
    }

    #[test]
    fn test_reflect_generates_insights() {
        let mut reflector = Reflector::new();
        let plan = Plan { task: "test".to_string(), steps: vec![], strategy: PlanningStrategy::TopDown };

        for i in 0..4 {
            let step = make_step(i, &format!("step {}", i));
            let result = make_result(i, i % 2 == 0, &format!("output {}", i));
            reflector.reflect_step(&step, &result, &format!("step {}", i));
        }

        let results: Vec<ExecutionResult> = (0..4).map(|i| make_result(i, i % 2 == 0, &format!("output {}", i))).collect();
        let reflection = reflector.reflect(&plan, &results);
        assert!(!reflection.insights.is_empty());
    }

    #[test]
    fn test_reflect_adjusts_plan_on_low_confidence() {
        let mut reflector = Reflector::new();
        let plan = Plan { task: "test".to_string(), steps: vec![], strategy: PlanningStrategy::TopDown };

        for i in 0..3 {
            let step = make_step(i, &format!("step {}", i));
            let result = make_result(i, false, "failed");
            reflector.reflect_step(&step, &result, "expected");
        }

        let results: Vec<ExecutionResult> = (0..3).map(|i| make_result(i, false, "failed")).collect();
        let reflection = reflector.reflect(&plan, &results);
        assert!(reflection.adjusted_plan.is_some());
        assert!(reflection.confidence < 0.5);
    }

    #[test]
    fn test_clear_history() {
        let mut reflector = Reflector::new();
        let step = make_step(0, "x");
        let result = make_result(0, true, "x");
        reflector.reflect_step(&step, &result, "x");
        assert_eq!(reflector.history().len(), 1);
        reflector.clear_history();
        assert_eq!(reflector.history().len(), 0);
    }
}
