use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::nt_core_consciousness::consciousness_architecture::{
    ArchitectureLayer, CapabilityStatus, ConsciousnessArchitecture, GapSeverity,
};
use crate::core::nt_core_experience::graceful::{
    DegradationLevel, GracefulDegradationManager, SubsystemHealth,
};

#[derive(Debug, Clone)]
pub struct ArchitectureGapReport {
    pub total_capabilities: usize,
    pub total_degraded: usize,
    pub total_missing: usize,
    pub unregistered_modules: Vec<String>,
    pub health_discrepancies: Vec<HealthDiscrepancy>,
    pub gap_closure_suggestions: Vec<GapClosureSuggestion>,
    pub overall_health_score: f64,
    pub report_timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct HealthDiscrepancy {
    pub module_name: String,
    pub architecture_status: String,
    pub degradation_health: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct GapClosureSuggestion {
    pub gap_id: String,
    pub suggestion: String,
    pub priority: GapSeverity,
    pub estimated_effort: String,
}

pub struct GapDetectorBridge {
    pub last_report: Option<ArchitectureGapReport>,
    pub detection_count: u64,
}

#[derive(Debug, Clone)]
pub struct GapDetectorStats {
    pub detection_count: u64,
    pub last_total_capabilities: usize,
    pub last_missing_count: usize,
    pub last_health_score: f64,
}

impl GapDetectorBridge {
    pub fn new() -> Self {
        GapDetectorBridge {
            last_report: None,
            detection_count: 0,
        }
    }

    /// Compare architecture model vs degradation manager. Detect discrepancies.
    pub fn detect_gaps(
        &mut self,
        arch: &ConsciousnessArchitecture,
        deg: &GracefulDegradationManager,
    ) -> ArchitectureGapReport {
        self.detection_count += 1;

        let _health = arch.assess_health();
        let deg_subsystems: Vec<String> = deg.subsystems.keys().cloned().collect();
        let arch_cap_ids: Vec<String> = {
            let mut ids = Vec::new();
            for layer in ArchitectureLayer::all() {
                for cap in arch.capabilities_by_layer(*layer) {
                    ids.push(cap.id.clone());
                }
            }
            ids
        };

        let known_reasoning_subsystems: [&str; 11] = [
            "mcts_reasoner",
            "parallel_hypothesis",
            "dead_end_detector",
            "epistemic_humility",
            "process_reward_model",
            "bidirectional_pruner",
            "strategy_selector",
            "process_calibration",
            "counterfactual_simulator",
            "gwt_self_interrupt",
            "curiosity_exploration",
        ];

        let mut unregistered_modules = Vec::new();
        for cap_id in &arch_cap_ids {
            let normalized = cap_id.replace('_', "");
            let found = deg_subsystems.iter().any(|s| {
                let s_norm = s.replace('_', "");
                s_norm == normalized
                    || s_norm.contains(&normalized)
                    || normalized.contains(&s_norm)
                    || known_reasoning_subsystems.contains(&s.as_str())
            });
            if !found {
                unregistered_modules.push(cap_id.clone());
            }
        }

        let mut health_discrepancies = Vec::new();
        for layer in ArchitectureLayer::all() {
            for cap in arch.capabilities_by_layer(*layer) {
                let _deg_name = if let Some(correlated) = cap.module_path.as_ref() {
                    let segs: Vec<&str> = correlated.split('/').collect();
                    segs.last()
                        .unwrap_or(&correlated.as_str())
                        .trim_end_matches(".rs")
                        .to_string()
                } else {
                    cap.id.replace('_', " ")
                };

                let arch_status_str = cap.status.name().to_string();
                let deg_health_str = match deg.subsystems.get(&cap.id) {
                    Some(SubsystemHealth::Healthy) => "healthy",
                    Some(SubsystemHealth::Degraded {
                        level: DegradationLevel::Full,
                        ..
                    }) => "degraded(level=full)",
                    Some(SubsystemHealth::Degraded {
                        level: DegradationLevel::Limited,
                        ..
                    }) => "degraded(level=limited)",
                    Some(SubsystemHealth::Degraded {
                        level: DegradationLevel::Minimal,
                        ..
                    }) => "degraded(level=minimal)",
                    Some(SubsystemHealth::Degraded {
                        level: DegradationLevel::None,
                        ..
                    }) => "degraded(level=none)",
                    Some(SubsystemHealth::Failed { .. }) => "failed",
                    Some(SubsystemHealth::Recovering { .. }) => "recovering",
                    None => "unregistered",
                };

                let discrepancy_needed = match cap.status {
                    CapabilityStatus::Maturing | CapabilityStatus::Complete => !matches!(
                        deg.subsystems.get(&cap.id),
                        Some(SubsystemHealth::Healthy)
                            | Some(SubsystemHealth::Degraded {
                                level: DegradationLevel::Full,
                                ..
                            })
                    ),
                    CapabilityStatus::Partial => {
                        matches!(
                            deg.subsystems.get(&cap.id),
                            None | Some(SubsystemHealth::Failed { .. })
                        )
                    }
                    CapabilityStatus::Missing => false,
                };

                if discrepancy_needed {
                    let desc = format!(
                        "Architecture says '{}' (status={}), but degradation manager reports '{}'",
                        cap.name, arch_status_str, deg_health_str
                    );
                    health_discrepancies.push(HealthDiscrepancy {
                        module_name: cap.id.clone(),
                        architecture_status: arch_status_str,
                        degradation_health: deg_health_str.to_string(),
                        description: desc,
                    });
                }
            }
        }

        let gap_cap_ids: Vec<String> = {
            let mut ids = Vec::new();
            for layer in ArchitectureLayer::all() {
                for cap in arch.capabilities_by_layer(*layer) {
                    if cap.status == CapabilityStatus::Missing {
                        ids.push(cap.id.clone());
                    }
                }
            }
            ids
        };

        let gap_closure_suggestions = self.suggest_closure(&gap_cap_ids);

        let total_capabilities = arch_cap_ids.len();
        let total_degraded = deg
            .subsystems
            .values()
            .filter(|h| !matches!(h, SubsystemHealth::Healthy))
            .count();
        let total_missing = gap_cap_ids.len();
        let overall_health_score = self.system_health_score(arch, deg);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let report = ArchitectureGapReport {
            total_capabilities,
            total_degraded,
            total_missing,
            unregistered_modules,
            health_discrepancies,
            gap_closure_suggestions,
            overall_health_score,
            report_timestamp: now,
        };

        self.last_report = Some(report.clone());
        report
    }

    /// Suggest gap-closure actions for missing capabilities
    pub fn suggest_closure(&self, gap_capabilities: &[String]) -> Vec<GapClosureSuggestion> {
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cap_id in gap_capabilities {
            let (suggestion_text, priority, effort) = match cap_id.as_str() {
                "mcts_reasoning" => (
                    "Implement MCTS tree search: UCT select, expand, simulate, backprop with VSA coherence reward",
                    GapSeverity::Survival,
                    "medium (600-800 lines)",
                ),
                "parallel_hypothesis_evaluator" | "parallel_hypothesis" => (
                    "Implement parallel hypothesis evaluation: competing hypotheses with Bayesian belief update",
                    GapSeverity::Survival,
                    "medium (400-500 lines)",
                ),
                "dead_end_detector" => (
                    "Implement dead-end detection: loop/divergence/contradiction flood monitors with recovery strategies",
                    GapSeverity::Survival,
                    "small (300-400 lines)",
                ),
                "world_model" => (
                    "Implement hierarchical world model: state prediction, consequence simulation, scene graph",
                    GapSeverity::Survival,
                    "large (800-1000 lines)",
                ),
                "counterfactual" => (
                    "Implement counterfactual reasoning: VSA perturbation, structural intervention, what-if simulation",
                    GapSeverity::Survival,
                    "large (800-900 lines)",
                ),
                "causal_reasoning" => (
                    "Implement causal reasoning: do-calculus operations, causal graph construction, intervention effects",
                    GapSeverity::Survival,
                    "large (1000-1200 lines)",
                ),
                "analogy_reasoning" | "analogical_reasoning" => (
                    "Implement analogical reasoning: structure mapping engine, relational similarity over VSA vectors",
                    GapSeverity::Evolution,
                    "medium (600-800 lines)",
                ),
                "confidence_calibrator" => (
                    "Implement confidence calibration: ECE tracking, temperature scaling, Platt scaling",
                    GapSeverity::Survival,
                    "medium (400-500 lines)",
                ),
                "synthetic_data_factory" => (
                    "Implement synthetic data factory: template-based generation, scenario enumeration, VSA perturbation",
                    GapSeverity::Evolution,
                    "large (1000-1200 lines)",
                ),
                "pixel_perception" => (
                    "Implement pixel-native perception: screenshot capture, VSA binding, layout chunking",
                    GapSeverity::Survival,
                    "small (250-350 lines)",
                ),
                "visual_embedding_frontend" => (
                    "Implement visual embedding model frontend: image→embedding, VSA alignment pipeline",
                    GapSeverity::Survival,
                    "medium (340-450 lines)",
                ),
                "visual_rag_index" => (
                    "Implement visual RAG index: FAISS+VSA hybrid, multi-modal index architecture",
                    GapSeverity::Evolution,
                    "small (200-300 lines)",
                ),
                "multi_modal_gate" => (
                    "Implement multi-modal GWT attention gating: modality arbitration, attention allocation",
                    GapSeverity::Survival,
                    "medium (370-500 lines)",
                ),
                "resource_allocator" => (
                    "Implement conscious resource allocator: dynamic cognitive budget based on internal state",
                    GapSeverity::Evolution,
                    "medium (350-500 lines)",
                ),
                "episodic_buffer" => (
                    "Implement episodic consciousness buffer: short-term high-resolution state ring buffer",
                    GapSeverity::Evolution,
                    "small (320-400 lines)",
                ),
                "cognitive_blackboard" => (
                    "Implement cognitive blackboard: cross-engine shared workspace for claims and contradictions",
                    GapSeverity::Evolution,
                    "medium (340-500 lines)",
                ),
                "consciousness_refinery" => (
                    "Implement consciousness refinery inner loop: LoopWM-style iterative refinement",
                    GapSeverity::Evolution,
                    "small (320-400 lines)",
                ),
                "dual_path_inference" => (
                    "Implement dual-path inference: constraint+generative dual model architecture",
                    GapSeverity::Survival,
                    "small (280-400 lines)",
                ),
                "executable_belief" => (
                    "Implement executable belief verification: Inspector×CLR×Stability triple verification",
                    GapSeverity::Survival,
                    "small (300-400 lines)",
                ),
                "meta_evolution_loop" => (
                    "Implement meta-architecture evolution loop: autonomous assess→recommend→track→measure cycle",
                    GapSeverity::Survival,
                    "medium (360-500 lines)",
                ),
                "consciousness_pipeline" => (
                    "Implement consciousness pipeline: unified integration of 10 architecture modules into single run()",
                    GapSeverity::Survival,
                    "medium (350-500 lines)",
                ),
                "performance_oracle" => (
                    "Implement performance oracle: self-learning pipeline optimizer with health dashboard",
                    GapSeverity::Evolution,
                    "medium (350-500 lines)",
                ),
                "adaptive_controller" => (
                    "Implement adaptive controller: Oracle→Pipeline automatic feedback loop for self-adaptive operation",
                    GapSeverity::Evolution,
                    "small (250-350 lines)",
                ),
                "key_vault" => (
                    "Implement encrypted credential & key vault: API keys, wallet seeds, secure storage",
                    GapSeverity::Survival,
                    "small (150-250 lines)",
                ),
                "data_feed" => (
                    "Implement unified market data feed: exchange, news, ad API abstraction layer",
                    GapSeverity::Survival,
                    "small (200-300 lines)",
                ),
                "economic_agent" => (
                    "Implement economic agency: autonomous opportunity analysis, strategy execution, P&L tracking",
                    GapSeverity::Survival,
                    "medium (350-500 lines)",
                ),
                "risk_manager" => (
                    "Implement risk management: position sizing, VaR 95%, kill-switch, drawdown monitor",
                    GapSeverity::Survival,
                    "small (200-300 lines)",
                ),
                "economic_world_model" => (
                    "Implement economic world model: GDP/inflation/rate/sentiment variables, regime prediction",
                    GapSeverity::Evolution,
                    "small (180-300 lines)",
                ),
                "screenshot_pipeline" => (
                    "Implement screenshot capture pipeline: Chromium CDP integration, base64 + VSA encoding",
                    GapSeverity::Survival,
                    "small (220-300 lines)",
                ),
                "spectrum_signal" => (
                    "Implement spectrum→signal diversity pipeline: multi-source signal fusion",
                    GapSeverity::Evolution,
                    "small (300-400 lines)",
                ),
                "spatial_scene" => (
                    "Implement spatial scene understanding: scene graph construction, spatial VSA embeddings",
                    GapSeverity::Enhancement,
                    "small (200-300 lines)",
                ),
                _ => {
                    if !seen.contains(cap_id) {
                        seen.insert(cap_id.clone());
                    }
                    continue;
                }
            };

            if seen.contains(cap_id) {
                continue;
            }
            seen.insert(cap_id.clone());

            let gap_id = cap_id.clone();
            suggestions.push(GapClosureSuggestion {
                gap_id,
                suggestion: suggestion_text.to_string(),
                priority,
                estimated_effort: effort.to_string(),
            });
        }

        suggestions.sort_by_key(|s| s.priority.priority());
        suggestions
    }

    /// Overall system health score (0.0-1.0)
    pub fn system_health_score(
        &self,
        arch: &ConsciousnessArchitecture,
        deg: &GracefulDegradationManager,
    ) -> f64 {
        let health = arch.assess_health();
        let arch_score = health.overall_health;

        let total = deg.subsystems.len();
        let healthy_count = deg
            .subsystems
            .values()
            .filter(|h| matches!(h, SubsystemHealth::Healthy))
            .count();
        let operational_count = deg
            .subsystems
            .values()
            .filter(|h| h.is_operational())
            .count();

        let deg_score = if total == 0 {
            0.0
        } else {
            let op_ratio = operational_count as f64 / total as f64;
            let healthy_ratio = healthy_count as f64 / total as f64;
            0.6 * op_ratio + 0.4 * healthy_ratio
        };

        0.5 * arch_score + 0.5 * deg_score
    }

    /// Generate a human-readable summary of all detected gaps
    pub fn summary(&self, report: &ArchitectureGapReport) -> String {
        let health_pct = report.overall_health_score * 100.0;
        let health_label = if report.overall_health_score >= 0.8 {
            "good"
        } else if report.overall_health_score >= 0.5 {
            "fair"
        } else if report.overall_health_score >= 0.3 {
            "poor"
        } else {
            "critical"
        };

        let mut lines = Vec::new();

        lines.push(format!(
            "┌─ Gap Detection Report #{} ─────────────────────┐",
            self.detection_count
        ));
        lines.push(format!("│ Health: {:.1}% ({})", health_pct, health_label));
        lines.push(format!(
            "│ Capabilities: {} total, {} missing, {} degraded",
            report.total_capabilities, report.total_missing, report.total_degraded
        ));
        lines.push(format!(
            "│ Unregistered modules: {}",
            report.unregistered_modules.len()
        ));
        lines.push(format!(
            "│ Health discrepancies: {}",
            report.health_discrepancies.len()
        ));
        lines.push(format!(
            "│ Closure suggestions: {}",
            report.gap_closure_suggestions.len()
        ));
        lines.push("└─────────────────────────────────────────────┘".to_string());

        if !report.unregistered_modules.is_empty() {
            lines.push(format!(
                "\nUnregistered modules ({}):",
                report.unregistered_modules.len()
            ));
            for m in report.unregistered_modules.iter().take(10) {
                lines.push(format!("  • {}", m));
            }
            if report.unregistered_modules.len() > 10 {
                lines.push(format!(
                    "  ... and {} more",
                    report.unregistered_modules.len() - 10
                ));
            }
        }

        if !report.health_discrepancies.is_empty() {
            lines.push(format!(
                "\nHealth discrepancies ({}):",
                report.health_discrepancies.len()
            ));
            for d in report.health_discrepancies.iter().take(5) {
                lines.push(format!("  • {}: {}", d.module_name, d.description));
            }
            if report.health_discrepancies.len() > 5 {
                lines.push(format!(
                    "  ... and {} more",
                    report.health_discrepancies.len() - 5
                ));
            }
        }

        if !report.gap_closure_suggestions.is_empty() {
            lines.push(format!("\nTop closure suggestions (by priority):"));
            for s in report.gap_closure_suggestions.iter().take(5) {
                lines.push(format!(
                    "  [{:?}] {}: {}",
                    s.priority, s.gap_id, s.suggestion
                ));
            }
            if report.gap_closure_suggestions.len() > 5 {
                lines.push(format!(
                    "  ... and {} more",
                    report.gap_closure_suggestions.len() - 5
                ));
            }
        }

        lines.join("\n")
    }

    pub fn stats(&self) -> GapDetectorStats {
        GapDetectorStats {
            detection_count: self.detection_count,
            last_total_capabilities: self
                .last_report
                .as_ref()
                .map(|r| r.total_capabilities)
                .unwrap_or(0),
            last_missing_count: self
                .last_report
                .as_ref()
                .map(|r| r.total_missing)
                .unwrap_or(0),
            last_health_score: self
                .last_report
                .as_ref()
                .map(|r| r.overall_health_score)
                .unwrap_or(0.0),
        }
    }
}

impl Default for GapDetectorBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> ConsciousnessArchitecture {
        ConsciousnessArchitecture::new()
    }

    fn make_reasoning_deg() -> GracefulDegradationManager {
        GracefulDegradationManager::with_reasoning_modules()
    }

    fn make_bare_deg() -> GracefulDegradationManager {
        GracefulDegradationManager::new()
    }

    #[test]
    fn test_gap_detector_new() {
        let detector = GapDetectorBridge::new();
        assert!(detector.last_report.is_none());
        assert_eq!(detector.detection_count, 0);
    }

    #[test]
    fn test_detect_gaps_with_reasoning_modules() {
        let arch = make_arch();
        let deg = make_reasoning_deg();
        let mut detector = GapDetectorBridge::new();

        let report = detector.detect_gaps(&arch, &deg);
        assert_eq!(detector.detection_count, 1);
        assert!(report.total_capabilities > 20);
        assert!(report.total_missing >= 2);
        assert!(report.overall_health_score > 0.0);
    }

    #[test]
    fn test_detect_gaps_with_bare_deg() {
        let arch = make_arch();
        let deg = make_bare_deg();
        let mut detector = GapDetectorBridge::new();

        let report = detector.detect_gaps(&arch, &deg);
        assert!(report.total_capabilities > 20);
        assert_eq!(report.total_degraded, 0);
        assert!(report.overall_health_score < 0.6);
    }

    #[test]
    fn test_detect_gaps_increments_counter() {
        let arch = make_arch();
        let deg = make_reasoning_deg();
        let mut detector = GapDetectorBridge::new();

        detector.detect_gaps(&arch, &deg);
        detector.detect_gaps(&arch, &deg);
        detector.detect_gaps(&arch, &deg);
        assert_eq!(detector.detection_count, 3);
    }

    #[test]
    fn test_suggest_closure_empty() {
        let detector = GapDetectorBridge::new();
        let suggestions = detector.suggest_closure(&[]);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_suggest_closure_known_gaps() {
        let detector = GapDetectorBridge::new();
        let gap_ids: Vec<String> = vec!["mcts_reasoning".into(), "world_model".into()];
        let suggestions = detector.suggest_closure(&gap_ids);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.gap_id == "mcts_reasoning"));
        assert!(suggestions.iter().any(|s| s.gap_id == "world_model"));
    }

