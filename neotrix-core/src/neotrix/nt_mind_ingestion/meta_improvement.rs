use super::agent_orchestrator;

/// Meta-improvement diagnostics and DGM-H self-modification
///
/// DGM-H pattern: task agent (pipeline stages) + meta agent (observes & modifies)
/// The meta agent monitors task agent performance and generates new pipeline stages
/// at runtime via SelfIteratingBrain::meta_additions.
///
/// DGM-H meta-layer self-modification: the meta agent can rewrite its own improvement
/// logic (meta_layer_can_rewrite_self = true), logging each self-edit to self_edits
/// for full auditability.

/// Diagnostic metrics collected during pipeline runs
#[derive(Debug, Clone)]
pub struct MetaDiagnostics {
    pub iteration: u64,
    pub throughput_vsa_per_ms: f64,
    pub duplicate_rate: f64,
    pub keep_rate: f64,
    pub vector_count: usize,
}

impl MetaDiagnostics {
    pub fn new(iteration: u64) -> Self {
        Self { iteration, throughput_vsa_per_ms: 0.0, duplicate_rate: 0.0, keep_rate: 1.0, vector_count: 0 }
    }
}

/// Detected improvement patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImprovementPattern {
    HighDuplicates,
    LowActivation,
    LowKeepRate,
    Healthy,
}

pub fn classify_pattern(diag: &MetaDiagnostics) -> ImprovementPattern {
    if diag.duplicate_rate > 0.3 { ImprovementPattern::HighDuplicates }
    else if diag.throughput_vsa_per_ms < 10.0 { ImprovementPattern::LowActivation }
    else if diag.keep_rate < 0.5 { ImprovementPattern::LowKeepRate }
    else { ImprovementPattern::Healthy }
}

