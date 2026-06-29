use super::attention_head::AttentionDomain;
use super::reasoning_strategy::StrategyKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ReflectionGrade {
    Excellent,
    Good,
    Adequate,
    Poor,
    Failed,
}

impl ReflectionGrade {
    pub fn score(&self) -> f64 {
        match self {
            ReflectionGrade::Excellent => 1.0,
            ReflectionGrade::Good => 0.75,
            ReflectionGrade::Adequate => 0.5,
            ReflectionGrade::Poor => 0.25,
            ReflectionGrade::Failed => 0.0,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ReflectionGrade::Excellent => "excellent",
            ReflectionGrade::Good => "good",
            ReflectionGrade::Adequate => "adequate",
            ReflectionGrade::Poor => "poor",
            ReflectionGrade::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThinkingStep {
    pub step_number: usize,
    pub description: String,
    pub strategy: StrategyKind,
    pub domain: AttentionDomain,
    pub duration_ms: u64,
    pub tokens_used: usize,
    pub tools_used: Vec<String>,
    pub intermediate_result: String,
    pub confidence: f64,
}

impl ThinkingStep {
    pub fn new(step_number: usize, description: &str, strategy: StrategyKind) -> Self {
        Self {
            step_number,
            description: description.to_string(),
            strategy,
            domain: AttentionDomain::PatternMatch,
            duration_ms: 0,
            tokens_used: 0,
            tools_used: Vec::new(),
            intermediate_result: String::new(),
            confidence: 0.5,
        }
    }

    pub fn with_domain(mut self, domain: AttentionDomain) -> Self {
        self.domain = domain;
        self
    }

    pub fn with_result(mut self, result: &str) -> Self {
        self.intermediate_result = result.to_string();
        self
    }

    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tools_used.push(tool.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ThinkingTrace {
    pub id: usize,
    pub task: String,
    pub steps: Vec<ThinkingStep>,
    pub grade: ReflectionGrade,
    pub total_duration_ms: u64,
    pub total_tokens: usize,
    pub final_answer: String,
    pub errors: Vec<String>,
    pub timestamp: f64,
}

impl ThinkingTrace {
    pub fn new(id: usize, task: &str) -> Self {
        Self {
            id,
            task: task.to_string(),
            steps: Vec::new(),
            grade: ReflectionGrade::Adequate,
            total_duration_ms: 0,
            total_tokens: 0,
            final_answer: String::new(),
            errors: Vec::new(),
            timestamp: 0.0,
        }
    }

    pub fn add_step(&mut self, step: ThinkingStep) {
        self.total_duration_ms += step.duration_ms;
        self.total_tokens += step.tokens_used;
        self.steps.push(step);
    }

    pub fn num_steps(&self) -> usize {
        self.steps.len()
    }

    pub fn strategies_used(&self) -> Vec<StrategyKind> {
        let mut strategies: Vec<StrategyKind> = self.steps.iter().map(|s| s.strategy).collect();
        strategies.sort_by_key(|s| *s as u8);
        strategies.dedup();
        strategies
    }

    pub fn domains_used(&self) -> Vec<AttentionDomain> {
        let mut domains: Vec<AttentionDomain> = self.steps.iter().map(|s| s.domain).collect();
        domains.sort_by_key(|d| *d as u8);
        domains.dedup();
        domains
    }

    pub fn tools_used(&self) -> Vec<String> {
        let mut tools: Vec<String> = self
            .steps
            .iter()
            .flat_map(|s| s.tools_used.clone())
            .collect();
        tools.sort();
        tools.dedup();
        tools
    }

    pub fn avg_confidence(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }
        self.steps.iter().map(|s| s.confidence).sum::<f64>() / self.steps.len() as f64
    }

    pub fn record_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn set_grade_from_accuracy(&mut self, accuracy: f64) {
        self.grade = if accuracy >= 0.9 {
            ReflectionGrade::Excellent
        } else if accuracy >= 0.7 {
            ReflectionGrade::Good
        } else if accuracy >= 0.5 {
            ReflectionGrade::Adequate
        } else if accuracy >= 0.2 {
            ReflectionGrade::Poor
        } else {
            ReflectionGrade::Failed
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::attention_head::AttentionDomain;
    use super::super::reasoning_strategy::StrategyKind;
    use super::*;

    #[test]
    fn test_thinking_trace_new() {
        let trace = ThinkingTrace::new(0, "analyze architecture");
        assert_eq!(trace.task, "analyze architecture");
        assert_eq!(trace.num_steps(), 0);
    }

    #[test]
    fn test_add_step() {
        let mut trace = ThinkingTrace::new(0, "test");
        let step = ThinkingStep::new(1, "parse input", StrategyKind::Direct);
        trace.add_step(step);
        assert_eq!(trace.num_steps(), 1);
    }

    #[test]
    fn test_strategies_used_dedup() {
        let mut trace = ThinkingTrace::new(0, "test");
        trace.add_step(ThinkingStep::new(1, "step1", StrategyKind::Direct));
        trace.add_step(ThinkingStep::new(2, "step2", StrategyKind::ChainOfThought));
        trace.add_step(ThinkingStep::new(3, "step3", StrategyKind::Direct));
        let strategies = trace.strategies_used();
        assert_eq!(strategies.len(), 2);
    }

    #[test]
    fn test_avg_confidence() {
        let mut trace = ThinkingTrace::new(0, "test");
        trace.add_step(ThinkingStep::new(1, "step1", StrategyKind::Direct).with_result("ok"));
        let mut step2 = ThinkingStep::new(2, "step2", StrategyKind::ChainOfThought);
        step2.confidence = 0.9;
        trace.add_step(step2);
        assert!((trace.avg_confidence() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_errors_tracking() {
        let mut trace = ThinkingTrace::new(0, "test");
        assert!(!trace.has_errors());
        trace.record_error("tool failed");
        assert!(trace.has_errors());
        assert_eq!(trace.errors.len(), 1);
    }

    #[test]
    fn test_set_grade_from_accuracy() {
        let mut trace = ThinkingTrace::new(0, "test");
        trace.set_grade_from_accuracy(0.95);
        assert_eq!(trace.grade.label(), "excellent");
        trace.set_grade_from_accuracy(0.8);
        assert_eq!(trace.grade.label(), "good");
        trace.set_grade_from_accuracy(0.3);
        assert_eq!(trace.grade.label(), "poor");
        trace.set_grade_from_accuracy(0.1);
        assert_eq!(trace.grade.label(), "failed");
    }

    #[test]
    fn test_tools_used_dedup() {
        let mut trace = ThinkingTrace::new(0, "test");
        trace
            .add_step(ThinkingStep::new(1, "search", StrategyKind::ToolAssisted).with_tool("grep"));
        trace.add_step(
            ThinkingStep::new(2, "analyze", StrategyKind::ToolAssisted).with_tool("grep"),
        );
        trace.add_step(ThinkingStep::new(3, "read", StrategyKind::ToolAssisted).with_tool("read"));
        let tools = trace.tools_used();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_step_with_domain() {
        let step = ThinkingStep::new(1, "reflect", StrategyKind::Reflection)
            .with_domain(AttentionDomain::SelfReflection);
        assert_eq!(step.domain, AttentionDomain::SelfReflection);
    }

    #[test]
    fn test_reflection_grade_scores() {
        assert!((ReflectionGrade::Excellent.score() - 1.0).abs() < 1e-6);
        assert!((ReflectionGrade::Failed.score() - 0.0).abs() < 1e-6);
    }
}
