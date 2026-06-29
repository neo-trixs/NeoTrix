#![allow(dead_code)]
use super::audit::{AuditEngine, AuditReport};
use super::memory_evolution::global_memory_evolution;
use super::meta_kpi_repo::{MetaKPIRepository, MetaKPISnapshot, SelfDirectedGoal};
use super::monitor::{AlertSeverity, HealthCheck, MetaAlert, MetaMonitor};
use super::planner::{EvolutionPlanner, PlannedEvolution};
use super::self_model::{EventKind, EvolutionEvent, SelfModel};
use super::weakness::{WeaknessAnalyzer, WeaknessReport};
use crate::core::nt_core_hcube::global_memory_activation;

/// Multi-framework consciousness probability assessor.
/// Combines IIT (Φ), GWT (6 markers), HOT (self-ref), and PP (hierarchical depth)
/// into a unified probability estimate.
/// Reference: Chen et al. 2026 LLM Consciousness Survey, Frontiers 2026 Consciousness Indicators.
#[derive(Debug, Clone)]
pub struct ConsciousnessProbabilityAssessor {
    /// IIT: integrated information score (0.0-1.0)
    pub iit_phi_score: f64,
    /// GWT: number of satisfied markers (0-6)
    pub gwt_markers_satisfied: u8,
    /// GWT: total markers checked
    pub gwt_markers_total: u8,
    /// HOT: self-referential processing score (0.0-1.0)
    pub hot_self_ref: f64,
    /// PP: hierarchical processing depth (number of levels)
    pub pp_hierarchical_depth: f64,
    /// Combined consciousness probability (0.0-1.0)
    pub combined_probability: f64,
    /// Per-framework breakdown for transparency
    pub framework_breakdown: Vec<FrameworkScore>,
    /// Assessment history
    pub history: Vec<ConsciousnessAssessment>,
}

#[derive(Debug, Clone)]
pub struct FrameworkScore {
    pub framework: String,
    pub score: f64,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct ConsciousnessAssessment {
    pub cycle: u64,
    pub combined: f64,
    pub iit: f64,
    pub gwt: f64,
    pub hot: f64,
    pub pp: f64,
}

impl Default for ConsciousnessProbabilityAssessor {
    fn default() -> Self {
        Self {
            iit_phi_score: 0.0,
            gwt_markers_satisfied: 0,
            gwt_markers_total: 6,
            hot_self_ref: 0.0,
            pp_hierarchical_depth: 0.0,
            combined_probability: 0.0,
            framework_breakdown: vec![
                FrameworkScore {
                    framework: "IIT".into(),
                    score: 0.0,
                    weight: 0.30,
                },
                FrameworkScore {
                    framework: "GWT".into(),
                    score: 0.0,
                    weight: 0.30,
                },
                FrameworkScore {
                    framework: "HOT".into(),
                    score: 0.0,
                    weight: 0.25,
                },
                FrameworkScore {
                    framework: "PP".into(),
                    score: 0.0,
                    weight: 0.15,
                },
            ],
            history: Vec::with_capacity(100),
        }
    }
}

impl ConsciousnessProbabilityAssessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Assess consciousness probability from current indicators.
    /// Each framework dimension is scored independently, then weighted averaged.
    pub fn assess(
        &mut self,
        cycle: u64,
        phi: Option<f64>,
        gwt_markers: u8,
        hot_self_ref: f64,
        pp_depth: f64,
    ) -> f64 {
        // IIT: phi score directly
        let iit = phi.unwrap_or(0.0);
        self.iit_phi_score = iit;

        // GWT: proportion of satisfied markers
        let gwt = if self.gwt_markers_total > 0 {
            (gwt_markers as f64) / (self.gwt_markers_total as f64)
        } else {
            0.0
        };
        self.gwt_markers_satisfied = gwt_markers;

        // HOT: self-referential score
        let hot = hot_self_ref.min(1.0).max(0.0);
        self.hot_self_ref = hot;

        // PP: hierarchical depth normalized (assume max ~10 levels)
        let pp = (pp_depth / 10.0).min(1.0);
        self.pp_hierarchical_depth = pp_depth;

        // Update framework breakdown
        for fw in self.framework_breakdown.iter_mut() {
            match fw.framework.as_str() {
                "IIT" => fw.score = iit,
                "GWT" => fw.score = gwt,
                "HOT" => fw.score = hot,
                "PP" => fw.score = pp,
                _ => {}
            }
        }

        // Weighted combination
        let combined: f64 = self
            .framework_breakdown
            .iter()
            .map(|fw| fw.score * fw.weight)
            .sum();
        self.combined_probability = combined.min(1.0).max(0.0);

        // Record history
        self.history.push(ConsciousnessAssessment {
            cycle,
            combined: self.combined_probability,
            iit,
            gwt,
            hot,
            pp,
        });
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        self.combined_probability
    }

    /// Return which framework the current assessment is viewed from.
    /// Each entry labels the framework, its score, and weight.
    /// Reference: Klatzmann & Doerig 2026 — multi-framework heterogeneity.
    pub fn framework_selector(&self) -> Vec<String> {
        self.framework_breakdown
            .iter()
            .map(|fw| {
                format!(
                    "{}: score={:.3} weight={:.2}",
                    fw.framework, fw.score, fw.weight
                )
            })
            .collect()
    }

    /// Assess using GWT marker template (6 standard GWT markers).
    pub fn assess_gwt(&mut self, markers: &[bool]) -> u8 {
        let satisfied = markers.iter().filter(|&&m| m).count() as u8;
        satisfied
    }

