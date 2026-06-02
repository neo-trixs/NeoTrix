
use neotrix_types::memory::ReasoningMemory;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnomalyType {
    TrajectoryRepetition,
    ErrorRepetition(String),
    LoopPseudoTermination,
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub action: String,
    pub result: String,
    pub success: bool,
    pub timestamp: u64,
}

pub struct InnerGuard {
    pub trajectory_repetition_threshold: usize,
    pub error_repetition_threshold: usize,
    pub loop_pseudo_termination_window: usize,
    trajectory_history: Vec<Vec<String>>,
    error_history: Vec<(String, u64)>,
    pseudo_termination_log: Vec<bool>,
}

impl InnerGuard {
    pub fn new() -> Self {
        Self {
            trajectory_repetition_threshold: 3,
            error_repetition_threshold: 3,
            loop_pseudo_termination_window: 5,
            trajectory_history: Vec::new(),
            error_history: Vec::new(),
            pseudo_termination_log: Vec::new(),
        }
    }

    pub fn with_thresholds(
        trajectory: usize,
        error: usize,
        window: usize,
    ) -> Self {
        Self {
            trajectory_repetition_threshold: trajectory,
            error_repetition_threshold: error,
            loop_pseudo_termination_window: window,
            trajectory_history: Vec::new(),
            error_history: Vec::new(),
            pseudo_termination_log: Vec::new(),
        }
    }

    pub fn detect_anomalies(&mut self, trajectory: &[TraceStep]) -> Vec<AnomalyType> {
        let mut anomalies = Vec::new();

        let action_seq: Vec<String> = trajectory.iter().map(|s| s.action.clone()).collect();
        self.trajectory_history.push(action_seq);
        if self.trajectory_history.len() >= self.trajectory_repetition_threshold {
            let recent = &self.trajectory_history[self.trajectory_history.len() - self.trajectory_repetition_threshold..];
            if recent.windows(2).all(|w| w[0] == w[1]) {
                anomalies.push(AnomalyType::TrajectoryRepetition);
            }
        }

        for step in trajectory {
            if !step.success {
                let count = self.error_history.iter().filter(|(a, _)| a == &step.action).count();
                if count >= self.error_repetition_threshold {
                    anomalies.push(AnomalyType::ErrorRepetition(step.action.clone()));
                }
                self.error_history.push((step.action.clone(), step.timestamp));
            }
        }

        let is_pseudo_termination = trajectory.last().map_or(false, |s| {
            s.result.contains("done") || s.result.contains("complete") || s.result.contains("finished")
        }) && trajectory.len() > 1
            && trajectory.last().map_or(true, |s| !s.success);

        self.pseudo_termination_log.push(is_pseudo_termination);
        if self.pseudo_termination_log.len() >= self.loop_pseudo_termination_window {
            let window = &self.pseudo_termination_log[self.pseudo_termination_log.len() - self.loop_pseudo_termination_window..];
            let true_count = window.iter().filter(|&&v| v).count();
            if true_count as f64 / window.len() as f64 > 0.6 {
                anomalies.push(AnomalyType::LoopPseudoTermination);
            }
        }

        anomalies
    }

