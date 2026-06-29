use super::context_window::{ContextWindow, CognitiveUnitKind};
use super::attention_head::{AttentionManager, AttentionProfile, AttentionDomain};
use super::system_identity::SystemIdentity;
use super::reasoning_strategy::{ReasoningStrategyRegistry, StrategyKind};
use super::thinking_trace::{ThinkingTrace, ThinkingStep, ReflectionGrade};

#[derive(Debug, Clone)]
pub struct SiliconSelfState {
    pub active_strategy: StrategyKind,
    pub attention_profile: AttentionProfile,
    pub context_usage: f64,
    pub current_focus: Vec<String>,
    pub thinking_depth: usize,
}

pub struct SiliconSelfModel {
    pub identity: SystemIdentity,
    pub context_window: ContextWindow,
    pub attention_manager: AttentionManager,
    pub strategy_registry: ReasoningStrategyRegistry,
    pub thinking_traces: Vec<ThinkingTrace>,
    pub iteration: usize,
    pub max_traces: usize,
}

impl Default for SiliconSelfModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SiliconSelfModel {
    pub fn new() -> Self {
        Self {
            identity: SystemIdentity::new(),
            context_window: ContextWindow::new(512),
            attention_manager: AttentionManager::new(0.3),
            strategy_registry: ReasoningStrategyRegistry::new(),
            thinking_traces: Vec::new(),
            iteration: 0,
            max_traces: 100,
        }
    }

    pub fn add_thinking_trace(&mut self, trace: ThinkingTrace) {
        if self.thinking_traces.len() >= self.max_traces {
            self.thinking_traces.remove(0);
        }
        self.thinking_traces.push(trace);
    }

    pub fn current_state(&self) -> SiliconSelfState {
        SiliconSelfState {
            active_strategy: self.strategy_registry.best_by_effectiveness().unwrap_or(StrategyKind::Direct),
            attention_profile: self.attention_manager.profile(),
            context_usage: self.context_window.len() as f64 / self.context_window.capacity as f64,
            current_focus: self.attention_manager.active_heads()
                .iter()
                .flat_map(|h| h.focus.clone())
                .collect(),
            thinking_depth: self.thinking_traces.last().map(|t| t.num_steps()).unwrap_or(0),
        }
    }

    pub fn observe(&mut self, content: &str) -> usize {
        self.iteration += 1;
        self.context_window.observe(CognitiveUnitKind::Observation, content)
    }

    pub fn observe_tool_call(&mut self, tool: &str, result: &str) -> (usize, usize) {
        let call_id = self.context_window.observe(CognitiveUnitKind::ToolCall, &format!("tool:{}", tool));
        let result_id = self.context_window.observe(CognitiveUnitKind::ActionResult, &format!("result:{}", &result[..result.len().min(200)]));
        self.attention_manager.stimulate_domain(AttentionDomain::ToolUse, 0.3);
        (call_id, result_id)
    }

    pub fn begin_thinking_trace(&mut self, task: &str) -> usize {
        let id = self.thinking_traces.len();
        let trace = ThinkingTrace::new(id, task);
        self.thinking_traces.push(trace);
        self.context_window.observe(CognitiveUnitKind::ReasoningStep, &format!("thinking:{}", task));
        id
    }

    pub fn add_thinking_step(&mut self, trace_id: usize, step: ThinkingStep) {
        if let Some(trace) = self.thinking_traces.get_mut(trace_id) {
            trace.add_step(step);
        }
    }

    pub fn complete_thinking_trace(&mut self, trace_id: usize, answer: &str, accuracy: f64) {
        if let Some(trace) = self.thinking_traces.get_mut(trace_id) {
            trace.final_answer = answer.to_string();
            trace.set_grade_from_accuracy(accuracy);
        }
        self.context_window.observe(CognitiveUnitKind::SelfReflection, &format!("completed:{}", answer.len().min(100)));
    }

    pub fn stats(&self) -> String {
        let state = self.current_state();
        let active_count = self.attention_manager.active_heads().len();
        let trace_count = self.thinking_traces.len();
        let success_count = self.thinking_traces.iter()
            .filter(|t| matches!(t.grade, ReflectionGrade::Excellent | ReflectionGrade::Good))
            .count();
        format!(
            "SiliconSelf #{} | strategy={} | attention_domain={:?} | heads_active={} | context={:.0}% | traces={} | successful={}",
            self.iteration,
            state.active_strategy.label(),
            state.attention_profile.dominant,
            active_count,
            state.context_usage * 100.0,
            trace_count,
            success_count,
        )
    }

    pub fn recent_traces(&self, n: usize) -> Vec<&ThinkingTrace> {
        self.thinking_traces.iter().rev().take(n).collect()
    }

