//! # ConsciousnessLoop — Subsystem Runtime Integrator
//!
//! Connects E₈ → GWT → HyperCube → IntrinsicMotivation → +1 Observer → CapabilityVector
//! into a single coherent consciousness iteration loop.

use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::nt_core_hex::{
    optimal_starting_mode, FullReasoningState, MetaState, ReasoningHexagram,
};
use crate::core::nt_core_meta::ConsolidationOutcome;
use crate::core::nt_core_meta::MetacognitiveState;
use crate::core::nt_core_observer::{ObserverReport, OneObserver};
use crate::core::nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
use crate::core::nt_core_self::silicon_self::SiliconSelfModel;
use crate::neotrix::nt_mind::attention_router::{AttentionRouter, RoutedContext};
use crate::neotrix::nt_mind::curiosity_drive::CuriosityDrive;
use crate::neotrix::nt_mind::hypercube_bridge::HyperCubeBridge;
use std::collections::HashMap;
use std::collections::VecDeque;

/// Configuration for the consciousness loop.
#[derive(Debug, Clone)]
pub struct ConsciousnessConfig {
    /// Minimum trajectory length before observer analysis triggers
    pub min_trajectory_for_observer: usize,
    /// How often to auto-run a full loop when idle (in seconds)
    pub idle_tick_interval_secs: u64,
    /// Whether to apply capability deltas automatically
    pub auto_apply_capability_deltas: bool,
    /// Whether to seed exploration on motivation trigger
    pub auto_explore_on_curiosity: bool,
    /// Max states in trajectory before pruning oldest
    pub max_trajectory_len: usize,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            min_trajectory_for_observer: 3,
            idle_tick_interval_secs: 60,
            auto_apply_capability_deltas: true,
            auto_explore_on_curiosity: true,
            max_trajectory_len: 100,
        }
    }
}

/// Result of a single consciousness iteration.
pub struct LoopResult {
    pub routed_context: RoutedContext,
    pub e8_mode: ReasoningHexagram,
    pub e8_mode_name: String,
    pub meta_state: MetaState,
    pub motivation: Option<MotivationState>,
    pub observer_report: Option<ObserverReport>,
    pub capability_deltas_applied: Vec<(String, f64)>,
    pub iteration: u64,
    pub knowledge_gap_detected: bool,
    pub curiosity_triggered: bool,
    pub msv: MetacognitiveState,
}

/// The ConsciousnessLoop — main runtime integrator.
///
/// Ownership of subsystems is centralized here. The loop receives input,
/// routes through all subsystems, and returns integrated results.
pub struct ConsciousnessLoop {
    /// AttentionRouter: GWT + HyperCube bridge
    pub attention_router: AttentionRouter,
    /// +1 Observer: trajectory pattern analysis
    pub observer: OneObserver,
    /// Intrinsic motivation: curiosity/exploration drive
    pub intrinsic_motivation: IntrinsicMotivation,
    /// Current capability vector (updated by observer deltas)
    pub capability_vector: CapabilityVector,
    /// Silicon self-model (thinking traces + patterns)
    pub silicon_self: SiliconSelfModel,
    /// E₈ state trajectory for observer analysis
    pub state_trajectory: Vec<FullReasoningState>,
    /// Current meta-state (reflection/planning bits)
    pub current_meta: MetaState,
    /// Iteration counter
    pub iteration: u64,
    /// Config
    pub config: ConsciousnessConfig,
    /// History of observer reports
    pub observer_reports: Vec<ObserverReport>,
    /// Knowledge gap map: domain → sparsity level
    pub knowledge_gaps: HashMap<String, f64>,
    /// MetacognitiveState — unified self-awareness snapshot
    pub msv: MetacognitiveState,
    /// CuriosityDrive — prediction-error-driven exploration
    pub curiosity_drive: CuriosityDrive,
    /// Prediction error ring buffer (JEPA placeholder)
    pub prediction_error_buffer: VecDeque<f64>,
}

impl ConsciousnessLoop {
    pub fn new() -> Self {
        let mut router = AttentionRouter::new();
        router.seed_knowledge();
        Self {
            attention_router: router,
            observer: OneObserver::new(),
            intrinsic_motivation: IntrinsicMotivation::new(),
            capability_vector: CapabilityVector::default(),
            silicon_self: SiliconSelfModel::new(),
            state_trajectory: Vec::new(),
            current_meta: MetaState::new(0),
            iteration: 0,
            config: ConsciousnessConfig::default(),
            observer_reports: Vec::new(),
            knowledge_gaps: HashMap::new(),
            msv: MetacognitiveState::new(),
            curiosity_drive: CuriosityDrive::default(),
            prediction_error_buffer: VecDeque::with_capacity(20),
        }
    }

