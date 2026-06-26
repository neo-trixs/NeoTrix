use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A recorded step in the reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_id: u64,
    pub handler_name: String,
    pub input_summary: String,
    pub output_summary: String,
    pub duration_ms: u64,
    pub confidence: f64,
    pub error_flag: bool,
    pub cycle: u64,
}

/// Meta-cognitive observation about a reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaObservation {
    pub observation_type: ObsType,
    pub handler: String,
    pub severity: f64,
    pub detail: String,
    pub cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ObsType {
    HighConfidence,
    LowConfidence,
    ErrorSpike,
    RepeatedFailure,
    CognitiveFatigue,
    StrategyShift,
    ConvergenceSlow,
    ResourceWaste,
}

/// MERA-style meta-cognitive monitor
pub struct MetaCogMonitor {
    pub trace: VecDeque<ReasoningStep>,
    pub observations: Vec<MetaObservation>,
    pub meta_state: MetaState,
    pub max_trace_len: usize,
    pub error_threshold: f64,
    pub fatigue_window: usize,
    pub plan: Option<MetaPlan>,
    pub step_counter: u64,
    cycle: u64,
}

/// Current meta-state of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaState {
    pub current_strategy: String,
    pub cognitive_load: f64,
    pub error_rate: f64,
    pub avg_confidence: f64,
    pub convergence_progress: f64,
    pub intervention_active: bool,
}

/// A plan produced by proactive planning phase
#[derive(Debug, Clone)]
pub struct MetaPlan {
    pub strategy: String,
    pub allocated_budget: u64,
    pub target_handlers: Vec<String>,
    pub difficulty_estimate: f64,
    pub cycle: u64,
}

impl MetaCogMonitor {
    pub fn new() -> Self {
        Self {
            trace: VecDeque::new(),
            observations: Vec::new(),
            meta_state: MetaState {
                current_strategy: "exploratory".to_string(),
                cognitive_load: 0.3,
                error_rate: 0.0,
                avg_confidence: 0.5,
                convergence_progress: 0.0,
                intervention_active: false,
            },
            max_trace_len: 200,
            error_threshold: 0.3,
            fatigue_window: 10,
            plan: None,
            step_counter: 0,
            cycle: 0,
        }
    }

    /// Record a reasoning step into the trace
    pub fn record_step(&mut self, step: ReasoningStep) {
        self.cycle = step.cycle;
        self.trace.push_back(step);
        self.update_meta_state();
        self.prune();
    }

    fn update_meta_state(&mut self) {
        let recent: Vec<&ReasoningStep> =
            self.trace.iter().rev().take(self.fatigue_window).collect();
        if recent.is_empty() {
            return;
        }
        let error_count = recent.iter().filter(|s| s.error_flag).count();
        let avg_conf = recent.iter().map(|s| s.confidence).sum::<f64>() / recent.len() as f64;
        self.meta_state.error_rate = error_count as f64 / recent.len() as f64;
        self.meta_state.avg_confidence = avg_conf;
    }

    /// Phase 1: Proactive Planning — assess current state, produce plan
    pub fn proactive_plan(
        &mut self,
        available_handlers: &[String],
        cognitive_load: f64,
        cycle: u64,
    ) -> MetaPlan {
        self.cycle = cycle;
        let recent: Vec<&ReasoningStep> = self.trace.iter().rev().take(5).collect();
        let error_rate = if recent.is_empty() {
            0.0
        } else {
            recent.iter().filter(|s| s.error_flag).count() as f64 / recent.len() as f64
        };
        let avg_conf = if recent.is_empty() {
            0.5
        } else {
            recent.iter().map(|s| s.confidence).sum::<f64>() / recent.len() as f64
        };

        let strategy = if error_rate > 0.3 {
            "conservative"
        } else if cognitive_load > 0.7 {
            "minimal"
        } else {
            "exploratory"
        };

        let target_handlers = match strategy {
            "conservative" => available_handlers
                .iter()
                .filter(|h| !h.contains("research") && !h.contains("explore"))
                .cloned()
                .collect(),
            "minimal" => vec!["ne_evaluator".to_string(), "meta_agent".to_string()],
            "exploratory" => available_handlers.to_vec(),
            _ => available_handlers.to_vec(),
        };

        let plan = MetaPlan {
            strategy: strategy.to_string(),
            allocated_budget: if strategy == "exploratory" { 3 } else { 1 },
            target_handlers,
            difficulty_estimate: error_rate * 0.6 + (1.0 - avg_conf) * 0.4,
            cycle,
        };
        self.meta_state.current_strategy = strategy.to_string();
        self.meta_state.cognitive_load = cognitive_load;
        self.plan = Some(plan.clone());
        plan
    }

    /// Phase 2: Online Regulation — analyze trace for errors/fatigue
    pub fn online_regulate(&mut self) -> Vec<MetaObservation> {
        let mut obs = Vec::new();
        let recent: Vec<&ReasoningStep> =
            self.trace.iter().rev().take(self.fatigue_window).collect();

        // Error spike detection
        let error_count = recent.iter().filter(|s| s.error_flag).count();
        if recent.len() >= 3 && error_count as f64 / recent.len() as f64 > self.error_threshold {
            obs.push(MetaObservation {
                observation_type: ObsType::ErrorSpike,
                handler: recent
                    .last()
                    .map(|s| s.handler_name.clone())
                    .unwrap_or_default(),
                severity: error_count as f64 / recent.len() as f64,
                detail: format!(
                    "error rate {}/{} above threshold",
                    error_count,
                    recent.len()
                ),
                cycle: self.cycle,
            });
        }

        // Cognitive fatigue: duration trending up
        if recent.len() >= 5 {
            let recent_durations: Vec<u64> = recent.iter().map(|s| s.duration_ms).collect();
            let avg_early: f64 = recent_durations.iter().take(3).sum::<u64>() as f64 / 3.0;
            let avg_late: f64 = recent_durations.iter().rev().take(3).sum::<u64>() as f64 / 3.0;
            if avg_late > avg_early * 1.5 && avg_late > 100.0 {
                obs.push(MetaObservation {
                    observation_type: ObsType::CognitiveFatigue,
                    handler: "system".to_string(),
                    severity: (avg_late / avg_early - 1.5).min(1.0),
                    detail: format!("duration increased {:.1}→{:.1}ms", avg_early, avg_late),
                    cycle: self.cycle,
                });
            }
        }

        self.observations.extend(obs.clone());
        obs
    }

    /// Phase 3: Adaptive Stopping — should we stop processing?
    pub fn should_stop(&self, confidence_target: f64) -> bool {
        if self.trace.is_empty() {
            return false;
        }
        let recent: Vec<&ReasoningStep> = self.trace.iter().rev().take(5).collect();
        let avg_conf = recent.iter().map(|s| s.confidence).sum::<f64>() / recent.len() as f64;
        avg_conf >= confidence_target
    }

    /// Clean up old trace entries
    pub fn prune(&mut self) {
        while self.trace.len() > self.max_trace_len {
            self.trace.pop_front();
        }
        while self.observations.len() > 100 {
            self.observations.remove(0);
        }
    }
}

impl Default for MetaCogMonitor {
    fn default() -> Self {
        Self::new()
    }
}