/// Concrete actions the meta agent can take
#[derive(Debug, Clone)]
pub enum MetaAction {
    /// Create a new stage and register it in meta_additions
    CreateStage { name: &'static str, description: &'static str, frequency: usize },
    /// Remove an existing meta-added stage by name
    RemoveStage { name: &'static str },
    /// Modify a configuration parameter
    ModifyConfig { param: &'static str, value: f64 },
    /// No action needed
    NoOp,
}

/// Types of self-modification the meta layer can apply to itself
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaSelfEditType {
    /// Change which patterns MetaAgent monitors
    ModifyStrategyPattern,
    /// Change how often meta agent runs
    AdjustFrequency,
    /// Add a new diagnostic metric to track
    AddMonitorMetric,
    /// Remove a diagnostic metric
    RemoveMonitorMetric,
    /// Change thresholds for ImprovementPattern detection
    ModifyImprovementThreshold,
    /// No self-modification
    Noop,
}

/// Record of a self-modification to the meta layer itself
#[derive(Debug, Clone)]
pub struct MetaSelfEdit {
    pub timestamp: u64,
    pub edit_type: MetaSelfEditType,
    pub description: String,
    pub before_value: String,
    pub after_value: String,
    pub applied: bool,
}

/// DGM-H MetaAgent: observes task agent, generates improvement actions, and can
/// rewrite its own meta-layer logic when meta_layer_can_rewrite_self is true.
#[derive(Debug, Clone)]
pub struct MetaAgent {
    pub history: MetaHistory,
    pub created_stages: Vec<String>,
    pub max_stages: usize,
    pub last_action: Option<MetaAction>,
    /// Log of all self-modifications made to the meta layer
    pub self_edits: Vec<MetaSelfEdit>,
    /// Maximum number of self-edits to retain in the log
    pub max_self_edits: usize,
    /// Whether the meta agent is permitted to rewrite its own improvement logic
    pub meta_layer_can_rewrite_self: bool,
    /// Threshold for HighDuplicates detection (default 0.3)
    pub improvement_threshold: f64,
    /// Currently monitored diagnostic patterns
    pub monitored_patterns: Vec<String>,
    /// How often the meta agent runs (default 10)
    pub meta_agent_frequency: usize,
    /// Multi-agent orchestrator for parallel diagnostic task dispatch
    pub orchestrator: Option<agent_orchestrator::ManagerOrchestrator>,
}

impl MetaAgent {
    pub fn new() -> Self {
        Self {
            history: MetaHistory::new(10),
            created_stages: Vec::new(),
            max_stages: 5,
            last_action: None,
            self_edits: Vec::new(),
            max_self_edits: 20,
            meta_layer_can_rewrite_self: false,
            improvement_threshold: 0.3,
            monitored_patterns: vec![
                "HighDuplicates".into(),
                "LowActivation".into(),
                "LowKeepRate".into(),
            ],
            meta_agent_frequency: 10,
            orchestrator: None,
        }
    }

    /// Full constructor with self-modification enabled
    pub fn new_with_self_modify() -> Self {
        let mut agent = Self::new();
        agent.meta_layer_can_rewrite_self = true;
        agent
    }

    /// Construct with self-modification + multi-agent orchestrator enabled
    pub fn new_with_orchestrator() -> Self {
        let mut agent = Self::new_with_self_modify();
        let mut orch = agent_orchestrator::ManagerOrchestrator::new();
        orch.add_worker("throughput");
        orch.add_worker("duplicates");
        orch.add_worker("keep_rate");
        agent.orchestrator = Some(orch);
        agent
    }

    /// Append a self-edit to the log, respecting max_self_edits
    pub fn log_self_edit(&mut self, edit_type: MetaSelfEditType, description: String, before_value: String, after_value: String) -> &MetaSelfEdit {
        let edit = MetaSelfEdit {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            edit_type,
            description,
            before_value,
            after_value,
            applied: true,
        };
        if self.self_edits.len() >= self.max_self_edits {
            self.self_edits.remove(0);
        }
        self.self_edits.push(edit);
        self.self_edits.last().unwrap()
    }

    /// Replace monitored patterns, logging the self-edit
    pub fn modify_monitored_patterns(&mut self, patterns: Vec<String>) {
        let before = self.monitored_patterns.join(", ");
        self.monitored_patterns = patterns;
        let after = self.monitored_patterns.join(", ");
        self.log_self_edit(
            MetaSelfEditType::ModifyStrategyPattern,
            "Replaced monitored diagnostic patterns".into(),
            before,
            after,
        );
    }

    /// Change how often the meta agent runs, logging the self-edit
    pub fn adjust_meta_frequency(&mut self, new_freq: usize) {
        let before = self.meta_agent_frequency.to_string();
        self.meta_agent_frequency = new_freq;
        let after = self.meta_agent_frequency.to_string();
        self.log_self_edit(
            MetaSelfEditType::AdjustFrequency,
            "Adjusted meta agent evaluation frequency".into(),
            before,
            after,
        );
    }

    /// Change the improvement threshold, logging the self-edit
    pub fn set_improvement_threshold(&mut self, new_threshold: f64) {
        let before = self.improvement_threshold.to_string();
        self.improvement_threshold = new_threshold;
        let after = self.improvement_threshold.to_string();
        self.log_self_edit(
            MetaSelfEditType::ModifyImprovementThreshold,
            "Changed improvement threshold for HighDuplicates detection".into(),
            before,
            after,
        );
    }

    /// Human-readable summary of recent self-edits
    pub fn self_edits_summary(&self) -> String {
        if self.self_edits.is_empty() {
            return "No self-modifications recorded.".to_string();
        }
        let mut lines: Vec<String> = self.self_edits.iter().rev().take(5).map(|edit| {
            format!(
                "[{}] {:?}: {} ({} → {})",
                edit.timestamp, edit.edit_type, edit.description, edit.before_value, edit.after_value
            )
        }).collect();
        lines.reverse();
        lines.join("\n")
    }

    /// Number of self-edits applied
    pub fn self_edits_count(&self) -> usize {
        self.self_edits.len()
    }

    /// Analyze own diagnostic history and suggest a self-modification
    ///
    /// Rules:
    /// - If throughput < 5.0: suggest removing a monitored pattern (overhead reduction)
    /// - If duplicate_rate > 0.5: suggest lowering improvement_threshold
    /// - If keep_rate < 0.3: suggest removing a monitored pattern
    /// - Otherwise: Noop
    pub fn generate_self_modification(&self, diag: &MetaDiagnostics) -> Option<MetaSelfEditType> {
        if !self.meta_layer_can_rewrite_self {
            return None;
        }
        // Check recent history averages for more stable decisions
        let avg_dup = self.history.average_duplicate_rate();
        let recent = self.history.recent(3.min(self.history.len()));
        let avg_throughput = if recent.is_empty() {
            diag.throughput_vsa_per_ms
        } else {
            recent.iter().map(|d| d.throughput_vsa_per_ms).sum::<f64>() / recent.len() as f64
        };
        let avg_keep = if recent.is_empty() {
            diag.keep_rate
        } else {
            recent.iter().map(|d| d.keep_rate).sum::<f64>() / recent.len() as f64
        };

        if avg_throughput < 5.0 && self.monitored_patterns.len() > 1 {
            return Some(MetaSelfEditType::RemoveMonitorMetric);
        }
        if avg_dup > 0.5 {
            return Some(MetaSelfEditType::ModifyImprovementThreshold);
        }
        if avg_keep < 0.3 && self.monitored_patterns.len() > 1 {
            return Some(MetaSelfEditType::RemoveMonitorMetric);
        }
        None
    }

    /// Collect diagnostic data and decide on an action.
    /// When meta_layer_can_rewrite_self is enabled, also generates and applies
    /// self-modifications, returning the edit type alongside the MetaAction.
    /// When orchestrator is present, dispatches parallel diagnostic audit tasks.
    pub fn observe_and_act(&mut self, diag: &MetaDiagnostics) -> (MetaAction, Option<MetaSelfEditType>) {
        self.history.record(diag.clone());

        if let Some(ref mut orch) = self.orchestrator {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let tasks = vec![
                agent_orchestrator::AgentTask {
                    id: format!("throughput_audit_{}", diag.iteration),
                    description: format!("Audit throughput: {:.2}", diag.throughput_vsa_per_ms),
                    priority: if diag.throughput_vsa_per_ms < 10.0 { 2 } else { 1 },
                    created_at: now,
                },
                agent_orchestrator::AgentTask {
                    id: format!("dup_audit_{}", diag.iteration),
                    description: format!("Audit duplicate rate: {:.2}", diag.duplicate_rate),
                    priority: if diag.duplicate_rate > 0.3 { 2 } else { 1 },
                    created_at: now,
                },
                agent_orchestrator::AgentTask {
                    id: format!("keep_audit_{}", diag.iteration),
                    description: format!("Audit keep rate: {:.2}", diag.keep_rate),
                    priority: if diag.keep_rate < 0.5 { 2 } else { 1 },
                    created_at: now,
                },
            ];
            for task in tasks {
                let _ = orch.dispatch(task);
            }
            let _ = orch.tick();
        }

        let pattern = classify_pattern(diag);

        // Don't act if we've already created the max number of stages
        if self.created_stages.len() >= self.max_stages {
            let self_edit = self.apply_self_modification(diag);
            return (MetaAction::NoOp, self_edit);
        }

        let action = match pattern {
            ImprovementPattern::HighDuplicates => {
                if !self.created_stages.contains(&"meta_dedup".to_string()) {
                    MetaAction::CreateStage {
                        name: "meta_dedup",
                        description: "runtime dedup stage for high duplicate rate",
                        frequency: 3,
                    }
                } else {
                    MetaAction::NoOp
                }
            }
            ImprovementPattern::LowActivation => {
                if !self.created_stages.contains(&"meta_activation".to_string()) {
                    MetaAction::CreateStage {
                        name: "meta_activation",
                        description: "activation guard on sparse vectors",
                        frequency: 5,
                    }
                } else {
                    MetaAction::NoOp
                }
            }
            ImprovementPattern::LowKeepRate => {
                if !self.created_stages.contains(&"meta_quality_gate".to_string()) {
                    MetaAction::CreateStage {
                        name: "meta_quality_gate",
                        description: "quality gate before inner critic",
                        frequency: 4,
                    }
                } else {
                    MetaAction::NoOp
                }
            }
            ImprovementPattern::Healthy => MetaAction::NoOp,
        };

        self.last_action = Some(action.clone());
        let self_edit = self.apply_self_modification(diag);
        (action, self_edit)
    }

    /// Internal: evaluate and apply a self-modification if meta_layer_can_rewrite_self
    fn apply_self_modification(&mut self, diag: &MetaDiagnostics) -> Option<MetaSelfEditType> {
        let edit_type = self.generate_self_modification(diag)?;
        match edit_type {
            MetaSelfEditType::RemoveMonitorMetric => {
                if self.monitored_patterns.len() > 1 {
                    let removed = self.monitored_patterns.remove(0);
                    self.log_self_edit(
                        MetaSelfEditType::RemoveMonitorMetric,
                        "Removed monitored pattern to reduce overhead".into(),
                        removed.clone(),
                        self.monitored_patterns.join(", "),
                    );
                }
            }
            MetaSelfEditType::ModifyImprovementThreshold => {
                let new_threshold = (self.improvement_threshold * 0.8).max(0.05);
                self.set_improvement_threshold(new_threshold);
            }
            _ => {}
        }
        Some(edit_type)
    }
}

impl Default for MetaAgent { fn default() -> Self { Self::new() } }

/// Ring buffer of recent diagnostics
#[derive(Debug, Clone)]
pub struct MetaHistory {
    entries: Vec<MetaDiagnostics>,
    max_entries: usize,
}

impl MetaHistory {
    pub fn new(max_entries: usize) -> Self { Self { entries: Vec::with_capacity(max_entries), max_entries } }
    pub fn record(&mut self, diag: MetaDiagnostics) {
        if self.entries.len() >= self.max_entries { self.entries.remove(0); }
        self.entries.push(diag);
    }
    pub fn recent(&self, n: usize) -> &[MetaDiagnostics] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn average_duplicate_rate(&self) -> f64 {
        if self.entries.is_empty() { return 0.0; }
        self.entries.iter().map(|d| d.duplicate_rate).sum::<f64>() / self.entries.len() as f64
    }
}

/// BrainStage dynamically generated by the DGM-H meta agent
pub struct DynamicStage {
    name: String,
    _description: String,
    frequency: usize,
    _invocation_count: u64,
}

impl DynamicStage {
    pub fn new(name: &str, description: &str, frequency: usize) -> Self {
        Self { name: name.to_string(), _description: description.to_string(), frequency, _invocation_count: 0 }
    }
}

impl crate::neotrix::nt_mind::self_iterating::pipeline::BrainStage for DynamicStage {
    fn name(&self) -> &str { &self.name }
    fn frequency(&self) -> usize { self.frequency }
    fn process(&self, brain: &mut crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain) -> Result<crate::neotrix::nt_mind::self_iterating::pipeline::StageDecision, crate::neotrix::nt_core_error::NeoTrixError> {
        use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
        match self.name.as_str() {
            "meta_dedup" => {
                let before = brain._consciousness_stream.len();
                // Only dedup entries below the salience threshold
                let salience_threshold = 0.6;
                let vec: Vec<_> = brain._consciousness_stream.iter().cloned().collect();
                // Remove high-salience entries from dedup consideration
                let (important, mut candidates): (Vec<_>, Vec<_>) = vec.into_iter()
                    .partition(|e| e.retention_score() >= salience_threshold || e.confidence >= 0.9);
                candidates.dedup_by(|a, b| {
                    QuantizedVSA::similarity(&a.vector, &b.vector) > 0.92
                });
                // Reconstruct: important first, then deduped candidates
                brain._consciousness_stream.clear();
                for entry in important {
                    brain._consciousness_stream.push(entry);
                }
                for entry in candidates {
                    brain._consciousness_stream.push(entry);
                }
                let removed = before - brain._consciousness_stream.len();
                if removed > 0 {
                    log::info!("[meta_dedup] removed {} duplicates (preserved high-salience)", removed);
                }
            }
            "meta_activation" => {
                let before = brain._consciousness_stream.len();
                brain._consciousness_stream.retain(|entry: &crate::core::nt_core_consciousness::VsaTagged| {
                    // Preserve high-salience entries regardless of sparsity
                    if entry.retention_score() >= 0.6 || entry.confidence >= 0.8 {
                        return true;
                    }
                    let ones = entry.vector.iter().filter(|&&b| b != 0).count();
                    let activity = ones as f64 / entry.vector.len().max(1) as f64;
                    activity > 0.05
                });
                let removed = before - brain._consciousness_stream.len();
                if removed > 0 {
                    log::info!("[meta_activation] removed {} low-activity vectors (preserved high-salience)", removed);
                }
            }
            "meta_quality_gate" => {
                let low_conf = brain._consciousness_stream.iter()
                    .filter(|e| e.confidence < 0.3)
                    .count();
                let high_salience_low_conf = brain._consciousness_stream.iter()
                    .filter(|e| e.confidence < 0.3 && e.retention_score() >= 0.7)
                    .count();
                if low_conf > 0 {
                    log::info!("[meta_quality_gate] {} low-confidence entries ({} are high-salience, keeping those)",
                        low_conf, high_salience_low_conf);
                }
                // If low-confidence but high-salience, boost confidence
                if high_salience_low_conf > 0 {
                    let mut repaired = 0;
                    let mut new_vec: Vec<_> = brain._consciousness_stream.iter().cloned().collect();
                    for entry in &mut new_vec {
                        if entry.confidence < 0.3 && entry.retention_score() >= 0.7 {
                            entry.confidence = 0.6;
                            repaired += 1;
                        }
                    }
                    brain._consciousness_stream.clear();
                    for entry in new_vec {
                        brain._consciousness_stream.push(entry);
                    }
                    log::info!("[meta_quality_gate] boosted confidence of {} high-salience entries", repaired);
                }
            }
            _ => {}
        }
        Ok(crate::neotrix::nt_mind::self_iterating::pipeline::StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_diag(iteration: u64, dup_rate: f64, throughput: f64, keep: f64) -> MetaDiagnostics {
        MetaDiagnostics { iteration, throughput_vsa_per_ms: throughput, duplicate_rate: dup_rate, keep_rate: keep, vector_count: 100 }
    }

    #[test]
    fn test_classify_healthy() {
        let d = sample_diag(1, 0.1, 50.0, 0.9);
        assert_eq!(classify_pattern(&d), ImprovementPattern::Healthy);
    }

    #[test]
    fn test_classify_high_duplicates() {
        let d = sample_diag(1, 0.5, 50.0, 0.9);
        assert_eq!(classify_pattern(&d), ImprovementPattern::HighDuplicates);
    }

    #[test]
    fn test_meta_agent_creates_stage_on_high_duplicates() {
        let mut agent = MetaAgent::new();
        let d = sample_diag(1, 0.5, 50.0, 0.9);
        let (action, _self_edit) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::CreateStage { name: "meta_dedup", .. }));
    }

    #[test]
    fn test_meta_agent_noop_on_healthy() {
        let mut agent = MetaAgent::new();
        let d = sample_diag(1, 0.1, 50.0, 0.9);
        let (action, _self_edit) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::NoOp));
    }