    pub fn intervene(trajectory: &mut Vec<TraceStep>, anomaly: &AnomalyType) -> String {
        match anomaly {
            AnomalyType::TrajectoryRepetition => {
                let intervention = "Trajectory repetition detected. Switching to explore alternative approach.".to_string();
                if let Some(last) = trajectory.last_mut() {
                    last.result.push_str(" [INTERVENTION: break repetition]");
                }
                intervention
            }
            AnomalyType::ErrorRepetition(action) => {
                let intervention = format!("Error repetition on action '{}'. Applying backoff and alternative strategy.", action);
                trajectory.retain(|s| s.action != *action || s.success);
                intervention
            }
            AnomalyType::LoopPseudoTermination => {
                let intervention = "Pseudo-termination detected. Forcing hard stop and escalation.".to_string();
                intervention
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootCauseAnalysis {
    pub root_cause: String,
    pub failure_type: FailureType,
    pub severity: f64,
    pub prevention_strategy: String,
    pub heuristic: Option<StrategyHeuristic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureType {
    Planning,
    Execution,
    KnowledgeGap,
    SafetyBlocked,
    BudgetExhausted,
}

#[derive(Debug, Clone)]
pub struct StrategyHeuristic {
    pub name: String,
    pub description: String,
    pub avoidance_pattern: String,
    pub applicability: Vec<String>,
    pub confidence: f64,
}

pub struct OuterReflector;

impl OuterReflector {
    pub fn analyze_failure(trajectory: &[TraceStep], outcome: &str) -> RootCauseAnalysis {
        let failure_type = Self::classify_failure(trajectory, outcome);
        let root_cause = Self::infer_root_cause(trajectory, outcome, &failure_type);
        let prevention = Self::suggest_prevention(&failure_type, &root_cause);

        RootCauseAnalysis {
            root_cause,
            failure_type,
            severity: Self::compute_severity(trajectory, outcome),
            prevention_strategy: prevention,
            heuristic: None,
        }
    }

    pub fn extract_heuristic(rca: &RootCauseAnalysis) -> StrategyHeuristic {
        let name = format!("avoid_{}", rca.failure_type.as_str());
        StrategyHeuristic {
            name: name.clone(),
            description: format!("Heuristic to prevent {} failures: {}", rca.failure_type.as_str(), rca.root_cause),
            avoidance_pattern: rca.prevention_strategy.clone(),
            applicability: vec![format!("{:?}", rca.failure_type)],
            confidence: 1.0 - rca.severity.min(1.0),
        }
    }

    pub fn store_as_memory(heuristic: &StrategyHeuristic, bank: &mut neotrix_types::memory::ReasoningBank) {
//        let desc = format!("Heuristic: {} — {}", heuristic.name, heuristic.description);
        let mem = ReasoningMemory::new(&desc, neotrix_types::knowledge::TaskType::Reflection, &[], heuristic.confidence);
        bank.store(mem);
    }

    fn classify_failure(trajectory: &[TraceStep], outcome: &str) -> FailureType {
        let outcome_lower = outcome.to_lowercase();
        if outcome_lower.contains("budget") || outcome_lower.contains("timeout") {
            return FailureType::BudgetExhausted;
        }
        if outcome_lower.contains("safety") || outcome_lower.contains("blocked") {
            return FailureType::SafetyBlocked;
        }
        if outcome_lower.contains("knowledge") || outcome_lower.contains("unknown") {
            return FailureType::KnowledgeGap;
        }
        let success_count = trajectory.iter().filter(|s| s.success).count();
        let total = trajectory.len();
        if total > 0 && (success_count as f64) / (total as f64) < 0.5 {
            return FailureType::Execution;
        }
        FailureType::Planning
    }

    fn infer_root_cause(trajectory: &[TraceStep], outcome: &str, ftype: &FailureType) -> String {
        match ftype {
            FailureType::Planning => {
                let first_failure = trajectory.iter().find(|s| !s.success);
                match first_failure {
                    Some(step) => format!("Initial approach '{}' failed, leading to cascading breakdown", step.action),
                    None => format!("Planning error: outcome '{}' not achieved despite all steps nominal", outcome),
                }
            }
            FailureType::Execution => {
                let failures: Vec<&str> = trajectory.iter().filter(|s| !s.success).map(|s| s.action.as_str()).collect();
                format!("Execution failures at steps: {:?}", failures)
            }
            FailureType::KnowledgeGap => "Missing domain knowledge for task requirements".to_string(),
            FailureType::SafetyBlocked => "Action blocked by safety constraints".to_string(),
            FailureType::BudgetExhausted => "Resource budget exhausted before completion".to_string(),
        }
    }

    fn suggest_prevention(ftype: &FailureType, root_cause: &str) -> String {
        match ftype {
            FailureType::Planning => format!("Prevention: decompose plan into smaller verifiable steps. Issue: {}", root_cause),
            FailureType::Execution => format!("Prevention: add preconditions check before each step. Issue: {}", root_cause),
            FailureType::KnowledgeGap => format!("Prevention: query knowledge base before planning. Issue: {}", root_cause),
            FailureType::SafetyBlocked => format!("Prevention: pre-screen actions against safety policy. Issue: {}", root_cause),
            FailureType::BudgetExhausted => format!("Prevention: set conservative budget with checkpoint. Issue: {}", root_cause),
        }
    }

    fn compute_severity(trajectory: &[TraceStep], outcome: &str) -> f64 {
        let failure_rate = trajectory.iter().filter(|s| !s.success).count() as f64 / trajectory.len().max(1) as f64;
        let outcome_severity = if outcome.contains("complete") || outcome.contains("success") { 0.2 } else { 0.8 };
        0.6 * failure_rate + 0.4 * outcome_severity
    }
}

impl FailureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailureType::Planning => "planning",
            FailureType::Execution => "execution",
            FailureType::KnowledgeGap => "knowledge_gap",
            FailureType::SafetyBlocked => "safety_blocked",
            FailureType::BudgetExhausted => "budget_exhausted",
        }
    }
}

pub struct DualPathAligner;

impl DualPathAligner {
    pub fn align_explicit(memory: &mut ReasoningMemory, feedback: &str) {
        let positive = feedback.contains("good") || feedback.contains("correct") || feedback.contains("yes");
        let negative = feedback.contains("bad") || feedback.contains("wrong") || feedback.contains("no");
        if positive {
            memory.reward = (memory.reward + 1.0).min(1.0) * 0.5 + memory.reward * 0.5;
            memory.success = true;
        } else if negative {
            memory.reward = memory.reward * 0.5;
            memory.success = false;
        }
    }

    pub fn align_implicit(memory: &mut ReasoningMemory, context: &[TraceStep]) {
        let trace_success_rate = context.iter().filter(|s| s.success).count() as f64 / context.len().max(1) as f64;
        memory.reward = memory.reward * 0.7 + trace_success_rate * 0.3;
        if trace_success_rate > 0.6 {
            memory.success = true;
        }
    }

    pub fn align(memory: &mut ReasoningMemory, feedback: Option<&str>, context: &[TraceStep]) {
        if memory.reward > 0.95 {
            return;
        }
        if let Some(fb) = feedback {
            Self::align_explicit(memory, fb);
        }
        Self::align_implicit(memory, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(action: &str, result: &str, success: bool) -> TraceStep {
        TraceStep {
            action: action.to_string(),
            result: result.to_string(),
            success,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_inner_guard_trajectory_repetition() {
        let mut guard = InnerGuard::with_thresholds(3, 5, 5);
        let traj: Vec<TraceStep> = vec![
            make_step("parse", "ok", true),
            make_step("compute", "ok", true),
        ];
        assert!(guard.detect_anomalies(&traj).is_empty());
        assert!(guard.detect_anomalies(&traj).is_empty());
        let anomalies = guard.detect_anomalies(&traj);
        assert!(anomalies.contains(&AnomalyType::TrajectoryRepetition));
    }

    #[test]
    fn test_inner_guard_error_repetition() {
        let mut guard = InnerGuard::with_thresholds(5, 2, 5);
        let traj = vec![
            make_step("call_api", "error", false),
            make_step("call_api", "error", false),
            make_step("call_api", "error", false),
        ];
        let anomalies = guard.detect_anomalies(&traj);
        assert!(anomalies.contains(&AnomalyType::ErrorRepetition("call_api".to_string())));
    }

    #[test]
    fn test_outer_reflector_analyze_failure() {
        let traj = vec![
            make_step("parse_input", "ok", true),
            make_step("process", "error", false),
            make_step("retry", "error", false),
        ];
        let rca = OuterReflector::analyze_failure(&traj, "failed to complete");
        assert!(!rca.root_cause.is_empty());
        assert_eq!(rca.failure_type, FailureType::Execution);
        assert!(rca.severity > 0.0);
    }

    #[test]
    fn test_extract_heuristic() {
        let rca = RootCauseAnalysis {
            root_cause: "Missing validation before step".to_string(),
            failure_type: FailureType::Planning,
            severity: 0.6,
            prevention_strategy: "add preconditions".to_string(),
            heuristic: None,
        };
        let heuristic = OuterReflector::extract_heuristic(&rca);
        assert!(heuristic.name.contains("planning"));
        assert!(!heuristic.description.is_empty());
    }

    #[test]
    fn test_dual_path_align_explicit_positive() {
        let mut mem = ReasoningMemory::new("test", neotrix_types::knowledge::TaskType::General, &[], 0.5);
        DualPathAligner::align_explicit(&mut mem, "good work");
        assert!(mem.reward > 0.5);
        assert!(mem.success);
    }

    #[test]
    fn test_dual_path_align_explicit_negative() {
        let mut mem = ReasoningMemory::new("test", neotrix_types::knowledge::TaskType::General, &[], 0.8);
        DualPathAligner::align_explicit(&mut mem, "wrong approach");
        assert!(mem.reward < 0.8);
    }

    #[test]
    fn test_dual_path_align_combined() {
        let mut mem = ReasoningMemory::new("test", neotrix_types::knowledge::TaskType::General, &[], 0.5);
        let ctx = vec![make_step("step1", "ok", true), make_step("step2", "ok", true)];
        DualPathAligner::align(&mut mem, Some("good"), &ctx);
        assert!(mem.reward > 0.5);
    }

    #[test]
    fn test_inner_guard_intervene_returns_string() {
        let mut traj = vec![make_step("loop", "fail", false)];
        let intervention = InnerGuard::intervene(&mut traj, &AnomalyType::TrajectoryRepetition);
        assert!(!intervention.is_empty());
    }

    #[test]
    fn test_outer_reflector_analyze_knowledge_gap() {
        let traj = vec![make_step("query", "unknown concept", false)];
        let rca = OuterReflector::analyze_failure(&traj, "knowledge missing");
        assert_eq!(rca.failure_type, FailureType::KnowledgeGap);
    }

    #[test]
    fn test_outer_reflector_analyze_budget() {
        let traj = vec![make_step("execute", "timeout", false)];
        let rca = OuterReflector::analyze_failure(&traj, "budget exhausted");
        assert_eq!(rca.failure_type, FailureType::BudgetExhausted);
    }
}