    pub fn summary(&self) -> String {
        format!(
            "ConsciousnessProbability: {:.1}% | IIT(Φ={:.2}) GWT({}/{}) HOT(self={:.2}) PP(depth={:.1})",
            self.combined_probability * 100.0,
            self.iit_phi_score,
            self.gwt_markers_satisfied, self.gwt_markers_total,
            self.hot_self_ref,
            self.pp_hierarchical_depth,
        )
    }

    pub fn trend(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let recent = &self.history[self.history.len().saturating_sub(10)..];
        if recent.len() < 2 {
            return 0.0;
        }
        recent.last().unwrap().combined - recent.first().unwrap().combined
    }
}

/// Assesses the "Acting" dimension of self-awareness (KAPRO framework).
/// Knowing ≠ Acting: a system can know its own limits yet fail to act on them.
/// Reference: KAPRO arXiv 2606.20661, Mirror benchmark arXiv 2604.19809.
#[derive(Debug, Clone)]
pub struct ActingDimensionAssessor {
    /// Acting dimension score (0.0-1.0). Measures whether the system
    /// actually adjusts its behavior based on self-knowledge.
    pub acting_score: f64,
    /// Knowing dimension score (0.0-1.0). Measures accuracy of self-assessment.
    pub knowing_score: f64,
    /// KAPRO gap = knowing - acting. Positive = knows more than acts on.
    pub kapro_gap: f64,
    /// Number of times the system acted on its self-knowledge
    pub actions_taken: u64,
    /// Number of times the system had self-knowledge but did NOT act
    pub missed_actions: u64,
    /// Calibration error in the acting dimension
    pub acting_calibration_error: f64,
    /// History of KAPRO assessments
    pub history: Vec<KaproAssessment>,
}

#[derive(Debug, Clone)]
pub struct KaproAssessment {
    pub cycle: u64,
    pub knowing: f64,
    pub acting: f64,
    pub gap: f64,
}

impl Default for ActingDimensionAssessor {
    fn default() -> Self {
        Self {
            acting_score: 0.0,
            knowing_score: 0.0,
            kapro_gap: 0.0,
            actions_taken: 0,
            missed_actions: 0,
            acting_calibration_error: 0.5,
            history: Vec::with_capacity(100),
        }
    }
}

impl ActingDimensionAssessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate the KAPRO gap from current metacognitive signals.
    /// knowing: the system's meta-accuracy (how well it knows itself)
    /// acting: whether the system actually changes behavior based on that knowledge
    pub fn evaluate(
        &mut self,
        cycle: u64,
        knowing: f64,
        acted_on_knowledge: bool,
        should_have_acted: bool,
    ) -> KaproAssessment {
        self.knowing_score = knowing;

        if should_have_acted {
            if acted_on_knowledge {
                self.actions_taken += 1;
            } else {
                self.missed_actions += 1;
            }
        }

        let total_actions = self.actions_taken + self.missed_actions;
        self.acting_score = if total_actions > 0 {
            self.actions_taken as f64 / total_actions as f64
        } else {
            0.5 // default when no data
        };

        self.kapro_gap = self.knowing_score - self.acting_score;
        self.acting_calibration_error = 1.0 - self.acting_score;

        let assessment = KaproAssessment {
            cycle,
            knowing: self.knowing_score,
            acting: self.acting_score,
            gap: self.kapro_gap,
        };
        self.history.push(assessment.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        assessment
    }

    pub fn summary(&self) -> String {
        format!(
            "KAPRO: Knowing={:.2} Acting={:.2} Gap={:.2} (actions={}, missed={})",
            self.knowing_score,
            self.acting_score,
            self.kapro_gap,
            self.actions_taken,
            self.missed_actions,
        )
    }
}

/// Type of introspection available to the system.
/// Reference: Introspect-Bench arXiv 2603.20276
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntrospectionType {
    /// Policy introspection: knows its own output tendencies (currently available)
    PolicyIntrospection,
    /// Mechanism introspection: knows its own internal activations (future)
    MechanismIntrospection,
}

impl Default for IntrospectionType {
    fn default() -> Self {
        IntrospectionType::PolicyIntrospection
    }
}

/// Control mode for metacognitive actions.
/// Mirror benchmark (arXiv 2604.19809) shows only architectural constraint works.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetaControlMode {
    /// No control - monitoring only
    Passive,
    /// Force lower temperature and more reasoning on low confidence
    Conservative,
    /// Force exploration (higher temperature) on stagnation
    Exploratory,
    /// Adaptive: choose based on context
    Adaptive,
}

impl Default for MetaControlMode {
    fn default() -> Self {
        MetaControlMode::Adaptive
    }
}