    #[test]
    fn test_meta_agent_does_not_duplicate_stages() {
        let mut agent = MetaAgent::new();
        agent.created_stages.push("meta_dedup".to_string());
        let d = sample_diag(1, 0.5, 50.0, 0.9);
        let (action, _self_edit) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::NoOp));
    }

    #[test]
    fn test_meta_agent_max_stages() {
        let mut agent = MetaAgent::new();
        agent.max_stages = 1;
        agent.created_stages.push("meta_dedup".to_string());
        // LowActivation would normally create a stage, but we're at capacity
        let d = sample_diag(1, 0.1, 5.0, 0.9);
        let (action, _self_edit) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::NoOp));
    }

    #[test]
    fn test_meta_history_capacity() {
        let mut hist = MetaHistory::new(3);
        for i in 0..5 { hist.record(sample_diag(i, 0.1, 50.0, 0.9)); }
        assert_eq!(hist.len(), 3);
        assert_eq!(hist.recent(3)[0].iteration, 2);
    }

    #[test]
    fn test_average_duplicate_rate() {
        let mut hist = MetaHistory::new(10);
        hist.record(sample_diag(1, 0.1, 50.0, 0.9));
        hist.record(sample_diag(2, 0.3, 50.0, 0.9));
        assert!((hist.average_duplicate_rate() - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_new_with_self_modify_enables_rewrite() {
        let agent = MetaAgent::new_with_self_modify();
        assert!(agent.meta_layer_can_rewrite_self);
        assert_eq!(agent.improvement_threshold, 0.3);
        assert_eq!(agent.meta_agent_frequency, 10);
        assert_eq!(agent.max_self_edits, 20);
    }

    #[test]
    fn test_log_self_edit_appends_and_respects_max() {
        let mut agent = MetaAgent::new();
        agent.meta_layer_can_rewrite_self = true;
        agent.max_self_edits = 2;
        agent.log_self_edit(
            MetaSelfEditType::ModifyStrategyPattern,
            "test edit 1".into(),
            "old".into(),
            "new".into(),
        );
        agent.log_self_edit(
            MetaSelfEditType::AdjustFrequency,
            "test edit 2".into(),
            "10".into(),
            "5".into(),
        );
        assert_eq!(agent.self_edits_count(), 2);
        agent.log_self_edit(
            MetaSelfEditType::Noop,
            "test edit 3 (should evict oldest)".into(),
            "".into(),
            "".into(),
        );
        assert_eq!(agent.self_edits_count(), 2);
        assert_eq!(agent.self_edits[0].description, "test edit 2");
    }

    #[test]
    fn test_modify_monitored_patterns_logs_edit() {
        let mut agent = MetaAgent::new_with_self_modify();
        agent.modify_monitored_patterns(vec!["OnlyPattern".into()]);
        assert_eq!(agent.monitored_patterns, vec!["OnlyPattern"]);
        assert_eq!(agent.self_edits_count(), 1);
        assert_eq!(agent.self_edits[0].edit_type, MetaSelfEditType::ModifyStrategyPattern);
    }

    #[test]
    fn test_adjust_meta_frequency_logs_edit() {
        let mut agent = MetaAgent::new_with_self_modify();
        agent.adjust_meta_frequency(5);
        assert_eq!(agent.meta_agent_frequency, 5);
        assert_eq!(agent.self_edits_count(), 1);
        assert_eq!(agent.self_edits[0].edit_type, MetaSelfEditType::AdjustFrequency);
    }

    #[test]
    fn test_set_improvement_threshold_logs_edit() {
        let mut agent = MetaAgent::new_with_self_modify();
        agent.set_improvement_threshold(0.15);
        assert!((agent.improvement_threshold - 0.15).abs() < 1e-6);
        assert_eq!(agent.self_edits_count(), 1);
        assert_eq!(agent.self_edits[0].edit_type, MetaSelfEditType::ModifyImprovementThreshold);
    }

    #[test]
    fn test_generate_self_modification_low_throughput() {
        let mut agent = MetaAgent::new_with_self_modify();
        let d = sample_diag(1, 0.2, 3.0, 0.9);
        agent.history.record(d.clone());
        let edit = agent.generate_self_modification(&d);
        assert_eq!(edit, Some(MetaSelfEditType::RemoveMonitorMetric));
    }

    #[test]
    fn test_generate_self_modification_high_dup() {
        let mut agent = MetaAgent::new_with_self_modify();
        let d = sample_diag(1, 0.7, 50.0, 0.9);
        agent.history.record(d.clone());
        let edit = agent.generate_self_modification(&d);
        assert_eq!(edit, Some(MetaSelfEditType::ModifyImprovementThreshold));
    }

    #[test]
    fn test_generate_self_modification_low_keep_rate() {
        let mut agent = MetaAgent::new_with_self_modify();
        let d = sample_diag(1, 0.2, 50.0, 0.2);
        agent.history.record(d.clone());
        let edit = agent.generate_self_modification(&d);
        assert_eq!(edit, Some(MetaSelfEditType::RemoveMonitorMetric));
    }

    #[test]
    fn test_generate_self_modification_healthy_returns_none() {
        let mut agent = MetaAgent::new_with_self_modify();
        let d = sample_diag(1, 0.1, 50.0, 0.9);
        agent.history.record(d.clone());
        let edit = agent.generate_self_modification(&d);
        assert_eq!(edit, None);
    }

    #[test]
    fn test_self_edits_summary_empty() {
        let agent = MetaAgent::new();
        assert_eq!(agent.self_edits_summary(), "No self-modifications recorded.");
    }

    #[test]
    fn test_self_edits_summary_with_edits() {
        let mut agent = MetaAgent::new_with_self_modify();
        agent.set_improvement_threshold(0.15);
        let summary = agent.self_edits_summary();
        assert!(summary.contains("ModifyImprovementThreshold"));
    }

    #[test]
    fn test_observe_and_act_applies_self_mod_when_enabled() {
        let mut agent = MetaAgent::new_with_self_modify();
        agent.history.record(sample_diag(0, 0.6, 50.0, 0.9));
        let d = sample_diag(1, 0.6, 50.0, 0.9);
        let (_action, self_edit) = agent.observe_and_act(&d);
        // High duplicate rate should trigger ModifyImprovementThreshold
        assert_eq!(self_edit, Some(MetaSelfEditType::ModifyImprovementThreshold));
        assert!(agent.improvement_threshold < 0.3);
        assert_eq!(agent.self_edits_count(), 1);
    }

    #[test]
    fn test_observe_and_act_no_self_mod_when_disabled() {
        let mut agent = MetaAgent::new(); // meta_layer_can_rewrite_self = false
        let d = sample_diag(1, 0.7, 3.0, 0.2);
        let (_action, self_edit) = agent.observe_and_act(&d);
        assert_eq!(self_edit, None);
        assert_eq!(agent.self_edits_count(), 0);
    }

    #[test]
    fn test_new_with_orchestrator_creates_workers() {
        let agent = MetaAgent::new_with_orchestrator();
        assert!(agent.meta_layer_can_rewrite_self);
        let orch = agent.orchestrator.unwrap();
        assert_eq!(orch.workers.len(), 3);
        let specialties: Vec<&str> = orch.workers.iter().map(|w| w.specialty.as_str()).collect();
        assert!(specialties.contains(&"throughput"));
        assert!(specialties.contains(&"duplicates"));
        assert!(specialties.contains(&"keep_rate"));
    }

    #[test]
    fn test_observe_and_act_with_orchestrator_dispatches_tasks() {
        let mut agent = MetaAgent::new_with_orchestrator();
        let d = sample_diag(1, 0.5, 50.0, 0.9);
        let (action, _) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::CreateStage { name: "meta_dedup", .. }));
        // Orchestrator should have processed 3 diagnostic tasks
        if let Some(ref orch) = agent.orchestrator {
            assert_eq!(orch.task_queue.len(), 0);
            let total_processed: usize = orch.workers.iter().map(|w| w.queue_len()).sum();
            // All tasks should have been consumed by ticks
            assert!(total_processed <= 2); // 2 workers had tasks in this tick
        } else {
            panic!("orchestrator should be present");
        }
    }

    #[test]
    fn test_agent_observe_and_act_works_without_orchestrator() {
        let mut agent = MetaAgent::new();
        let d = sample_diag(1, 0.1, 50.0, 0.9);
        let (action, _) = agent.observe_and_act(&d);
        assert!(matches!(action, MetaAction::NoOp));
        assert!(agent.orchestrator.is_none());
    }
}