    /// Main consciousness iteration: input → E₈ → GWT → HyperCube → Motivation → Observer → output
    pub fn step(&mut self, input: &str) -> LoopResult {
        self.iteration += 1;

        // 1. E₈ mode selection
        let e8_mode = optimal_starting_mode(input);
        let mode_name = e8_mode.mode_name().to_string();

        // 2. Record state in trajectory
        let full_state = FullReasoningState::new(e8_mode, self.current_meta);
        self.state_trajectory.push(full_state);
        if self.state_trajectory.len() > self.config.max_trajectory_len {
            self.state_trajectory.remove(0);
        }

        // 3. GWT routing via AttentionRouter
        let routed_context = self.attention_router.route(input);

        // 4. SiliconSelfModel observation
        let trace_id = self.silicon_self.begin_thinking_trace(input);
        self.silicon_self.observe(&format!(
            "E8:{} GWT:{} Knowledge:{}",
            mode_name,
            routed_context.winning_topic,
            routed_context.knowledge_lines.len()
        ));

        // 5. Intrinsic motivation computation
        let motivation_state = self.intrinsic_motivation.compute(&self.silicon_self);
        let curiosity_triggered = motivation_state.should_explore;

        // 6. Curiosity-driven exploration: seed knowledge gaps
        let knowledge_gap_detected = if curiosity_triggered && self.config.auto_explore_on_curiosity
        {
            self.seed_knowledge_from_curiosity(&motivation_state);
            true
        } else {
            false
        };

        // 6b. Integrate prediction error into curiosity drive
        let avg_prediction_error = self.prediction_error_buffer.back().copied().unwrap_or(0.0);
        self.curiosity_drive
            .ingest_prediction_error(avg_prediction_error, &[] /* no gap reports here */);

        // 6c. Update MetacognitiveState from subsystems
        self.msv.update_prediction_error(avg_prediction_error);
        self.msv
            .update_curiosity(self.curiosity_drive.curiosity_level.salience_multiplier());
        self.msv
            .update_reasoning_quality(mode_name.len() as f64 / 20.0);
        self.msv.update_from_load(0.3); // placeholder until cognitive_load is wired

        // 7. +1 Observer analysis (if enough trajectory)
        let (observer_report, capability_deltas_applied) =
            if self.state_trajectory.len() >= self.config.min_trajectory_for_observer {
                let keywords: Vec<&str> = input.split_whitespace().collect();
                let report = self.observer.analyze(&self.state_trajectory, &keywords);
                let deltas = self.apply_observer_deltas(&report);
                self.observer_reports.push(report.clone());
                if self.observer_reports.len() > 50 {
                    self.observer_reports.remove(0);
                }
                // Apply meta recommendation
                if let Some(meta) = report.recommended_meta {
                    self.current_meta = meta;
                }
                (Some(report), deltas)
            } else {
                (None, Vec::new())
            };

        // 8. Complete thinking trace
        let trace_quality = observer_report
            .as_ref()
            .map(|r| r.quality_score)
            .unwrap_or(0.5);
        self.silicon_self.complete_thinking_trace(
            trace_id,
            &routed_context.winning_topic,
            trace_quality,
        );

        // 8b. Update MSV from observer quality
        self.msv.update_from_critic(trace_quality, trace_quality);

        // 8c. Feed consolidation quality into MSV
        self.msv
            .update_from_consolidation(&ConsolidationOutcome::default());

        // 8d. Detect MSV conflicts and log if any
        if let Some(recommendation) = self.msv.meta_awareness_trigger() {
            self.silicon_self.observe(recommendation);
        }

        LoopResult {
            routed_context,
            e8_mode,
            e8_mode_name: mode_name,
            meta_state: self.current_meta,
            motivation: Some(motivation_state),
            observer_report,
            capability_deltas_applied,
            iteration: self.iteration,
            knowledge_gap_detected,
            curiosity_triggered,
            msv: self.msv.clone(),
        }
    }

    /// Apply observer capability deltas to the CapabilityVector.
    fn apply_observer_deltas(&mut self, report: &ObserverReport) -> Vec<(String, f64)> {
        if !self.config.auto_apply_capability_deltas {
            return Vec::new();
        }
        let mut applied = Vec::new();
        for (name, delta) in &report.capability_deltas {
            // Map to capability vector dimensions
            let mapped = match name.as_str() {
                "exploration_bias" => ("curiosity_drive", *delta),
                "task_adaptability" => ("adaptability", *delta),
                "decisiveness" => ("decisiveness", *delta),
                "analysis_depth" => ("analysis_depth", *delta),
                "planning_ahead" => ("planning_depth", *delta),
                "pattern_matching" => ("pattern_recognition", *delta),
                "route_planning" => ("planning_depth", *delta * 0.5),
                "task_success_rate" => ("task_completion", *delta),
                "knowledge_scope" => ("knowledge_breadth", *delta),
                "convergence_speed" => ("convergence_rate", *delta),
                "learning_rate" => ("learning_speed", *delta),
                "cross_domain_transfer" => ("transfer_ability", *delta),
                _ => continue,
            };
            self.capability_vector.add_extension_dim(mapped.0, mapped.1);
            applied.push((mapped.0.to_string(), mapped.1));
        }
        applied
    }