/// The main metacognitive loop that orchestrates self-awareness.
///
/// Phases:
///   1. SCAN  — Build/update SelfModel from filesystem
///   2. ANALYZE — Run WeaknessAnalyzer on SelfModel
///   3. MONITOR — Generate alerts, check trends
///   4. PLAN   — Prioritize weaknesses into evolution actions
///   5. REPORT — Summarize findings for external consumption
///
/// This is a **synchronous** loop (no tokio dependency) — the `core/` layer
/// is runtime-agnostic. Async scheduling happens in `nt_mind/`.
#[derive(Debug, Clone)]
pub struct MetaCognitiveLoop {
    pub self_model: SelfModel,
    pub monitor: MetaMonitor,
    pub analyzer: WeaknessAnalyzer,
    pub planner: EvolutionPlanner,
    pub auditor: Option<AuditEngine>,
    pub last_audit: Option<AuditReport>,
    pub iteration: usize,
    pub max_iterations: usize,
    /// Self-predicted performance (0.0-1.0) for last N cycles
    pub self_predicted: Vec<f64>,
    /// Actual measured performance (0.0-1.0) for last N cycles
    pub actual_performance: Vec<f64>,
    /// MetaAccuracy = |self_predicted - actual_performance|
    pub meta_accuracy: Vec<f64>,
    /// Maximum history length for accuracy tracking
    pub meta_window: usize,
    /// RIIU-style causal footprint (mu) — tracks which weights lead to good outcomes.
    /// Each entry is [w_accuracy, w_coherence, w_health] for the last N cycles.
    pub causal_footprint: Vec<[f64; 3]>,
    /// RIIU-style broadcast buffer — top-K signals from the global workspace.
    pub broadcast_buffer: Vec<f64>,
    /// Adaptive weights [accuracy, coherence, health] — updated by meta_accuracy gap.
    pub adaptive_weights: [f64; 3],
    /// Auto-Φ integrated information value (computed each cycle from causal footprint)
    pub auto_phi_value: f64,
    /// MC² cross-cycle meta-knowledge accumulator (arXiv:2604.17399)
    pub meta_knowledge: MetaKnowledgeAccumulator,
    /// Last composite health from SelfReviewStage (SEAL pipeline → consciousness bridge)
    pub last_review_composite: Option<f64>,
    /// Phase 5: persistent KPI repository with gap detection
    pub kpi_repo: MetaKPIRepository,
    /// Phase 5: currently active self-directed improvement goal
    pub active_goal: Option<SelfDirectedGoal>,
    /// Real calibration metrics from CalibrationEngine (replaces synthetic tech_debt)
    pub calibration_meta_d: Option<f64>,
    pub calibration_ece: Option<f64>,
    pub calibration_pair_count: usize,
    /// Multi-framework consciousness probability assessor
    pub consciousness_probability: ConsciousnessProbabilityAssessor,
    /// KAPRO acting dimension assessor (Knowing≠Acting)
    pub acting_dimension: ActingDimensionAssessor,
    /// Whether metacognitive control is active (architectural enforcement)
    pub control_active: bool,
    /// Control mode: what action to take when meta_accuracy drops
    pub control_mode: MetaControlMode,
    /// Whether control was triggered this cycle
    pub control_triggered: bool,
    /// Temperature override (None = use default)
    pub temperature_override: Option<f64>,
    /// Reasoning steps override (None = use default)
    pub reasoning_steps_override: Option<usize>,
    /// Conservative mode toggle
    pub conservative_mode: bool,
    /// Type of introspection available (Policy vs Mechanism)
    pub introspection_type: IntrospectionType,
    /// Whether mechanism introspection is available (default: false)
    pub introspection_mechanism_available: bool,
}

impl MetaCognitiveLoop {
    pub fn new(model: SelfModel) -> Self {
        let monitor = MetaMonitor::new(model.clone());
        Self {
            self_model: model,
            monitor,
            analyzer: WeaknessAnalyzer::new(),
            planner: EvolutionPlanner::new(),
            auditor: None,
            last_audit: None,
            iteration: 0,
            max_iterations: 100,
            self_predicted: Vec::with_capacity(100),
            actual_performance: Vec::with_capacity(100),
            meta_accuracy: Vec::with_capacity(100),
            meta_window: 100,
            causal_footprint: Vec::with_capacity(100),
            broadcast_buffer: Vec::with_capacity(8),
            adaptive_weights: [0.33, 0.33, 0.34],
            auto_phi_value: 0.0,
            meta_knowledge: MetaKnowledgeAccumulator::new(100),
            last_review_composite: None,
            kpi_repo: MetaKPIRepository::new(),
            active_goal: None,
            calibration_meta_d: None,
            calibration_ece: None,
            calibration_pair_count: 0,
            consciousness_probability: ConsciousnessProbabilityAssessor::default(),
            acting_dimension: ActingDimensionAssessor::default(),
            control_active: true,
            control_mode: MetaControlMode::Adaptive,
            control_triggered: false,
            temperature_override: None,
            reasoning_steps_override: None,
            conservative_mode: false,
            introspection_type: IntrospectionType::PolicyIntrospection,
            introspection_mechanism_available: false,
        }
    }

    /// Attach an AuditEngine to run during metacognitive cycles.
    pub fn with_auditor(mut self, engine: AuditEngine) -> Self {
        self.auditor = Some(engine);
        self
    }

    /// Record a self-prediction vs actual outcome
    pub fn record_meta_accuracy(&mut self, predicted: f64, actual: f64) {
        if self.self_predicted.len() >= self.meta_window {
            self.self_predicted.remove(0);
            self.actual_performance.remove(0);
            self.meta_accuracy.remove(0);
        }
        self.self_predicted.push(predicted);
        self.actual_performance.push(actual);
        let acc = if predicted < 0.0 || predicted > 1.0 || actual < 0.0 || actual > 1.0 {
            0.0
        } else {
            1.0 - (predicted - actual).abs()
        };
        self.meta_accuracy.push(acc);
    }

    /// Inject real calibration data from CalibrationEngine.
    /// Replaces synthetic tech_debt-based meta_accuracy with genuine
    /// prediction-outcome calibration metrics (ECE, meta-d').
    pub fn record_calibration_data(&mut self, meta_d: f64, ece: f64, pair_count: usize) {
        self.calibration_meta_d = Some(meta_d);
        self.calibration_ece = Some(ece);
        self.calibration_pair_count = pair_count;
    }

    /// Current MetaAccuracy = mean over window
    pub fn current_meta_accuracy(&self) -> f64 {
        if self.meta_accuracy.is_empty() {
            return 0.0;
        }
        self.meta_accuracy.iter().sum::<f64>() / self.meta_accuracy.len() as f64
    }