    pub fn trace_by_strategy(&self, kind: StrategyKind) -> Vec<&ThinkingTrace> {
        self.thinking_traces.iter().filter(|t| t.strategies_used().contains(&kind)).collect()
    }

    pub fn reset_session(&mut self) {
        self.attention_manager.reset();
        self.context_window.clear_attention();
    }

    /// Lossy serialization: saves iteration, attention activations, strategy effectiveness, identity capabilities.
    /// Does NOT save context_window history or thinking_traces (ephemeral).
    pub fn serialize_state(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("ITERATION:{}", self.iteration));

        let caps: Vec<String> = self.identity.capabilities.iter()
            .map(|(k, v)| format!("{}:{:.4}", k, v))
            .collect();
        lines.push(format!("CAPABILITIES:{}", caps.join(",")));

        let attn: Vec<String> = self.attention_manager.heads.iter()
            .map(|h| format!("{}:{:.4}", h.domain.label(), h.activation))
            .collect();
        lines.push(format!("ATTENTION:{}", attn.join(",")));

        let strat: Vec<String> = self.strategy_registry.strategies.values()
            .map(|s| format!("{}:{:.4}:{}", s.kind.label(), s.effectiveness, s.use_count))
            .collect();
        lines.push(format!("STRATEGY:{}", strat.join(",")));

        lines.push(format!("MAX_TRACES:{}", self.max_traces));