    /// Seed hypercube with exploration topics based on curiosity gaps.
    fn seed_knowledge_from_curiosity(&mut self, _motivation: &MotivationState) {
        let gap_topics = self.attention_router.sparse_topics();
        for topic in gap_topics {
            let coord = HyperCoord::with(DimensionAxis::Abstraction, 0.5);
            self.attention_router.bridge.hypercube.insert(
                &coord,
                "curiosity-exploration",
                &format!("{:?}", topic),
            );
        }
    }

    /// Access the attention router's hypercube bridge.
    pub fn bridge(&self) -> &HyperCubeBridge {
        &self.attention_router.bridge
    }

    pub fn bridge_mut(&mut self) -> &mut HyperCubeBridge {
        &mut self.attention_router.bridge
    }

    /// Get current loop status summary.
    pub fn status(&self) -> String {
        format!(
            "ConsciousnessLoop | iter={} | trajectory={} | meta={:?} | observer_reports={} | gaps={} | caps_ext={}",
            self.iteration,
            self.state_trajectory.len(),
            self.current_meta,
            self.observer_reports.len(),
            self.knowledge_gaps.len(),
            self.capability_vector.extension().len(),
        )
    }

    /// Reset the loop state for a new session.
    pub fn reset_session(&mut self) {
        self.state_trajectory.clear();
        self.observer_reports.clear();
        self.current_meta = MetaState::new(0);
        self.silicon_self.reset_session();
    }
}

impl Default for ConsciousnessLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_loop() {
        let loop_ = ConsciousnessLoop::new();
        assert_eq!(loop_.iteration, 0);
        assert!(loop_.state_trajectory.is_empty());
        assert!(loop_.observer_reports.is_empty());
    }

    #[test]
    fn test_step_routes_input() {
        let mut loop_ = ConsciousnessLoop::new();
        let result = loop_.step("find patterns in system error logs");
        assert!(!result.e8_mode_name.is_empty());
        assert!(!result.routed_context.winning_topic.is_empty());
        assert_eq!(result.iteration, 1);
    }

    #[test]
    fn test_step_accumulates_trajectory() {
        let mut loop_ = ConsciousnessLoop::new();
        let _ = loop_.step("first input");
        assert_eq!(loop_.state_trajectory.len(), 1);
        let _ = loop_.step("second input");
        assert_eq!(loop_.state_trajectory.len(), 2);
    }

    #[test]
    fn test_step_triggers_observer_after_min_trajectory() {
        let mut loop_ = ConsciousnessLoop::new();
        loop_.config.min_trajectory_for_observer = 2;

        let r1 = loop_.step("test one");
        assert!(r1.observer_report.is_none());

        let r2 = loop_.step("test two");
        assert!(r2.observer_report.is_some());
    }

    #[test]
    fn test_observer_updates_capability_vector() {
        let mut loop_ = ConsciousnessLoop::new();
        loop_.config.min_trajectory_for_observer = 2;

        let _ = loop_.step("error bug crash fail");
        let r = loop_.step("error bug crash fail again");
        assert!(r.observer_report.is_some());
        assert!(
            !loop_.capability_vector.extension().is_empty()
                || r.observer_report
                    .as_ref()
                    .map_or(true, |o| o.capability_deltas.is_empty())
        );
    }

    #[test]
    fn test_curiosity_detection() {
        let mut loop_ = ConsciousnessLoop::new();
        let result = loop_.step("explore novel ideas");
        assert!(result.curiosity_triggered);
    }

    #[test]
    fn test_status_format() {
        let mut loop_ = ConsciousnessLoop::new();
        let _ = loop_.step("test");
        let status = loop_.status();
        assert!(status.contains("ConsciousnessLoop"));
        assert!(status.contains("iter="));
    }

    #[test]
    fn test_reset_session_clears_state() {
        let mut loop_ = ConsciousnessLoop::new();
        let _ = loop_.step("test");
        loop_.reset_session();
        assert!(loop_.state_trajectory.is_empty());
        assert_eq!(loop_.state_trajectory.len(), 0);
    }

    #[test]
    fn test_multiple_steps_track_iteration() {
        let mut loop_ = ConsciousnessLoop::new();
        for i in 1..=5 {
            let r = loop_.step(&format!("step {}", i));
            assert_eq!(r.iteration, i as u64);
        }
        assert_eq!(loop_.iteration, 5);
    }

    #[test]
    fn test_trajectory_pruning() {
        let mut loop_ = ConsciousnessLoop::new();
        loop_.config.max_trajectory_len = 3;
        for i in 1..=5 {
            let _ = loop_.step(&format!("step {}", i));
        }
        assert_eq!(loop_.state_trajectory.len(), 3);
    }

    #[test]
    fn test_e8_mode_changes_with_input() {
        let mut loop_ = ConsciousnessLoop::new();
        let r1 = loop_.step("crash bug null pointer");
        let r2 = loop_.step("brainstorm creative novel ideas");
        assert_ne!(r1.e8_mode, r2.e8_mode);
    }
}