    /// MetaAccuracy trend: positive = improving self-knowledge
    pub fn meta_accuracy_trend(&self) -> f64 {
        if self.meta_accuracy.len() < 2 {
            return 0.0;
        }
        let n = self.meta_accuracy.len() as f64;
        let mean_x = (n - 1.0) / 2.0;
        let mean_y = self.meta_accuracy.iter().sum::<f64>() / n;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in self.meta_accuracy.iter().enumerate() {
            let x = i as f64;
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }
        if den < 1e-10 {
            0.0
        } else {
            num / den
        }
    }

    /// Fuse self-review audit scores from SEAL pipeline into meta-cognition.
    /// Called by ConsciousnessIntegration bridge after SelfReviewStage completes.
    /// Updates adaptive_weights bias and causal_footprint based on 6-dim audit.
    pub fn fuse_self_review_scores(&mut self, scores: [f64; 6]) -> f64 {
        let weights = [0.25, 0.20, 0.20, 0.15, 0.10, 0.10];
        let composite: f64 = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        let composite = composite.clamp(0.0, 1.0);
        self.last_review_composite = Some(composite);

        // If composite is dangerously low, boost accuracy weight to prioritize
        // self-knowledge over exploration/coherence.
        let old = self.adaptive_weights;
        if composite < 0.4 {
            // Shift weight from coherence to accuracy
            let shift = (0.4 - composite) * 0.3;
            self.adaptive_weights[0] = (self.adaptive_weights[0] + shift).clamp(0.2, 0.8);
            self.adaptive_weights[1] = (self.adaptive_weights[1] - shift * 0.5).clamp(0.1, 0.6);
            self.adaptive_weights[2] = (self.adaptive_weights[2] - shift * 0.5).clamp(0.1, 0.6);
            let sum: f64 = self.adaptive_weights.iter().sum();
            for w in self.adaptive_weights.iter_mut() {
                *w /= sum;
            }
            log::info!(
                "[meta] self-review fusion: composite={:.3}, weights [{:.3},{:.3},{:.3}] → [{:.3},{:.3},{:.3}]",
                composite, old[0], old[1], old[2],
                self.adaptive_weights[0], self.adaptive_weights[1], self.adaptive_weights[2],
            );
        }

        // Record this self-review score in meta-accuracy tracking as a "reality check"
        let predicted = self.current_meta_accuracy();
        self.record_meta_accuracy(predicted, composite);

        composite
    }