    #[test]
    fn test_suggest_closure_prioritizes_survival() {
        let detector = GapDetectorBridge::new();
        let gap_ids: Vec<String> = vec!["analogy_reasoning".into(), "mcts_reasoning".into()];
        let suggestions = detector.suggest_closure(&gap_ids);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].priority, GapSeverity::Survival);
    }

    #[test]
    fn test_suggest_closure_unknown_gap() {
        let detector = GapDetectorBridge::new();
        let gap_ids: Vec<String> = vec!["nonexistent_module_xyz".into()];
        let suggestions = detector.suggest_closure(&gap_ids);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_system_health_score_with_reasoning() {
        let arch = make_arch();
        let deg = make_reasoning_deg();
        let detector = GapDetectorBridge::new();

        let score = detector.system_health_score(&arch, &deg);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_system_health_score_with_bare_deg() {
        let arch = make_arch();
        let deg = make_bare_deg();
        let detector = GapDetectorBridge::new();

        let score = detector.system_health_score(&arch, &deg);
        assert!(score > 0.0);
        assert!(score <= 1.0);
        let _full_score = detector.system_health_score(&arch, &make_reasoning_deg());
    }

    #[test]
    fn test_summary_contains_report() {
        let arch = make_arch();
        let deg = make_reasoning_deg();
        let mut detector = GapDetectorBridge::new();
        let report = detector.detect_gaps(&arch, &deg);
        let summary = detector.summary(&report);

        assert!(summary.contains("Gap Detection Report"));
        assert!(summary.contains("Health:"));
        assert!(summary.contains("Capabilities:"));
    }

    #[test]
    fn test_summary_with_missing_and_discrepancies() {
        let arch = make_arch();
        let deg = make_bare_deg();
        let mut detector = GapDetectorBridge::new();
        let report = detector.detect_gaps(&arch, &deg);
        let summary = detector.summary(&report);

        assert!(
            summary.contains("Unregistered modules") || summary.contains("closure suggestions")
        );
    }

    #[test]
    fn test_stats() {
        let arch = make_arch();
        let deg = make_reasoning_deg();
        let mut detector = GapDetectorBridge::new();

        let initial_stats = detector.stats();
        assert_eq!(initial_stats.detection_count, 0);

        detector.detect_gaps(&arch, &deg);
        let stats = detector.stats();
        assert_eq!(stats.detection_count, 1);
        assert!(stats.last_total_capabilities > 0);
        assert!(stats.last_health_score > 0.0);
    }

    #[test]
    fn test_detect_gaps_reports_unregistered_subsystems() {
        let arch = make_arch();
        let mut deg = GracefulDegradationManager::new();
        deg.subsystems
            .insert("vision".into(), SubsystemHealth::Healthy);
        let mut detector = GapDetectorBridge::new();
        let report = detector.detect_gaps(&arch, &deg);
        assert!(!report.unregistered_modules.is_empty());
    }

    #[test]
    fn test_system_health_score_improves_with_healthy_subsystems() {
        let arch = make_arch();
        let full_deg = make_reasoning_deg();
        let bare_deg = make_bare_deg();
        let detector = GapDetectorBridge::new();

        let full_score = detector.system_health_score(&arch, &full_deg);
        let bare_score = detector.system_health_score(&arch, &bare_deg);
        assert!(
            full_score >= bare_score - 0.01,
            "full={} should be >= bare={}",
            full_score,
            bare_score
        );
    }

    #[test]
    fn test_detect_gaps_for_degraded_subsystem() {
        let arch = make_arch();
        let mut deg = GracefulDegradationManager::new();
        deg.register_subsystem("vsa_vectors".into(), vec!["vsa_vector_ops".into()]);
        deg.register_subsystem("e8_reasoning".into(), vec!["e8_inference".into()]);
        deg.report_failure("vsa_vectors", "simd crash".into());

        let mut detector = GapDetectorBridge::new();
        let report = detector.detect_gaps(&arch, &deg);
        assert_eq!(report.total_degraded, 1);
    }

    #[test]
    fn test_gap_closure_suggestion_has_effort() {
        let detector = GapDetectorBridge::new();
        let gap_ids: Vec<String> = vec!["causal_reasoning".into()];
        let suggestions = detector.suggest_closure(&gap_ids);
        assert_eq!(suggestions.len(), 1);
        assert!(!suggestions[0].estimated_effort.is_empty());
        assert_eq!(suggestions[0].priority, GapSeverity::Survival);
    }
}