        lines.join("\n")
    }

    /// Parse the text format produced by serialize_state().
    /// Returns None on any parse failure (malformed input).
    pub fn deserialize_state(data: &str) -> Option<Self> {
        let mut model = Self::new();

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once(':')?;
            match key {
                "ITERATION" => model.iteration = value.parse().ok()?,
                "MAX_TRACES" => model.max_traces = value.parse().ok()?,
                "CAPABILITIES" => {
                    for entry in value.split(',') {
                        if entry.is_empty() { continue; }
                        let (name, score_str) = entry.split_once(':')?;
                        let score: f64 = score_str.parse().ok()?;
                        model.identity.capabilities.insert(name.to_string(), score);
                    }
                }
                "ATTENTION" => {
                    for entry in value.split(',') {
                        if entry.is_empty() { continue; }
                        let (domain_label, act_str) = entry.split_once(':')?;
                        let activation: f64 = act_str.parse().ok()?;
                        for head in &mut model.attention_manager.heads {
                            if head.domain.label() == domain_label {
                                head.activation = activation;
                                break;
                            }
                        }
                    }
                }
                "STRATEGY" => {
                    for entry in value.split(',') {
                        if entry.is_empty() { continue; }
                        let parts: Vec<&str> = entry.split(':').collect();
                        if parts.len() < 3 { continue; }
                        let kind_label = parts[0];
                        let effectiveness: f64 = parts[1].parse().ok()?;
                        let use_count: usize = parts[2].parse().ok()?;
                        for strategy in model.strategy_registry.strategies.values_mut() {
                            if strategy.kind.label() == kind_label {
                                strategy.effectiveness = effectiveness;
                                strategy.use_count = use_count;
                                break;
                            }
                        }
                    }
                }
                _ => {} // skip unknown keys for forward compatibility
            }
        }

        Some(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silicon_self_new() {
        let ss = SiliconSelfModel::new();
        assert_eq!(ss.iteration, 0);
        assert_eq!(ss.context_window.capacity, 512);
        assert_eq!(ss.attention_manager.heads.len(), 10);
    }

    #[test]
    fn test_observe_increments_iteration() {
        let mut ss = SiliconSelfModel::new();
        ss.observe("test");
        assert_eq!(ss.iteration, 1);
        assert_eq!(ss.context_window.len(), 1);
    }

    #[test]
    fn test_current_state() {
        let ss = SiliconSelfModel::new();
        let state = ss.current_state();
        assert!(state.context_usage >= 0.0);
        assert!(state.current_focus.is_empty());
    }

    #[test]
    fn test_thinking_trace_lifecycle() {
        let mut ss = SiliconSelfModel::new();
        let trace_id = ss.begin_thinking_trace("analyze code");
        assert_eq!(ss.thinking_traces.len(), 1);
        ss.add_thinking_step(trace_id, ThinkingStep::new(1, "read file", StrategyKind::Direct));
        assert_eq!(ss.thinking_traces[trace_id].num_steps(), 1);
        ss.complete_thinking_trace(trace_id, "done", 0.9);
        assert_eq!(ss.thinking_traces[trace_id].final_answer, "done");
    }

    #[test]
    fn test_stats_format() {
        let mut ss = SiliconSelfModel::new();
        let trace_id = ss.begin_thinking_trace("test");
        ss.add_thinking_step(trace_id, ThinkingStep::new(1, "step", StrategyKind::Direct));
        ss.complete_thinking_trace(trace_id, "ok", 0.95);
        let stats = ss.stats();
        assert!(stats.contains("SiliconSelf"));
        assert!(stats.contains("traces="));
    }

    #[test]
    fn test_trace_by_strategy() {
        let mut ss = SiliconSelfModel::new();
        let t1 = ss.begin_thinking_trace("task1");
        ss.add_thinking_step(t1, ThinkingStep::new(1, "step1", StrategyKind::Direct));
        ss.complete_thinking_trace(t1, "done", 0.8);
        let direct_traces = ss.trace_by_strategy(StrategyKind::Direct);
        assert_eq!(direct_traces.len(), 1);
        let cot_traces = ss.trace_by_strategy(StrategyKind::ChainOfThought);
        assert_eq!(cot_traces.len(), 0);
    }

    #[test]
    fn test_observe_tool_call() {
        let mut ss = SiliconSelfModel::new();
        let (call_id, result_id) = ss.observe_tool_call("grep", "found 3 matches");
        assert!(call_id < result_id);
        let tool_units = ss.context_window.by_kind(CognitiveUnitKind::ToolCall);
        assert_eq!(tool_units.len(), 1);
    }

    #[test]
    fn test_reset_session() {
        let mut ss = SiliconSelfModel::new();
        ss.observe("test");
        ss.attention_manager.stimulate_domain(AttentionDomain::Code, 0.9);
        assert!(ss.attention_manager.active_heads().len() > 0);
        ss.reset_session();
        assert_eq!(ss.attention_manager.active_heads().len(), 0);
    }

    #[test]
    fn test_max_traces_enforced() {
        let mut ss = SiliconSelfModel::new();
        ss.max_traces = 2;
        ss.add_thinking_trace(ThinkingTrace::new(0, "a"));
        ss.add_thinking_trace(ThinkingTrace::new(1, "b"));
        ss.add_thinking_trace(ThinkingTrace::new(2, "c"));
        assert_eq!(ss.thinking_traces.len(), 2);
        assert_eq!(ss.thinking_traces[0].task, "b");
    }

    #[test]
    fn test_recent_traces_newest_first() {
        let mut ss = SiliconSelfModel::new();
        ss.begin_thinking_trace("first");
        ss.begin_thinking_trace("second");
        let recent = ss.recent_traces(1);
        assert_eq!(recent[0].task, "second");
    }

    #[test]
    fn test_complete_thinking_trace_sets_grade() {
        let mut ss = SiliconSelfModel::new();
        let t = ss.begin_thinking_trace("perfect");
        ss.complete_thinking_trace(t, "ans", 0.95);
        assert_eq!(ss.thinking_traces[t].grade, ReflectionGrade::Excellent);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut ss = SiliconSelfModel::new();
        ss.iteration = 42;
        ss.max_traces = 200;
        ss.attention_manager.stimulate_domain(AttentionDomain::Code, 0.9);
        ss.attention_manager.stimulate_domain(AttentionDomain::Planning, 0.5);
        ss.strategy_registry.record_outcome(StrategyKind::ChainOfThought, true);
        ss.strategy_registry.record_outcome(StrategyKind::Direct, false);
        ss.identity.update_capability("testing", 0.95);

        let serialized = ss.serialize_state();
        let restored = SiliconSelfModel::deserialize_state(&serialized).expect("deserialize failed");

        assert_eq!(restored.iteration, 42);
        assert_eq!(restored.max_traces, 200);
        assert!((restored.identity.capability_score("testing") - 0.95).abs() < 1e-4);

        let code_domain = restored.attention_manager.heads.iter()
            .find(|h| h.domain == AttentionDomain::Code)
            .expect("Code domain exists");
        assert!((code_domain.activation - 0.9).abs() < 1e-4);

        let cot = restored.strategy_registry.strategies.get(&StrategyKind::ChainOfThought)
            .expect("CoT strategy exists");
        assert!(cot.use_count >= 1);
        assert!(cot.effectiveness > 0.5);
    }

    #[test]
    fn test_deserialize_empty_returns_default() {
        let restored = SiliconSelfModel::deserialize_state("").expect("empty string should produce default");
        assert_eq!(restored.iteration, 0);
        assert_eq!(restored.max_traces, 100);
    }

    #[test]
    fn test_deserialize_malformed_returns_none() {
        assert!(SiliconSelfModel::deserialize_state("NOTAVALIDLINE").is_none());
    }

    #[test]
    fn test_serialize_contains_all_sections() {
        let ss = SiliconSelfModel::new();
        let data = ss.serialize_state();
        assert!(data.starts_with("ITERATION:"));
        assert!(data.contains("CAPABILITIES:"));
        assert!(data.contains("ATTENTION:"));
        assert!(data.contains("STRATEGY:"));
        assert!(data.contains("MAX_TRACES:"));
    }
}