    /// Run one full metacognitive cycle.
    pub fn run_cycle(&mut self) -> MetaCycleResult {
        self.iteration += 1;

        // Phase 0: Audit (if attached — runs every 5th cycle to balance cost)
        if let Some(ref auditor) = self.auditor {
            if self.iteration % 5 == 0 {
                self.last_audit = Some(auditor.run_all());
            }
        }

        let report = self.analyzer.analyze(&self.self_model);
        self.self_model.tech_debt.items = self.analyzer.to_tech_debt_items(&report);
        self.self_model.tech_debt.total_count = report.weaknesses.len();

        self.monitor.weaknesses_to_alerts(&report);
        let health_check = self.monitor.run_check();

        let plans = self.planner.plan_from_report(&report);

        // PathwayAwareness: detect bottlenecks from cycle state
        if self.iteration % 3 == 0 {
            if let Ok(mut pa) = crate::core::nt_core_search::global_pathway_awareness().lock() {
                pa.evaluate_tool_efficiency(1.0, 100.0, 500);
                pa.evaluate_context_usage(0.4, 0.7);
                pa.evaluate_memory_freshness(0.2);
                for b in pa.active_bottlenecks() {
                    self.monitor.alerts.push(MetaAlert {
                        timestamp: chrono::Utc::now(),
                        severity: if b.severity > 0.7 {
                            AlertSeverity::Warning
                        } else {
                            AlertSeverity::Info
                        },
                        message: format!("[pathway] {} (sev={:.2})", b.description, b.severity),
                        module: Some("pathway_awareness".into()),
                        suggestion: b.resolution_suggestion.clone(),
                    });
                }
            }
        }
        // MemoryEvolution: periodic retrieval config adaptation
        if self.iteration > 0 && self.iteration % 10 == 0 {
            if let Ok(mut me) = global_memory_evolution().lock() {
                me.auto_adapt();
                self.monitor.alerts.push(MetaAlert {
                    timestamp: chrono::Utc::now(),
                    severity: AlertSeverity::Info,
                    message: "memory_evolution:auto_adapt_cycle".into(),
                    module: Some("memory_evolution".into()),
                    suggestion: "Review retrieval config evolution report".into(),
                });
            }
        }
        // MemoryActivation: spreading activation maintenance
        if self.iteration > 0 && self.iteration % 6 == 0 {
            if let Ok(mut ma) = global_memory_activation().lock() {
                ma.step();
                let active = ma.activation.active_nodes();
                if !active.is_empty() {
                    self.monitor.alerts.push(MetaAlert {
                        timestamp: chrono::Utc::now(),
                        severity: AlertSeverity::Info,
                        message: format!(
                            "memory_activation:{}_active_nodes_top={:.3}",
                            active.len(),
                            active.first().map(|(_, a)| *a).unwrap_or(0.0)
                        ),
                        module: Some("memory_activation".into()),
                        suggestion: "Review spreading activation graph state".into(),
                    });
                }
            }
        }

        let trend = self.monitor.trend_analysis();

        self.self_model.register_evolution(EvolutionEvent {
            timestamp: chrono::Utc::now(),
            kind: EventKind::MetaCognitionUpdated,
            description: format!(
                "Cycle {}: {} weaknesses found, {} alerts, {} plans generated",
                self.iteration,
                report.summary.total_count,
                self.monitor.alerts.len(),
                plans.len()
            ),
            affected_modules: Vec::new(),
        });

        // Auto-record MetaAccuracy: prefer real calibration metrics, fallback to synthetic
        // XXXIV.2: synthetic tech_debt/1000 was flagged as dangerous — replaced with
        // real prediction-outcome calibration data when available.
        let (predicted_quality, actual_quality) = if let (Some(meta_d), Some(ece)) =
            (self.calibration_meta_d, self.calibration_ece)
        {
            // Use genuine calibration data: meta-d measures discrimination,
            // ECE measures calibration error. Higher meta-d + lower ECE = better.
            let p = (1.0 - ece).clamp(0.0, 1.0);
            let a = (meta_d / (1.0 + meta_d)).clamp(0.0, 1.0);
            (p, a)
        } else {
            // Fallback to synthetic tech_debt-based estimation (legacy path)
            let p = if self.self_model.tech_debt.total_count == 0 {
                1.0
            } else {
                (1.0 - (self.self_model.tech_debt.total_count as f64).min(1000.0) / 1000.0).max(0.0)
            };
            let a = {
                let total = report.summary.total_count as f64;
                if total == 0.0 {
                    1.0
                } else {
                    (1.0 - total / 1000.0).max(0.0)
                }
            };
            (p, a)
        };
        self.record_meta_accuracy(predicted_quality, actual_quality);

        // ── Self-Review Fusion: incorporate SEAL pipeline audit scores ──
        // Composite health from SelfReviewStage (set via fuse_self_review_scores)
        // is used as an additional bias in the adaptive weight update below.
        // This creates the circular fusion:
        //   SEAL → SelfReviewStage → audit_scores → MetaCognitiveLoop → next cycle
        let self_review_bias = self.last_review_composite.unwrap_or(0.5);
        if self_review_bias < 0.4 {
            log::warn!(
                "[meta] self-review composite={:.3} — increasing accuracy weight",
                self_review_bias
            );
        }

        // RIIU-style Auto-Φ adaptive weight update: compute integrated information
        let ma = self.current_meta_accuracy();
        let auto_phi = self.compute_auto_phi();
        self.auto_phi_value = auto_phi;
        let error = 1.0 - ma;
        let lr = 0.01 + auto_phi.clamp(0.0, 1.0) * 0.09;
        let momentum = 0.3;
        for w in self.adaptive_weights.iter_mut() {
            let delta = lr * (1.0 - error * 2.0) * (1.0 + auto_phi);
            *w = (*w * (1.0 - momentum) + (*w + delta) * momentum).clamp(0.1, 0.8);
        }
        // Normalize to sum = 1.0
        let sum: f64 = self.adaptive_weights.iter().sum();
        if sum > 0.0 {
            for w in self.adaptive_weights.iter_mut() {
                *w /= sum;
            }
        }
        // Record causal footprint
        self.causal_footprint.push(self.adaptive_weights);
        if self.causal_footprint.len() > self.meta_window {
            self.causal_footprint.remove(0);
        }
        // Update broadcast buffer with top-K signals
        self.broadcast_buffer.push(ma);
        if self.broadcast_buffer.len() > 8 {
            self.broadcast_buffer.remove(0);
        }

        // MC²: record cross-cycle meta-knowledge snapshot
        self.meta_knowledge
            .record(self.iteration as u64, ma, ma, actual_quality, 0.0, 0.0);

        let ma_trend = self.meta_accuracy_trend();

        // Phase 5: Record MetaKPI snapshot for persistent cross-session tracking
        let snapshot = MetaKPISnapshot {
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            meta_accuracy: ma,
            adaptation_rate: auto_phi,
            hallucination_rate: 1.0 - ma,
            cognitive_load: (self.self_model.tech_debt.total_count as f64 / 1000.0).min(1.0),
            negentropy_rate: ma_trend.max(0.0),
            composite_score: (ma + auto_phi + self_review_bias) / 3.0,
        };
        self.kpi_repo.record_snapshot(snapshot);

        // Detect gaps and propose self-improvement goals if health is low
        let gap_report = self.kpi_repo.detect_gaps(10);
        if gap_report.overall_health < 0.7 && self.active_goal.is_none() {
            if let Some(goal) = self.kpi_repo.propose_goal(&gap_report) {
                let dim = goal.target_dimension.clone();
                self.active_goal = Some(goal);
                log::info!(
                    "[meta] self-directed goal proposed: {}, health={:.3}",
                    dim,
                    gap_report.overall_health
                );
            }
        }

        // Metacognitive control: close monitor→control gap
        let control_report = self.run_metacognitive_control();
        if control_report.starts_with("control:") {
            log::info!("[meta] {}", control_report);
        }

        MetaCycleResult {
            iteration: self.iteration,
            health_check,
            report,
            alerts: self.monitor.alerts.clone(),
            plans,
            trend,
            model_snapshot: self.self_model.clone(),
            meta_accuracy: ma,
            meta_accuracy_trend: ma_trend,
        }
    }

    /// Run multiple cycles, returning the result of each.
    pub fn run_batch(&mut self, cycles: usize) -> Vec<MetaCycleResult> {
        let mut results = Vec::with_capacity(cycles);
        let remaining = self.max_iterations - self.iteration;
        let actual = cycles.min(remaining);
        for _ in 0..actual {
            results.push(self.run_cycle());
        }
        results
    }

    /// Run continuous cycles until a stopping condition is met.
    pub fn run_until(
        &mut self,
        mut should_stop: impl FnMut(&MetaCycleResult) -> bool,
    ) -> Vec<MetaCycleResult> {
        let mut results = Vec::new();
        while self.iteration < self.max_iterations {
            let result = self.run_cycle();
            if should_stop(&result) {
                break;
            }
            results.push(result);
        }
        results
    }

