use std::collections::VecDeque;

/// Defect patterns that the introspection engine can detect at runtime.
/// Each variant maps to a known cognitive defect distilled from experience.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DefectPattern {
    OverDiagnosis {
        handler: String,
        snapshot_count: usize,
    },
    AccumulationWithoutPruning {
        component: String,
        size: usize,
    },
    MissingPreconditionCheck {
        operation: String,
    },
    PlatformBlindness {
        attempted: String,
        actual: String,
    },
    RedundantSnapshotStorage {
        component: String,
        entries: usize,
    },
    ExcessiveProbing {
        pattern: String,
        count: usize,
    },
    LockBeforeGc {
        operation: String,
    },
    DistillThenDelete {
        component: String,
    },
    AuditBeforeAct {
        component: String,
        missed_calls: usize,
    },
    // ── Meta patterns (detected by meta_audit) ──
    SelfOverDiagnosis {
        pattern_name: String,
        occurrence_count: usize,
    },
    ActionAccumulation {
        unexecuted_count: usize,
    },
    IntrospectionStagnation {
        idle_cycles: usize,
    },
    FalsePositiveBias {
        false_positive_rate: f64,
    },
    // ── Confirmation patterns (detected by behavior confirmation loop) ──
    CorrectionFailed {
        pattern_name: String,
        original_priority: u8,
    },
}

/// A corrective action generated from defect analysis.
#[derive(Debug, Clone)]
pub struct CorrectiveAction {
    pub pattern: DefectPattern,
    pub suggestion: String,
    pub priority: u8,
    pub detected_at_cycle: u64,
    pub executed: bool,
}

impl CorrectiveAction {
    pub fn description(&self) -> String {
        format!(
            "[{}] {} — {}",
            self.priority,
            self.suggestion,
            if self.executed { "executed" } else { "pending" }
        )
    }
}

/// A distilled experience node — automatically generated from runtime pattern detection.
/// Written to AGENTS.md (or other persistent store) for cross-session learning.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistilledExperience {
    pub title: String,
    pub rule: String,
    pub pattern_name: String,
    pub confidence: f64,
    pub generated_at_cycle: u64,
}

/// A point-in-time diagnostic observation of consciousness-internal state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSnapshot {
    pub cycle: u64,
    pub active_handler_count: usize,
    pub pending_actions: usize,
    pub component_sizes: Vec<(String, usize)>,
    pub handler_frequencies: Vec<(String, usize)>,
}

/// Introspection engine state snapshot for NTSSEG persistence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntrospectionState {
    pub cycle: u64,
    pub max_history: usize,
    pub action_count: usize,
    pub executed_action_count: usize,
    pub distilled_count: usize,
}

/// Tunable detection parameters. The engine self-adjusts these via
/// experience-driven calibration (Loop 5).
#[derive(Debug, Clone)]
pub struct DetectionThresholds {
    pub overdiagnosis_rate: f64,
    pub accumulation_growth: usize,
    pub excessive_probing_handlers: usize,
    pub stagnation_idle: u32,
    pub false_positive_exec_rate: f64,
    pub min_exec_priority: u8,
    pub self_overdiagnosis_count: usize,
    pub action_accumulation_threshold: usize,
    pub confirmation_window: u64,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            overdiagnosis_rate: 0.6,
            accumulation_growth: 1000,
            excessive_probing_handlers: 10,
            stagnation_idle: 10,
            false_positive_exec_rate: 0.2,
            min_exec_priority: 120,
            self_overdiagnosis_count: 3,
            action_accumulation_threshold: 20,
            confirmation_window: 30,
        }
    }
}

/// Tracks a correction that was executed, pending confirmation of its effectiveness.
#[derive(Debug, Clone)]
struct ConfirmationEntry {
    detected_at_cycle: u64,
    pattern_debug: String,
    original_priority: u8,
    suggestion: String,
}

