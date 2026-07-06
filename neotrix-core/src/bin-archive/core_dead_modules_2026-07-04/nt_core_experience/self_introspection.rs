use std::collections::VecDeque;

/// Defect patterns that the introspection engine can detect at runtime.
/// Each variant maps to a known cognitive defect distilled from experience.
#[derive(Debug, Clone, PartialEq)]
pub enum DefectPattern {
    OverDiagnosis { handler: String, snapshot_count: usize },
    AccumulationWithoutPruning { component: String, size: usize },
    MissingPreconditionCheck { operation: String },
    PlatformBlindness { attempted: String, actual: String },
    RedundantSnapshotStorage { component: String, entries: usize },
    ExcessiveProbing { pattern: String, count: usize },
    LockBeforeGc { operation: String },
    DistillThenDelete { component: String },
    AuditBeforeAct { component: String, missed_calls: usize },
}

/// A corrective action generated from defect analysis.
#[derive(Debug, Clone)]
pub struct CorrectiveAction {
    pub pattern: DefectPattern,
    pub suggestion: String,
    pub priority: u8,
    pub detected_at_cycle: u64,
}

/// A point-in-time diagnostic observation of consciousness-internal state.
#[derive(Debug, Clone)]
pub struct DiagnosticSnapshot {
    pub cycle: u64,
    pub active_handler_count: usize,
    pub pending_actions: usize,
    pub component_sizes: Vec<(String, usize)>,
    pub handler_frequencies: Vec<(String, usize)>,
}

/// Runtime introspection engine — maps system resource patterns to cognitive defects.
///
/// Mirrors the AGENTS.md introspection session at runtime:
/// - Over-diagnosis (opencode.db event table storing full snapshots) → handler polling too often
/// - Only-accumulate-never-prune (target/incremental/) → component sizes growing without limit
/// - Platform-blindness (ps syntax) → failing to pre-check environment
/// - Lock-before-gc → running destructive ops without lock check
/// - Distill-then-delete → deleting data without first extracting insight
pub struct IntrospectionEngine {
    history: VecDeque<DiagnosticSnapshot>,
    actions: Vec<CorrectiveAction>,
    max_history: usize,
    cycle: u64,
}

impl Default for IntrospectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IntrospectionEngine {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(100),
            actions: Vec::new(),
            max_history: 100,
            cycle: 0,
        }
    }

    /// Process a new snapshot and return any newly detected corrective actions.
    pub fn tick(&mut self, snapshot: DiagnosticSnapshot) -> Vec<CorrectiveAction> {
        self.cycle += 1;
        self.history.push_back(snapshot);
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        let new_actions = self.analyze();
        self.actions.extend(new_actions.clone());
        new_actions
    }

    fn analyze(&self) -> Vec<CorrectiveAction> {
        let mut new_actions = Vec::new();
        if self.history.len() < 3 {
            return new_actions;
        }

        let latest = match self.history.back() {
            Some(l) => l,
            None => return new_actions,
        };

        // Pattern 1: Over-diagnosis — same handler appears in top frequencies every tick
        if !latest.handler_frequencies.is_empty() {
            let top = &latest.handler_frequencies[0];
            let appearance_rate = self.count_handler_appearances(&top.0);
            if appearance_rate as f64 > self.history.len() as f64 * 0.6 {
                new_actions.push(CorrectiveAction {
                    pattern: DefectPattern::OverDiagnosis {
                        handler: top.0.clone(),
                        snapshot_count: appearance_rate,
                    },
                    suggestion: format!(
                        "Handler '{}' polled in {} of {} snapshots. Cache results, act on deltas.",
                        top.0, appearance_rate, self.history.len()
                    ),
                    priority: 100,
                    detected_at_cycle: self.cycle,
                });
            }
        }

        // Pattern 2: Accumulation without pruning — monotonic growth across recent ticks
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
                if sizes.len() >= 3
                    && sizes.windows(2).all(|w| w[1] > w[0])
                    && sizes.last().zip(sizes.first()).map(|(l, f)| l - f).unwrap_or(0) > 1000
                {
                    let last_size = *sizes.last().unwrap_or(&0);
                    let first_size = *sizes.first().unwrap_or(&0);
                    new_actions.push(CorrectiveAction {
                        pattern: DefectPattern::AccumulationWithoutPruning {
                            component: comp_name.clone(),
                            size: last_size,
                        },
                        suggestion: format!(
                            "'{}' grew from {} to {} in 5 ticks. Trigger GC/prune.",
                            comp_name,
                            first_size,
                            last_size,
                        ),
                        priority: 150,
                        detected_at_cycle: self.cycle,
                    });
                }
            }
        }

        // Pattern 3: Redundant snapshot storage — same component size unchanged across many ticks
        if self.history.len() >= 5 {
            let recent: Vec<&DiagnosticSnapshot> = self.history.iter().rev().take(5).collect();
            for (comp_name, size) in &latest.component_sizes {
                let all_same = recent
                    .iter()
                    .all(|s| s.component_sizes.iter().any(|(n, sz)| n == comp_name && sz == size));
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
                    });
                }
            }
        }

        // Pattern 4: Excessive probing — too many active handlers with low pending actions
        if latest.active_handler_count > 10 && latest.pending_actions < 3 {
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

    pub fn actions(&self) -> &[CorrectiveAction] {
        &self.actions
    }

    pub fn drain_actions(&mut self) -> Vec<CorrectiveAction> {
        self.actions.drain(..).collect()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn report(&self) -> String {
        format!(
            "Introspection: {} snapshots, {} open actions, {} patterns active",
            self.history.len(),
            self.actions.len(),
            self.active_pattern_count()
        )
    }

    fn active_pattern_count(&self) -> usize {
        self.actions.len()
    }
}