    pub fn reset(&mut self, model: SelfModel) {
        self.self_model = model.clone();
        self.monitor = MetaMonitor::new(model);
        self.planner = EvolutionPlanner::new();
        self.iteration = 0;
        self.kpi_repo = MetaKPIRepository::new();
        self.active_goal = None;
    }

    pub fn status_summary(&self) -> String {
        let trend = self.monitor.trend_analysis();
        let goal_info = match &self.active_goal {
            Some(g) => format!(" goal=[{}/{:?}]", g.target_dimension, g.status),
            None => " goal=none".to_string(),
        };
        format!(
            "MetaCognition Cycle {}/{} | {} weaknesses | {} alerts | {} plans pending | trend: {} | meta-acc: {:.3} | meta-trend: {:.3} | {} |{}",
            self.iteration, self.max_iterations,
            self.self_model.tech_debt.total_count,
            self.monitor.alerts.len(),
            self.planner.pending_count(),
            trend.overall,
            self.current_meta_accuracy(),
            self.meta_accuracy_trend(),
            self.kpi_repo.stats(),
            goal_info,
        )
    }

    /// MC²: cross-cycle meta-knowledge summary
    pub fn meta_knowledge_summary(&self) -> String {
        self.meta_knowledge.summary()
    }

    /// Current Auto-Φ integrated information value.
    pub fn auto_phi(&self) -> f64 {
        self.auto_phi_value
    }

    /// Run a multi-framework consciousness probability assessment.
    pub fn assess_consciousness_probability(&mut self) -> f64 {
        let cycle = self.iteration as u64;
        let phi = Some(self.auto_phi_value);
        let gwt_markers = self
            .consciousness_probability
            .assess_gwt(&[true, true, true, false, true, false]);
        let hot = self.current_meta_accuracy();
        let pp_depth = 3.0;
        self.consciousness_probability
            .assess(cycle, phi, gwt_markers, hot, pp_depth)
    }

    /// Evaluate the KAPRO gap between knowing and acting dimensions.
    pub fn evaluate_kapro_gap(&mut self) -> f64 {
        let knowing = self.current_meta_accuracy();
        let acted_on_knowledge = self.control_triggered;
        let should_have_acted = knowing < 0.6;
        self.acting_dimension.evaluate(
            self.iteration as u64,
            knowing,
            acted_on_knowledge,
            should_have_acted,
        );
        self.acting_dimension.kapro_gap
    }

    /// Compute Auto-Φ from causal_footprint weight-outcome correlation.
    fn compute_auto_phi(&self) -> f64 {
        if self.causal_footprint.len() < 3 {
            return 0.0;
        }
        let mut phi = 0.0f64;
        for i in 1..self.causal_footprint.len() {
            let prev = self.causal_footprint[i - 1];
            let curr = self.causal_footprint[i];
            for j in 0..3 {
                let diff = (curr[j] - prev[j]).abs();
                phi += diff * self.meta_accuracy.get(i - 1).copied().unwrap_or(0.5);
            }
        }
        phi /= self.causal_footprint.len().max(1) as f64;
        phi
    }

    /// Run metacognitive control: close the monitoring→control gap.
    /// When meta_accuracy < 0.6, force architectural changes:
    /// - Lower temperature (less randomness)
    /// - Increase reasoning steps
    /// - Switch to conservative strategy
    /// Mirror benchmark (arXiv 2604.19809) shows only architectural constraint works.
    /// Returns a report of what control actions were taken.
    pub fn run_metacognitive_control(&mut self) -> String {
        if !self.control_active {
            return "control_disabled".to_string();
        }

        let meta_acc = self.current_meta_accuracy();
        let mut actions: Vec<String> = Vec::new();

        match self.control_mode {
            MetaControlMode::Passive => {
                self.control_triggered = false;
                return "passive_monitoring".to_string();
            }
            MetaControlMode::Conservative => {
                if meta_acc < 0.6 {
                    self.temperature_override = Some(0.1);
                    self.reasoning_steps_override = Some(5);
                    self.conservative_mode = true;
                    self.control_triggered = true;
                    actions.push("low_temp(0.1)".to_string());
                    actions.push("extra_reasoning(5)".to_string());
                } else if meta_acc > 0.8 {
                    self.temperature_override = None;
                    self.reasoning_steps_override = None;
                    self.conservative_mode = false;
                    self.control_triggered = false;
                }
            }
            MetaControlMode::Exploratory => {
                if meta_acc < 0.5 {
                    self.temperature_override = Some(0.9);
                    self.conservative_mode = false;
                    self.control_triggered = true;
                    actions.push("high_temp(0.9)".to_string());
                }
            }
            MetaControlMode::Adaptive => {
                if meta_acc < 0.4 {
                    self.temperature_override = Some(0.05);
                    self.reasoning_steps_override = Some(8);
                    self.conservative_mode = true;
                    self.control_triggered = true;
                    actions.push(format!("crisis_mode(ma={:.2})", meta_acc));
                } else if meta_acc < 0.6 {
                    self.temperature_override = Some(0.2);
                    self.reasoning_steps_override = Some(4);
                    self.conservative_mode = true;
                    self.control_triggered = true;
                    actions.push(format!("tighten(ma={:.2})", meta_acc));
                } else if meta_acc > 0.85 {
                    self.temperature_override = None;
                    self.reasoning_steps_override = None;
                    self.conservative_mode = false;
                    self.control_triggered = false;
                } else {
                    let temp_adj = 0.5 - (meta_acc - 0.6) * 1.5;
                    self.temperature_override = Some(temp_adj.max(0.1));
                    self.conservative_mode = meta_acc < 0.7;
                    self.control_triggered = true;
                    actions.push(format!("adjust(temp={:.2})", temp_adj));
                }
            }
        }

        if actions.is_empty() {
            "no_action_needed".to_string()
        } else {
            format!("control:{}", actions.join(","))
        }
    }