/// Runtime introspection engine — maps system resource patterns to cognitive defects.
///
/// Nested loops:
///   Loop 1 (analyze):  detect defects from snapshots → queue corrective actions
///   Loop 2 (meta_audit): detect defects in own detection patterns
///   Loop 3 (distill):   novel patterns → DistilledExperience nodes
///   Loop 4 (confirm):   verify whether executed corrections actually fixed the root cause
///   Loop 5 (calibrate): use distilled experiences to tune detection thresholds
pub struct IntrospectionEngine {
    history: VecDeque<DiagnosticSnapshot>,
    actions: Vec<CorrectiveAction>,
    distilled: Vec<DistilledExperience>,
    max_history: usize,
    cycle: u64,
    recent_patterns: VecDeque<String>,
    idle_counter: u32,
    total_actions_generated: u64,
    total_actions_executed: u64,
    /// Behavior confirmation state (Loop 4)
    confirmation_queue: VecDeque<ConfirmationEntry>,
    confirmed_successes: u64,
    confirmed_failures: u64,
    /// Self-calibration thresholds (Loop 5)
    pub thresholds: DetectionThresholds,
}

impl IntrospectionEngine {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(100),
            actions: Vec::new(),
            distilled: Vec::new(),
            max_history: 100,
            cycle: 0,
            recent_patterns: VecDeque::with_capacity(20),
            idle_counter: 0,
            total_actions_generated: 0,
            total_actions_executed: 0,
            confirmation_queue: VecDeque::with_capacity(50),
            confirmed_successes: 0,
            confirmed_failures: 0,
            thresholds: DetectionThresholds::default(),
        }
    }

    /// Process a new snapshot and return any newly detected corrective actions.
    ///
    /// This is the main entry point that orchestrates all 5 loops:
    ///   1. analyze()     — first-order defect detection
    ///   2. meta_audit()  — second-order introspection
    ///   3. auto_distill() called externally — experience generation
    ///   4. check_confirmations() — verify past corrections
    ///   5. apply_experience_calibration() — tune thresholds
    pub fn tick(&mut self, snapshot: DiagnosticSnapshot) -> Vec<CorrectiveAction> {
        self.cycle += 1;
        self.history.push_back(snapshot);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // ── Loop 4: Check behavior confirmations before new analysis ──
        let confirmation_actions = self.check_confirmations();

        // ── Loop 1: Analyze ──
        let new_actions = self.analyze();
        if new_actions.is_empty() {
            self.idle_counter += 1;
        } else {
            self.idle_counter = 0;
        }
        self.total_actions_generated += new_actions.len() as u64;

        // Track recent pattern names for meta-audit dedup
        for a in new_actions.iter().chain(confirmation_actions.iter()) {
            let name = format!("{:?}", a.pattern);
            if !self.recent_patterns.iter().any(|p| p == &name) {
                if self.recent_patterns.len() >= 20 {
                    self.recent_patterns.pop_front();
                }
                self.recent_patterns.push_back(name);
            }
        }

        // ── Loop 2: Meta-audit ──
        let meta_actions = self.meta_audit();
        for a in &meta_actions {
            let name = format!("{:?}", a.pattern);
            if !self.recent_patterns.iter().any(|p| p == &name) {
                self.recent_patterns.push_back(name);
            }
        }

        // Merge all actions
        let mut all_new =
            Vec::with_capacity(new_actions.len() + meta_actions.len() + confirmation_actions.len());
        all_new.extend(new_actions);
        all_new.extend(meta_actions);
        all_new.extend(confirmation_actions);

        self.actions.extend(all_new.clone());
        all_new
    }

    // ── Loop 2: Meta-audit — the engine introspects its own detection patterns ──

    fn meta_audit(&self) -> Vec<CorrectiveAction> {
        let mut meta = Vec::new();
        let t = &self.thresholds;

        // Meta 1: SelfOverDiagnosis — same pattern name appears N+ times
        let pattern_counts: std::collections::HashMap<&str, usize> = self
            .recent_patterns
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, p| {
                *acc.entry(p.as_str()).or_insert(0) += 1;
                acc
            });
        for (name, count) in &pattern_counts {
            if *count >= t.self_overdiagnosis_count {
                meta.push(CorrectiveAction {
                    pattern: DefectPattern::SelfOverDiagnosis {
                        pattern_name: name.to_string(),
                        occurrence_count: *count,
                    },
                    suggestion: format!(
                        "Pattern '{}' triggered {} times — engine may be over-sensitive. Consider raising threshold.",
                        name, count
                    ),
                    priority: 130,
                    detected_at_cycle: self.cycle,
                    executed: false,
                });
            }
        }

        // Meta 2: ActionAccumulation
        let executed_count = self.actions.iter().filter(|a| a.executed).count();
        if executed_count > t.action_accumulation_threshold {
            meta.push(CorrectiveAction {
                pattern: DefectPattern::ActionAccumulation {
                    unexecuted_count: self.actions.len(),
                },
                suggestion: format!(
                    "{} executed actions accumulated — call drain_actions() to clean up.",
                    executed_count
                ),
                priority: 140,
                detected_at_cycle: self.cycle,
                executed: false,
            });
        }

        // Meta 3: Stagnation
        if self.idle_counter >= t.stagnation_idle && self.cycle > 30 {
            meta.push(CorrectiveAction {
                pattern: DefectPattern::IntrospectionStagnation {
                    idle_cycles: self.idle_counter as usize,
                },
                suggestion: format!(
                    "No defects detected in {} cycles — engine may need recalibration or system is genuinely healthy.",
                    self.idle_counter
                ),
                priority: 60,
                detected_at_cycle: self.cycle,
                executed: false,
            });
        }

        // Meta 4: FalsePositiveBias
        if self.total_actions_generated > 20 {
            let exec_rate =
                self.total_actions_executed as f64 / self.total_actions_generated as f64;
            if exec_rate < t.false_positive_exec_rate {
                meta.push(CorrectiveAction {
                    pattern: DefectPattern::FalsePositiveBias {
                        false_positive_rate: 1.0 - exec_rate,
                    },
                    suggestion: format!(
                        "Only {:.0}% of actions executed ({}/{}). Lower priority thresholds may be too aggressive.",
                        exec_rate * 100.0, self.total_actions_executed, self.total_actions_generated
                    ),
                    priority: 100,
                    detected_at_cycle: self.cycle,
                    executed: false,
                });
            }
        }

        meta
    }

    // ── Loop 3: Auto-distillation — novel patterns → DistilledExperience ──

    pub fn auto_distill(&mut self) -> Vec<DistilledExperience> {
        if self.cycle < 10 {
            return Vec::new();
        }

        let mut new_distilled = Vec::new();
        let known_titles: std::collections::HashSet<&str> =
            self.distilled.iter().map(|d| d.title.as_str()).collect();

        for action in self.actions.iter().filter(|a| !a.executed) {
            let title = match action.pattern {
                DefectPattern::SelfOverDiagnosis { .. } => "Recursive Over-Diagnosis Guard",
                DefectPattern::ActionAccumulation { .. } => "Action Buffer Drain Discipline",
                DefectPattern::IntrospectionStagnation { .. } => "Stagnation Recovery Trigger",
                DefectPattern::FalsePositiveBias { .. } => "False Positive Rate Gate",
                DefectPattern::OverDiagnosis { .. } => "Handler Polling Frequency Limit",
                DefectPattern::AccumulationWithoutPruning { .. } => "Component GC Threshold",
                DefectPattern::ExcessiveProbing { .. } => "Handler Population Cap",
                DefectPattern::CorrectionFailed { .. } => "Correction Escalation Trigger",
                _ => continue,
            };

            if known_titles.contains(title) {
                continue;
            }

            new_distilled.push(DistilledExperience {
                title: title.to_string(),
                rule: action.suggestion.clone(),
                pattern_name: format!("{:?}", action.pattern),
                confidence: 0.3,
                generated_at_cycle: self.cycle,
            });
        }

        self.distilled.extend(new_distilled.clone());

        // ── Loop 5: Apply experience-driven calibration after each distill ──
        self.apply_experience_calibration();

        new_distilled
    }

    pub fn distilled_experiences(&self) -> &[DistilledExperience] {
        &self.distilled
    }

    // ── Loop 4: Behavior Confirmation ──

    /// Check if previously corrected patterns have reappeared.
    /// If a pattern is confirmed as fixed (no recurrence within window), it is removed.
    /// If a pattern recurs, an escalated CorrectionFailed action is generated.
    fn check_confirmations(&mut self) -> Vec<CorrectiveAction> {
        let mut escalated = Vec::new();
        let limit = self.thresholds.confirmation_window;
        let mut keep = VecDeque::new();

        while let Some(entry) = self.confirmation_queue.pop_front() {
            let age = self.cycle.saturating_sub(entry.detected_at_cycle);
            if age < limit {
                // Not yet due for confirmation — check if pattern recently appeared
                let reappeared = self
                    .actions
                    .iter()
                    .any(|a| format!("{:?}", a.pattern) == entry.pattern_debug);
                if reappeared {
                    // Pattern recurred before confirmation window — escalation
                    self.confirmed_failures += 1;
                    escalated.push(CorrectiveAction {
                        pattern: DefectPattern::CorrectionFailed {
                            pattern_name: entry.pattern_debug.clone(),
                            original_priority: entry.original_priority,
                        },
                        suggestion: format!(
                            "Correction for '{}' failed (recurred within {} cycles). Escalating.",
                            entry.suggestion, age
                        ),
                        priority: entry.original_priority.saturating_add(20).min(255),
                        detected_at_cycle: self.cycle,
                        executed: false,
                    });
                } else {
                    keep.push_back(entry);
                }
            } else {
                // Window expired and pattern did not reappear — confirmed success
                self.confirmed_successes += 1;
            }
        }

        self.confirmation_queue = keep;
        escalated
    }

    /// Register a corrected pattern for future confirmation checking.
    fn push_confirmation(&mut self, executed_action: &CorrectiveAction) {
        let pattern_debug = format!("{:?}", executed_action.pattern);
        // Avoid duplicate queue entries for the same pattern
        if !self
            .confirmation_queue
            .iter()
            .any(|e| e.pattern_debug == pattern_debug)
        {
            if self.confirmation_queue.len() >= 50 {
                self.confirmation_queue.pop_front();
            }
            self.confirmation_queue.push_back(ConfirmationEntry {
                detected_at_cycle: executed_action.detected_at_cycle,
                pattern_debug,
                original_priority: executed_action.priority,
                suggestion: executed_action.suggestion.clone(),
            });
        }
    }

    pub fn confirmation_stats(&self) -> (u64, u64) {
        (self.confirmed_successes, self.confirmed_failures)
    }

    // ── Loop 5: Experience-Driven Self-Calibration ──

    /// Tune detection thresholds based on auto-distilled experiences.
    fn apply_experience_calibration(&mut self) {
        for exp in &self.distilled {
            match exp.title.as_str() {
                "False Positive Rate Gate" => {
                    // Few actions being executed → raise min priority
                    self.thresholds.min_exec_priority =
                        self.thresholds.min_exec_priority.saturating_add(5).min(200);
                }
                "Handler Polling Frequency Limit" => {
                    // Over-diagnosis detected → lower sensitivity
                    self.thresholds.overdiagnosis_rate =
                        (self.thresholds.overdiagnosis_rate + 0.05).min(1.0);
                }
                "Component GC Threshold" => {
                    // Accumulation detected → lower growth trigger
                    self.thresholds.accumulation_growth = self
                        .thresholds
                        .accumulation_growth
                        .saturating_sub(100)
                        .max(100);
                }
                "Recursive Over-Diagnosis Guard" => {
                    // Self-diagnosing too often → raise threshold
                    self.thresholds.self_overdiagnosis_count = self
                        .thresholds
                        .self_overdiagnosis_count
                        .saturating_add(1)
                        .min(10);
                }
                "Handler Population Cap" => {
                    // Too many handlers → raise threshold (need more evidence to trigger)
                    self.thresholds.excessive_probing_handlers = self
                        .thresholds
                        .excessive_probing_handlers
                        .saturating_add(2)
                        .min(50);
                }
                _ => {}
            }
        }
    }

    /// Read the current threshold values (for external display).
    pub fn thresholds(&self) -> &DetectionThresholds {
        &self.thresholds
    }

    // ── NTSSEG Persistence ──

    pub fn capture_state(&self) -> IntrospectionState {
        IntrospectionState {
            cycle: self.cycle,
            max_history: self.max_history,
            action_count: self.actions.len(),
            executed_action_count: self.actions.iter().filter(|a| a.executed).count(),
            distilled_count: self.distilled.len(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&self.capture_state()).map_err(|e| e.to_string())
    }

    pub fn load_state(&mut self, bytes: &[u8]) -> Result<(), String> {
        let state: IntrospectionState = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        self.cycle = state.cycle;
        self.max_history = state.max_history;
        self.total_actions_generated = state.action_count as u64;
        self.total_actions_executed = state.executed_action_count as u64;
        Ok(())
    }

    // ── Loop 1: First-order analysis ──

    fn analyze(&self) -> Vec<CorrectiveAction> {
        let mut new_actions = Vec::new();
        if self.history.len() < 3 {
            return new_actions;
        }

        let latest = self
            .history
            .back()
            .expect("history.len() >= 3 per earlier guard");
        let t = &self.thresholds;

        // Pattern 1: Over-diagnosis
        if !latest.handler_frequencies.is_empty() {
            let top = &latest.handler_frequencies[0];
            let appearance_rate = self.count_handler_appearances(&top.0);
            if appearance_rate as f64 > self.history.len() as f64 * t.overdiagnosis_rate {
                new_actions.push(CorrectiveAction {
                    pattern: DefectPattern::OverDiagnosis {
                        handler: top.0.clone(),
                        snapshot_count: appearance_rate,
                    },
                    suggestion: format!(
                        "Handler '{}' polled in {} of {} snapshots. Cache results, act on deltas.",
                        top.0,
                        appearance_rate,
                        self.history.len()
                    ),
                    priority: 100,
                    detected_at_cycle: self.cycle,
                    executed: false,
                });
            }
        }

        // Pattern 2: Accumulation without pruning
        if self.history.len() >= 5 {
            let recent: Vec<&DiagnosticSnapshot> = self.history.iter().rev().take(5).collect();
            for (comp_name, _) in &recent[0].component_sizes {
                let sizes: Vec<usize> = recent
                    .iter()
                    .filter_map(|s| {
                        s.component_sizes
                            .iter()
                            .find(|(n, _)| n == comp_name)
                            .map(|(_, sz)| *sz)
                    })
                    .collect();
                if sizes.len() >= 3 && sizes.windows(2).all(|w| w[1] > w[0]) && {
                    let first = sizes.first().copied().unwrap_or(0);
                    let last = sizes.last().copied().unwrap_or(0);
                    last - first > t.accumulation_growth
                } {
                    let size = *sizes.last().unwrap_or(&0);
                    let first_sz = sizes.first().copied().unwrap_or(0);
                    let last_sz = sizes.last().copied().unwrap_or(0);
                    new_actions.push(CorrectiveAction {
                        pattern: DefectPattern::AccumulationWithoutPruning {
                            component: comp_name.clone(),
                            size,
                        },
                        suggestion: format!(
                            "'{}' grew from {} to {} in 5 ticks. Trigger GC/prune.",
                            comp_name, first_sz, last_sz
                        ),
                        priority: 150,
                        detected_at_cycle: self.cycle,
                        executed: false,
                    });
                }
            }
        }

        // Pattern 3: Redundant snapshot storage
        if self.history.len() >= 5 {
            let recent: Vec<&DiagnosticSnapshot> = self.history.iter().rev().take(5).collect();
            for (comp_name, size) in &latest.component_sizes {
                let all_same = recent.iter().all(|s| {
                    s.component_sizes
                        .iter()
                        .any(|(n, sz)| n == comp_name && sz == size)
                });
                if all_same && *size > 100 {
                    new_actions.push(CorrectiveAction {
                        pattern: DefectPattern::RedundantSnapshotStorage {
                            component: comp_name.clone(),
                            entries: *size,
                        },
                        suggestion: format!(
                            "'{}' unchanged at {} across 5 ticks. Skip snapshotting until delta detected.",
                            comp_name, size
                        ),
                        priority: 80,
                        detected_at_cycle: self.cycle,
                        executed: false,
                    });
                }
            }
        }

        // Pattern 4: Excessive probing
        if latest.active_handler_count > t.excessive_probing_handlers && latest.pending_actions < 3
        {
            new_actions.push(CorrectiveAction {
                pattern: DefectPattern::ExcessiveProbing {
                    pattern: "many_handlers_few_actions".to_string(),
                    count: latest.active_handler_count,
                },
                suggestion: format!(
                    "{} handlers active but only {} pending actions. Reduce polling, batch dispatch.",
                    latest.active_handler_count, latest.pending_actions
                ),
                priority: 120,
                detected_at_cycle: self.cycle,
                executed: false,
            });
        }

        new_actions
    }

    fn count_handler_appearances(&self, name: &str) -> usize {
        self.history
            .iter()
            .filter(|s| s.handler_frequencies.iter().any(|(n, _)| n == name))
            .count()
    }

    /// Three-pass self-dialectic (Hermes-inspired): audit → reconcile → write.
    ///
    /// **Pass 1 — Audit**: reviews all pending actions, identifies patterns of
    ///   corrections (high-priority actions) vs approvals (low-priority that resolved),
    ///   and flags repeat offenses (same pattern appearing in multiple actions).
    ///
    /// **Pass 2 — Reconcile**: cross-checks audit findings against existing
    ///   DistilledExperience nodes. If a pending action matches an existing experience
    ///   pattern, promotes that experience (updates confidence). If no match,
    ///   prepares a new candidate.
    ///
    /// **Pass 3 — Write**: generates up to 3 new DistilledExperience entries from
    ///   unresolved audit findings that don't already exist.
    pub fn run_self_dialectic(&mut self) -> Vec<DistilledExperience> {
        let mut new_experiences = Vec::new();

        // ── Pass 1: Audit — categorize pending actions ──
        let pending: Vec<&CorrectiveAction> = self.actions.iter().filter(|a| !a.executed).collect();
        let mut correction_patterns: Vec<String> = Vec::new();
        let mut repeat_offenses: Vec<&CorrectiveAction> = Vec::new();

        for action in &pending {
            let pat_name = format!("{:?}", action.pattern);
            if correction_patterns.contains(&pat_name) {
                repeat_offenses.push(action);
            } else {
                correction_patterns.push(pat_name);
            }
        }

        // ── Pass 2: Reconcile — cross-check with existing experiences ──
        for action in &repeat_offenses {
            let pat_name = format!("{:?}", action.pattern);
            // Check if this pattern has a distilled experience
            let existing = self
                .distilled
                .iter_mut()
                .find(|d| d.pattern_name == pat_name);
            if let Some(exp) = existing {
                // Promote: repeat offense means pattern is important → raise confidence
                exp.confidence = (exp.confidence + 0.1).min(1.0);
            } else {
                // New pattern: prepare candidate experience
                let title = format!("recurring:{}", &pat_name[..pat_name.len().min(60)]);
                if !self.distilled.iter().any(|d| d.title == title) {
                    new_experiences.push(DistilledExperience {
                        title,
                        rule: format!("Detected recurring pattern: {}", action.suggestion),
                        pattern_name: pat_name.clone(),
                        confidence: 0.5,
                        generated_at_cycle: self.cycle,
                    });
                }
            }
        }

        // ── Pass 3: Write — add new experiences (max 3 per cycle) ──
        for exp in new_experiences.iter().take(3) {
            if !self.distilled.iter().any(|d| d.title == exp.title) {
                self.distilled.push(exp.clone());
            }
        }

        new_experiences.truncate(3);
        new_experiences
    }

    pub fn actions(&self) -> &[CorrectiveAction] {
        &self.actions
    }

    pub fn drain_actions(&mut self) -> Vec<CorrectiveAction> {
        let drained = self.actions.drain(..).collect::<Vec<_>>();
        self.total_actions_executed += drained.iter().filter(|a| a.executed).count() as u64;
        drained
    }

    /// Mark high-priority actions as executed, and register them for behavior confirmation.
    pub fn mark_executed(&mut self) {
        let to_confirm: Vec<CorrectiveAction> = self
            .actions
            .iter()
            .filter(|a| a.priority >= self.thresholds.min_exec_priority && !a.executed)
            .cloned()
            .collect();

        for a in &self.actions {
            if a.priority >= self.thresholds.min_exec_priority {
                // executed will be set below
            }
        }

        for a in &mut self.actions {
            if a.priority >= self.thresholds.min_exec_priority {
                a.executed = true;
            }
        }

        for action in &to_confirm {
            self.push_confirmation(action);
        }
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn report(&self) -> String {
        let meta_count = self
            .actions
            .iter()
            .filter(|a| {
                matches!(
                    a.pattern,
                    DefectPattern::SelfOverDiagnosis { .. }
                        | DefectPattern::ActionAccumulation { .. }
                        | DefectPattern::IntrospectionStagnation { .. }
                        | DefectPattern::FalsePositiveBias { .. }
                )
            })
            .count();

        format!(
            "Introspect: {} scope={} meta={} distill={} exec_ratio={:.0}% idle={} confirm={}ok/{}fail",
            self.history.len(),
            self.actions.len(),
            meta_count,
            self.distilled.len(),
            if self.total_actions_generated > 0 {
                self.total_actions_executed as f64 / self.total_actions_generated as f64 * 100.0
            } else {
                0.0
            },
            self.idle_counter,
            self.confirmed_successes,
            self.confirmed_failures,
        )
    }
}