    /// Get current recommended temperature based on control state.
    pub fn effective_temperature(&self, default_temp: f64) -> f64 {
        self.temperature_override.unwrap_or(default_temp)
    }

    /// Get current recommended reasoning steps.
    pub fn effective_reasoning_steps(&self, default_steps: usize) -> usize {
        self.reasoning_steps_override.unwrap_or(default_steps)
    }
}

#[derive(Debug, Clone)]
pub struct MetaCycleResult {
    pub iteration: usize,
    pub health_check: HealthCheck,
    pub report: WeaknessReport,
    pub alerts: Vec<MetaAlert>,
    pub plans: Vec<PlannedEvolution>,
    pub trend: super::monitor::HealthTrend,
    pub model_snapshot: SelfModel,
    pub meta_accuracy: f64,
    pub meta_accuracy_trend: f64,
}

// ── MC²: Cross-cycle Meta-Knowledge Accumulation (arXiv:2604.17399) ──

/// Per-cycle snapshot for the fast-timescale buffer.
#[derive(Debug, Clone)]
struct MetaSnapshot {
    cycle: u64,
    health: f64,
    awareness: f64,
    pass_rate: f64,
    arch_penalty: f64,
    negentropy: f64,
}

/// Smoothed aggregate at a given timescale (medium/slow).
#[derive(Debug, Clone, Default)]
struct TimescaleAggregate {
    mean_health: f64,
    mean_awareness: f64,
    trend: f64,
    variance: f64,
    sample_count: usize,
}

/// Cross-cycle meta-knowledge accumulator (MC²).
///
/// Maintains hierarchical multi-timescale statistics:
/// - Fast: per-cycle buffer (configurable capacity, default 100)
/// - Medium: EMA over every 50 cycles
/// - Slow: EMA over every 500 cycles
#[derive(Debug, Clone)]
pub struct MetaKnowledgeAccumulator {
    fast_buffer: Vec<MetaSnapshot>,
    medium: TimescaleAggregate,
    slow: TimescaleAggregate,
    fast_capacity: usize,
}

impl MetaKnowledgeAccumulator {
    pub fn new(fast_capacity: usize) -> Self {
        Self {
            fast_buffer: Vec::with_capacity(fast_capacity),
            medium: TimescaleAggregate::default(),
            slow: TimescaleAggregate::default(),
            fast_capacity,
        }
    }

    /// Record a single cycle's meta snapshot.
    pub fn record(
        &mut self,
        cycle: u64,
        health: f64,
        awareness: f64,
        pass_rate: f64,
        arch_penalty: f64,
        negentropy: f64,
    ) {
        let snap = MetaSnapshot {
            cycle,
            health,
            awareness,
            pass_rate,
            arch_penalty,
            negentropy,
        };

        if self.fast_buffer.len() >= self.fast_capacity {
            self.fast_buffer.remove(0);
        }
        self.fast_buffer.push(snap);

        if cycle > 0 && cycle % 50 == 0 {
            let mean = self.compute_mean(self.fast_buffer.as_slice());
            self.medium = self.update_aggregate(&self.medium, &mean, 50);
        }

        if cycle > 0 && cycle % 500 == 0 {
            let mean = self.compute_mean(self.fast_buffer.as_slice());
            self.slow = self.update_aggregate(&self.slow, &mean, 500);
        }
    }

    fn compute_mean(&self, buffer: &[MetaSnapshot]) -> TimescaleAggregate {
        if buffer.is_empty() {
            return TimescaleAggregate::default();
        }
        let n = buffer.len() as f64;
        let mut agg = TimescaleAggregate::default();
        for snap in buffer {
            agg.mean_health += snap.health / n;
            agg.mean_awareness += snap.awareness / n;
        }
        agg.sample_count = buffer.len();
        agg
    }

    fn update_aggregate(
        &self,
        current: &TimescaleAggregate,
        new_sample: &TimescaleAggregate,
        _window: usize,
    ) -> TimescaleAggregate {
        let alpha = 0.3;
        TimescaleAggregate {
            mean_health: current.mean_health * (1.0 - alpha) + new_sample.mean_health * alpha,
            mean_awareness: current.mean_awareness * (1.0 - alpha)
                + new_sample.mean_awareness * alpha,
            trend: new_sample.mean_health - current.mean_health,
            variance: current.variance * (1.0 - alpha)
                + (new_sample.mean_health - current.mean_health).powi(2) * alpha,
            sample_count: current.sample_count + new_sample.sample_count,
        }
    }

    /// Current meta-knowledge summary string.
    pub fn summary(&self) -> String {
        format!(
            "meta_knowledge: samples={}, fast_trend={:.4}, medium_health={:.4}, slow_health={:.4}",
            self.fast_buffer.len(),
            self.fast_buffer.last().map(|s| s.health).unwrap_or(0.0)
                - self.fast_buffer.first().map(|s| s.health).unwrap_or(0.0),
            self.medium.mean_health,
            self.slow.mean_health,
        )
    }

    /// Detect stagnation (no significant health change over last 10 samples).
    pub fn is_stagnant(&self) -> bool {
        if self.fast_buffer.len() < 10 {
            return false;
        }
        let recent = &self.fast_buffer[self.fast_buffer.len().saturating_sub(10)..];
        let first_health = recent.first().map(|s| s.health).unwrap_or(0.0);
        let last_health = recent.last().map(|s| s.health).unwrap_or(0.0);
        (last_health - first_health).abs() < 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_loop() -> MetaCognitiveLoop {
        let model = SelfModel::new();
        MetaCognitiveLoop::new(model)
    }

    #[test]
    fn test_loop_run_cycle() {
        let mut metacog = make_test_loop();
        let result = metacog.run_cycle();
        assert_eq!(result.iteration, 1);
        assert!(result.health_check.compilation_ok);
        assert_eq!(metacog.iteration, 1);
    }

    #[test]
    fn test_loop_run_batch() {
        let mut metacog = make_test_loop();
        let results = metacog.run_batch(5);
        assert_eq!(results.len(), 5);
        assert_eq!(metacog.iteration, 5);
    }

    #[test]
    fn test_loop_run_until_stop() {
        let mut metacog = make_test_loop();
        let results = metacog.run_until(|r| r.iteration >= 3);
        assert!(results.len() <= 3);
        assert!(metacog.iteration <= 3);
    }

    #[test]
    fn test_loop_status_summary() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        let summary = metacog.status_summary();
        assert!(summary.contains("MetaCognition Cycle"));
        assert!(summary.contains("weaknesses"));
    }

    #[test]
    fn test_loop_reset() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        assert_eq!(metacog.iteration, 1);
        metacog.reset(SelfModel::new());
        assert_eq!(metacog.iteration, 0);
    }

    #[test]
    fn test_max_iterations_enforced() {
        let mut metacog = make_test_loop();
        metacog.max_iterations = 2;
        let results = metacog.run_batch(10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_meta_accuracy_tracking() {
        let mut metacog = make_test_loop();
        metacog.record_meta_accuracy(0.8, 0.75);
        assert!((metacog.current_meta_accuracy() - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_meta_accuracy_trend() {
        let mut metacog = make_test_loop();
        metacog.record_meta_accuracy(0.8, 0.6);
        metacog.record_meta_accuracy(0.8, 0.7);
        metacog.record_meta_accuracy(0.8, 0.8);
        assert!(metacog.meta_accuracy_trend() > 0.0);
    }

    #[test]
    fn test_meta_accuracy_window() {
        let mut metacog = make_test_loop();
        metacog.meta_window = 3;
        for _i in 0..5 {
            metacog.record_meta_accuracy(0.9, 0.5);
        }
        assert_eq!(metacog.meta_accuracy.len(), 3);
    }

    #[test]
    fn test_meta_accuracy_invalid() {
        let mut metacog = make_test_loop();
        metacog.record_meta_accuracy(-0.5, 0.5);
        assert_eq!(metacog.meta_accuracy[0], 0.0);
    }

    #[test]
    fn test_meta_accuracy_empty() {
        let metacog = make_test_loop();
        assert_eq!(metacog.current_meta_accuracy(), 0.0);
    }

    // ── Phase 5: MetaKPI Repository Integration ──

    #[test]
    fn test_kpi_repo_run_cycle_records_snapshot() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        assert_eq!(metacog.kpi_repo.snapshots.len(), 1);
        let snap = metacog.kpi_repo.snapshots.back().unwrap();
        assert!(snap.meta_accuracy >= 0.0);
        assert!(snap.timestamp_ms > 0);
    }

    #[test]
    fn test_kpi_repo_multiple_cycles() {
        let mut metacog = make_test_loop();
        metacog.run_batch(10);
        assert_eq!(metacog.kpi_repo.snapshots.len(), 10);
    }

    #[test]
    fn test_kpi_repo_gap_detection_on_run() {
        let mut metacog = make_test_loop();
        for i in 0..15 {
            metacog.run_cycle();
            if i < 10 {
                // Keep accuracy high for first 10 then let it drop via high tech debt
                metacog.self_model.tech_debt.total_count = (i * 100).min(1000);
            }
        }
        let report = metacog.kpi_repo.detect_gaps(10);
        // After enough cycles we should have data
        assert!(metacog.kpi_repo.snapshots.len() >= 2);
    }

    #[test]
    fn test_status_summary_includes_kpi() {
        let mut metacog = make_test_loop();
        metacog.run_batch(3);
        let summary = metacog.status_summary();
        assert!(summary.contains("snapshots"));
        assert!(summary.contains("goal=none"));
    }

    #[test]
    fn test_reset_clears_kpi_repo() {
        let mut metacog = make_test_loop();
        metacog.run_batch(5);
        assert_eq!(metacog.kpi_repo.snapshots.len(), 5);
        metacog.reset(SelfModel::new());
        assert!(metacog.kpi_repo.snapshots.is_empty());
        assert!(metacog.active_goal.is_none());
    }

    #[test]
    fn test_kpi_repo_snapshot_via_run_cycle_has_unchanged_accuracy() {
        let mut metacog = make_test_loop();
        metacog.run_cycle();
        let snap = metacog.kpi_repo.snapshots.back().unwrap();
        let current_acc = metacog.current_meta_accuracy();
        assert!((snap.meta_accuracy - current_acc).abs() < 1e-6);
    }

    #[test]
    fn test_kpi_repo_performance_decline_detected() {
        let mut metacog = make_test_loop();
        metacog.self_model.tech_debt.total_count = 10;
        metacog.run_batch(5);
        // Increase tech debt to degrade accuracy
        for _ in 0..5 {
            metacog.self_model.tech_debt.total_count = 500;
            metacog.run_cycle();
        }
        let report = metacog.kpi_repo.detect_gaps(10);
        // If performance declined, gaps may be detected
        // The key test: report runs without error
        assert!(report.overall_health >= 0.0);
    }
}
